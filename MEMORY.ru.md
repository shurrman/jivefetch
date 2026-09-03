[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# Память проекта JiveFetch

Обновлено: 2026-09-03

## Текущее состояние

- `0.1.0` — рабочий первый download preview: Tauri 2, React, Rust/SQLite, локальные
  `yt-dlp` и FFmpeg.
- Rust-scheduler на два слота владеет typed process plans, attempts, progress и
  Pause/Resume/Stop/Remove с optimistic concurrency по revision.
- UI и документы имеют EN/RU/简体中文; первый запуск всегда EN, явный выбор сохраняется.
- Supervisor владеет Unix process group или Windows Job Object, ограниченно дренирует
  output и завершает только созданное им дерево.
- На macOS прошли real-engine loopback smoke с yt-dlp `2026.07.04`/FFmpeg `8.1.2`
  и тест сохранения постороннего процесса.
- Первая remote native source CI успешно прошла на Windows, macOS и Linux в GitHub
  Actions run `33779282453`.
- GitHub CLI авторизован как `shurrman`; приватный репозиторий
  `https://github.com/shurrman/jivefetch` настроен как `origin`. Версия `v0.1.0` —
  первый native release candidate для macOS, Linux и Windows.
- Исходники JiveFetch — Apache-2.0; системные движки не распространяются.
- Linux получает `glib 0.18.5` через Tauri/GTK. `GHSA-wrw7-89jp-8q8g` затрагивает API,
  который JiveFetch не вызывает; исправленный `glib 0.20` несовместим с GTK `^0.18`.
  Исключение допустимо только для private preview и блокирует stable-релиз.

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
- Код и документация JiveFetch используют Apache-2.0. Распространение bundled/managed
  движков остаётся отдельным gate по точной лицензии бинарника.
- Tag, совпадающий с package version, запускает native preview-сборки macOS/Linux/Windows;
  публикация происходит только после загрузки пакетов и checksums всей matrix.

## Инварианты

- Не сохранять cookies, токены, auth headers и расшифрованные секреты в БД, логах и Git.
- Не считать PID доказательством владения и не принимать неизвестный процесс.
- Удаление записи задачи и удаление файлов — разные явные действия.
- Не объявлять кроссплатформенность до native-тестов на Windows/macOS/Linux.
- Новый или изменённый документ нельзя считать готовым без трёх переводов и ссылок.

## Следующее

1. Проконтролировать первый release workflow на Windows/macOS/Linux; исправить ошибки
   packaging до признания artifacts `v0.1.0` проверенными.
2. Добавить metadata probe, выбор формата и отчёт capabilities ffprobe.
3. Расширить storage: artifacts/events, idempotency, retry policy и pagination.
4. Добавить cookies через credential store и проверяемые managed engines до release.
