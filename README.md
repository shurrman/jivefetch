[English](README.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md)

# JiveFetch

> Your media. Your queue. Your rules.

JiveFetch is a local-first cross-platform desktop manager for lawful media
downloads. It combines a Tauri 2 shell, a Rust scheduling and process-control
core, and a React/TypeScript interface with `yt-dlp`, `ffmpeg`/`ffprobe`, and
optionally `aria2` as managed external engines.

Version `0.2.0` is a working settings and live-queue preview. It provides a Tauri 2 desktop
shell, a React interface in English, Russian, and Simplified Chinese, and a Rust-owned
SQLite queue. Users can add validated HTTP(S) URLs, download through locally installed
`yt-dlp` and FFmpeg, select 1–10 or a custom bounded concurrency, apply a global speed
budget, choose the output folder, switch system/light/dark themes, and see live progress,
speed, ETA, and sizes. Interrupted work is reconciled on restart and can resume from
partial files. A browser dropdown can pass one of yt-dlp's supported browser identifiers
to `--cookies-from-browser`; JiveFetch never copies cookie values. The default output
remains the platform Downloads directory plus `JiveFetch`.

## Product goals

- Run multiple downloads through a bounded, fair, persistent queue.
- Restore queue state after a crash, reboot, or application upgrade without
  duplicating active work.
- Probe current source formats and let the user choose quality, codecs, container,
  subtitles, and audio-only modes from real metadata.
- Support browser authentication and imported cookies without storing secrets in
  plain text or leaking them into logs and command previews.
- Apply both an application-wide bandwidth ceiling and per-task limits.
- Provide Pause, Resume, Stop, Retry, and Remove for each task with explicit file
  retention semantics.
- Own and terminate complete `yt-dlp`/`ffmpeg`/`aria2` process trees safely on
  Windows, macOS, and Linux.
- Ship reproducible, signed installers with pinned and verifiable sidecar tools.

## Non-goals

- DRM, paywall, or access-control bypass.
- Cloud execution, hosted download relays, or telemetry in the initial product.
- A general-purpose shell around arbitrary `yt-dlp` arguments in the MVP.
- Copying or forking MediaHarbor, FlowGrab, or their UI/code. They are research
  references only.

Users remain responsible for permissions, copyright, and service terms for the
content they download.

## Architecture at a glance

```text
React UI
   │ typed Tauri commands + versioned events
   ▼
Tauri adapter
   ▼
Rust application services ──► persistent scheduler ──► process supervisor
          │                         │                         │
          ▼                         ▼                         ▼
    format policy              SQLite/WAL          yt-dlp / ffmpeg / aria2
          │                                                   │
          └──────────── secure credential broker ◄────────────┘
```

SQLite is the durable source of truth. The React store is a projection rebuilt
from a snapshot plus ordered backend events. Only the Rust scheduler may reserve
queue slots or perform task-state transitions.

See [Architecture](docs/architecture.md) and
[Task lifecycle](docs/task-lifecycle.md) for the complete model.

## Repository layout

```text
.
├── src/                         # React/TypeScript UI
├── src-tauri/                   # Tauri shell and Rust/SQLite queue commands
├── .github/workflows/           # native Windows/macOS/Linux source verification
├── docs/                        # requirements, architecture and roadmap
├── package.json                 # pinned frontend/Tauri CLI dependencies
└── README.*.md                  # localized project entry points
```

The first vertical slice keeps the scheduler, storage, engine adapter, and process
supervisor as explicit Rust modules in `src-tauri`; the target crate boundaries remain
documented in [Architecture](docs/architecture.md).

## Documentation

- [Product requirements](docs/product-requirements.md)
- [Architecture](docs/architecture.md)
- [Task lifecycle and recovery](docs/task-lifecycle.md)
- [Security and authentication](docs/security.md)
- [Cross-platform packaging](docs/packaging.md)
- [Install an unsigned macOS release](docs/macos-installation.md)
- [Licensing and third-party engines](docs/licensing.md)
- [Roadmap](docs/roadmap.md)
- [Research references](docs/research-references.md)
- [Localization policy](docs/localization.md)
- [Change log](CHANGES.md)
- [v0.1.1 release notes](docs/releases/v0.1.1.md)
- [v0.2.0 release notes](docs/releases/v0.2.0.md)

## Run locally on macOS

Prerequisites currently verified on macOS: Node.js 22+, npm 10+, Rust 1.88+,
`yt-dlp`, and FFmpeg.

```bash
npm install
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --test real_engine_smoke -- --ignored
npm run tauri dev
```

The queue database is created in the platform application-data directory, not inside
the repository. English is the first-run default; an explicit language choice is
remembered locally.

The ignored smoke test creates a tiny media fixture with FFmpeg, serves it over a
loopback-only HTTP listener, downloads it through the real Rust scheduler and
`yt-dlp`, and verifies the output path. It requires no public media or external network.

## Identity

- Product name: **JiveFetch**
- Repository slug: `jivefetch`
- Private repository: [github.com/shurrman/jivefetch](https://github.com/shurrman/jivefetch)
- Application identifier: `top.jivejournal.jivefetch`
- Intended site: `fetch.jivejournal.top`

- Source license: [Apache-2.0](LICENSE)
- Current engine model: validated system executables; no engine binaries are redistributed
