import { useState, useEffect } from "react";
import { Plus } from "lucide-react";
import { Sidebar } from "@/components/sidebar/Sidebar";
import { Editor } from "@/components/editor/Editor";
import { WelcomeScreen } from "@/components/editor/WelcomeScreen";
import { EditorToolbar } from "@/components/editor/EditorToolbar";
import { listNotes, writeNote, deleteNote } from "@/lib/notes";
import { generateNoteId, nowISO } from "@/lib/utils";
import type { NoteSummary } from "@/types";

export default function App() {
  const [notes, setNotes] = useState<NoteSummary[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    refreshNotes();
  }, []);

  async function refreshNotes() {
    try {
      const list = await listNotes();
      setNotes(list);
    } finally {
      setLoading(false);
    }
  }

  async function handleNewNote() {
    const id = generateNoteId();
    const now = nowISO();
    await writeNote(id, "", {
      title: "Untitled Meeting",
      participants: [],
      tags: [],
      created_at: now,
      updated_at: now,
    });
    await refreshNotes();
    setActiveId(id);
  }

  async function handleDeleteNote(id: string) {
    await deleteNote(id);
    if (activeId === id) setActiveId(null);
    await refreshNotes();
  }

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden">
      {/* Full-width titlebar — always draggable, traffic lights live here */}
      <div
        className="h-11 flex items-center shrink-0 border-b border-[hsl(var(--border))]"
        style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
      >
        {/* Traffic light spacer (80px matches macOS inset) */}
        <div className="w-20 shrink-0" />

        {/* Right side: editor toolbar or new-note button */}
        <div
          className="flex-1 flex items-center justify-end px-3"
          style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}
        >
          {activeId ? (
            <EditorToolbar noteId={activeId} />
          ) : (
            <button
              onClick={handleNewNote}
              className="p-1.5 rounded-md text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))] hover:bg-[hsl(var(--accent))] transition-colors"
              title="New Note (⌘N)"
            >
              <Plus size={16} />
            </button>
          )}
        </div>
      </div>

      {/* Body: sidebar + main */}
      <div className="flex flex-1 overflow-hidden">
        <Sidebar
          notes={notes}
          activeId={activeId}
          loading={loading}
          onSelect={setActiveId}
          onNew={handleNewNote}
          onDelete={handleDeleteNote}
        />
        <main className="flex-1 overflow-hidden">
          {activeId ? (
            <Editor key={activeId} noteId={activeId} onSave={refreshNotes} />
          ) : (
            <WelcomeScreen onNew={handleNewNote} />
          )}
        </main>
      </div>
    </div>
  );
}
