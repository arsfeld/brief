---
id: TASK-1
status: groomed
updated: 2026-02-18
---

# TASK-1 Implementation Plan: Audio Recording & Transcription

## Context

Brief currently has no way to capture speech. All note-taking is manual. Adding mic recording + Whisper transcription lets users speak during a meeting and get a transcript appended to their note — the core value prop of tools like Granola.

The feature fits naturally into the existing pattern:
- A new button in `EditorToolbar` (alongside the Enhance button)
- A new event bus entry, like the existing `registerEnhanceListener`, to signal the Editor
- A new `src-tauri/src/commands/transcribe.rs` module, registered in `lib.rs` like the existing `notes` and `ai` commands

## Approach

**Audio capture: Rust (`cpal` crate) — not the browser MediaRecorder API**

The browser's `MediaRecorder` outputs WebM/Opus, which whisper.cpp cannot consume directly. Converting would require ffmpeg or a WASM audio processor. Instead, `cpal` captures raw PCM from Core Audio on macOS, and `hound` writes it to a 16kHz mono 16-bit WAV file — exactly what whisper expects. The entire audio pipeline stays in Rust.

**Transcription: `whisper-rs` crate (FFI to whisper.cpp)**

`whisper-rs` compiles whisper.cpp into the Rust binary at build time — no sidecar, no subprocess management. The downside is a longer first build (~2–3 minutes) and a C++ compiler requirement (`cmake`, `clang`). The model file (`ggml-base.en.bin`, ~148MB) is downloaded to `~/Brief/models/` on first use.

**Alternatives considered and rejected:**
- `macOS SpeechRecognizer` via AppleScript: less accurate, requires internet on some OS versions
- External `whisper-cli` sidecar: requires shipping a separate binary and managing a subprocess
- Cloud transcription (Deepgram, AssemblyAI): contradicts "works offline" acceptance criterion

**Transcript placement in the note:**
Append the transcript inline to the note's existing content, separated by a blank line:

```
\n\n{transcript}
```

---

## Stage 1: Recording UI Controls

**Goal**: Record/Stop button appears in EditorToolbar; recording state is visible to the user. No backend yet — the IPC calls are stubbed.

**Status**: Complete

**Files:**
- `src/components/editor/EditorToolbar.tsx` — add Record/Stop button, recording timer, event bus for start/stop
- `src/components/editor/Editor.tsx` — register recording listener, stub `startRecording`/`stopRecording` IPC calls

**Steps:**
1. Add a `registerRecordListener(noteId, fn)` + `listeners` map to `EditorToolbar.tsx`, parallel to the existing `registerEnhanceListener`.
2. Add `recording` and `transcribing` boolean states to `EditorToolbar`.
3. Render a "Record" button (mic icon) that toggles to a "Stop" button (square icon) when `recording === true`. Show a `Loader2` spinner when `transcribing === true`.
4. In `Editor.tsx`, call `registerRecordListener(noteId, handler)` in a `useEffect`, same pattern as the enhance listener. The handler will call `invoke("start_recording")` / `invoke("stop_recording")` (stubbed for now with `console.log`).

**Success criteria:**
- Record button renders in toolbar with no TypeScript errors
- Clicking cycles through idle → recording → transcribing → idle states visually
- `pnpm build` passes cleanly

---

## Stage 2: Rust Audio Capture (cpal + hound)

**Goal**: `start_recording` and `stop_recording` Tauri commands work. Mic audio is saved to `~/Brief/recordings/{timestamp}.wav` (16kHz, mono, 16-bit PCM).

**Status**: Complete

**Files:**
- `src-tauri/Cargo.toml` — add `cpal`, `hound`, `uuid`
- `src-tauri/src/commands/transcribe.rs` (new) — `start_recording`, `stop_recording` commands
- `src-tauri/src/commands/mod.rs` — `pub mod transcribe;`
- `src-tauri/src/lib.rs` — register commands, manage `RecordingState`
- `src-tauri/tauri.conf.json` — add macOS `NSMicrophoneUsageDescription`
- `src-tauri/capabilities/default.json` — no changes needed (core:default covers microphone dialog on macOS via the entitlement)

**Steps:**
1. Add to `Cargo.toml`:
   ```toml
   cpal = "0.15"
   hound = "3.5"
   uuid = { version = "1", features = ["v4"] }
   ```
2. Define `RecordingState` in `transcribe.rs`:
   ```rust
   pub struct RecordingState {
       pub stream: Option<cpal::Stream>,
       pub samples: Arc<Mutex<Vec<i16>>>,
       pub sample_rate: u32,
       pub out_path: Option<PathBuf>,
   }
   ```
3. `start_recording` command: get default input device via cpal, build an input stream that collects i16 samples into a shared `Arc<Mutex<Vec<i16>>>`, store stream + samples in `RecordingState` (via `tauri::State`).
4. `stop_recording` command: drop the cpal stream, write samples to WAV using `hound::WavWriter` (16kHz, 1ch, 16-bit), return the path to the caller.
5. In `lib.rs`, call `app.manage(Mutex::new(RecordingState::default()))` and register the two commands.
6. Add `NSMicrophoneUsageDescription` to `tauri.conf.json`:
   ```json
   "bundle": {
     "macOS": {
       "entitlements": null,
       "infoPlist": {
         "NSMicrophoneUsageDescription": "Brief needs microphone access to transcribe meeting audio."
       }
     }
   }
   ```

**Success criteria:**
- `pnpm tauri dev` — clicking Record asks for mic permission on macOS
- A `.wav` file appears in `~/Brief/recordings/` after clicking Stop
- WAV is audible when played back

---

## Stage 3: whisper-rs Transcription

**Goal**: `transcribe_audio(path)` Tauri command returns a transcript string from a WAV file.

**Status**: Complete

**Files:**
- `src-tauri/Cargo.toml` — add `whisper-rs`
- `src-tauri/src/commands/transcribe.rs` — add `transcribe_audio` command, `WhisperState` for cached context
- `src-tauri/src/lib.rs` — manage `WhisperState`

**Steps:**
1. Add to `Cargo.toml`:
   ```toml
   whisper-rs = { version = "0.13", features = ["whisper-cpp-metal"] }
   ```
   (metal feature enables GPU acceleration on Apple Silicon)
2. Define `WhisperState`:
   ```rust
   pub struct WhisperState(pub Mutex<Option<WhisperContext>>);
   ```
3. `transcribe_audio(path: String, state: State<WhisperState>)` command:
   - Determine model path: `~/Brief/models/ggml-base.en.bin`
   - If `WhisperState` is `None`, load the model via `WhisperContext::new_with_params`
   - Read the WAV file with `hound`, convert samples to `Vec<f32>` (divide by 32768.0)
   - Run `ctx.create_state()?.full(params, &samples)`
   - Collect segment text, join with spaces, return
4. Merge `stop_recording` and `transcribe_audio` into a single `stop_and_transcribe` command so the frontend makes one IPC call instead of two.
5. Register `stop_and_transcribe` (and keep `start_recording`) in `lib.rs`.

**Note on build time:** The first `cargo build` after adding `whisper-rs` will compile whisper.cpp from source (~2–3 min). Requires `cmake` and `clang` (both present on macOS with Xcode CLI tools).

**Success criteria:**
- `invoke("stop_and_transcribe")` returns a non-empty string for a recorded sentence
- Accuracy is reasonable (base.en model understands clear English speech)

---

## Stage 4: Model Download UX

**Goal**: If `ggml-base.en.bin` is missing, the UI prompts the user to download it (~148MB) and shows progress. Recording is disabled until the model is present.

**Status**: Complete

**Files:**
- `src-tauri/src/commands/transcribe.rs` — add `check_whisper_model`, `download_whisper_model` commands
- `src/lib/audio.ts` (new) — typed wrappers for `start_recording`, `stop_and_transcribe`, `check_whisper_model`, `download_whisper_model`
- `src/components/editor/EditorToolbar.tsx` — show model-missing state, download progress

**Steps:**
1. `check_whisper_model` command: return `{ exists: bool, path: String }`.
2. `download_whisper_model` command: use `reqwest` to stream `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin` to `~/Brief/models/ggml-base.en.bin`. Emit Tauri events for progress (`tauri::Emitter`).
3. In `src/lib/audio.ts`, export `checkWhisperModel()`, `downloadWhisperModel()`, `startRecording()`, `stopAndTranscribe()` — all thin wrappers around `invoke()`.
4. In `EditorToolbar`, run `checkWhisperModel()` on mount. If missing, show a "Download model (148MB)" button instead of the Record button. Show a progress bar while downloading.

**Success criteria:**
- First run: "Download model" button appears; clicking it downloads the file with visible progress
- After download: Record button replaces the download button without a page reload

---

## Stage 5: End-to-End Wiring

**Goal**: Full flow works — Record → Stop → transcript appended to note → note saved.

**Status**: Complete

**Files:**
- `src/components/editor/Editor.tsx` — handle `record` event from toolbar; append transcript to content
- `src/components/editor/EditorToolbar.tsx` — wire `startRecording` / `stopAndTranscribe` IPC calls into the record handler

**Steps:**
1. In `EditorToolbar.tsx`, the record handler:
   - Set `recording = true`, call `startRecording()`
   - On Stop click: set `transcribing = true`, call `stopAndTranscribe()`
   - Dispatch result back to Editor via the event bus
2. In `Editor.tsx`, the record listener receives the transcript and:
   - Appends `\n\n{transcript}` to `content`
   - Updates `contentRef.current` (matches the pattern in the enhance handler)
   - Calls `save(newContent, titleRef.current)` immediately (no debounce)
3. Add error display for transcription failures (re-use the existing `error` state + red banner pattern in Editor).
4. Clean up the temp WAV file from `~/Brief/recordings/` after transcription completes (in the Rust `stop_and_transcribe` command).

**Success criteria** (all acceptance criteria from spec):
- [ ] Record button appears when a note is open
- [ ] Pressing Record starts capturing mic audio
- [ ] Pressing Stop triggers transcription
- [ ] Transcript is appended inline to note content (blank line separator, no heading)
- [ ] Note is saved immediately after transcript is inserted
- [ ] Works offline (no internet required after model download)

---

## Verification

1. **Happy path**: Open a note → click Record → speak for 10 seconds → click Stop → verify transcript appears in note and note is saved.
2. **Empty recording**: Start and immediately stop → verify graceful handling (no crash, note unchanged).
3. **No model**: Delete `~/Brief/models/ggml-base.en.bin` → restart app → verify "Download model" button appears.
4. **Permission denied**: Deny mic access in macOS System Settings → verify error message appears in the editor (red banner).
5. **Long recording**: Record for 60 seconds → verify transcription completes without timeout.
6. **Existing content preserved**: Note with existing text → record + transcribe → verify original content is intact above the `---` separator.
