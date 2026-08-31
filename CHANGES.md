[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# Changes

This file records validated project-level changes and before/after metrics.

## Unreleased

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

### Baseline metrics

- Before: 2 pre-existing repository files, 1 generic Markdown file, 0 application
  source files, and 0 project-specific documents.
- After: 113 versioned files, including 36 Markdown documents in 12 structurally
  aligned EN/RU/Simplified Chinese sets, 1,341 lines of application/check source,
  a runnable desktop bundle, 4 passing Rust tests, and persistent SQLite state.

### Pre-existing issues

- The repository has no configured remote; the first commit remains local only.
- The original `.gitignore` and generic `AGENTS.md` contain legacy Ansible-oriented
  rules that are preserved for compatibility but are not JiveFetch architecture.
- Product license and installer signing identities are not yet selected.
- `yt-dlp`/FFmpeg execution, process supervision, real downloads, and native CI are
  not implemented yet.
