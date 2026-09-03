[English](README.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md)

# JiveFetch

> Ваши медиа. Ваша очередь. Ваши правила.

JiveFetch — локальный кроссплатформенный desktop-менеджер законных медиазагрузок.
Целевой стек: Tauri 2, Rust, React/TypeScript, `yt-dlp`, `ffmpeg`/`ffprobe` и,
опционально, `aria2`.

Версия `0.2.0` — рабочий preview настроек и живой очереди: Tauri-оболочка, интерфейс
на английском, русском и упрощённом китайском и очередь SQLite под управлением Rust.
Можно добавить проверенную HTTP(S)-ссылку, загрузить через локальные `yt-dlp` и FFmpeg,
выбрать 1–10 или своё ограниченное число одновременных задач, задать общий бюджет
скорости, папку загрузок и системную/тёмную/светлую тему. Очередь показывает прогресс,
скорость, ETA, текущий и общий размер. Папка по умолчанию — `Загрузки/JiveFetch`.
Выпадающий список браузеров передаёт выбранный поддерживаемый идентификатор в
`yt-dlp --cookies-from-browser`; JiveFetch не копирует значения cookies.

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
src-tauri/        Tauri shell, scheduler, supervisor и Rust/SQLite
.github/workflows/ native-проверки Windows/macOS/Linux
docs/             требования, архитектура и roadmap в EN/RU/简体中文
package.json      frontend и Tauri CLI
README.*.md       локализованные точки входа
```

Первый срез держит scheduler, storage, engine adapter и process supervisor отдельными
Rust-модулями в `src-tauri`; целевые границы crates описаны в архитектуре.

## Документация

- [Требования](docs/product-requirements.ru.md)
- [Архитектура](docs/architecture.ru.md)
- [Жизненный цикл задач](docs/task-lifecycle.ru.md)
- [Безопасность](docs/security.ru.md)
- [Упаковка](docs/packaging.ru.md)
- [Лицензирование и сторонние движки](docs/licensing.ru.md)
- [Roadmap](docs/roadmap.ru.md)
- [Исследовательские ориентиры](docs/research-references.ru.md)
- [Локализация](docs/localization.ru.md)
- [Изменения](CHANGES.ru.md)
- [Примечания к выпуску v0.1.1](docs/releases/v0.1.1.ru.md)
- [Примечания к выпуску v0.2.0](docs/releases/v0.2.0.ru.md)

## Локальный запуск на macOS

Проверенные требования: Node.js 22+, npm 10+, Rust 1.88+, `yt-dlp` и FFmpeg.

```bash
npm install
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --test real_engine_smoke -- --ignored
npm run tauri dev
```

База создаётся в системном каталоге данных приложения. При первом запуске язык —
английский; явный выбор пользователя сохраняется локально.

Игнорируемый smoke-тест создаёт короткий fixture через FFmpeg, раздаёт его только на
loopback, загружает реальным Rust-scheduler и `yt-dlp`, затем проверяет выходной путь.
Публичный контент и внешняя сеть для теста не нужны.

## Идентичность

- Продукт: **JiveFetch**
- Репозиторий: `jivefetch`
- Приватный репозиторий: [github.com/shurrman/jivefetch](https://github.com/shurrman/jivefetch)
- Application ID: `top.jivejournal.jivefetch`
- Планируемый сайт: `fetch.jivejournal.top`

- Лицензия исходного кода: [Apache-2.0](LICENSE)
- Текущая модель движков: проверенные системные executables без их распространения
