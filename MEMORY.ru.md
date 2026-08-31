[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# Память проекта JiveFetch

Обновлено: 2026-09-01

## Текущее состояние

- `0.0.1` — рабочий macOS foundation: Tauri 2, React и Rust/SQLite.
- URL добавляются в устойчивую очередь; Pause/Resume/Stop/Remove меняют состояние с
  optimistic concurrency по revision.
- UI и документы имеют EN/RU/简体中文; первый запуск всегда EN, явный выбор сохраняется.
- `yt-dlp` и FFmpeg найдены в системе, но ещё не подключены к приложению.
- Работа локальная; remote и push отсутствуют.

## Устойчивые решения

- Product ID `top.jivejournal.jivefetch`, сайт `fetch.jivejournal.top`.
- SQLite WAL — источник истины; React — только проекция.
- Один Rust scheduler будет владельцем переходов, concurrency, retry и bandwidth.
- Pause переносимо означает контролируемую остановку и новый resumable attempt.
- Владение процессами: Unix process group/session, Windows Job Object; не kill по имени.
- Browser mode хранит только browser/profile reference; импортированные cookies
  шифруются ключом из OS credential store.
- Формат выбирается после живого probe и компилируется в Rust без shell-строк.
- Документы и пользовательские строки всегда поддерживаются синхронно на EN/RU/简体中文.
- MediaHarbor и FlowGrab — только ориентиры идей; код и структура не копируются.
- Лицензия остаётся открытым решением до аудита sidecar-бинарников.

## Инварианты

- Не сохранять cookies, токены, auth headers и расшифрованные секреты в БД, логах и Git.
- Не считать PID доказательством владения и не принимать неизвестный процесс.
- Удаление записи задачи и удаление файлов — разные явные действия.
- Не объявлять кроссплатформенность до native-тестов на Windows/macOS/Linux.
- Новый или изменённый документ нельзя считать готовым без трёх переводов и ссылок.

## Следующее

1. Fake engine и process-tree helper.
2. Первая миграция полной модели tasks/attempts/artifacts/events.
3. Реальный `yt-dlp` probe и single-download vertical slice.
4. Выбор лицензии и sidecar policy до публичной упаковки.
