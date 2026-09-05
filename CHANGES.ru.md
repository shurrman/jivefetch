[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# Изменения

## Не выпущено

## 0.4.0 - 2026-09-05

### Добавлено

- Единая модель общего progress задачи суммирует плановые размеры video/audio-компонентов,
  загруженные bytes, speed и ETA; текущий этап «Видео/Аудио/Объединение» локализован.
- Для completed-задач добавлено действие «Открыть». Rust получает путь по task ID, заново
  проверяет непустой canonical output внутри настроенной папки и открывает его приложением
  ОС по умолчанию; webview не передаёт filesystem path.
- Добавлены structured local JSON diagnostics со стабильными task/attempt/error fields,
  лимитом 2 MiB и одной rotation без raw engine lines, URL, cookies и paths.
- Добавлены узкие engine/process-spawner traits и scheduler test doubles.

### Изменено

- Внутренние `Result<T, String>` заменены typed validation/storage/engine/scheduler/input
  errors; стабильные локализованные строки остаются только на Tauri IPC boundary.
  `AppSettings` сам проверяет себя, browser cookies представлены typed allowlist.
- Discovery бинарников отделён от `yt-dlp` execution при сохранении одного scheduler и
  одного RAII owner для каждого supervised process tree.
- Сообщение о локальном контроле и готовности движков перенесено одной компактной строкой
  в header с зелёным/красным indicator; URL, format picker и добавление в очередь сохранены.
- Unsigned и unnotarized сборки публикуются как GitHub pre-release до прохождения
  остальных production release gates.

### Исправлено

- Событие 100% отдельного видеопотока перед аудиопотоком больше не выглядит как завершение
  и повторный старт. Component progress монотонно агрегируется, active work остаётся ниже
  100%, и только проверенное завершение достигает 100%.
- Completed-задача зелёная и предлагает «Открыть» только пока итоговый файл реально
  существует; удалённый или пустой файл становится output error при следующем refresh.
- Legacy-settings migration test использует нативный временный absolute path, поэтому
  validation contract одинаково проверяется на Windows, macOS и Linux.

### Метрики

- SQLite schema: версия 5 -> 6 с проверенной in-place migration `download_stage`.
- Обычный Rust suite: 22 -> 26 tests плюс проходящий opt-in real-engine loopback smoke.
- Error boundary: string codes по всему Rust -> 5 typed error families и одно string
  mapping на Tauri IPC.
- Progress model: один перезаписываемый per-component snapshot -> один монотонный aggregate
  по плановым компонентам и 4 локализованных stage labels на каждом языке приложения.

## 0.3.1 - 2026-09-05

### Добавлено

- Полное руководство по установке для macOS, Windows, Ubuntu/Debian, Fedora,
  Arch Linux, AppImage и DEB на английском, русском и упрощённом китайском. Оно описывает
  установку и проверку `yt-dlp`, FFmpeg и Deno, сверку checksum JiveFetch, первый запуск и
  обновление движков.
- Требование установить движки для кроссплатформенного приложения и ссылка на руководство
  теперь заметны в самом начале каждого перевода README.
- Добавлен нейтральный для всех языков README hero: пользовательская очередь, состояния
  прогресса, медиаформаты и работа на разных устройствах без стороннего branding.
- Каждый README перестроен в пользовательском порядке: сначала результат и возможности,
  затем требования, пакеты для платформ и ссылки на установку.
- Версия JiveFetch добавлена рядом с версиями `yt-dlp` и FFmpeg в верхней строке статуса.

### Исправлено

- Переключатель языка релиза теперь ведёт на страницу конкретного tag и anchor секции,
  например `/releases/tag/v0.3.0#russian`, а не на anchor общей ленты релизов. Переходы
  работают, и Assets остаются доступны.
- Инструкция для неподписанного macOS-приложения больше не привязана к версии, поэтому
  после публикации нового Latest в ней не остаются устаревшие имя DMG и checksum.
- Общие ошибки `yt-dlp` заменены локализованными и полезными категориями: доступ к
  cookies браузера, авторизация, недоступное медиа, rate limit, формат, сеть и права
  файловой системы. Сырой вывод движка не сохраняется.
- Описаны Full Disk Access и доступ к Keychain, нужные текущей неподписанной macOS-сборке
  для чтения cookies Chrome, а также безопасный режим для публичных медиа и Firefox как
  временная альтернатива.
- Проверка версий движков перенесена из защищённой папки Downloads в собственный app-data
  каталог. Это устраняет startup race macOS TCC, из-за которого Homebrew `yt-dlp` мог
  показываться отсутствующим одновременно с найденным FFmpeg.
- В очищенный `PATH` загрузчика добавлены стандартные каталоги пакетных менеджеров и
  пользовательский каталог Deno. Запущенная из Finder macOS-сборка теперь передаёт
  Homebrew Deno в `yt-dlp`, поэтому YouTube не отвечает HTTP 403 после успешной проверки
  метаданных только из-за отсутствующего JavaScript runtime.
- HTTP 403 выделен в отдельную локализованную ошибку с советом обновить `yt-dlp`,
  установить Deno или включить cookies браузера для закрытого медиа.

### Метрики

- Документация: с 48 до 54 отслеживаемых Markdown-файлов и с 16 до 18 проверяемых
  комплектов EN/RU/简体中文.
- Суммарный объём README: с 389 до 93 строк; архитектура и детали разработки остаются
  в отдельных документах, а не на входной странице продукта.
- Навигация релиза: 3 неоднозначных относительных anchor общей ленты заменены на 3 явных
  anchor страницы tag; это проверяет release-note checker.
- Индикаторы версий сверху: с 2 версий движков до 1 версии приложения и 2 версий движков.
- Пользовательские категории ошибок движка: с 1 общей до 8 конкретных локализованных
  категорий плюс общий fallback; обычных Rust-тестов: с 19 до 22.

## 0.3.0 - 2026-09-03

### Добавлено

- Живой cookie-aware JSON probe `yt-dlp` и выбор для каждой задачи из реальных
  видеоформатов источника. Picker показывает разрешение, FPS, битрейт источника, кодек,
  контейнер и оценку размера, по умолчанию выбирает максимум и скрывает внутренние ID
  и похожие на дубликаты строки без оценки размера.
- Собственное меню задачи по правой кнопке: «Запустить», «Остановить», «Пауза»,
  «Копировать URL» и «Удалить»; недоступные переходы видны, но отключены.
- Выбранный проверенный format selector сохраняется в SQLite schema version 5 и
  передаётся движку только как типизированный argument vector.

### Исправлено

- Явно включён `yt-dlp --progress`, поэтому вывод финального пути больше не подавляет
  события текущего и общего размера, скорости, ETA и progress.
- В карточках сохранены несекретные query-параметры URL с маскированием похожих на
  секреты значений: разные YouTube-задачи `/watch?v=…` снова различимы.
- Completion требует проверенный непустой output file; его реальный размер сохраняется,
  а старые нулевые метрики completed-задач исправляются при запуске.
- После выхода быстрого version-процесса приложение дожидается drain вывода, поэтому при
  чистом desktop-запуске больше не возникает случайное ложное «yt-dlp не найден».

### Метрики

- Было (`v0.2.0`): 140 файлов, 48 Markdown-документов, 4 029 строк application/check
  source и 14 обычных Rust-тестов плюс opt-in real-engine smoke.
- Стало: 141 файл, 48 Markdown-документов в 16 комплектах EN/RU/简体中文, 4 975 строк,
  19 обычных Rust-тестов и проходящий real-engine smoke.

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
- Исправлены опубликованные ссылки на переводы/документацию `v0.2.0`; release workflow
  теперь преобразует относительные ссылки в стабильные абсолютные URL конкретного tag.
- Удалён неполный draft-релиз `v0.1.0`, `v0.2.0` назначен GitHub `Latest` без пересборки
  artifacts, а следующие успешно опубликованные релизы будут становиться `Latest`
  автоматически. Signing и notarization остаются отдельными production gate.
- Добавлена инструкция EN/RU/简体中文 по установке текущего неподписанного приложения
  на macOS: сначала checksum, затем локальная ad-hoc подпись и точечное снятие quarantine.
- Переводы release notes встроены в одну страницу GitHub Release с внутренней навигацией:
  при выборе языка пользователь остаётся рядом с общим блоком Assets.

### Метрики

- Было: 135 файлов, 45 Markdown-документов в 15 translation sets, 3 262 строки
  application/check source и 8 обычных Rust-тестов.
- Стало: 135 файлов, 42 Markdown-документа в 14 translation sets, 4 075 строк,
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
  `GHSA-wrw7-89jp-8q8g`; проблемный API не используется, но advisory блокирует полностью
  поддерживаемый production-релиз.
