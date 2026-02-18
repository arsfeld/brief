import { invoke } from "@tauri-apps/api/core";
import type { AIProvider, EnhanceMode } from "@/types";

interface EnhanceRequest {
  content: string;
  mode: EnhanceMode;
  provider: AIProvider;
  api_key?: string;
  model?: string;
}

interface EnhanceResponse {
  result: string;
}

export async function enhanceNote(
  content: string,
  mode: EnhanceMode,
  provider: AIProvider,
  apiKey?: string,
  model?: string,
): Promise<string> {
  const req: EnhanceRequest = {
    content,
    mode,
    provider,
    api_key: apiKey,
    model,
  };
  const resp: EnhanceResponse = await invoke("enhance_note", { request: req });
  return resp.result;
}
