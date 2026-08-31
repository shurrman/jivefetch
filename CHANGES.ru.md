[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# Изменения

## Unreleased

### Добавлено

- Архитектурный baseline, требования, безопасность, lifecycle, packaging и roadmap.
- Рабочий Tauri 2 + React/TypeScript + Rust прототип для macOS.
- SQLite-очередь с ревизиями, проверкой HTTP(S) URL и командами Pause/Resume/Stop/Remove.
- Интерфейс и документация на английском, русском и упрощённом китайском; EN по умолчанию.
- Начальный icon asset JiveFetch и проверяемые npm/Cargo lockfiles.

### Метрики

- Было: 2 исходных файла, 1 общий Markdown, 0 файлов приложения и 0 проектных документов.
- Стало: 113 versioned-файлов, в том числе 36 Markdown-документов в 12 структурно
  согласованных комплектах EN/RU/简体中文, 1 341 строка application/check source,
  запускаемый `.app`, 4 проходящих Rust-теста и устойчивая SQLite-очередь.

### Остаётся

- `yt-dlp`/FFmpeg ещё не запускаются из приложения.
- Нет process supervisor, engine probe, реальной загрузки и native CI.
- Remote и выбранная лицензия отсутствуют.
