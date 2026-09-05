[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# Changes

This file records validated project-level changes and before/after metrics.

## Unreleased

## 0.4.0 - 2026-09-05

### Added

- Added one task-wide progress model that combines planned video and audio component
  sizes, downloaded bytes, speed, and ETA, plus localized Video/Audio/Merging stage text.
- Added an Open action for completed tasks. Rust resolves the task ID, revalidates a
  non-empty canonical output inside the configured folder, and opens it with the OS
  default application; the webview never supplies a filesystem path.
- Added structured local JSON diagnostics with stable task/attempt/error fields, a 2 MiB
  bound, and one rotation while excluding raw engine lines, URLs, cookies, and paths.
- Added narrow engine and process-spawner traits with scheduler test doubles.

### Changed

- Replaced internal `Result<T, String>` flows with typed validation, storage, engine,
  scheduler, and input errors; stable localized strings now exist only at the Tauri IPC
  boundary. `AppSettings` owns its validation and browser cookies use a typed allowlist.
- Separated binary discovery from `yt-dlp` execution while preserving one scheduler and
  one RAII owner for each supervised process tree.
- Moved the local-control and engine-ready message into one compact header line with a
  green/red indicator; the URL, format picker, and Add to queue form remain in the card.
- Publish unsigned and unnotarized builds as GitHub pre-releases until the remaining
  production release gates are complete.

### Fixed

- A 100% video-only event followed by an audio-only download no longer makes a task look
  finished and then restarted. Component progress is aggregated monotonically, active
  work stays below 100%, and only verified completion reaches 100%.
- Completed tasks are green and offer Open only while their verified final file still
  exists; a removed or empty file is shown as an output error on the next queue refresh.
- Made the legacy-settings migration test use a native temporary absolute path, so the
  validation contract is exercised consistently on Windows, macOS, and Linux.

### Metrics

- SQLite schema: version 5 to 6 with a tested in-place `download_stage` migration.
- Regular Rust suite: 22 to 26 tests, plus the passing opt-in real-engine loopback smoke.
- Internal error boundary: string codes throughout Rust to 5 typed error families and one
  stable string mapping at Tauri IPC.
- Progress model: one replaceable per-component snapshot to one monotonic aggregate across
  planned components, with 4 localized stage labels in each application language.

## 0.3.1 - 2026-09-05

### Added

- Added a complete installation guide for macOS, Windows, Ubuntu/Debian, Fedora,
  Arch Linux, AppImage, and DEB in English, Russian, and Simplified Chinese. It covers
  installing and verifying `yt-dlp`, FFmpeg, and Deno, matching JiveFetch checksums, first
  launch, and engine updates.
- Made the cross-platform engine prerequisite and installation-guide link prominent at
  the beginning of every README translation.
- Added a language-neutral README hero that shows the user-facing download queue,
  progress states, media formats, and cross-device use without third-party branding.
- Reworked every README into a user-first sequence: what JiveFetch provides, followed
  by the requirements, platform packages, and installation links.
- Added the JiveFetch application version beside the `yt-dlp` and FFmpeg versions in
  the top status region.

### Fixed

- Release language navigation now targets the concrete tag page and section anchor,
  for example `/releases/tag/v0.3.0#russian`, instead of an anchor relative to the
  multi-release listing. This keeps language jumps functional and Assets accessible.
- Made the unsigned macOS installation guide version-independent so it no longer names
  an obsolete DMG or checksum after a newer release becomes Latest.
- Replaced generic `yt-dlp` failures with localized, actionable error classes for
  browser-cookie access, authentication, unavailable media, rate limits, formats,
  network failures, and filesystem permissions without persisting raw engine output.
- Documented the Full Disk Access and Keychain steps required when the current unsigned
  macOS build cannot read Chrome cookies, plus safe public-media and Firefox fallbacks.
- Moved engine version probes from the protected Downloads folder to the application's
  own data directory, preventing a macOS TCC startup race that could report Homebrew
  `yt-dlp` as missing while FFmpeg was found.
- Added standard package-manager directories and Deno's per-user directory to the
  sanitized downloader `PATH`. Finder-launched macOS builds can now expose Homebrew
  Deno to `yt-dlp`, preventing YouTube media requests from failing with HTTP 403 after
  metadata probing succeeds.
- Classified HTTP 403 separately with localized guidance to update `yt-dlp`, install
  Deno, or enable browser cookies for restricted media.

### Metrics

- Documentation: 48 to 54 tracked Markdown files and 16 to 18 validated
  EN/RU/Simplified Chinese sets.
- Combined README length: 389 to 93 lines; architecture and development details remain
  in the dedicated documents instead of the product entry page.
- Release navigation: 3 ambiguous listing-relative anchors to 3 explicit tag-page
  anchors, validated by the release-note checker.
- Header version indicators: 2 engine versions to 1 application plus 2 engine versions.
- User-facing engine failure classes: 1 generic fallback to 8 actionable localized
  classes plus the generic fallback; regular Rust tests: 19 to 22.

## 0.3.0 - 2026-09-03

### Added

- Added a live, cookie-aware `yt-dlp` JSON probe and per-task selection from actual
  source video formats. The picker shows resolution, FPS, source bitrate, codec,
  container, and estimated size, defaults to maximum quality, and hides internal IDs
  and duplicate-looking rows without an estimated size.
- Added a custom right-click task menu with Start, Stop, Pause, Copy URL, and Remove;
  unavailable state transitions remain visible but disabled.
- Persisted the selected validated format selector in SQLite schema version 5 and pass
  it to the engine only as a typed argument vector.

### Fixed

- Explicitly enabled `yt-dlp --progress` so final-path printing no longer suppresses
  current bytes, total size, speed, ETA, and progress events.
- Preserved non-secret URL query parameters in queue cards while redacting secret-like
  values, so distinct YouTube `/watch?v=…` tasks remain distinguishable.
- Required a verified non-empty output file before completion, persisted its actual
  size, and reconciled zero-byte metrics for existing completed tasks on startup.
- Waited for fast version-process output to drain after exit, preventing intermittent
  false `yt-dlp not found` results on a cold desktop launch.

### Metrics

- Before (`v0.2.0`): 140 repository files, 48 Markdown documents, 4,029 lines of
  application/check source, and 14 regular Rust tests plus the opt-in real-engine smoke.
- After: 141 repository files, 48 Markdown documents in 16 EN/RU/Simplified Chinese
  sets, 4,975 source lines, and 19 regular Rust tests plus the passing real-engine smoke.

## 0.2.0 - 2026-09-03

### Added

- Added persistent Rust/SQLite settings for 1–10 concurrency presets plus a validated
  custom value, an optional aggregate speed budget, browser-cookie authentication, and
  a native output-directory picker that defaults to `Downloads/JiveFetch`.
- Added system, dark, and light themes and a new high-contrast JiveFetch icon across
  desktop and generated platform icon sizes.
- Added a five-second authoritative queue refresh with blue active, green completed,
  and red failed/interrupted progress bars plus speed, ETA, and current/total size.
- Moved `yt-dlp` and FFmpeg versions to the header and removed the SQLite and engine
  implementation-detail cards and redundant default-language message.
- Added typed `--limit-rate` and `--cookies-from-browser` plan arguments with allowlist,
  migration, persistence, allocation, and argument-vector tests.
- Published `v0.1.1` as the first cross-platform prerelease with DMG, AppImage, DEB,
  NSIS EXE, MSI, and per-platform SHA-256 manifests.
- Published `v0.2.0` after Native CI and Native Release succeeded on macOS, Linux,
  and Windows, with eight installer/checksum assets.
- Moved local `AGENTS*` and `MEMORY*` context out of version control while preserving
  local working copies through explicit ignore rules.
- Repaired the published translation/documentation links for `v0.2.0` and made the
  release workflow render every relative note link as a tag-stable absolute URL.
- Removed the incomplete `v0.1.0` draft release, promoted `v0.2.0` to GitHub `Latest`
  without rebuilding its artifacts, and made future successful releases become
  `Latest` automatically. Signing and notarization remain separate production gates.
- Added checksum-first macOS installation instructions in English, Russian, and
  Simplified Chinese for locally ad-hoc signing and narrowly removing quarantine from
  the current unsigned application.
- Kept release downloads visible while switching language by embedding all three
  translations in one GitHub release description with in-page language navigation.

### Metrics

- Before: 135 repository files, 45 Markdown documents in 15 translation sets, 3,262
  application/check source lines, and 8 regular Rust tests.
- After: 135 repository files, 42 Markdown documents in 14 translation sets, 4,075
  application/check source lines, and 14 regular Rust tests plus the opt-in real-engine smoke.

## 0.1.1 - 2026-09-03

### Fixed

- Declared the complete PNG/ICNS/ICO bundle icon set so Linux AppImage and Windows
  MSI packaging can select their required square and `.ico` assets.
- Limited Native CI push runs to branches so a release tag does not duplicate the
  source-verification matrix already performed by Native Release.

## 0.1.0 - 2026-09-03

### Added

- Established the JiveFetch product baseline and project identity.
- Documented product requirements, architecture, persistent task lifecycle,
  authentication/security boundaries, process supervision, packaging, and roadmap.
- Added project-specific agent rules while retaining the pre-existing `AGENTS.md`
  content.
- Added repository memory for durable, secret-free continuation context.
- Added a working Tauri 2 + React/TypeScript + Rust foundation for macOS.
- Added a Rust-owned SQLite queue with URL validation, revisions, and working
  Pause/Resume/Stop/Remove state controls.
- Added complete English, Russian, and Simplified Chinese application dictionaries
  and documentation variants; English is the first-run default.
- Added pinned npm/Cargo lockfiles and an initial original JiveFetch icon asset.
- Added real `yt-dlp` execution with typed arguments, verified FFmpeg discovery,
  bounded progress parsing, and output-path verification.
- Added a Rust scheduler with two execution slots, transactional attempts, startup
  recovery, and portable Pause/Resume/Stop semantics.
- Added owned process-tree supervision using Unix process groups and Windows Job
  Objects, plus a cross-platform helper test that preserves an unrelated process.
- Added a local real-engine smoke test, high-confidence secret scan, native
  Windows/macOS/Linux CI, and a live progress UI in all three application languages.
- Selected Apache-2.0 for JiveFetch and documented the system-engine licensing gate.
- Created the private `shurrman/jivefetch` GitHub repository and configured it as
  `origin`.
- Added a tag-driven native release workflow for macOS DMG, Linux AppImage/deb, and
  Windows NSIS/MSI preview packages with per-platform SHA-256 checksums.
- Passed the first remote native CI matrix on macOS, Linux, and Windows: frontend,
  multilingual docs, secret scan, rustfmt, clippy, Rust tests, and desktop compilation.

### Baseline metrics

- Before: 2 pre-existing repository files, 1 generic Markdown file, 0 application
  source files, and 0 project-specific documents.
- After: 135 repository files, including 45 Markdown documents in 15 structurally
  aligned EN/RU/Simplified Chinese sets, 3,262 lines of application/check source,
  a runnable desktop build, 8 regular passing Rust tests, 1 passing opt-in real-engine
  smoke test, and persistent SQLite task/attempt state.

### Pre-existing issues

- The original `.gitignore` and generic `AGENTS.md` contain legacy Ansible-oriented
  rules that are preserved for compatibility but are not JiveFetch architecture.
- Installer signing identities, managed sidecar distribution, FFmpeg redistribution
  review, SBOM/provenance, and clean-machine release tests are not yet complete.
- Linux transitively uses `glib 0.18.5` through Tauri/GTK and remains flagged by
  `GHSA-wrw7-89jp-8q8g`; the affected API is unused, but the advisory blocks a fully
  supported production release.
