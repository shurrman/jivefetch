[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# JiveFetch project memory

Last updated: 2026-09-03

## Current state

- Version `0.1.0` is a working initial download preview using Tauri 2, React, Rust/SQLite,
  locally installed `yt-dlp`, and FFmpeg.
- A two-slot Rust scheduler owns typed process plans, attempts, progress, and
  Pause/Resume/Stop/Remove with optimistic task revisions.
- The UI and every Markdown document have EN/RU/Simplified Chinese variants. A clean
  first run defaults to EN; an explicit language choice is stored locally.
- The supervisor owns Unix process groups or Windows Job Objects, drains bounded
  output, and terminates only the tree it created.
- The real-engine loopback smoke test passes with yt-dlp `2026.07.04` and FFmpeg
  `8.1.2`; the unrelated-process supervisor test also passes on macOS.
- The first remote native source CI passed on Windows, macOS, and Linux in GitHub
  Actions run `33779282453`.
- GitHub CLI is authenticated as `shurrman`; private repository
  `https://github.com/shurrman/jivefetch` is configured as `origin`. Version `v0.1.0`
  is the first native release candidate for macOS, Linux, and Windows.
- Architecture and delivery contracts are documented under `docs/`.
- JiveFetch source is Apache-2.0. System engines are not redistributed.

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
- JiveFetch source and documentation use Apache-2.0. Bundled/managed engine
  distribution remains separately gated by exact binary license review.
- Tags matching the package version trigger native macOS/Linux/Windows preview builds
  and publish only after every matrix build uploads packages and checksums.
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

1. Observe the first Windows/macOS/Linux release workflow; fix any packaging failure
   before treating `v0.1.0` artifacts as verified.
2. Add metadata probing, format choice, and ffprobe capability reporting.
3. Expand storage with artifacts/events, idempotency, retry policy, and pagination.
4. Add credential-store-backed cookies and managed-engine verification before release.
