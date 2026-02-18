import { invoke } from "@tauri-apps/api/core";
import type { Note, NoteMeta, NoteSummary } from "@/types";

export async function listNotes(): Promise<NoteSummary[]> {
  return invoke("list_notes");
}

export async function readNote(id: string): Promise<Note> {
  return invoke("read_note", { id });
}

export async function writeNote(id: string, content: string, meta: NoteMeta): Promise<void> {
  return invoke("write_note", { id, content, meta });
}

export async function deleteNote(id: string): Promise<void> {
  return invoke("delete_note", { id });
}
