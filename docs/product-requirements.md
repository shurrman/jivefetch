[English](product-requirements.md) | [Русский](product-requirements.ru.md) | [简体中文](product-requirements.zh-CN.md)

# Product requirements

## 1. Product statement

JiveFetch is a local-first desktop application for collecting, inspecting, queuing,
and controlling lawful media downloads. It should feel as immediate as a simple
paste-and-download utility while behaving internally like a durable job system.

The central promise is not merely that a download can start. It is that the user can
understand what will happen, control every task, close the application, recover from
failure, and continue without losing the queue or accidentally duplicating work.

## 2. Design principles

1. **Simple surface, explicit semantics.** Common choices are one click; destructive
   choices describe exactly what happens to partial and completed files.
2. **Durability before animation.** The database is updated before the UI reports a
   transition as accepted.
3. **Current source truth.** Formats and authentication requirements are discovered
   from a live probe, not a static quality list.
4. **Least-secret handling.** Prefer browser-mediated authentication that does not
   copy cookies. Encrypt imported material and minimize its lifetime in plaintext.
5. **Owned processes only.** A task may stop its own complete process tree and
   nothing else on the machine.
6. **Useful degradation.** Missing `ffmpeg`, expired cookies, unsupported pause, low
   disk space, or a stale format selection produce actionable states rather than a
   frozen progress bar.
7. **Local-first and quiet.** No account, cloud relay, analytics, or telemetry is
   required for the initial product.

## 3. Primary user journeys

### 3.1 Quick download

1. Paste or capture a URL.
2. JiveFetch probes metadata and available formats.
3. Choose a preset such as Best, 1080p, Audio only, or a detailed format pair.
4. Add the task to the queue.
5. Watch progress, speed, ETA, current phase, and output path.

### 3.2 Batch queue

1. Paste several URLs or a playlist.
2. Review the number of resulting items and naming/output policy.
3. Apply common defaults and override individual items where needed.
4. Reorder, prioritize, pause, resume, stop, retry, or remove tasks independently.

### 3.3 Authenticated source

1. Select an installed browser/profile or import an approved Netscape cookie file.
2. Probe without exposing credential contents to the UI or logs.
3. If authentication expires, the task enters an actionable failure state and can be
   retried after credentials are refreshed.

### 3.4 Restart recovery

1. Close or crash the application while tasks are active.
2. Reopen it.
3. JiveFetch reconciles transient states and partial artifacts before dispatching new
   work.
4. Tasks appear as interrupted, paused, queued, or completed based on evidence; an
   optional policy resumes eligible tasks automatically.

## 4. Functional requirements

### FR-1 URL intake and probing

- Accept a single URL, multiple URLs, clipboard capture, and OS deep links.
- Treat clipboard monitoring and deep links as opt-in features.
- Normalize only safe superficial aspects of a URL; preserve extractor-significant
  query parameters while redacting sensitive parameters from display and logs.
- Probe through a dedicated engine command and return title, source, duration,
  thumbnail URL, playlist information, subtitles, chapters, and format inventory.
- Cancel or supersede stale probes when the input changes.
- Cache probe results with an age and engine-version marker; re-probe before start if
  the snapshot is stale or the selected format is no longer available.

### FR-2 Dynamic format selection

- Offer intent-based presets and an advanced view of actual audio/video formats.
- Show resolution, frame rate, codecs, container, HDR, audio channels/bitrate,
  approximate size, and whether merging/transcoding is required when known.
- Keep normalized selections separate from engine-specific format IDs.
- Compile typed intent into `yt-dlp` arguments in Rust. Do not build a shell command
  in React.
- Explain fallbacks, for example: “prefer AV1 up to 1440p, otherwise best compatible
  video plus audio.”
- Make remux and transcode distinct choices; never silently transcode when remux is
  sufficient.

### FR-3 Persistent queue and concurrency

- Persist tasks, priority, order, selected policy, retries, progress checkpoints,
  attempts, and artifacts in SQLite.
- Enforce a configurable maximum number of simultaneous downloading tasks and a
  separate post-processing limit.
- The desktop control offers 1–10 presets and a validated custom value; the first-run
  default is 2. Reducing the value does not terminate already running tasks.
- Prevent two attempts from owning the same task or output path concurrently.
- Support priorities without permanent starvation of older tasks.
- Apply retry classification and exponential backoff with jitter only to errors
  judged transient.
- Continue scheduling unrelated tasks when one task fails.

### FR-4 Per-task controls

- **Pause:** reach a resumable paused state, preserving useful partial artifacts.
- **Resume:** start a new attempt using the same task intent and compatible partials.
- **Stop:** terminate active work and move to a stable stopped state; preserve files
  unless the user separately chooses deletion.
- **Retry:** create a new attempt after a failure or stop, optionally re-probing first.
- **Remove:** hide/tombstone queue history only after active work is stopped. File
  deletion is a separate explicit option with a path preview.
- Commands must be idempotent: duplicate clicks or IPC retries cannot spawn duplicate
  work or fail destructively.

### FR-5 Process supervision

- Track an attempt as a process tree, not a single PID.
- Capture structured stdout/stderr without allowing a blocked pipe to deadlock the
  child.
- Terminate gracefully first, wait a bounded interval, then terminate the owned tree
  forcibly if required.
- Use process groups/sessions on Unix and Job Objects with kill-on-close on Windows.
- Never kill all processes with a matching executable name.
- Treat `yt-dlp`-spawned `ffmpeg` and external downloader children as part of the same
  attempt ownership boundary.
- Detect and report an orphan-risk condition instead of claiming a successful stop.

### FR-6 Authentication and cookies

- Support `yt-dlp` browser-cookie integration using a chosen browser/profile without
  persisting cookie values.
- Support imported Netscape cookie files encrypted at rest.
- Use the OS credential store for the random key protecting imported secret blobs.
- Materialize decrypted data only in a user-only temporary location and remove it on
  process exit and startup cleanup.
- Allow authentication configuration per task and as a default profile reference.
- Never display, export, or log cookie contents, authorization headers, passwords, or
  signed URL query values.

### FR-7 Bandwidth limits

- Support an optional per-task download-rate ceiling.
- Support an application-wide ceiling shared fairly among active network tasks.
- Effective task allocation is bounded by both policies; `unlimited` is explicit.
- Rebalancing must not discard partial data. When a running engine cannot change its
  limit dynamically, JiveFetch uses a resumable controlled restart.
- Surface that source throttling, protocol overhead, and post-processing can make
  observed rates lower than the configured maximum.
- Acceptance target for sustained aggregate traffic is no more than 110% of the
  configured global ceiling after a short stabilization window.
- The `0.2.0` conservative implementation divides the global ceiling across configured
  slots for new attempts; a future broker may reclaim capacity from idle slots.

### FR-8 History, artifacts, and diagnostics

- Keep task and attempt history with redacted, user-readable diagnostics.
- Record final and partial artifacts by canonical path, size, and role.
- Open the containing folder using platform APIs only after canonical path checks.
- Provide a support bundle that excludes secrets by construction and previews every
  included file.
- Distinguish extractor, network, authentication, disk, format, post-processing,
  cancellation, and internal failures.

### FR-9 Dependency management

- Detect bundled baseline and managed override versions of `yt-dlp`, `ffmpeg`,
  `ffprobe`, and optional `aria2c`.
- Verify signed manifests and cryptographic hashes before activating a downloaded
  tool.
- Keep the previous known-good version for rollback.
- Update engines independently from the application only within the declared
  compatibility range.

### FR-10 Cross-platform delivery

- Produce native, signed/notarized packages for supported Windows, macOS, and Linux
  targets.
- Run process-tree, path, secret-storage, migration, and sidecar-discovery tests on
  each target OS.
- Display exact engine versions and licenses in the application.

## 5. Non-functional requirements

### Reliability

- A hard application kill during probe, download, merge, or stop must not corrupt the
  queue database.
- Recovery never silently marks uncertain work completed.
- Repeating a recovered scheduler tick is safe.

### Performance

- A queue containing 10,000 historical tasks remains usable through pagination and
  indexed queries.
- Progress-event coalescing prevents UI or database write amplification; durable
  checkpoints are less frequent than transient UI updates.
- Slow thumbnail loading cannot block queue control.

### Security and privacy

- Tauri capabilities are minimal and scoped to the application windows that need
  them.
- Remote content is not granted Tauri IPC access.
- Logs and diagnostics are redacted before persistence, not only before display.
- No telemetry is enabled by default in the initial release.

### Accessibility and UX

- Every control has a keyboard path, accessible label, visible focus, and state that
  does not rely on color alone.
- Destructive actions say whether task records, partials, or completed files will be
  affected.
- A progress indicator identifies Downloading, Merging, Converting, Paused, Stopping,
  and Waiting for retry as different phases.
- Active queue snapshots refresh at least every five seconds and display progress,
  state, speed, ETA, downloaded bytes, and estimated total bytes per task.
- Application and engine versions, concurrency, speed, theme, language, and output-directory controls
  remain visible in the top settings region; implementation-only storage cards are absent.

## 6. MVP boundary

The first public MVP includes single and batch URL intake, dynamic probing, persistent
bounded concurrency, reliable recovery, task controls, browser/imported cookies,
global and per-task rate limits, progress/history, and signed packages for the chosen
initial target matrix.

Library playback, subscriptions, scheduled recurring downloads, browser extensions,
remote control, cloud synchronization, plugins, and arbitrary expert arguments are
post-MVP candidates.

## 7. Release acceptance scenarios

1. Kill the app during download, relaunch, and resume without a duplicate attempt or
   lost queue entry.
2. Pause and resume a task on all supported OSes while preserving a valid partial.
3. Stop a task that has spawned `ffmpeg` or `aria2c`; verify the owned tree is gone
   and an unrelated process with the same executable remains alive.
4. Change the global rate while three tasks run; verify eventual aggregate compliance
   without losing partial data.
5. Use browser authentication and imported cookies; scan database, logs, support
   bundle, command preview, and crash artifacts for secret leakage.
6. Select a format that disappears before dispatch; verify re-probe and a clear user
   decision instead of an opaque failure.
7. Upgrade and roll back an engine from verified artifacts.
8. Migrate a database created by the previous released version and recover active
   states safely.
