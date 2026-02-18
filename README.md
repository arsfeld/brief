# Brief

A local-first, AI-powered meeting notes app. Take raw notes during meetings, then let AI turn them into structured, actionable records — all without accounts or cloud lock-in.

Built with Tauri v2 (Rust) + React 19 (TypeScript).

## Features

- **Distraction-free editor** — minimal UI that stays out of your way
- **AI enhancement** — polish, summarize, extract action items, surface decisions
- **Local-first storage** — notes live as plain Markdown files in `~/Brief/`; readable without the app
- **Multiple AI providers** — local Llama (via llama-server), OpenAI, or Anthropic — bring your own key
- **No account required** — works fully offline with a local model

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) + [pnpm](https://pnpm.io/)
- [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/) for your platform

### Install & Run

```bash
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

## Storage Format

Notes are stored in `~/Brief/` as plain Markdown with JSON sidecar files:

```
~/Brief/
  2026-02-18-product-kickoff.md         ← note content
  2026-02-18-product-kickoff.meta.json  ← title, participants, tags, timestamps
```

## AI Providers

Configure your preferred provider in the toolbar:

| Provider | Setup |
|---|---|
| **Local** (default) | Runs `llama-server` at `localhost:8080` |
| **OpenAI** | Set `OPENAI_API_KEY` |
| **Anthropic** | Set `ANTHROPIC_API_KEY` |

## Tech Stack

```
App shell:   Tauri v2 (Rust core, WebView frontend)
Frontend:    React 19 + TypeScript + Vite + Tailwind CSS
Local AI:    llama.cpp (llama-server)
Cloud AI:    OpenAI / Anthropic (optional, BYOK)
Storage:     Markdown + JSON via Tauri FS plugin
```

## Development

```bash
pnpm dev          # Frontend only (Vite, port 1420)
pnpm build        # Type-check + build frontend
pnpm tauri dev    # Full app with hot reload
```

TypeScript strict mode (`noUnusedLocals`, `noUnusedParameters`) is the quality gate — `pnpm build` must pass cleanly.
