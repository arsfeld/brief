import { useState, useEffect } from "react";
import { Sparkles, Loader2, ChevronDown, Mic, Square, Download } from "lucide-react";
import { checkWhisperModel, downloadWhisperModel } from "@/lib/audio";
import type { EnhanceMode } from "@/types";

const ENHANCE_MODES: { value: EnhanceMode; label: string }[] = [
  { value: "polish", label: "Polish notes" },
  { value: "summarize", label: "Summarize" },
  { value: "action_items", label: "Action items" },
  { value: "decisions", label: "Decisions" },
];

// Simple event bus so EditorToolbar can trigger actions in Editor
// without prop drilling through App.
const listeners = new Map<string, (mode: EnhanceMode) => void>();

export function registerEnhanceListener(noteId: string, fn: (mode: EnhanceMode) => void) {
  listeners.set(noteId, fn);
  return () => { listeners.delete(noteId); };
}

// Event bus for record actions. "stop" returns a promise so the toolbar
// can show the transcribing spinner until the Editor finishes.
type RecordAction = "start" | "stop";
const recordListeners = new Map<string, (action: RecordAction) => Promise<void>>();

export function registerRecordListener(noteId: string, fn: (action: RecordAction) => Promise<void>) {
  recordListeners.set(noteId, fn);
  return () => { recordListeners.delete(noteId); };
}

type ModelStatus = "checking" | "missing" | "downloading" | "ready";

export function EditorToolbar({ noteId }: { noteId: string }) {
  const [enhancing, setEnhancing] = useState(false);
  const [enhanceMode, setEnhanceMode] = useState<EnhanceMode>("polish");
  const [showModeMenu, setShowModeMenu] = useState(false);
  const [recording, setRecording] = useState(false);
  const [transcribing, setTranscribing] = useState(false);
  const [modelStatus, setModelStatus] = useState<ModelStatus>("checking");
  const [downloadPercent, setDownloadPercent] = useState(0);

  useEffect(() => {
    checkWhisperModel()
      .then(({ exists }) => setModelStatus(exists ? "ready" : "missing"))
      .catch(() => setModelStatus("missing"));
  }, []);

  async function handleEnhance() {
    const listener = listeners.get(noteId);
    if (!listener) return;
    setEnhancing(true);
    try {
      await new Promise<void>((resolve) => {
        listener(enhanceMode);
        // Editor signals completion by calling back; we just wait briefly
        setTimeout(resolve, 100);
      });
    } finally {
      setEnhancing(false);
    }
  }

  async function handleDownloadModel() {
    setModelStatus("downloading");
    setDownloadPercent(0);
    try {
      await downloadWhisperModel((p) => setDownloadPercent(p.percent));
      setModelStatus("ready");
    } catch {
      setModelStatus("missing");
    }
  }

  async function handleRecordToggle() {
    const listener = recordListeners.get(noteId);
    if (!listener) return;

    if (!recording) {
      setRecording(true);
      listener("start"); // fire-and-forget
    } else {
      setRecording(false);
      setTranscribing(true);
      try {
        await listener("stop");
      } finally {
        setTranscribing(false);
      }
    }
  }

  function renderRecordSection() {
    if (modelStatus === "checking") {
      return (
        <button disabled className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[hsl(var(--muted))] text-[hsl(var(--muted-foreground))] text-xs font-medium opacity-50">
          <Loader2 size={13} className="animate-spin" />
          Checking…
        </button>
      );
    }

    if (modelStatus === "missing") {
      return (
        <button
          onClick={handleDownloadModel}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[hsl(var(--muted))] text-[hsl(var(--foreground))] text-xs font-medium hover:opacity-90 transition-opacity"
        >
          <Download size={13} />
          Download model (148 MB)
        </button>
      );
    }

    if (modelStatus === "downloading") {
      return (
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[hsl(var(--muted))] text-[hsl(var(--foreground))] text-xs font-medium">
            <Loader2 size={13} className="animate-spin" />
            Downloading… {downloadPercent}%
          </div>
          <div className="w-24 h-1.5 rounded-full bg-[hsl(var(--border))] overflow-hidden">
            <div
              className="h-full bg-[hsl(var(--primary))] transition-all duration-200"
              style={{ width: `${downloadPercent}%` }}
            />
          </div>
        </div>
      );
    }

    // modelStatus === "ready"
    return (
      <button
        onClick={handleRecordToggle}
        disabled={transcribing}
        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium hover:opacity-90 disabled:opacity-50 transition-opacity ${
          recording
            ? "bg-red-500 text-white"
            : "bg-[hsl(var(--muted))] text-[hsl(var(--foreground))]"
        }`}
      >
        {transcribing ? (
          <>
            <Loader2 size={13} className="animate-spin" />
            Transcribing…
          </>
        ) : recording ? (
          <>
            <Square size={13} />
            Stop
          </>
        ) : (
          <>
            <Mic size={13} />
            Record
          </>
        )}
      </button>
    );
  }

  return (
    <div className="flex items-center gap-2">
      {renderRecordSection()}

      <button
        onClick={handleEnhance}
        disabled={enhancing}
        className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[hsl(var(--primary))] text-white text-xs font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
      >
        {enhancing ? <Loader2 size={13} className="animate-spin" /> : <Sparkles size={13} />}
        {ENHANCE_MODES.find((m) => m.value === enhanceMode)?.label}
      </button>

      <div className="relative">
        <button
          onClick={() => setShowModeMenu((v) => !v)}
          className="p-1.5 rounded-lg hover:bg-[hsl(var(--accent))] text-[hsl(var(--muted-foreground))] transition-colors"
        >
          <ChevronDown size={13} />
        </button>

        {showModeMenu && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setShowModeMenu(false)} />
            <div className="absolute right-0 top-full mt-1 w-40 rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--background))] shadow-lg z-50 py-1">
              {ENHANCE_MODES.map((mode) => (
                <button
                  key={mode.value}
                  onClick={() => {
                    setEnhanceMode(mode.value);
                    setShowModeMenu(false);
                  }}
                  className="w-full text-left px-3 py-1.5 text-xs hover:bg-[hsl(var(--accent))] text-[hsl(var(--foreground))] transition-colors"
                >
                  {mode.label}
                </button>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
