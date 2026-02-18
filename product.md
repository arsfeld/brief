# Granola Clone — Product Definition

## Vision

An AI-powered meeting notepad that transforms raw, messy notes into structured, actionable records. Everything runs locally-first, with optional sync. No accounts required to get started.

---

## Core Problem

During meetings, people either:
1. Type frantically and lose the thread of conversation
2. Pay attention and capture nothing useful
3. Record audio and never review it

After meetings, notes are messy, incomplete, and hard to act on. Action items get buried. Decisions go undocumented.

**Granola's job**: Make your notes useful with minimal effort.

---

## Who It's For

- **Primary**: Knowledge workers who attend many meetings (PMs, founders, engineers, consultants)
- **Secondary**: Students, researchers, anyone who needs structured notes from conversations

---

## Core Features

### 1. Meeting Notes Editor
- Minimal, distraction-free editor
- Markdown support with live preview toggle
- Timestamps you can insert manually or automatically
- Quick capture mode: just start typing

### 2. AI Enhancement
- **Polish**: Fix grammar, structure, add headers — keep your voice
- **Summarize**: TL;DR of the meeting in 3–5 bullets
- **Action Items**: Extract and list todos with owners and deadlines
- **Decisions**: Highlight key decisions made
- **Questions**: Surface open questions that were raised

The AI runs on your choice of provider (OpenAI, Anthropic, local via Ollama). You bring your own API key.

### 3. Meeting Context
- Set meeting title, participants, date/time before or after
- Optional: paste in a meeting agenda to give AI more context
- Optional: attach audio transcript (paste from Zoom/Meet/etc.)

### 4. Organization
- Notes stored as Markdown files on disk (like Obsidian)
- Folder-based organization: `~/Granola/` by default, configurable
- Tags with `#hashtag` syntax
- Search across all notes (full-text)

### 5. Optional Sync
- Sync via any folder sync tool (iCloud, Dropbox, Syncthing) — it's just files
- Optional self-hosted backend for teams: share notes, search across team's notes
- No vendor lock-in: your notes are always readable without the app

---

## What It Is NOT

- Not a meeting recorder or transcription tool (v1)
- Not a project management tool
- Not a calendar app
- Not a real-time collaboration tool (v1)
- Not a subscription SaaS (optional self-hosted backend only)

---

## User Flow (v1)

```
Open app
  → New meeting note (Cmd+N)
  → Set title / participants (optional, can do later)
  → Type raw notes during meeting
  → Hit "Enhance" when done (Cmd+E)
  → AI cleans up notes, extracts action items
  → Review / edit AI output
  → Save (auto-saves continuously)
  → Done
```

---

## Storage Model

```
~/Granola/
  2026-02-18-product-kickoff.md        ← note file
  2026-02-18-product-kickoff.meta.json ← participants, tags, meeting time
  2026-01-15-eng-standup.md
  ...
```

Markdown files are human-readable and portable. Meta files are small JSON sidecars. The app is just a better editor for these files.

---

## AI Model Strategy

The app ships with everything needed to run AI locally. No external dependencies, no accounts required.

### Local AI (Default, built-in)
- `llama-server` (from llama.cpp) bundled as a Tauri sidecar binary
- App manages the process lifecycle — start/stop automatically
- In-app model browser: download GGUF models (Llama 3, Mistral, Phi, etc.)
- Models stored in `~/.brief/models/`, configurable
- Runs entirely on-device; zero network calls for AI

### Cloud AI (Optional, bring your own key)
- OpenAI or Anthropic API key stored in macOS Keychain
- Faster, higher quality output for users who prefer it
- Keys never leave the device (calls go direct to API, not through any server)

### Model Management UI
- Built-in model browser with curated list of recommended GGUF models
- Download progress, disk usage, delete models
- Active model selector (swap without restarting)
- Quantization hint: recommend Q4_K_M as the default balance of speed/quality

### Default prompt templates, fully editable in settings
- No data sent anywhere except the chosen AI provider

---

## Tech Stack

```
App shell:     Tauri v2 (Rust core, WebView frontend)
Frontend:      React 19 + TypeScript + Vite
UI:            shadcn/ui + Tailwind CSS
               Tauri window controls (native traffic lights, vibrancy)
Editor:        CodeMirror 6 or TipTap (Markdown)
Local AI:      llama.cpp (llama-server binary, Tauri sidecar)
Cloud AI:      OpenAI / Anthropic SDK (optional, BYOK)
Storage:       Markdown + JSON files via Tauri FS plugin
Keychain:      Tauri stronghold plugin → macOS Keychain
Search:        tantivy (Rust full-text search, via Tauri command)
Build:         cargo (Rust) + pnpm (frontend)
```

### Architecture Diagram

```
┌─────────────────────────────────────────┐
│              Tauri App                  │
│  ┌─────────────────┐  ┌──────────────┐ │
│  │  React Frontend │  │  Rust Core   │ │
│  │  (WebView)      │◄─►│              │ │
│  │                 │  │ • File I/O   │ │
│  │ • Editor        │  │ • Search     │ │
│  │ • Model mgmt    │  │ • Keychain   │ │
│  │ • Settings      │  │ • AI router  │ │
│  └─────────────────┘  └──────┬───────┘ │
│                               │         │
│                    ┌──────────▼───────┐ │
│                    │  llama-server    │ │
│                    │  (sidecar)       │ │
│                    └──────────────────┘ │
└─────────────────────────────────────────┘
         │ optional
         ▼
   OpenAI / Anthropic API
```

---

## Self-Hosted Backend (Optional, v2)

A simple Go or Node server teams can run internally:
- Central search across all team notes
- Shared templates
- Note sharing via link
- No AI processing on the server — stays on client

---

## Success Metrics (MVP)

- A user can go from raw notes → polished output in under 60 seconds
- Notes are readable without the app (plain Markdown)
- App works fully offline including AI (local model bundled)
- Zero required accounts or sign-ups

---

## Non-Goals for v1

- Mobile app
- Audio recording / transcription
- Real-time multiplayer editing
- Calendar integration
- Plugin system

---

## Competitive Differentiation

| Feature | Granola (this) | Granola (real) | Notion AI | Obsidian |
|---|---|---|---|---|
| Local-first files | ✅ | ❌ | ❌ | ✅ |
| Built-in local AI (no setup) | ✅ | ❌ | ❌ | ❌ |
| Bring your own AI key | ✅ | ❌ | ❌ | Partial |
| Self-hostable | ✅ | ❌ | ❌ | ❌ |
| Meeting-specific UX | ✅ | ✅ | ❌ | ❌ |
| No account required | ✅ | ❌ | ❌ | ✅ |
| Works offline | ✅ | ❌ | ❌ | ✅ |
