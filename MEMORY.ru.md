[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# Память проекта JiveFetch

Обновлено: 2026-09-03

## Текущее состояние

- `0.2.0` — активный следующий preview: Tauri 2, React, Rust/SQLite, локальные
  `yt-dlp` и FFmpeg.
- Настраиваемый Rust-scheduler владеет typed process plans, attempts, progress,
  распределением общего лимита скорости и Pause/Resume/Stop/Remove с revision.
- Устойчивые несекретные настройки включают concurrency (по умолчанию 2, presets 1–10,
  своё проверяемое значение до 64), общий лимит скорости, выбор browser cookies для
  yt-dlp и output directory по умолчанию `Загрузки/JiveFetch`.
- UI раз в пять секунд читает авторитетную очередь, показывает цветной progress,
  скорость, ETA и размеры каждой задачи, имеет system/dark/light темы и новую яркую иконку.
- UI и документы имеют EN/RU/简体中文; первый запуск всегда EN, явный выбор сохраняется.
- Supervisor владеет Unix process group или Windows Job Object, ограниченно дренирует
  output и завершает только созданное им дерево.
- Regular Rust suite содержит 14 тестов: migration v2/v3, типизированные engine args
  и владение process tree. Opt-in real-engine loopback smoke — последний локальный
  runtime gate для `v0.2.0`.
- Первая remote native source CI успешно прошла на Windows, macOS и Linux в GitHub
  Actions run `33779282453`.
- GitHub CLI авторизован как `shurrman`; приватный репозиторий
  `https://github.com/shurrman/jivefetch` настроен как `origin`. Candidate `v0.1.0`
  выявил неявные Linux/Windows bundle icons. Native CI run `33783089736` и Native
  Release run `33784331491` успешно завершились для опубликованного `v0.1.1` с
  восемью macOS/Linux/Windows artifacts.
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
- Browser-cookie mode хранит только browser identifier из allowlist и передаёт его
  типизированным аргументом `yt-dlp --cookies-from-browser`; значения cookies не копируются.
  Будущий импорт cookie-файла должен шифроваться ключом из OS credential store.
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

1. Завершить локальные app/real-engine/package проверки `v0.2.0`, затем потребовать
   зелёную native CI на Windows/macOS/Linux перед tag и публикацией prerelease.
2. Добавить metadata probe, выбор формата и отчёт capabilities ffprobe.
3. Расширить storage: artifacts/events, idempotency, retry policy и pagination.
4. Добавить profile/keyring details и зашифрованный import cookie-файла без показа значений.
5. Добавить signing/notarization и проверяемые managed engines до stable release.
