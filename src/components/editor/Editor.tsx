import { useState, useEffect, useRef, useCallback } from "react";
import { Loader2 } from "lucide-react";
import { readNote, writeNote } from "@/lib/notes";
import { enhanceNote } from "@/lib/ai";
import { startRecording, stopAndTranscribe } from "@/lib/audio";
import { nowISO } from "@/lib/utils";
import { registerEnhanceListener, registerRecordListener } from "@/components/editor/EditorToolbar";
import type { Note, EnhanceMode } from "@/types";

interface EditorProps {
  noteId: string;
  onSave: () => void;
}

export function Editor({ noteId, onSave }: EditorProps) {
  const [note, setNote] = useState<Note | null>(null);
  const [content, setContent] = useState("");
  const [title, setTitle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  // Keep refs so the enhance callback always sees latest values
  const contentRef = useRef(content);
  const titleRef = useRef(title);
  const noteRef = useRef(note);

  useEffect(() => { contentRef.current = content; }, [content]);
  useEffect(() => { titleRef.current = title; }, [title]);
  useEffect(() => { noteRef.current = note; }, [note]);

  useEffect(() => {
    readNote(noteId).then((n: Note) => {
      setNote(n);
      setContent(n.content);
      setTitle(n.meta.title);
    });
  }, [noteId]);

  const save = useCallback(
    async (newContent: string, newTitle: string) => {
      const currentNote = noteRef.current;
      if (!currentNote) return;
      await writeNote(noteId, newContent, {
        ...currentNote.meta,
        title: newTitle,
        updated_at: nowISO(),
      });
      onSave();
    },
    [noteId, onSave],
  );

  function scheduleSave(newContent: string, newTitle: string) {
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => save(newContent, newTitle), 800);
  }

  // Register with EditorToolbar's event bus
  useEffect(() => {
    const unregister = registerEnhanceListener(noteId, async (mode: EnhanceMode) => {
      if (!contentRef.current.trim()) return;
      setError(null);
      try {
        const result = await enhanceNote(contentRef.current, mode, "local");
        setContent(result);
        contentRef.current = result;
        save(result, titleRef.current);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    });
    return unregister;
  }, [noteId, save]);

  // Register recording listener
  useEffect(() => {
    const unregister = registerRecordListener(noteId, async (action) => {
      if (action === "start") {
        setError(null);
        await startRecording();
      } else {
        try {
          const transcript = await stopAndTranscribe();
          if (transcript) {
            const newContent = contentRef.current
              ? `${contentRef.current}\n\n${transcript}`
              : transcript;
            setContent(newContent);
            contentRef.current = newContent;
            save(newContent, titleRef.current);
          }
        } catch (e) {
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    });
    return unregister;
  }, [noteId, save]);

  function handleContentChange(e: React.ChangeEvent<HTMLTextAreaElement>) {
    setContent(e.target.value);
    scheduleSave(e.target.value, titleRef.current);
  }

  function handleTitleChange(e: React.ChangeEvent<HTMLInputElement>) {
    setTitle(e.target.value);
    scheduleSave(contentRef.current, e.target.value);
  }

  if (!note) {
    return (
      <div className="h-full flex items-center justify-center">
        <Loader2 size={20} className="animate-spin text-[hsl(var(--muted-foreground))]" />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Title */}
      <div className="px-8 pt-8 pb-2 shrink-0">
        <input
          value={title}
          onChange={handleTitleChange}
          placeholder="Meeting title…"
          className="selectable w-full text-xl font-semibold bg-transparent outline-none text-[hsl(var(--foreground))] placeholder:text-[hsl(var(--muted-foreground))]"
        />
      </div>

      {error && (
        <div className="mx-8 mb-2 px-3 py-2 rounded-lg bg-red-50 border border-red-200 text-red-700 text-xs">
          {error}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-8 py-2 pb-8">
        <textarea
          value={content}
          onChange={handleContentChange}
          placeholder="Start typing your meeting notes…"
          className="selectable w-full h-full min-h-full resize-none bg-transparent outline-none text-sm text-[hsl(var(--foreground))] placeholder:text-[hsl(var(--muted-foreground))] leading-relaxed"
        />
      </div>
    </div>
  );
}
