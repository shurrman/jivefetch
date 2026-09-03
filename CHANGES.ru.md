[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# Изменения

## 0.2.0 - 2026-09-03

### Добавлено

- Устойчивые Rust/SQLite settings: concurrency presets 1–10 и своё проверяемое значение,
  необязательный суммарный бюджет скорости, browser-cookie authentication и нативный
  выбор папки с default `Загрузки/JiveFetch`.
- Системная, тёмная и светлая темы и новая контрастная иконка JiveFetch во всех
  сгенерированных platform-размерах.
- Authoritative refresh очереди каждые пять секунд: синий active, зелёный completed,
  красный failed/interrupted progress, а также speed, ETA и текущий/общий размер.
- Версии `yt-dlp`/FFmpeg перенесены в шапку; технические SQLite/engine-плашки и лишнее
  сообщение о языке удалены.
- Типизированные аргументы `--limit-rate` и `--cookies-from-browser` с allowlist,
  migration, persistence, allocation и argument-vector тестами.
- `v0.1.1` опубликован как первый cross-platform prerelease: DMG, AppImage, DEB,
  NSIS EXE, MSI и SHA-256 manifests для каждой платформы.
- `v0.2.0` опубликован после успешных Native CI и Native Release на macOS, Linux и
  Windows с восемью installer/checksum artifacts.
- Локальные `AGENTS*` и `MEMORY*` убраны из version control, но рабочие копии сохранены
  точными правилами ignore.

### Метрики

- Было: 135 файлов, 45 Markdown-документов в 15 translation sets, 3 262 строки
  application/check source и 8 обычных Rust-тестов.
- Стало: 134 файла, 42 Markdown-документа в 14 translation sets, 4 039 строк,
  14 обычных Rust-тестов и opt-in real-engine smoke.

## 0.1.1 - 2026-09-03

### Исправлено

- Явно задан полный комплект PNG/ICNS/ICO, чтобы Linux AppImage и Windows MSI
  находили квадратный и `.ico` bundle assets.
- Native CI на push ограничен ветками, чтобы release tag не дублировал проверочную
  matrix, уже выполняемую Native Release.

## 0.1.0 - 2026-09-03

### Добавлено

- Архитектурный baseline, требования, безопасность, lifecycle, packaging и roadmap.
- Рабочий Tauri 2 + React/TypeScript + Rust прототип для macOS.
- SQLite-очередь с ревизиями, проверкой HTTP(S) URL и командами Pause/Resume/Stop/Remove.
- Интерфейс и документация на английском, русском и упрощённом китайском; EN по умолчанию.
- Начальный icon asset JiveFetch и проверяемые npm/Cargo lockfiles.
- Реальный запуск `yt-dlp` с типизированными аргументами, обнаружение FFmpeg,
  ограниченный parser прогресса и проверка выходного пути.
- Rust-scheduler на два слота, транзакционные attempts, startup recovery и переносимые
  Pause/Resume/Stop.
- Supervisor собственного дерева через Unix process groups и Windows Job Objects,
  включая cross-platform тест с сохранением постороннего процесса.
- Локальный smoke-тест реальных движков, secret scan, native CI Windows/macOS/Linux
  и live progress UI на трёх языках.
- Выбрана Apache-2.0 и задокументирован gate лицензий системных движков.
- Создан приватный GitHub-репозиторий `shurrman/jivefetch` и настроен `origin`.
- Добавлен tag-driven native release workflow для macOS DMG, Linux AppImage/deb и
  Windows NSIS/MSI preview-пакетов с SHA-256 checksums для каждой платформы.
- Первая remote native CI matrix прошла на macOS, Linux и Windows: frontend,
  три языка документации, secret scan, rustfmt, clippy, Rust-тесты и desktop compilation.

### Метрики

- Было: 2 исходных файла, 1 общий Markdown, 0 файлов приложения и 0 проектных документов.
- Стало: 135 файлов репозитория, в том числе 45 Markdown-документов в 15 структурно
  согласованных комплектах EN/RU/简体中文, 3 262 строки application/check source,
  запускаемый desktop build, 8 обычных проходящих Rust-тестов, 1 проходящий opt-in
  real-engine smoke и устойчивые SQLite tasks/attempts.

### Остаётся

- Не готовы signing identities, managed sidecars, проверка распространения FFmpeg,
  SBOM/provenance и clean-machine release tests.
- Linux транзитивно использует `glib 0.18.5` через Tauri/GTK и затронут
  `GHSA-wrw7-89jp-8q8g`; проблемный API не используется, но advisory блокирует stable-релиз.
