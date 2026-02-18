import { FileText, Trash2 } from "lucide-react";
import { cn, formatDate } from "@/lib/utils";
import type { NoteSummary } from "@/types";

interface SidebarProps {
  notes: NoteSummary[];
  activeId: string | null;
  loading: boolean;
  onSelect: (id: string) => void;
  onNew: () => void;
  onDelete: (id: string) => void;
}

export function Sidebar({ notes, activeId, loading, onSelect, onNew, onDelete }: SidebarProps) {
  return (
    <aside className="flex flex-col w-64 border-r border-[hsl(var(--sidebar-border))] bg-[hsl(var(--sidebar))] shrink-0">
      {/* Notes list */}
      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <div className="px-4 py-8 text-sm text-[hsl(var(--muted-foreground))] text-center">
            Loading…
          </div>
        ) : notes.length === 0 ? (
          <div className="px-4 py-8 text-sm text-[hsl(var(--muted-foreground))] text-center">
            No notes yet.
            <br />
            <button onClick={onNew} className="mt-2 underline hover:text-[hsl(var(--foreground))]">
              Create one
            </button>
          </div>
        ) : (
          <ul className="py-1">
            {notes.map((note) => (
              <NoteItem
                key={note.id}
                note={note}
                active={note.id === activeId}
                onSelect={() => onSelect(note.id)}
                onDelete={() => onDelete(note.id)}
              />
            ))}
          </ul>
        )}
      </div>
    </aside>
  );
}

function NoteItem({
  note,
  active,
  onSelect,
  onDelete,
}: {
  note: NoteSummary;
  active: boolean;
  onSelect: () => void;
  onDelete: () => void;
}) {
  return (
    <li
      className={cn(
        "group flex items-start gap-2 px-3 py-2.5 cursor-pointer rounded-md mx-1 transition-colors",
        active
          ? "bg-[hsl(var(--accent))] text-[hsl(var(--accent-foreground))]"
          : "hover:bg-[hsl(var(--accent))/60%] text-[hsl(var(--foreground))]",
      )}
      onClick={onSelect}
    >
      <FileText size={14} className="mt-0.5 shrink-0 text-[hsl(var(--muted-foreground))]" />
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium truncate">{note.title}</p>
        <p className="text-xs text-[hsl(var(--muted-foreground))] mt-0.5">
          {formatDate(note.updated_at)}
        </p>
        {note.preview && (
          <p className="text-xs text-[hsl(var(--muted-foreground))] mt-0.5 truncate opacity-70">
            {note.preview}
          </p>
        )}
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onDelete();
        }}
        className="opacity-0 group-hover:opacity-100 p-0.5 rounded text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--destructive))] transition-all shrink-0"
      >
        <Trash2 size={13} />
      </button>
    </li>
  );
}
