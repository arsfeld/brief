# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

**Brief** is a local-first, AI-powered meeting notes desktop app built with Tauri v2 (Rust backend) + React 19 (TypeScript frontend). Notes are stored as plain Markdown files in `~/Brief/` with accompanying `.meta.json` sidecar files.

## Commands

```bash
# Install dependencies
pnpm install

# Run in development (starts both Vite + Tauri)
pnpm tauri dev

# Build production app
pnpm tauri build

# Frontend-only dev server (port 1420)
pnpm dev

# Type check + frontend build
pnpm build
```

There is no test framework configured. TypeScript strict mode (`noUnusedLocals`, `noUnusedParameters`) serves as a quality gate — `pnpm build` must pass cleanly.

## Architecture

### IPC Boundary

The app is split cleanly at the Tauri IPC boundary:

- **Frontend** (`src/`) — React UI, never touches the filesystem directly
- **Backend** (`src-tauri/src/commands/`) — All file I/O and HTTP calls to AI providers

Frontend calls backend via `invoke()` from `@tauri-apps/api/core`. The five commands are declared in `src-tauri/src/lib.rs` and implemented in:
- `src-tauri/src/commands/notes.rs` — CRUD for notes in `~/Brief/`
- `src-tauri/src/commands/ai.rs` — Routes enhancement requests to local Llama, OpenAI, or Anthropic

### Frontend Structure

- `src/lib/notes.ts` — Typed wrappers around the five Tauri IPC calls
- `src/lib/ai.ts` — `enhanceNote()` wrapper for the AI command
- `src/types/index.ts` — Shared types: `Note`, `NoteMeta`, `NoteSummary`, `EnhanceMode`, `AIProvider`
- `src/components/editor/Editor.tsx` — Main editor (title + content textarea, 800ms auto-save debounce)
- `src/components/editor/EditorToolbar.tsx` — AI enhancement controls; communicates with Editor via event bus (not props)
- `src/components/sidebar/Sidebar.tsx` — Note list sorted by `updated_at`

### Storage Format

Each note consists of two files in `~/Brief/`:
- `{id}.md` — Raw markdown content
- `{id}.meta.json` — `NoteMeta` (title, participants, tags, created_at, updated_at)

Note IDs are generated as `YYYY-MM-DD-{random}` by `generateNoteId()` in `src/lib/utils.ts`.

### AI Providers

The `enhance_note` Rust command accepts a provider field:
- `"local"` → POST to `http://localhost:8080/completion` (llama-server)
- `"openai"` → OpenAI Chat API (default: gpt-4o-mini)
- `"anthropic"` → Anthropic API (default: claude-haiku-4-5-20251001)

### Key Patterns

- **Event bus** is used for Editor ↔ EditorToolbar communication to avoid prop drilling through `App.tsx`
- **Refs** maintain state consistency during async save/enhance operations
- **CSS custom properties (HSL)** drive the theme; dark mode uses `.dark` class
- The `cn()` utility in `src/lib/utils.ts` merges Tailwind classes (clsx + tailwind-merge)

## Path Alias

`@/*` maps to `src/*` in both TypeScript (`tsconfig.json`) and Vite (`vite.config.ts`).
