[English](roadmap.md) | [Русский](roadmap.ru.md) | [简体中文](roadmap.zh-CN.md)

# Roadmap

The roadmap is gated by evidence, not calendar dates. A phase is complete only when
its exit criteria pass on the stated target environment. Later UI polish cannot waive
an earlier durability, security, or process-ownership gate.

## Phase 0 — Reproducible project foundation

### Deliverables

- Create Tauri 2 + React + TypeScript scaffold and Rust workspace crates described in
  [Architecture](architecture.md).
- Pin Rust, Node, package manager, and Tauri CLI versions; commit lockfiles.
- Add formatting, linting, unit-test, Markdown-link, secret-scan, and `git diff
  --check` workflows.
- Add native CI skeleton for Windows, macOS, and Linux.
- Decide initial OS/architecture support, SQLite library, typed IPC approach, and
  product license review owner.
- Add ADR template and record the selected choices.

Current foundation status: the macOS Tauri shell, React production build, Rust/SQLite
queue, lockfiles, unit tests, multilingual UI/docs, Apache-2.0, and a native CI matrix
are present. The first remote matrix passed on macOS, Linux, and Windows; ADRs and the
planned Rust crate split remain before Phase 0 is fully closed.

### Exit criteria

- A clean checkout builds the empty application and runs tests on all initial native
  targets.
- No undocumented generated or secret file is tracked.
- README contains verified development commands.

## Phase 1 — Domain model and durable storage

### Deliverables

- Implement task, attempt, artifact, error, and state-transition types in pure Rust.
- Add first SQLite migrations, repositories, event sequence, idempotency keys, and
  queue revisions.
- Build state-machine and migration test suites.
- Add snapshot/pagination query service and a minimal read-only queue UI.

### Exit criteria

- Illegal transitions fail in property/table tests.
- Duplicate commands return the prior result without duplicate rows.
- A database fixture migrates forward and preserves queue intent.
- 10,000 historical tasks can be paged without loading all rows into the UI.

## Phase 2 — Fake engine and process ownership

### Deliverables

- Implement platform process supervisor and deterministic helper executable that
  spawns children/grandchildren and emits progress.
- Add graceful/forced stop deadlines, bounded output drains, and ownership verification.
- Implement scheduler slots, priorities, aging, output-path locks, and fake-engine
  attempts.
- Implement Pause, Resume, Stop, Retry, and Remove against fake work.

Current slice status: the owned process-group/Job-Object supervisor, bounded output,
two scheduler slots, durable controls, and an unrelated-process helper test are
implemented. Priority, aging, output locks, and the full crash-boundary matrix remain.

### Exit criteria

- Native tests terminate the complete owned tree on every OS.
- An unrelated helper with the same process name remains alive.
- Concurrent/raced commands converge to one stable state and no duplicate attempt.
- App termination at every transition boundary recovers without queue corruption.

## Phase 3 — yt-dlp probing and single-download vertical slice

### Deliverables

- Add verified engine registry and baseline sidecar discovery.
- Implement `yt-dlp` probe adapter, normalized metadata/formats, stale-probe policy,
  typed format intent compiler, and fixture-tested parsers.
- Build intake, metadata preview, format picker, destination choice, and one real
  queued download with progress.
- Detect `ffmpeg`/`ffprobe` capabilities and show actionable dependency errors.

Current slice status: system `yt-dlp`/FFmpeg discovery, a typed real download plan,
progress parsing, output verification, UI progress, and a loopback real-engine smoke
test are implemented. Metadata probing, format selection, and ffprobe capability
reporting remain.

### Exit criteria

- A sanitized fixture suite covers single media, playlists, audio-only, separated
  streams, subtitles, unavailable formats, and malformed output.
- A clean-machine smoke test completes one allowed public test download and verifies
  its final artifact.
- No shell string is assembled from user input.

## Phase 4 — Persistent parallel queue and recovery

### Deliverables

- Enable bounded parallel network tasks and separate post-processing capacity.
- Persist checkpoints, attempts, retry deadlines, and artifact fingerprints.
- Implement full startup recovery/reconciliation and optional eligible auto-resume.
- Add batch/playlist review, reordering, priorities, per-task overrides, and history.
- Add disk-space preflight and collision policy.

### Exit criteria

- Hard-kill tests during probe, download, post-processing, pause, stop, and completion
  reconcile correctly after restart.
- No task is dispatched twice and no conflicting output paths run together.
- One failed task does not block unrelated queue progress.

## Phase 5 — Authentication and bandwidth

### Deliverables

- Add browser/profile authentication references.
- Add encrypted imported-cookie storage using the OS credential service and
  short-lived temp leases.
- Implement per-task and application-wide rate policies with fair allocation,
  debounced resumable rebalance, and settings UI.
- Add sentinel-secret redaction/support-bundle tests.

### Exit criteria

- Authenticated probe/download works on each target using test credentials that are
  never committed.
- Database, logs, UI state, crash artifacts, and support bundle contain no synthetic
  sentinel secret.
- Aggregate sustained traffic respects the global ceiling acceptance target while
  per-task ceilings remain enforced.
- Limit changes do not discard compatible partial data.

## Phase 6 — Product-grade desktop UX

### Deliverables

- Complete accessible queue/task detail/history/settings views.
- Add opt-in clipboard URL capture, deep-link review, notifications, and Show in Folder.
- Add clear phase-specific progress, recovery prompts, error actions, and file-retention
  confirmation.
- Add list virtualization, thumbnail constraints/cache, keyboard paths, and theme.

### Exit criteria

- Keyboard-only and screen-reader smoke paths cover intake and all task controls.
- Deep links and clipboard data never auto-start a download.
- A 10,000-item history and active progress updates remain responsive.

## Phase 7 — Verified engine management and packaging

### Deliverables

- Implement signed engine manifests, downloads, hash checks, immutable installation,
  health checks, activation, and rollback.
- Finalize sidecar/license inventory, notices, SBOM, and provenance.
- Build signed Windows installers, signed/notarized macOS artifacts, and selected Linux
  packages.
- Add install/update/uninstall and clean-machine smoke tests.

### Exit criteria

- Tampered or incompatible tools never activate.
- Failed engine update rolls back to the known-good version.
- Every claimed target passes the gates in [Packaging](packaging.md).
- Exact application and engine licenses/versions are visible to the user.

## Phase 8 — Public MVP hardening

### Deliverables

- Run long-duration queue, flaky-network, low-disk, sleep/wake, and large-playlist tests.
- Fuzz parsers, path handling, redaction, and command payloads.
- Review Tauri capabilities, updater, credential lifecycle, and process ownership.
- Freeze schemas/contracts, write user documentation, troubleshooting, and release
  notes.

### Exit criteria

- All scenarios in [Product requirements](product-requirements.md) pass.
- No unresolved critical/high security or data-loss issue remains.
- Recovery metrics and process-tree tests show no orphaned owned process.
- Release artifacts, hashes, signatures, SBOM, notices, and provenance are published
  together.

## Post-MVP candidates

- Browser extension after deep-link security is mature.
- Local media library and player.
- Scheduled channels/subscriptions and download archive policies.
- Plugin/engine API with signed capability manifests.
- Remote control or synchronization with a separate threat model.
- Local transport broker for tighter dynamic aggregate bandwidth control.
- Advanced arguments through a constrained, auditable policy language rather than raw
  shell text.

## Definition of done for every phase

- Requirements and docs match behavior.
- Relevant unit/integration/native tests pass.
- Failure and restart paths are tested, not only the happy path.
- Logs/fixtures contain no secrets.
- `git diff --check` and relative Markdown-link validation pass.
- `CHANGES.md` records before/after metrics and known pre-existing issues.
