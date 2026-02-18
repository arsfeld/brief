import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface WhisperModelStatus {
  exists: boolean;
  path: string;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  percent: number;
}

export async function checkWhisperModel(): Promise<WhisperModelStatus> {
  return invoke("check_whisper_model");
}

export async function downloadWhisperModel(
  onProgress: (p: DownloadProgress) => void,
): Promise<void> {
  const unlisten = await listen<DownloadProgress>(
    "whisper-download-progress",
    (event) => onProgress(event.payload),
  );
  try {
    await invoke("download_whisper_model");
  } finally {
    unlisten();
  }
}

export async function startRecording(): Promise<void> {
  return invoke("start_recording");
}

export async function stopAndTranscribe(): Promise<string> {
  return invoke("stop_and_transcribe");
}
