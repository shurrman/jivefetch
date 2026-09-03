[English](roadmap.md) | [Русский](roadmap.ru.md) | [简体中文](roadmap.zh-CN.md)

# Roadmap

Фазы закрываются evidence, не датой. UI polish не отменяет durability/security gates.

## Phase 0 — foundation

### Deliverables

Tauri 2 + React/TS + Rust, pinned lockfiles/toolchains, format/lint/test/link/secret
checks, native CI skeleton, ADR choices. `0.1.0` уже имеет working macOS UI, SQLite
queue, три языка, Apache-2.0, unit/build checks и native CI matrix. Первый remote-run
matrix прошёл на macOS/Linux/Windows; ADR и crate split ещё впереди.

### Exit criteria

Clean checkout воспроизводимо собирается и запускается, а решения и проверки записаны.

## Phase 1 — domain/storage

### Deliverables

Pure Rust task/attempt/artifact/error transitions, полные migrations/events/idempotency,
snapshot/pagination. Exit: property tests, duplicate command safety, migration fixture,
10k history pagination.

### Exit criteria

Перечисленные property, idempotency, migration и pagination checks проходят.

## Phase 2 — fake engine/process ownership

### Deliverables

Platform supervisor и helper child/grandchild, graceful/forced stop, scheduler slots/
aging/locks, все controls. Exit: complete owned tree gone, unrelated same-name alive,
races converge, crash boundaries recover.

Уже реализованы process-group/Job-Object supervisor, bounded output, два scheduler
slot, durable controls и unrelated-process test. Priority, aging, output locks и
полный crash-boundary matrix остаются.

### Exit criteria

Owned tree исчезает полностью, unrelated process жив, races и crash recovery сходятся.

## Phase 3 — yt-dlp vertical slice

### Deliverables

Verified engine registry, probe/normalized formats, format compiler, intake/picker,
single real queued download, FFmpeg capability errors. Exit: fixture suite, clean-machine
allowed test download, zero shell strings.

Уже реализованы system engine discovery, typed real plan, progress parser, output
verification, UI progress и loopback smoke. Metadata probe, format selection и
ffprobe capabilities остаются.

### Exit criteria

Fixture suite и clean-machine test проходят; shell strings отсутствуют.

## Phase 4 — parallel queue/recovery

### Deliverables

Network/postprocess pools, checkpoints/retry/artifact fingerprints, startup reconcile,
batch/reorder/history, disk/collision policy. Exit: hard-kill matrix, no duplicate/path
conflict, failed task не блокирует queue.

### Exit criteria

Hard-kill matrix не создаёт duplicate/path conflict; failed task не блокирует queue.

## Phase 5 — auth/bandwidth

### Deliverables

Browser refs, encrypted imported cookies, per-task/global fair limits, resumable
rebalance, sentinel redaction. Exit: auth on targets, no secret leakage, aggregate
ceiling target, no partial loss.

### Exit criteria

Auth работает на targets, secrets не утекают, ceiling соблюдается, partials сохранены.

## Phase 6 — desktop UX

### Deliverables

Accessible views, opt-in clipboard/deep link, notifications/folder, clear phases/errors,
virtualization/thumbnails/theme. Exit: keyboard/screen-reader smoke, no external auto-start,
responsive 10k history.

### Exit criteria

Accessibility smoke проходит, external input не auto-start, 10k history responsive.

## Phase 7 — engines/packaging

### Deliverables

Signed manifests/hash/health/rollback, licenses/SBOM/provenance, signed Windows,
notarized macOS, Linux packages, installer tests. Exit: tamper denial, rollback, per-target
packaging gates, visible exact licenses/versions.

### Exit criteria

Tamper denial и rollback проверены, target gates пройдены, licenses/versions видимы.

## Phase 8 — MVP hardening

### Deliverables

Long queues, flaky network, low disk, sleep/wake, fuzz, capability/security review,
frozen contracts и user docs. Exit: requirements scenarios pass, no critical/high
security/data-loss issue, no orphan tree, artifacts+signatures+SBOM published together.

### Exit criteria

Requirements scenarios проходят; нет high-risk issue/orphan tree; release комплект полный.

## Post-MVP candidates

Post-MVP: extension, library/player, subscriptions, plugins, remote/cloud, transport
broker, constrained expert policy language.

## Definition of done для каждой phase

Каждая фаза: synced EN/RU/简体中文 docs/UI, relevant tests, failure/restart paths,
secret-free fixtures, diff/link checks и CHANGES metrics.
