[English](product-requirements.md) | [Русский](product-requirements.ru.md) | [简体中文](product-requirements.zh-CN.md)

# Требования к продукту

## 1. Назначение продукта

JiveFetch — local-first desktop-приложение для добавления, анализа, устойчивой очереди
и управления законными медиазагрузками. Оно должно быть простым снаружи и вести себя
как надёжная job system внутри.

## 2. Принципы проектирования

Принципы: простой UI с точной семантикой; durability раньше анимации; живой probe
вместо статического списка качества; минимальная работа с секретами; завершение только
своих процессов; понятная деградация; отсутствие обязательного cloud/telemetry.

## 3. Основные пользовательские сценарии

### 3.1 Быстрая загрузка

Вставить URL, получить metadata/formats, выбрать policy и поставить задачу в очередь.

### 3.2 Batch queue

Проверить batch/playlist, применить defaults, переопределить отдельные задачи,
  менять порядок и управлять независимо.

### 3.3 Аутентифицированный источник

Выбрать browser/profile или импортировать Netscape cookies, не показывая значения в UI.

### 3.4 Восстановление после restart

После crash/reboot примирить transient states и partial artifacts до новых запусков.

## 4. Функциональные требования

### FR-1 URL intake и probing

- Single/multiple URL, opt-in clipboard и deep link с review до старта.
- Metadata, playlist, subtitles, chapters и реальные formats через отдельный probe.
- Отмена устаревшего probe; cache с возрастом и версией engine; re-probe перед стартом.

### FR-2 Динамический выбор формата

- Простые presets и advanced inventory: resolution, FPS, codec, container, HDR, audio,
  размер и необходимость merge/transcode.
- Пользователь выбирает intent, а Rust компилирует typed `yt-dlp` args без shell.
- Fallback объясним; remux и transcode — разные явные действия.

### FR-3 Устойчивая очередь и concurrency

- SQLite хранит tasks, priority/order, policy, attempts, progress и artifacts.
- Отдельные limits для network и post-processing, output-path lock, priority aging.
- Desktop-контрол предлагает presets 1–10 и проверяемое своё значение; default первого
  запуска — 2. Уменьшение значения не завершает уже работающие задачи.
- Один live attempt на task; transient errors получают bounded retry/backoff.
- Ошибка одной задачи не блокирует остальные.

### FR-4 Управление отдельной задачей

- Pause сохраняет полезные partials и достигает состояния без живого процесса.
- Resume создаёт новый attempt с тем же intent; Stop запрещает auto-resume.
- Retry сохраняет историю; Remove отдельно спрашивает об удалении tracked files.
- Команды идемпотентны и защищены revision/idempotency key.

### FR-5 Process supervision

- Attempt владеет деревом: Unix session/process group, Windows Job Object.
- Graceful deadline, затем принудительное завершение только owned tree и проверка результата.
- Никогда не kill по имени; stdout/stderr дренируются асинхронно и ограниченно.

### FR-6 Authentication и cookies

- Browser reference без сохранения cookie values.
- Imported cookies шифруются random key из OS credential store; plaintext — только
  user-only temp lease на время child tree.
- Cookies, tokens, auth headers и signed query values не попадают в logs/DB/UI/Git.

### FR-7 Bandwidth limits

- Per-task ceiling и fair application-wide ceiling.
- Effective limit ограничен обоими; изменение может вызвать debounced resumable restart.
- После стабилизации aggregate traffic не превышает 110% global limit.
- В `0.2.0` общий предел консервативно делится между configured slots для новых attempts;
  будущий broker сможет перераспределять ёмкость свободных слотов.

### FR-8 History, artifacts и diagnostics

Redacted attempt history, typed errors, tracked canonical artifacts и безопасный Show in
Folder. Диагностика привязана к attempt и не содержит секретов.

### FR-9 Dependency management

Signed engine manifest, hashes, health check, immutable versions и rollback.

### FR-10 Cross-platform delivery

Native signed/notarized packages и process/path/secret/migration tests на каждой ОС.

## 5. Нефункциональные требования

### Reliability

Hard kill не повреждает очередь и не порождает duplicate attempt.

### Performance

10 000 history items работают через pagination/indices; progress coalescing не
  перегружает UI/SQLite.

### Security и privacy

Минимальные Tauri capabilities, запрет remote scripts, redaction до persistence.

### Accessibility и UX

Keyboard path, accessible labels, visible focus; states не кодируются только цветом.
Активная очередь обновляется минимум раз в пять секунд и показывает progress, state,
speed, ETA, загруженный и общий размер для каждой задачи. Версии движков и настройки
concurrency, скорости, темы, языка и папки остаются в верхней зоне без технических плашек.

## 6. Граница MVP

MVP включает URL/batch, dynamic probe, persistent concurrency/recovery, controls,
browser/imported cookies, rate limits, history и проверенную упаковку. Player, library,
subscriptions, extensions, remote/cloud, plugins и raw expert args отложены.

## 7. Сценарии приёмки release

Release scenarios: crash/restart без дублей; pause/resume на каждой ОС; stop дерева с
неприкосновенным unrelated process; global-limit rebalance; sentinel-secret scan;
исчезнувший format вызывает re-probe; engine update/rollback; DB migration/recovery.
