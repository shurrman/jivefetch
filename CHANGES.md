[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# Changes

This file records validated project-level changes and before/after metrics.

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

### Baseline metrics

- Before: 2 pre-existing repository files, 1 generic Markdown file, 0 application
  source files, and 0 project-specific documents.
- After: 132 repository files, including 42 Markdown documents in 14 structurally
  aligned EN/RU/Simplified Chinese sets, 3,262 lines of application/check source,
  a runnable desktop build, 8 regular passing Rust tests, 1 passing opt-in real-engine
  smoke test, and persistent SQLite task/attempt state.

### Pre-existing issues

- The original `.gitignore` and generic `AGENTS.md` contain legacy Ansible-oriented
  rules that are preserved for compatibility but are not JiveFetch architecture.
- Installer signing identities, managed sidecar distribution, FFmpeg redistribution
  review, SBOM/provenance, and clean-machine release tests are not yet complete.
- Native CI is defined but has not run because the local changes have not been
  committed or pushed to the GitHub remote.
