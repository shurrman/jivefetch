[English](architecture.md) | [Русский](architecture.ru.md) | [简体中文](architecture.zh-CN.md)

# Architecture

## 1. Architectural drivers

JiveFetch is a desktop UI around fallible, long-running external processes. The hard
parts are durable orchestration, process ownership, secret handling, and keeping the
UI truthful after crashes. Those concerns drive the architecture more strongly than
the initial screen layout.

The design follows four boundaries:

- **Domain:** tasks, attempts, state transitions, format intent, limits, and errors.
- **Application:** scheduler, recovery, probing, bandwidth allocation, and commands.
- **Infrastructure:** SQLite, filesystem, keychain, clock, and platform processes.
- **Adapters:** Tauri IPC/events and concrete `yt-dlp`/`ffmpeg`/`aria2` integration.

The domain and application layers must compile and test without a webview or real
download engines.

Version `0.4.0` keeps one scheduler authority while extracting typed error and diagnostics
modules, engine binary discovery, a `YtDlpExecutor`, and narrow engine/process traits used
by scheduler test doubles. A later crate split will follow these tested seams instead of
creating empty boundaries or splitting ownership of a live process tree.

## 2. Target component model

```text
┌──────────────────────────────── React UI ────────────────────────────────┐
│ intake · format picker · queue · task controls · history · settings     │
└───────────────────────────┬──────────────────────────────────────────────┘
                            │ typed commands / sequenced events
┌───────────────────────────▼──────── Tauri adapter ───────────────────────┐
│ validation · DTO mapping · capability boundary · window lifecycle       │
└───────────────┬───────────────────────────────┬──────────────────────────┘
                │                               │
┌───────────────▼──────────────┐  ┌─────────────▼─────────────────────────┐
│ Application services         │  │ Query/event service                   │
│ probe · enqueue · controls   │  │ snapshots · pagination · projections │
└───────────────┬──────────────┘  └─────────────┬─────────────────────────┘
                │                               │
┌───────────────▼────────────── persistent scheduler ─────────────────────┐
│ leases · priorities · concurrency · retries · bandwidth · recovery      │
└──────────┬────────────────────┬──────────────────────────┬──────────────┘
           │                    │                          │
┌──────────▼─────────┐ ┌────────▼──────────┐ ┌────────────▼───────────────┐
│ SQLite repositories│ │ Process supervisor│ │ Credential broker          │
│ tasks/events/files │ │ owned trees/logs  │ │ browser refs/encrypted blob│
└────────────────────┘ └────────┬──────────┘ └────────────────────────────┘
                                │
                    ┌───────────▼────────────────┐
                    │ Engine adapters            │
                    │ yt-dlp · ffmpeg · aria2    │
                    └────────────────────────────┘
```

## 3. Planned workspace

```text
src/                               React/TypeScript application
src-tauri/                         Tauri bootstrap, commands, events, capabilities
crates/jivefetch-core/             pure domain model and scheduler policies
crates/jivefetch-storage/          SQLite migrations and repositories
crates/jivefetch-process/          platform process ownership/supervision
crates/jivefetch-engines/          engine plans, parsers and capability detection
tests/helpers/process-tree/        deterministic child/grandchild test executable
tests/fixtures/                     sanitized engine output and migration fixtures
```

Rust crate dependencies point inward: `src-tauri` depends on adapters and core;
infrastructure crates implement core traits. `jivefetch-core` does not depend on
Tauri, SQLite, or process APIs.

## 4. Command and event contract

Tauri commands express user intent, not database mutations:

- `probe_url(request)`
- `enqueue_task(request)` and `enqueue_batch(request)`
- `pause_task(task_id, expected_revision)`
- `resume_task(task_id, expected_revision)`
- `stop_task(task_id, expected_revision)`
- `retry_task(task_id, expected_revision)`
- `remove_task(task_id, delete_files, expected_revision)`
- `reorder_queue(task_ids, expected_queue_revision)`
- `update_limits(request)`
- `get_snapshot(cursor)`

Each mutating command carries an idempotency key and an expected revision. The Rust
application service validates the command, commits the transition, and only then
emits a versioned event. Conflicts return the current revision so the UI can refresh.

Events contain a monotonic sequence number scoped to a database instance. On a gap
or webview reconnect, React requests a new snapshot rather than guessing missing
state. High-frequency progress is coalesced for UI delivery; durable checkpoints are
stored at a slower bounded cadence and at phase boundaries.

## 5. Persistence model

SQLite runs with foreign keys, WAL, a busy timeout, migrations, and an explicit
durability policy. A single application-owned write path avoids competing writers.

Initial logical tables:

| Table | Purpose |
| --- | --- |
| `tasks` | Stable user intent, current state, revision, priority, timestamps |
| `attempts` | One process execution, engine plan, owner run, result, diagnostics |
| `probes` | Raw redacted metadata, normalized formats, engine version, expiry |
| `artifacts` | Canonical paths, role, completeness, size, attempt ownership |
| `task_events` | Durable state/audit events with monotonic sequence |
| `settings` | Non-secret application settings and credential references |
| `idempotency_keys` | Command deduplication and prior result |
| `schema_migrations` | Applied migrations and checksums |

Raw cookies and credential values never enter SQLite. Raw probe payloads are filtered
for known secret-bearing headers/URLs before storage.

The current `app_settings` singleton stores validated concurrency, an optional aggregate
speed budget, a non-secret browser-kind reference, and the absolute output directory. Its
migration defaults to two slots, unlimited speed, no browser cookies, and the platform
Downloads directory plus `JiveFetch`.

Task state and its corresponding durable event are written in the same transaction.
Attempt ownership uses a generated `run_id` plus process-native ownership metadata;
PID alone is informational.

## 6. Scheduler

The scheduler is an actor-like Rust service with one authority over dispatch and task
state. It receives commands and process events through channels, evaluates policy,
commits changes, and schedules the next tick.

`AppSettings` validates itself, and storage, engine, validation, and scheduler failures
remain typed internally. Only the Tauri command boundary maps them to stable localized
error codes. This keeps causes testable without exposing raw engine output to React.

### 6.1 Dispatch invariants

- A task is dispatchable only from a stable eligible state.
- Slot reservation and transition to `starting` are atomic.
- The engine plan is immutable for an attempt and stored before spawn.
- One task has at most one live attempt.
- Conflicting canonical output paths cannot run concurrently.
- Download and post-processing pools have independent limits.
- Priority is combined with queue age to prevent starvation.

### 6.2 Retry policy

Errors are typed. Authentication, invalid input, unsupported format, permission, and
disk-full errors wait for user action. Timeouts, selected network errors, transient
extractor failures, and a crashed managed engine can retry with bounded exponential
backoff and jitter. Every retry is a new attempt; history is never overwritten.

### 6.3 Bandwidth policy

`BandwidthPolicy` calculates an effective limit for every active network attempt:

```text
effective(task) = min(task_limit_or_infinity, fair_share_of_global_limit)
```

Version `0.2.0` divides the aggregate budget by the configured execution slots and passes
that fixed per-attempt cap to `yt-dlp`. This conservative allocation never exceeds the
selected budget, although unused slots can leave bandwidth idle. Settings changes apply
to new attempts; already-owned process trees are not killed or silently restarted.

A later local transport broker may provide finer-grained aggregate throttling, but
it is not required to establish correct task semantics.

## 7. Engine abstraction

The scheduler never assembles raw command lines. An engine adapter implements:

```text
probe(request) -> ProbeResult
plan(task, probe, capabilities) -> ExecutionPlan
spawn(execution_plan, credential_lease) -> OwnedAttempt
parse(event_line) -> EngineEvent
classify(exit, recent_events) -> AttemptResult
```

`ExecutionPlan` contains an executable identity, argument vector, sanitized display
form, environment allowlist, working directory, expected artifacts, and required
capabilities. It contains no shell syntax.

The `yt-dlp` adapter owns extractor and format behavior. `ffmpeg` is normally invoked
by `yt-dlp`, but direct local processing can later use a separate adapter. `aria2` is
optional and activated only when its capabilities and requested protocol match.

Engine output parsers accept versioned JSON/progress templates where possible. Text
parsing is isolated, fixture-tested, and treated as untrusted input.

The `0.3.0` slice implements a bounded single-URL JSON probe through the same supervised
process boundary and browser-cookie selector used by downloads. React receives normalized
video-format metadata, hides internal format IDs and duplicate-looking choices without a
size estimate, and sends back only the selected validated selector. The selector is stored
with the task and later becomes a typed `--format` argument. Download plans explicitly pass
`--progress`, because yt-dlp's final-path printing can otherwise suppress progress output.

In `0.4.0`, discovery is separate from execution. Typed browser-cookie sources and engine
plans cross the scheduler boundary. The adapter also emits a safe component plan plus
component-tagged progress; the scheduler aggregates video and audio bytes into one
monotonic overall progress value and reserves 100% for a verified final file.

## 8. Process supervisor

Every attempt receives an ownership container before useful work begins:

- macOS/Linux: create a new session/process group and signal that group.
- Windows: create and assign a Job Object configured to terminate members when the
  job closes; use creation flags that prevent descendants escaping before assignment.

The supervisor drains stdout/stderr asynchronously, bounds memory, writes redacted
diagnostics, and emits heartbeat/progress events. Stop proceeds through:

1. mark `stopping` durably;
2. request graceful engine termination where supported;
3. wait a bounded grace period while draining output;
4. terminate the owned group/job;
5. verify no owned members remain;
6. reconcile artifacts and commit the stable result.

On application crash, Job Object kill-on-close handles Windows descendants. Unix
children receive parent-death protection where available, but recovery still treats
all previous transient attempts as uncertain. JiveFetch never adopts a process based
only on a reused PID.

## 9. Credential broker

The scheduler requests a short-lived `CredentialLease` from the broker. Browser mode
returns a non-secret browser/profile selector. Imported-cookie mode retrieves a data
encryption key from the OS credential store, decrypts the blob to a restrictive temp
file, and returns only its path and cleanup guard to the engine adapter.

The lease is released after the process tree exits. Startup removes stale app-owned
plaintext temp files after validating their path and ownership metadata.

See [Security](security.md) for the complete boundary.

## 10. Frontend architecture

React maintains normalized read models for intake, queue, task details, and settings.
It may optimistically show a command as pending, but it does not finalize state until
the backend acknowledges the durable revision.

Frontend concerns:

- typed IPC bindings generated or checked against Rust DTOs;
- list virtualization and paginated history;
- accessible controls and keyboard navigation;
- explicit current phase and pending command;
- thumbnail loading through a constrained fetch/cache path without Tauri privileges;
- no secret values in the webview state tree, browser storage, or developer logs.

## 11. Observability

Version `0.4.0` writes local structured JSON diagnostics to one 2 MiB current file and
one rotated file. Correlation fields include task and attempt IDs, engine version, state,
and stable error code. They exclude cookie content, authorization values, browser-profile
paths, raw engine lines, media URLs, signed URLs, and output paths.

Metrics remain local in the MVP: queue depth, dispatch latency, active counts,
throughput, retries, stop latency, and recovery outcomes. Any future telemetry is a
separate opt-in design decision.

## 12. Test architecture

- Pure state-machine property tests for legal transitions and command idempotency.
- Scheduler tests with narrow fake engine/process boundaries; fake clock and a
  deterministic bandwidth policy remain planned.
- Migration tests from every released schema fixture.
- Crash tests that terminate the app helper at each transition boundary.
- Native process-tree tests with a helper spawning children and grandchildren,
  including an unrelated same-name process.
- Golden fixtures for probe/progress/error parsing across supported engine versions.
- Security tests for redaction, path traversal, temp permissions, and support bundles.
- Native packaging smoke tests on Windows, macOS, and Linux.

## 13. Open decisions

- Exact Rust database library and migration tool.
- Typed IPC generation library versus a small hand-maintained schema.
- Initial supported OS versions and CPU architectures.
- Whether strict aggregate rate limiting eventually requires a transport broker.
- License and precise redistribution strategy for each sidecar build.

These decisions are made before their roadmap gate and then recorded as ADRs.
