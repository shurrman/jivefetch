[English](task-lifecycle.md) | [Русский](task-lifecycle.ru.md) | [简体中文](task-lifecycle.zh-CN.md)

# Task lifecycle and recovery

## 1. Why a formal lifecycle is required

External download engines can spawn grandchildren, change from network activity to
post-processing, fail after producing useful partial files, or outlive a webview.
JiveFetch therefore models user tasks separately from process attempts and stores
every important transition before presenting it as complete.

## 2. Task and attempt

- A **task** is durable user intent: source, selected format policy, destination,
  authentication reference, priority, and limits.
- An **attempt** is one immutable execution plan and its owned process tree.
- A task can have many attempts over time but at most one live attempt.
- Partial and completed files are **artifacts** attributed to an attempt and task.

## 3. States

| State | Stable | Meaning |
| --- | --- | --- |
| `probing` | no | Metadata/formats are being resolved |
| `queued` | yes | Eligible for scheduler dispatch |
| `starting` | no | Slot and attempt committed; process ownership is being created |
| `downloading` | no | Network acquisition is active |
| `postprocessing` | no | Merge, remux, extraction, or conversion is active |
| `pausing` | no | A resumable controlled stop is in progress |
| `paused` | yes | No owned process is alive; compatible partials are retained |
| `stopping` | no | A terminal controlled stop is in progress |
| `stopped` | yes | No attempt is active; automatic scheduling is disabled |
| `waiting_retry` | yes | A transient failure is waiting for its retry deadline |
| `completed` | yes | Expected final artifacts were verified |
| `failed` | yes | Work ended with an actionable or terminal error |
| `interrupted` | yes | Previous transient work is uncertain after recovery |
| `removed` | yes | Tombstoned and hidden; optional file deletion was handled separately |

The UI may display a short-lived pending command, but that is not a durable task
state.

## 4. Transition outline

```text
probe ──success──► queued ──dispatch──► starting
  │                                   │
  └──failure──► failed                ├──► downloading ──► postprocessing ──► completed
                                      │          │                │
                                      │          ├──pause──► pausing ──► paused ──resume──► queued
                                      │          ├──stop───► stopping ─► stopped
                                      │          └──error──► waiting_retry / failed
                                      └──spawn failure─────► waiting_retry / failed

Any transient state found at startup ──► interrupted ──reconcile──► paused / queued /
                                                          failed / completed
Any non-removed state ──remove request──► stop if active ──► removed
```

Every transition has a whitelist. An illegal transition is an internal error and
cannot be forced by the frontend.

## 5. Command semantics

### Pause

Pause is portable and durable, not merely a frozen process:

1. commit `pausing` and the pause intent;
2. request graceful interruption;
3. terminate the owned process tree if it does not exit within the grace period;
4. inspect compatible partial artifacts;
5. commit `paused` only after no owned process remains.

Resume creates a new attempt using the same task and compatible partials. A future
Unix fast suspend may improve responsiveness, but application shutdown must still
convert it to the durable paused contract.

### Resume

Resume is accepted only from `paused` or an eligible reconciled `interrupted` state.
It can require a new probe when the old format snapshot is stale. It transitions to
`queued`; the scheduler, not the command handler, decides when a slot is available.

### Stop

Stop disables automatic resumption. It commits `stopping`, terminates the owned tree,
reconciles files, then commits `stopped`. Partial and completed files are retained by
default. A subsequent Retry is explicit.

### Retry

Retry clears the actionable error, optionally refreshes probe data, increments the
attempt count, and returns the task to `queued`. It never overwrites the previous
attempt record.

### Remove

Remove first reaches a stable no-process state. It then tombstones the task. The UI
offers two separate choices:

- remove the task/history but keep files;
- remove and delete a previewed set of app-owned artifacts.

Deletion is limited to canonical artifact paths already attributed to the task. It
does not recursively delete a user-selected destination directory. Failed deletion
leaves a visible cleanup error rather than hiding uncertainty.

## 6. Durable transition protocol

For a start operation:

1. scheduler transaction verifies state/revision and reserves capacity;
2. create immutable attempt and engine plan;
3. set task to `starting` and append durable event;
4. commit;
5. create the platform ownership container and spawn the child;
6. persist process metadata and transition based on the first confirmed phase;
7. emit events after their corresponding commit.

If spawn fails after step 4, the attempt ends with a typed spawn failure; it does not
roll the task backward invisibly.

The same “commit intent before side effect, then reconcile outcome” pattern applies
to pause, stop, and remove.

## 7. Startup recovery

Recovery runs before normal dispatch:

1. open and migrate SQLite;
2. acquire the single-instance scheduler lock;
3. assign a fresh application `run_id`;
4. convert tasks in transient states from older runs to `interrupted` in one
   transaction;
5. clean stale app-owned plaintext credential files using strict path/owner checks;
6. inspect attempt metadata and attributed artifact paths;
7. validate completed outputs when evidence exists;
8. classify compatible partials as resumable, incompatible, or unknown;
9. apply the configured recovery policy: leave paused, requeue eligible tasks, or
   request user action;
10. start normal scheduler ticks only after reconciliation completes.

JiveFetch does not attach to an unknown prior PID. Windows Job Objects should have
terminated descendants when the prior process closed. On Unix, any possible orphan is
reported and handled by verified ownership metadata; PID plus executable name is not
enough to kill or adopt it.

## 8. Partial-file compatibility

Resume eligibility depends on an unchanged canonical destination, compatible output
template, source identity, format policy, and engine behavior. JiveFetch records an
attempt fingerprint containing the non-secret plan fields relevant to partial reuse.

If the fingerprint changes, the user chooses whether to keep, rename, or delete the
old partial. JiveFetch never feeds an incompatible partial into a new plan silently.

## 9. Progress and checkpoints

Transient progress events may arrive many times per second. The supervisor coalesces
them for UI display. Durable checkpoints are stored at phase boundaries and a bounded
interval, carrying downloaded bytes, total estimate, speed, ETA, fragment counts,
and last safe event time.

The `0.3.0` UI requests an authoritative queue snapshot every five seconds and keeps
manual refresh available. Blue, green, and red progress fills supplement explicit text
states for active, completed, and failed/interrupted work.

A checkpoint is informational; files and engine resume behavior remain the source of
truth for resumability. A 100% progress event alone never marks a task completed.
Completion additionally requires a verified non-empty regular output file inside the
configured directory. Its actual size is persisted, and startup repairs legacy zero-byte
completed metrics from the same verified file. Right-click task actions remain projections
of the same revision-checked Start/Stop/Pause/Remove transitions; Copy URL does not mutate state.

## 10. Concurrency races to test

- Pause and Stop submitted together.
- Remove while Pause is still completing.
- Duplicate Resume after an IPC retry.
- App crash after committing `starting` but before spawn.
- App crash after process exit but before committing `completed`.
- Retry deadline fires while the user stops the task.
- Output-path conflict appears after a destination setting change.
- Global limit change causes rebalance while another task finishes.

Each scenario must converge to one stable state with no duplicate live process tree.
