[English](README.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md)

# JiveFetch

> Your media. Your queue. Your rules.

JiveFetch is a planned cross-platform desktop manager for lawful, local media
downloads. It will combine a Tauri 2 shell, a Rust scheduling and process-control
core, and a React/TypeScript interface with `yt-dlp`, `ffmpeg`/`ffprobe`, and
optionally `aria2` as managed external engines.

Version `0.0.1` is a working macOS foundation build. It provides a Tauri 2 desktop
shell, a React interface in English, Russian, and Simplified Chinese, and a Rust-owned
SQLite queue. Users can add validated HTTP(S) URLs and persistently Pause, Resume,
Stop, or Remove queue entries. It deliberately does not claim to download media yet;
`yt-dlp`/FFmpeg execution is the next vertical slice.

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
├── docs/                        # requirements, architecture and roadmap
├── package.json                 # pinned frontend/Tauri CLI dependencies
└── README.*.md                  # localized project entry points
```

The workspace will be split into dedicated Rust crates when process supervision and
engine adapters are introduced; the target boundaries remain documented in
[Architecture](docs/architecture.md).

## Documentation

- [Product requirements](docs/product-requirements.md)
- [Architecture](docs/architecture.md)
- [Task lifecycle and recovery](docs/task-lifecycle.md)
- [Security and authentication](docs/security.md)
- [Cross-platform packaging](docs/packaging.md)
- [Roadmap](docs/roadmap.md)
- [Research references](docs/research-references.md)
- [Localization policy](docs/localization.md)
- [Change log](CHANGES.md)
- [Project memory](MEMORY.md)

## Run locally on macOS

Prerequisites currently verified on macOS: Node.js 22+, npm 10+, and Rust 1.88+.

```bash
npm install
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

The queue database is created in the platform application-data directory, not inside
the repository. English is the first-run default; an explicit language choice is
remembered locally.

The next implementation step is the fake-engine/process-supervision phase followed by
a real `yt-dlp` probe and single-download vertical slice.

## Identity

- Product name: **JiveFetch**
- Repository slug: `jivefetch`
- Application identifier: `top.jivejournal.jivefetch`
- Intended site: `fetch.jivejournal.top`

Licensing is intentionally undecided until the distribution model and bundled
binary obligations have been reviewed.
