[English](README.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md)

# JiveFetch

> Ваши медиа. Ваша очередь. Ваши правила.

JiveFetch — локальный кроссплатформенный desktop-менеджер законных медиазагрузок.
Целевой стек: Tauri 2, Rust, React/TypeScript, `yt-dlp`, `ffmpeg`/`ffprobe` и,
опционально, `aria2`.

Версия `0.0.1` уже запускается на macOS: Tauri-оболочка, интерфейс на английском,
русском и упрощённом китайском и очередь SQLite под управлением Rust. Можно добавить
проверенную HTTP(S)-ссылку и устойчиво выполнить Pause, Resume, Stop и Remove. Версия
честно не объявляет загрузку начавшейся: интеграция `yt-dlp`/FFmpeg — следующий этап.

## Цели

- Ограниченная параллельная очередь, переживающая сбои и перезапуски.
- Живой выбор доступных форматов, качества, кодеков, контейнера и субтитров.
- Browser authentication и импорт cookies без открытого хранения секретов.
- Общий и индивидуальные лимиты скорости.
- Формальные Pause/Resume/Stop/Retry/Remove с понятной судьбой файлов.
- Владение полным деревом `yt-dlp`/`ffmpeg`/`aria2` на Windows, macOS и Linux.
- Воспроизводимые подписанные пакеты с проверяемыми sidecar-инструментами.

## Не входит в задачу

- Обход DRM, paywall или контроля доступа.
- Cloud relay, учётная запись и телеметрия в первой версии.
- Произвольная shell-строка с аргументами `yt-dlp` в MVP.
- Копирование MediaHarbor, FlowGrab, их кода, UI или структуры.

Пользователь отвечает за разрешения, авторские права и условия сервисов.

## Архитектура

```text
React UI
   │ типизированные Tauri-команды + версионированные события
   ▼
Tauri adapter
   ▼
Rust application services ─► scheduler ─► process supervisor
          │                      │                 │
          ▼                      ▼                 ▼
    format policy            SQLite/WAL   yt-dlp / ffmpeg / aria2
          │                                        │
          └──────── secure credential broker ◄─────┘
```

SQLite — источник истины. React восстанавливается из snapshot и событий. Только
Rust-scheduler может резервировать слоты и менять устойчивое состояние задачи.

## Структура

```text
src/              React/TypeScript UI и три словаря
src-tauri/        Tauri shell и Rust/SQLite-команды очереди
docs/             требования, архитектура и roadmap в EN/RU/简体中文
package.json      frontend и Tauri CLI
README.*.md       локализованные точки входа
```

Выделение Rust crates для core/storage/process/engines произойдёт при добавлении
process supervisor и движков.

## Документация

- [Требования](docs/product-requirements.ru.md)
- [Архитектура](docs/architecture.ru.md)
- [Жизненный цикл задач](docs/task-lifecycle.ru.md)
- [Безопасность](docs/security.ru.md)
- [Упаковка](docs/packaging.ru.md)
- [Roadmap](docs/roadmap.ru.md)
- [Исследовательские ориентиры](docs/research-references.ru.md)
- [Локализация](docs/localization.ru.md)
- [Изменения](CHANGES.ru.md)
- [Память проекта](MEMORY.ru.md)

## Локальный запуск на macOS

Проверенные требования: Node.js 22+, npm 10+, Rust 1.88+.

```bash
npm install
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

База создаётся в системном каталоге данных приложения. При первом запуске язык —
английский; явный выбор пользователя сохраняется локально.

Следующий этап: fake engine и владение деревом процессов, затем реальный `yt-dlp`
probe и одна сквозная загрузка.

## Идентичность

- Продукт: **JiveFetch**
- Репозиторий: `jivefetch`
- Application ID: `top.jivejournal.jivefetch`
- Планируемый сайт: `fetch.jivejournal.top`

Лицензия будет выбрана после проверки обязательств распространяемых бинарников.
