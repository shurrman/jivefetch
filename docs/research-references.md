[English](research-references.md) | [Русский](research-references.ru.md) | [简体中文](research-references.zh-CN.md)

# Research references and differentiation

Last reviewed: 2026-09-01

## 1. Use policy

MediaHarbor and FlowGrab are public product/repository references, not upstreams.
JiveFetch may learn from their visible workflows and feature claims, but it does not
copy their source code, architecture, assets, interface text, branding, or docs.

Each borrowed product idea is rewritten as an independent requirement and implemented
from JiveFetch's own domain model. Claims found in another project's README are treated
as research leads, not proof of behavior; JiveFetch acceptance tests define its own
standard.

## 2. Sources

- [MediaHarbor repository](https://github.com/MediaHarbor/mediaharbor) — public
  cross-platform media downloader/player using Tauri, Rust, React, and multiple media
  tools. Its public README highlights unified search/download, quality selection,
  local library, dependency management, settings, credentials, and multi-platform
  packages.
- [FlowGrab Downloader repository](https://github.com/HrshD1eux/Flowgrab_Downloader)
  — public Tauri 2/React downloader around `yt-dlp` and FFmpeg. Its public README
  highlights clipboard capture, deep links, batch queueing, quality/audio choices,
  pause/resume/stop, history, progress, settings, and engine updates.
- [yt-dlp upstream](https://github.com/yt-dlp/yt-dlp) — authoritative engine options,
  formats, authentication, release files, updates, and binary licensing notes.
- [Tauri documentation](https://v2.tauri.app/) — application, capability, sidecar,
  updater, and packaging behavior.
- [FFmpeg legal page](https://ffmpeg.org/legal.html) and
  [aria2 repository](https://github.com/aria2/aria2) — redistribution and engine
  behavior references to review before release.

## 3. Idea-to-contract matrix

| Useful public idea | JiveFetch improvement |
| --- | --- |
| Paste a URL and preview metadata | Cancellable typed probe with stale-result suppression, normalized formats, expiry, engine-version marker, and revalidation before dispatch |
| Quality presets and detailed formats | Intent-based policy plus live audio/video format inventory, compatibility explanation, and explicit remux versus transcode |
| Clipboard capture and deep links | Opt-in intake, URL-only clipboard filtering, review screen, deduplication, and no auto-start from untrusted external input |
| Batch/playlist queue | Durable SQLite queue with bounded concurrency, priority aging, output-path locks, retries, and crash recovery |
| Live progress, speed, ETA, and history | Sequenced events, coalesced progress, durable checkpoints, attempt history, typed phases, and reconnect snapshots |
| Pause/Resume/Stop controls | Formally specified cross-platform state machine; Pause is durable resumable interruption, Stop verifies the owned process tree is gone |
| Process-tree cleanup | OS-native ownership container created at spawn, graceful deadline, forced owned-tree termination, and tests proving unrelated processes survive |
| Browser cookies and credentials UI | Browser/profile references without copied values; encrypted imported cookies using an OS-held random key and short-lived plaintext leases |
| Settings persistence | Versioned non-secret settings in SQLite, secret references only, validation/migration, and per-task overrides |
| Dependency setup and updates | Signed manifest, pinned compatibility, hash verification, immutable versions, health check, rollback, SBOM, and license inventory |
| Multiple simultaneous downloads | Scheduler-enforced slots plus independent post-processing pool, fairness, idempotent leases, and no duplicate live attempt |
| Speed configuration | Separate per-task cap and fair application-wide budget with resumable rebalance and measurable aggregate acceptance target |
| Cross-platform installers | Native process/security/package tests, signing/notarization, clean-machine smoke tests, and target-specific release gates |
| Friendly errors | Structured engine error taxonomy, redacted diagnostics, suggested recovery action, and support bundles that exclude secrets by construction |

## 4. Deliberate differentiators

### Durable queue, not a frontend list

The queue survives process death and is restored from SQLite. React never decides
that a task is running or finished by itself. Attempts, transitions, and artifacts
remain auditable.

### Honest controls

JiveFetch avoids promising an OS suspend as universal Pause. Its portable contract is
a controlled stop that keeps compatible partial data and a new resumable attempt.
Stop is complete only when the owned process tree is verified gone.

### Global resource policy

Concurrency, post-processing slots, output locks, retry timing, and aggregate
bandwidth are coordinated centrally instead of being independent process flags.

### Secrets outside ordinary state

Browser selection may be persisted; cookie values may not. Imported data is encrypted
using a random key held by the system credential store, and plaintext exists only for
a bounded child-process lease.

### Verifiable distribution

Engine convenience does not override supply-chain controls. Every activated binary is
known, hashed, compatible, attributable, and rollback-capable.

## 5. Ideas intentionally deferred

Media search across subscription services, playback, lyrics, a scanned local library,
browser extensions, scheduled subscriptions, cloud sync, and arbitrary expert flags
may be valuable later. They are excluded from the first architecture because they
would dilute the reliability and security work required for the core queue.

## 6. Review cadence

Revisit the reference products before a major UX milestone, but update this file only
with publicly verifiable observations and JiveFetch decisions. Never import code to
“study it quickly” into this repository or use reference implementation details as a
shortcut around JiveFetch tests.
