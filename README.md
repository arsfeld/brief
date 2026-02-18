> ⚠️ **Experimental / Vibe-coded** — This project is a work in progress built through AI-assisted exploration. Expect rough edges, incomplete features, and breaking changes.

# Brief

A local-first, AI-powered meeting notes app. Take raw notes during meetings, then let AI turn them into structured, actionable records.

Built with Tauri v2 (Rust) + React 19 (TypeScript).

## What works

- **Note editor** — title + content textarea with 800ms auto-save
- **Sidebar** — lists all notes sorted by last modified, with delete
- **AI enhancement** — four modes: Polish, Summarize, Action items, Decisions (requires a local [llama-server](https://github.com/ggml-org/llama.cpp) running at `localhost:8080`)
- **Audio recording + transcription** — record mic + system audio, transcribe via Whisper; model (~148 MB) is downloaded on first use

## What's not yet implemented

- UI for selecting AI provider or entering API keys (OpenAI/Anthropic backends exist but are unreachable from the UI)
- Editing participants and tags
- Settings screen

## Storage

Notes are stored as plain files in `~/Brief/`:

```
~/Brief/
  2026-02-18-abc12.md         ← note content (Markdown)
  2026-02-18-abc12.meta.json  ← title, participants, tags, timestamps
```

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) + [pnpm](https://pnpm.io/)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform
- A local llama-server on `localhost:8080` if you want AI enhancement

### Run

```bash
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

## Development

```bash
pnpm dev    # frontend only (Vite, port 1420)
pnpm build  # type-check + build frontend
```

TypeScript strict mode (`noUnusedLocals`, `noUnusedParameters`) is the quality gate — `pnpm build` must pass cleanly.
