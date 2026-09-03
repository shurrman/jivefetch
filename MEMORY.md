[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# JiveFetch project memory

Last updated: 2026-09-03

## Current state

- Version `0.2.0` is the active next preview using Tauri 2, React, Rust/SQLite,
  locally installed `yt-dlp`, and FFmpeg.
- A configurable Rust scheduler owns typed process plans, attempts, progress, aggregate
  speed allocation, and Pause/Resume/Stop/Remove with optimistic task revisions.
- Persistent non-secret settings cover concurrency (default 2, presets 1–10, validated
  custom values up to 64), aggregate speed, a yt-dlp browser-cookie selector, and an
  output directory that defaults to `Downloads/JiveFetch`.
- The UI polls the authoritative queue every five seconds, shows colored per-task
  progress/speed/ETA/sizes, exposes system/dark/light themes, and uses the new vivid icon.
- The UI and every Markdown document have EN/RU/Simplified Chinese variants. A clean
  first run defaults to EN; an explicit language choice is stored locally.
- The supervisor owns Unix process groups or Windows Job Objects, drains bounded
  output, and terminates only the tree it created.
- The regular Rust suite has 14 tests, including v2/v3 settings migrations, typed engine
  arguments, and unrelated-process ownership. The opt-in real-engine loopback smoke is
  the final local runtime gate for `v0.2.0`.
- The first remote native source CI passed on Windows, macOS, and Linux in GitHub
  Actions run `33779282453`.
- GitHub CLI is authenticated as `shurrman`; private repository
  `https://github.com/shurrman/jivefetch` is configured as `origin`. Candidate
  `v0.1.0` exposed missing explicit Linux/Windows bundle icons. Native CI run
  `33783089736` and Native Release run `33784331491` completed successfully for
  `v0.1.1`, which is published with eight macOS/Linux/Windows artifacts.
- Commit `a4792bb` passed Native CI run `33788132275` on macOS, Linux, and Windows.
  Native Release run `33789236621` then published the `v0.2.0` prerelease with eight
  verified DMG/AppImage/DEB/NSIS/MSI/checksum assets.
- Architecture and delivery contracts are documented under `docs/`.
- JiveFetch source is Apache-2.0. System engines are not redistributed.
- Linux inherits `glib 0.18.5` from Tauri/GTK. `GHSA-wrw7-89jp-8q8g` affects an API
  JiveFetch does not call; patched `glib 0.20` is incompatible with GTK `^0.18`.
  This is accepted only for the private preview and blocks a stable release.

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
- Browser-cookie mode stores only an allowlisted browser identifier and passes it as
  a typed `yt-dlp --cookies-from-browser` argument; cookie values are never copied.
  Future imported-cookie data must be encrypted with a random key held by the OS
  credential store and decrypted only into a restrictive short-lived file.
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

1. Add metadata probing, format choice, and ffprobe capability reporting.
2. Expand storage with artifacts/events, idempotency, retry policy, and pagination.
3. Add profile/keyring detail and encrypted cookie-file import without exposing values.
4. Add signing/notarization and verified managed engines before a stable release.
