[English](task-lifecycle.md) | [Русский](task-lifecycle.ru.md) | [简体中文](task-lifecycle.zh-CN.md)

# Жизненный цикл и восстановление

## 1. Зачем нужен формальный lifecycle

UI-команды, crash recovery и process side effects должны сходиться в одно устойчивое
состояние; иначе Pause/Stop будут лишь визуальными флагами.

## 2. Task и attempt

Task — устойчивый intent; attempt — один immutable execution plan и owned process tree;
artifact — attributed partial/final file. У task много attempts в истории, но не больше
одного live attempt.

## 3. States

States: `probing`, `queued`, `starting`, `downloading`, `postprocessing`, `pausing`,
`paused`, `stopping`, `stopped`, `waiting_retry`, `completed`, `failed`, `interrupted`,
`removed`. Foundation `0.0.1` реализует stable subset `queued/paused/stopped` плюс
зарезервированные runtime states.

## 4. Схема переходов

```text
probe -> queued -> starting -> downloading -> postprocessing -> completed
                    │             ├ pause -> pausing -> paused -> queued
                    │             ├ stop  -> stopping -> stopped
                    │             └ error -> waiting_retry / failed
old transient state at startup -> interrupted -> reconcile -> paused/queued/failed/completed
```

## 5. Семантика команд

### Pause

Commit intent, graceful interrupt, owned-tree termination при deadline,
  проверка partials, затем stable `paused` без живых процессов.

### Resume

Из `paused`/eligible `interrupted`, при необходимости re-probe, затем `queued`.

### Stop

Прекращает auto-resume, сохраняет files по умолчанию и достигает `stopped`.

### Retry

Создаёт новый attempt и сохраняет историю.

### Remove

Сначала stable/no-process, затем tombstone. Delete files — отдельный previewed
  выбор только tracked canonical paths, без recursive destination deletion.

Команды идемпотентны и проверяют revision. Illegal transition отклоняется backend.

## 6. Durable transition protocol

Scheduler transaction проверяет revision/slot, создаёт attempt+plan, записывает
`starting` и event, commit, затем создаёт ownership container и spawn. Ошибка после
commit становится typed spawn failure, а не скрытым rollback. Pause/Stop/Remove следуют
правилу «persist intent before side effect, reconcile outcome after».

## 7. Startup recovery

Открыть/migrate DB; взять single-instance scheduler lock; создать `run_id`; одним commit
перевести старые transient states в `interrupted`; удалить только проверенные app-owned
plaintext leases; проверить attempt metadata/artifacts; классифицировать partials;
применить recovery policy; только затем запустить scheduler.

Не attach к старому PID. Windows Job Object должен убить descendants on close; Unix
orphan обрабатывается только с verified ownership metadata.

## 8. Совместимость partial files

Partial reuse требует совместимых destination/template/source/format/engine fingerprint.
Несовместимые данные не используются молча.

## 9. Progress и checkpoints

Progress coalesced для UI, checkpoints —
на bounded interval и phase boundaries; 100% progress сам по себе не означает complete.

## 10. Concurrency races для тестирования

Обязательные races: Pause+Stop, Remove during Pause, duplicate Resume, crash между
commit/spawn, crash между process exit/completed commit, retry deadline vs Stop,
output conflict и bandwidth rebalance vs finish. Каждый сходится в один stable state.
