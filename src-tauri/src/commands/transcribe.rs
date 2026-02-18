use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ruhear::{RUBuffers, RUHear};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// ── Recording state ─────────────────────────────────────────────────────────

struct SysAudioHandle {
    samples: Arc<Mutex<Vec<f32>>>,
    stop_flag: Arc<Mutex<bool>>,
    thread: Option<thread::JoinHandle<()>>,
}

struct RecordingHandle {
    samples: Arc<Mutex<Vec<i16>>>,
    sample_rate: u32,
    stop_flag: Arc<Mutex<bool>>,
    thread: Option<thread::JoinHandle<()>>,
    sys_audio: Option<SysAudioHandle>,
}

pub struct RecordingState(Mutex<Option<RecordingHandle>>);

impl Default for RecordingState {
    fn default() -> Self {
        RecordingState(Mutex::new(None))
    }
}

// ── Whisper state ────────────────────────────────────────────────────────────

pub struct WhisperState(Mutex<Option<WhisperContext>>);

impl Default for WhisperState {
    fn default() -> Self {
        WhisperState(Mutex::new(None))
    }
}

// ── Paths ────────────────────────────────────────────────────────────────────

fn brief_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot find home dir")
        .join("Brief")
}

fn recordings_dir() -> PathBuf {
    brief_dir().join("recordings")
}

fn models_dir() -> PathBuf {
    brief_dir().join("models")
}

fn model_path() -> PathBuf {
    models_dir().join("ggml-base.en.bin")
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Linear-interpolation resample from `src_rate` Hz mono i16 → 16 kHz mono f32.
fn resample_to_16k(samples: &[i16], src_rate: u32) -> Vec<f32> {
    if src_rate == 16000 {
        return samples.iter().map(|&s| s as f32 / 32768.0).collect();
    }
    let ratio = src_rate as f64 / 16000.0;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos as usize;
        let frac = (pos - idx as f64) as f32;
        let s0 = samples.get(idx).copied().unwrap_or(0) as f32 / 32768.0;
        let s1 = samples.get(idx + 1).copied().unwrap_or(0) as f32 / 32768.0;
        out.push(s0 + (s1 - s0) * frac);
    }
    out
}

/// Linear-interpolation resample from `src_rate` Hz mono f32 → 16 kHz mono f32.
fn resample_f32_to_16k(samples: &[f32], src_rate: u32) -> Vec<f32> {
    if src_rate == 16000 {
        return samples.to_vec();
    }
    let ratio = src_rate as f64 / 16000.0;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos as usize;
        let frac = (pos - idx as f64) as f32;
        let s0 = samples.get(idx).copied().unwrap_or(0.0);
        let s1 = samples.get(idx + 1).copied().unwrap_or(0.0);
        out.push(s0 + (s1 - s0) * frac);
    }
    out
}

/// Start capturing system audio in a background thread.
/// Returns None (with a logged warning) if the platform doesn't support it.
fn start_system_audio() -> Result<SysAudioHandle, String> {
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
    let stop_flag = Arc::new(Mutex::new(false));

    let s = Arc::clone(&samples);
    let sf = Arc::clone(&stop_flag);

    let thread = thread::spawn(move || {
        // RUBuffers = Vec<Vec<f32>>: outer index = channel, inner = samples
        let cb_samples = Arc::clone(&s);
        let callback = move |data: RUBuffers| {
            let num_ch = data.len();
            if num_ch == 0 {
                return;
            }
            let frame_len = data[0].len();
            let mut buf = cb_samples.lock().unwrap();
            for i in 0..frame_len {
                let sum: f32 = data.iter().map(|ch| ch.get(i).copied().unwrap_or(0.0)).sum();
                buf.push(sum / num_ch as f32);
            }
        };

        let shared_cb: Arc<Mutex<dyn FnMut(RUBuffers) + Send>> =
            Arc::new(Mutex::new(callback));
        let mut ruhear = RUHear::new(shared_cb);

        if let Err(e) = ruhear.start() {
            eprintln!("System audio capture failed to start: {e}");
            return;
        }

        loop {
            thread::sleep(Duration::from_millis(100));
            if *sf.lock().unwrap() {
                break;
            }
        }

        let _ = ruhear.stop();
    });

    Ok(SysAudioHandle {
        samples,
        stop_flag,
        thread: Some(thread),
    })
}

/// Stop the active recording and return mic samples, mic sample rate, and system audio samples.
fn collect_samples(state: &RecordingState) -> Result<(Vec<i16>, u32, Vec<f32>), String> {
    let handle = {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        guard.take().ok_or_else(|| "Not recording".to_string())?
    };

    let RecordingHandle {
        samples,
        sample_rate,
        stop_flag,
        thread,
        sys_audio,
    } = handle;

    *stop_flag.lock().map_err(|e| e.to_string())? = true;
    drop(stop_flag);

    if let Some(t) = thread {
        let _ = t.join();
    }

    let sys_samples = if let Some(mut sa) = sys_audio {
        *sa.stop_flag.lock().map_err(|e| e.to_string())? = true;
        if let Some(t) = sa.thread.take() {
            let _ = t.join();
        }
        sa.samples.lock().map_err(|e| e.to_string())?.clone()
    } else {
        Vec::new()
    };

    let mic_samples = samples.lock().map_err(|e| e.to_string())?.clone();
    Ok((mic_samples, sample_rate, sys_samples))
}

/// Write a mono i16 PCM buffer to a WAV file and return the path.
fn write_wav(samples: &[i16], sample_rate: u32) -> Result<PathBuf, String> {
    let dir = recordings_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!(
        "{}.wav",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    ));
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
    for &s in samples {
        writer.write_sample(s).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(path)
}

// ── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn start_recording(state: tauri::State<RecordingState>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Err("Already recording".into());
    }

    // Gather device config on the calling thread
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No input device available".to_string())?;
    let supported = device
        .default_input_config()
        .map_err(|e| e.to_string())?;

    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels() as usize;
    let sample_format = supported.sample_format();
    let stream_config: cpal::StreamConfig = supported.into();

    let samples = Arc::new(Mutex::new(Vec::<i16>::new()));
    let stop_flag = Arc::new(Mutex::new(false));

    let samples_t = Arc::clone(&samples);
    let stop_flag_t = Arc::clone(&stop_flag);

    let handle = thread::spawn(move || {
        // Re-acquire device on this thread — required on macOS
        let host = cpal::default_host();
        let device = match host.default_input_device() {
            Some(d) => d,
            None => {
                eprintln!("Recording thread: no input device");
                return;
            }
        };

        let cb_samples = Arc::clone(&samples_t);
        let ch = channels;

        let stream_result: Result<cpal::Stream, _> = match sample_format {
            cpal::SampleFormat::I16 => device.build_input_stream::<i16, _, _>(
                &stream_config,
                move |data: &[i16], _| {
                    let mut s = cb_samples.lock().unwrap();
                    for frame in data.chunks(ch) {
                        let sum: i32 = frame.iter().map(|&x| x as i32).sum();
                        s.push((sum / ch as i32) as i16);
                    }
                },
                |e| eprintln!("cpal error: {e}"),
                None,
            ),
            cpal::SampleFormat::F32 => {
                let cb_samples_f = Arc::clone(&cb_samples);
                device.build_input_stream::<f32, _, _>(
                    &stream_config,
                    move |data: &[f32], _| {
                        let mut s = cb_samples_f.lock().unwrap();
                        for frame in data.chunks(ch) {
                            let sum: f32 = frame.iter().sum();
                            let mono = (sum / ch as f32).clamp(-1.0, 1.0);
                            s.push((mono * 32767.0) as i16);
                        }
                    },
                    |e| eprintln!("cpal error: {e}"),
                    None,
                )
            }
            fmt => {
                eprintln!("Unsupported sample format: {fmt:?}");
                return;
            }
        };

        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to build input stream: {e}");
                return;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("Failed to start stream: {e}");
            return;
        }

        loop {
            thread::sleep(Duration::from_millis(100));
            if *stop_flag_t.lock().unwrap() {
                break;
            }
        }
        // stream dropped here — recording stops
    });

    let sys_audio = start_system_audio()
        .map_err(|e| eprintln!("System audio unavailable: {e}"))
        .ok();

    *guard = Some(RecordingHandle {
        samples,
        sample_rate,
        stop_flag,
        thread: Some(handle),
        sys_audio,
    });

    Ok(())
}

#[tauri::command]
pub fn stop_recording(state: tauri::State<RecordingState>) -> Result<String, String> {
    let (samples, sample_rate, _sys_samples) = collect_samples(&state)?;
    let path = write_wav(&samples, sample_rate)?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn stop_and_transcribe(
    recording: tauri::State<RecordingState>,
    whisper: tauri::State<WhisperState>,
) -> Result<String, String> {
    let (mic_i16, mic_rate, sys_mono) = collect_samples(&recording)?;

    if mic_i16.is_empty() && sys_mono.is_empty() {
        return Ok(String::new());
    }

    // Write temp WAV of mic audio (useful for debugging; deleted after transcription)
    let wav_path = if !mic_i16.is_empty() {
        Some(write_wav(&mic_i16, mic_rate)?)
    } else {
        None
    };

    // Resample mic i16 @ mic_rate → 16 kHz f32
    let mic_f32 = resample_to_16k(&mic_i16, mic_rate);

    // Resample system audio f32 @ 48 kHz → 16 kHz f32
    let sys_f32 = resample_f32_to_16k(&sys_mono, 48000);

    // Mix: average both streams, padding the shorter one with silence
    let audio_f32: Vec<f32> = if sys_f32.is_empty() {
        mic_f32
    } else if mic_f32.is_empty() {
        sys_f32
    } else {
        let len = mic_f32.len().max(sys_f32.len());
        (0..len)
            .map(|i| {
                let m = mic_f32.get(i).copied().unwrap_or(0.0);
                let s = sys_f32.get(i).copied().unwrap_or(0.0);
                ((m + s) / 2.0).clamp(-1.0, 1.0)
            })
            .collect()
    };

    // Load the model if not already cached
    let model = model_path();
    if !model.exists() {
        return Err("Whisper model not found. Please download it first.".into());
    }

    let mut ctx_guard = whisper.0.lock().map_err(|e| e.to_string())?;
    if ctx_guard.is_none() {
        let ctx = WhisperContext::new_with_params(
            model.to_str().ok_or("Invalid model path")?,
            WhisperContextParameters::default(),
        )
        .map_err(|e| e.to_string())?;
        *ctx_guard = Some(ctx);
    }
    let ctx = ctx_guard.as_ref().unwrap();

    // Run transcription
    let mut wstate = ctx.create_state().map_err(|e| e.to_string())?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    wstate.full(params, &audio_f32).map_err(|e| e.to_string())?;

    let n = wstate.full_n_segments().map_err(|e| e.to_string())?;
    let mut transcript = String::new();
    for i in 0..n {
        let text = wstate
            .full_get_segment_text(i)
            .map_err(|e| e.to_string())?;
        transcript.push_str(text.trim());
        transcript.push(' ');
    }

    // Clean up temp WAV
    if let Some(path) = wav_path {
        let _ = std::fs::remove_file(&path);
    }

    Ok(transcript.trim().to_string())
}

#[tauri::command]
pub fn check_whisper_model() -> Result<serde_json::Value, String> {
    let path = model_path();
    Ok(serde_json::json!({
        "exists": path.exists(),
        "path": path.to_string_lossy()
    }))
}

#[tauri::command]
pub async fn download_whisper_model(app: tauri::AppHandle) -> Result<(), String> {
    const URL: &str =
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";

    let dir = models_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let final_path = model_path();
    let tmp_path = dir.join("ggml-base.en.bin.tmp");

    let client = reqwest::Client::new();
    let mut resp = client
        .get(URL)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Download failed: HTTP {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total > 0 {
            let percent = ((downloaded as f64 / total as f64) * 100.0) as u8;
            let _ = app.emit(
                "whisper-download-progress",
                serde_json::json!({
                    "downloaded": downloaded,
                    "total": total,
                    "percent": percent,
                }),
            );
        }
    }

    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    tokio::fs::rename(&tmp_path, &final_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
