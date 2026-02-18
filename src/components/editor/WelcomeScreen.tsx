import { Plus } from "lucide-react";

export function WelcomeScreen({ onNew }: { onNew: () => void }) {
  return (
    <div
      className="h-full flex flex-col items-center justify-center text-center gap-4 titlebar-drag"
    >
      <div className="titlebar-no-drag flex flex-col items-center gap-4">
        <div className="w-16 h-16 rounded-2xl bg-[hsl(var(--primary))] flex items-center justify-center">
          <span className="text-2xl font-bold text-white">G</span>
        </div>
        <div>
          <h1 className="text-2xl font-semibold text-[hsl(var(--foreground))]">Brief</h1>
          <p className="text-sm text-[hsl(var(--muted-foreground))] mt-1">
            AI-powered meeting notes
          </p>
        </div>
        <button
          onClick={onNew}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[hsl(var(--primary))] text-white text-sm font-medium hover:opacity-90 transition-opacity"
        >
          <Plus size={16} />
          New Meeting Note
          <kbd className="ml-1 text-xs opacity-70 font-mono">⌘N</kbd>
        </button>
      </div>
    </div>
  );
}
