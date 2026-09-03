[English](architecture.md) | [Русский](architecture.ru.md) | [简体中文](architecture.zh-CN.md)

# Архитектура

## 1. Архитектурные драйверы

JiveFetch — desktop UI над долгоживущими внешними процессами, которые могут завершаться
с ошибкой. Главные сложности — устойчивое управление очередью, владение процессами,
обращение с секретами и правдивое восстановление UI после сбоя.

Архитектура разделена на четыре границы:

- **Domain:** задачи, попытки, переходы состояний, форматы, лимиты и ошибки.
- **Application:** scheduler, recovery, probing, распределение bandwidth и команды.
- **Infrastructure:** SQLite, filesystem, keychain, clock и platform processes.
- **Adapters:** Tauri IPC/events и конкретные интеграции `yt-dlp`/`ffmpeg`/`aria2`.

Domain и Application должны компилироваться и тестироваться без webview и реальных
download engines. В `0.1.0` модули `model`, `storage`, `scheduler`, `engine` и
`process_supervisor` остаются в `src-tauri`; последующий crate split пойдёт по уже
проверенным интерфейсам, а не создаст пустые границы.

## 2. Целевая модель компонентов

```text
React UI ─ typed commands/events ─ Tauri adapter
                                      │
                         application services/query projection
                                      │
                            persistent Rust scheduler
                         ┌────────────┼────────────┐
                      SQLite   process supervisor  credential broker
                                      │
                               yt-dlp/ffmpeg/aria2
```

## 3. Планируемый workspace

```text
src/                               приложение React/TypeScript
src-tauri/                         bootstrap Tauri, commands, events, capabilities
crates/jivefetch-core/             чистая domain model и scheduler policies
crates/jivefetch-storage/          SQLite migrations и repositories
crates/jivefetch-process/          platform process ownership/supervision
crates/jivefetch-engines/          engine plans, parsers и capability detection
tests/helpers/process-tree/        test executable с children/grandchildren
tests/fixtures/                     очищенные engine output и migration fixtures
```

Rust-зависимости направлены внутрь: `src-tauri` зависит от adapters и core, а
infrastructure crates реализуют core traits. `jivefetch-core` не зависит от Tauri,
SQLite или process API.

## 4. Контракт команд и событий

Tauri-команды выражают намерение пользователя: probe, enqueue single/batch,
pause/resume/stop/retry/remove, reorder, update limits и snapshot. Каждая изменяющая
команда содержит idempotency key и ожидаемую revision. Rust service проверяет команду,
фиксирует transition и только затем публикует versioned event. При конфликте UI получает
текущую revision.

Events имеют монотонный sequence number в пределах экземпляра БД. При gap или reconnect
React запрашивает snapshot, а не угадывает состояние. Частые progress events объединяются;
durable checkpoints записываются реже и на границах phase.

## 5. Модель хранения

SQLite работает с foreign keys, WAL, busy timeout, migrations и явной durability policy.
Единственный write path принадлежит приложению.

| Таблица | Назначение |
| --- | --- |
| `tasks` | Намерение пользователя, state, revision, priority и timestamps |
| `attempts` | Один process run, engine plan, owner run, результат и diagnostics |
| `probes` | Очищенные metadata, normalized formats, engine version и expiry |
| `artifacts` | Canonical paths, роль, completeness, size и attempt owner |
| `task_events` | Durable audit/state events с monotonic sequence |
| `settings` | Несекретные settings и credential references |
| `idempotency_keys` | Deduplication команд и предыдущий результат |
| `schema_migrations` | Применённые migrations и checksums |

Cookies и credential values никогда не попадают в SQLite; probe payload очищается от
secret-bearing headers и signed URLs. State задачи и её event пишутся одной transaction.
Attempt ownership использует `run_id` и platform metadata; PID сам по себе информативен.

## 6. Scheduler

Scheduler — actor-like Rust service и единственный владелец dispatch/state. Он получает
commands и process events через channels, применяет policy, фиксирует изменения и
планирует следующий tick.

### 6.1 Инварианты dispatch

- Задача запускается только из стабильного разрешённого state.
- Reservation slot и переход в `starting` атомарны.
- Engine plan неизменяем для attempt и сохранён до spawn.
- У задачи не более одного live attempt.
- Задачи с конфликтующими canonical output paths не идут одновременно.
- Download и post-processing используют отдельные pools.
- Priority объединяется с queue age, чтобы исключить starvation.

### 6.2 Политика повторов

Errors типизированы. Authentication, invalid input, unsupported format, permission и
disk-full ждут действия пользователя. Timeouts, выбранные network errors, transient
extractor failures и crash управляемого engine получают ограниченный exponential backoff
с jitter. Каждый retry создаёт новый attempt; история не перезаписывается.

### 6.3 Политика bandwidth

`BandwidthPolicy` считает лимит каждого активного network attempt:

```text
effective(task) = min(task_limit_or_infinity, fair_share_of_global_limit)
```

Сначала фиксированный cap передаётся engine. При существенном изменении состава задач или
settings scheduler делает rebalance через pause/checkpoint и новый resumable attempt,
поскольку `yt-dlp` не везде умеет менять rate на лету. Изменения allocation debounced.
Позже возможен local transport broker, но для корректной task semantics он не обязателен.

## 7. Абстракция engine

Scheduler не собирает shell-команды. Engine adapter реализует:

```text
probe(request) -> ProbeResult
plan(task, probe, capabilities) -> ExecutionPlan
spawn(execution_plan, credential_lease) -> OwnedAttempt
parse(event_line) -> EngineEvent
classify(exit, recent_events) -> AttemptResult
```

`ExecutionPlan` содержит verified executable, argument vector, sanitized display,
environment allowlist, working directory, ожидаемые artifacts и capabilities — без shell
syntax. `yt-dlp` adapter владеет extractor/format behavior; прямой local processing может
позже получить отдельный `ffmpeg` adapter. `aria2` опционален. Parsers используют versioned
JSON/progress templates, а text input изолирован, fixture-tested и недоверен.

## 8. Process supervisor

До полезной работы attempt получает ownership container: на macOS/Linux — новая
session/process group; на Windows — terminate-on-close Job Object с creation flags,
не позволяющими descendants уйти до назначения.

Supervisor асинхронно дренирует stdout/stderr, ограничивает memory, пишет redacted logs и
публикует heartbeat/progress. Stop: durable `stopping`, graceful termination, ограниченное
ожидание с drain, termination owned group/job, проверка отсутствия members, reconcile
artifacts и stable result. После crash старые attempts считаются uncertain; JiveFetch
никогда не принимает процесс только по повторно использованному PID.

## 9. Credential broker

Scheduler получает краткоживущий `CredentialLease`. Browser mode возвращает несекретный
browser/profile selector. Imported-cookie mode берёт encryption key из OS credential
store, расшифровывает blob в restrictive temp file и передаёт adapter только путь и cleanup
guard. Lease освобождается после выхода всего process tree. На старте удаляются только
проверенные stale plaintext temp files, принадлежащие приложению.

Полная граница описана в [Security](security.ru.md).

## 10. Архитектура frontend

React хранит normalized read models intake, queue, task details и settings. Команда может
отображаться как pending, но state считается окончательным только после backend ack с
durable revision.

- typed IPC bindings согласованы с Rust DTO;
- history paginated, большие списки virtualized;
- controls доступны с keyboard и screen reader;
- текущая phase и pending command показаны явно;
- thumbnails идут через ограниченный fetch/cache path без Tauri privileges;
- secrets отсутствуют в webview state, browser storage и developer logs.

## 11. Наблюдаемость

Local logs структурированы, ограничены и redacted при ingestion. Correlation fields: task
ID, attempt ID, run ID, event sequence, engine/version и phase. Cookie content,
authorization, raw signed URLs и plaintext secret paths исключены.

MVP metrics остаются локальными: queue depth, dispatch latency, active counts, throughput,
retries, stop latency и recovery outcomes. Будущая telemetry требует отдельного opt-in
решения.

## 12. Архитектура тестов

- Property tests state machine и command idempotency.
- Scheduler tests с fake clock/engine и deterministic bandwidth policy.
- Migration fixtures от каждой released schema.
- Crash tests на каждой transition boundary.
- Native process-tree helper с children/grandchildren и unrelated same-name process.
- Golden fixtures probe/progress/error для поддерживаемых engine versions.
- Security tests redaction, path traversal, temp permissions и support bundles.
- Native packaging smoke tests на Windows, macOS и Linux.

## 13. Открытые решения

- Точная Rust DB library и migration tool.
- Генерация typed IPC или небольшая поддерживаемая вручную schema.
- Поддерживаемые OS versions и CPU architectures.
- Нужен ли transport broker для строгого aggregate rate limit.
- License и redistribution strategy для sidecar builds.

Решения принимаются до соответствующего roadmap gate и фиксируются как ADR.
