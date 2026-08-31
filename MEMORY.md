[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# JiveFetch project memory

Last updated: 2026-09-01

## Current state

- Version `0.0.1` is a working macOS foundation using Tauri 2, React, and Rust/SQLite.
- Validated HTTP(S) URLs can be persisted in the queue; Pause/Resume/Stop/Remove use
  optimistic concurrency through a task revision.
- The UI and every Markdown document have EN/RU/Simplified Chinese variants. A clean
  first run defaults to EN; an explicit language choice is stored locally.
- `yt-dlp` and FFmpeg exist on the development Mac but are not invoked by the app yet.
- Work is local only. No Git remote is configured, so do not push.
- Architecture and delivery contracts are documented under `docs/`.
- The next step is the fake-engine/process-supervision slice, then real probing.

## Durable decisions

- Product: **JiveFetch**; application ID `top.jivejournal.jivefetch`; intended site
  `fetch.jivejournal.top`.
- Stack: Tauri 2 + Rust + React/TypeScript.
- Engines: managed `yt-dlp`, `ffmpeg`/`ffprobe`, with optional `aria2` integration.
- SQLite in WAL mode is the durable source of truth for task definitions, attempts,
  normalized format snapshots, events, and artifacts.
- A single Rust scheduler owns queue leases, concurrency, retry policy, bandwidth
  allocation, and every durable task-state transition.
- React is a projection. It rebuilds from a backend snapshot plus ordered events and
  never owns authoritative task state.
- Cross-platform Pause means a resumable controlled stop followed by a new process
  attempt. OS process suspension is not the portable contract.
- JiveFetch terminates only process trees it created: process groups/sessions on
  Unix and Job Objects on Windows. Never kill by executable name.
- Browser-cookie mode stores only browser/profile selection. Imported cookie data
  is encrypted at rest with a random key held by the OS credential store and is
  decrypted into a restrictive temporary file only while a child process needs it.
- Format choices are probed dynamically and compiled by Rust from typed user intent;
  the UI must not pass arbitrary shell command strings.
- Global and per-task speed limits are separate policies. The scheduler computes an
  effective per-task cap and may perform a resumable restart to rebalance active
  tasks when limits change.
- MediaHarbor and FlowGrab are research references only. Do not copy code, assets,
  UI text, or internal structure.
- Licensing remains open pending a sidecar and distribution license review.
- Documentation and user-facing strings are complete only when EN/RU/Simplified
  Chinese are updated together and mutually linked; EN is the application default.

## Safety invariants

- Never persist credentials, raw cookies, authorization headers, or decrypted
  secret material in SQLite, logs, crash reports, fixtures, or Git.
- Persist intent before spawning work; after an unclean shutdown, transient states
  become `interrupted` and are reconciled before the scheduler starts.
- Never infer ownership from PID alone and never adopt an unknown surviving process.
- Removing a task and deleting its downloaded files are separate explicit choices.
- Do not claim cross-platform support until native packaging and process-tree tests
  pass on Windows, macOS, and Linux.
- Do not mark a document change complete without all three translations.

## Next actions

1. Implement a fake engine and process-tree helper.
2. Expand the first SQLite schema into tasks/attempts/artifacts/events migrations.
3. Add a real `yt-dlp` probe and single-download vertical slice.
4. Decide license and sidecar acquisition policy before publishing installers.
