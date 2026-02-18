---
id: TASK-1
title: Listen to audio and transcribe it into a note
status: done
created: 2026-02-18
---

# TASK-1: Listen to audio and transcribe it into a note

## Problem
Manually typing out meeting notes from recordings is tedious and error-prone. Users have no way to capture spoken content directly — everything has to be transcribed by hand.

## Goal
The app records microphone (and optionally system) audio during a meeting session. When the user stops recording, the audio is transcribed and the transcript is appended to the current note — similar to how Granola works.

## Acceptance Criteria
- [x] A "Record" button is available when a note is open
- [x] Pressing Record begins capturing microphone audio
- [x] Pressing Stop ends the recording and triggers transcription
- [x] The transcript is appended inline to the note content (blank line separator)
- [x] The note is saved after the transcript is inserted
- [x] The feature works offline / locally (no cloud required)

## Out of Scope
- Speaker diarization (identifying who said what)
- Uploading or storing audio files in the cloud
- Transcribing pre-existing audio/video file imports (separate task)

## Notes
- Granola (https://www.granola.ai) is the reference UX — study their flow for inspiration
- Transcription engine TBD: options include Whisper (local via whisper.cpp), macOS SpeechRecognizer, or a cloud API
- Audio capture will need Tauri permissions (microphone access) and likely a new Rust command
