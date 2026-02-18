export interface NoteMeta {
  title: string;
  participants: string[];
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface Note {
  id: string;
  content: string;
  meta: NoteMeta;
}

export interface NoteSummary {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  tags: string[];
  preview: string;
}

export type AIProvider = "local" | "openai" | "anthropic";
export type EnhanceMode = "polish" | "summarize" | "action_items" | "decisions";

export interface AISettings {
  provider: AIProvider;
  openaiKey?: string;
  anthropicKey?: string;
  localModel?: string;
}
