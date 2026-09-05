[English](INSTALL.md) | [Русский](INSTALL.ru.md) | [简体中文](INSTALL.zh-CN.md)

# Установка JiveFetch

JiveFetch работает на macOS, Windows и Linux, но текущая preview-версия не включает
движки загрузки. Сначала установите `yt-dlp`, FFmpeg и Deno, затем установите JiveFetch.
Deno — рекомендуемый `yt-dlp` JavaScript runtime для актуальной поддержки YouTube.
Регулярно обновляйте эти инструменты: поддерживаемые сайты со временем меняются.

Используйте JiveFetch только для медиа, которые вам разрешено скачивать. Приложение не
обходит DRM и другие средства контроля доступа.

## Поддерживаемые пакеты релиза

Текущий release pipeline выпускает следующие preview-пакеты:

| Операционная система | Архитектура | Пакет |
| --- | --- | --- |
| macOS | Apple Silicon (`arm64`) | DMG |
| Windows | 64-битная (`x64`) | NSIS EXE или MSI |
| Linux | 64-битная (`x86_64`/`amd64`) | AppImage или DEB |

Установочные пакеты JiveFetch для других архитектур пока не публикуются.

## 1. Установите yt-dlp, FFmpeg и Deno

### macOS

Установите [Homebrew](https://brew.sh/), если его ещё нет, затем откройте Terminal:

```bash
brew install yt-dlp ffmpeg deno
```

### Windows

Откройте PowerShell и используйте Windows Package Manager:

```powershell
winget install --id yt-dlp.yt-dlp --exact
winget install --id Gyan.FFmpeg --exact
winget install --id DenoLand.Deno --exact
```

Закройте PowerShell и откройте его снова, чтобы применился обновлённый `PATH`.

### Ubuntu или Debian

Установите FFmpeg из дистрибутива, а официальный release-бинарник `yt-dlp` — в системный
путь, доступный desktop-приложениям:

```bash
sudo apt update
sudo apt install ffmpeg curl
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /tmp/jivefetch-yt-dlp
sudo install -m 0755 /tmp/jivefetch-yt-dlp /usr/local/bin/yt-dlp
curl -fsSL https://deno.land/install.sh | sh
```

### Fedora

```bash
sudo dnf install ffmpeg-free curl
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /tmp/jivefetch-yt-dlp
sudo install -m 0755 /tmp/jivefetch-yt-dlp /usr/local/bin/yt-dlp
curl -fsSL https://deno.land/install.sh | sh
```

### Arch Linux

```bash
sudo pacman -Syu yt-dlp ffmpeg deno
```

Для другого дистрибутива Linux установите FFmpeg его пакетным менеджером, выполните
официальную инструкцию установки `yt-dlp` и установите Deno по официальной инструкции.
Инструменты должны быть в `PATH` desktop-сессии, стандартных системных путях либо в
каталоге Deno по умолчанию `$HOME/.deno/bin`.

## 2. Проверьте движки

Откройте новое окно Terminal или PowerShell и выполните:

```text
yt-dlp --version
ffmpeg -version
deno --version
```

Все три команды должны вывести версию. Если какая-либо команда не найдена, завершите
установку соответствующего инструмента и перезапустите как терминал, так и JiveFetch.

## 3. Скачайте и проверьте JiveFetch

Откройте [последний релиз JiveFetch](https://github.com/shurrman/jivefetch/releases/latest),
а затем из блока **Assets** скачайте пакет для своей ОС и соответствующий файл
`SHA256SUMS-*.txt`.

- macOS: в каталоге загрузок выполните
  `shasum -a 256 -c SHA256SUMS-macOS-ARM64.txt`.
- Linux: выполните `sha256sum JiveFetch_*_amd64.deb` или
  `sha256sum JiveFetch_*_amd64.AppImage` и сравните результат с соответствующей строкой
  в `SHA256SUMS-Linux-X64.txt`.
- Windows: выполните
  `Get-FileHash (Get-ChildItem .\JiveFetch_*_x64-setup.exe).FullName -Algorithm SHA256`
  либо аналогичную команду для `*_x64_en-US.msi` в PowerShell и сравните результат с
  соответствующей строкой в `SHA256SUMS-Windows-X64.txt`.

Не продолжайте установку, если checksum отличается.

## 4. Установите приложение

### macOS

Откройте DMG и перетащите `JiveFetch.app` в `Applications`. Текущий DMG не подписан и не
notarized, поэтому Gatekeeper может назвать приложение повреждённым. После проверки
checksum выполните [точечную инструкцию для неподписанного macOS-приложения](docs/macos-installation.ru.md).

### Windows

Запустите либо установщик `x64-setup.exe`, либо пакет `x64_en-US.msi`. Текущие
preview-установщики не подписаны, поэтому Windows может показать предупреждение издателя
или SmartScreen. Продолжайте только для пакета из этого репозитория с проверенным checksum.

### Linux

Для DEB-пакета:

```bash
sudo apt install ./JiveFetch_*_amd64.deb
```

Для AppImage:

```bash
chmod +x JiveFetch_*_amd64.AppImage
./JiveFetch_*_amd64.AppImage
```

## 5. Первый запуск

В верхней части JiveFetch убедитесь, что показаны версия приложения и реальные версии
`yt-dlp` и FFmpeg.
Если приложение сообщает, что движок не найден, полностью закройте JiveFetch, проверьте
движок в новом терминале и снова запустите приложение. По умолчанию файлы сохраняются в
системный каталог загрузок, в подпапку `JiveFetch`; каталог меняется в верхних настройках.
При первом запуске выбран английский язык, там же можно выбрать другой.

### Cookies браузера в текущей неподписанной сборке macOS

Если публичный URL работает в режиме **Не использовать cookies браузера**, но выдаёт
ошибку с выбранным Chrome, macOS не позволяет контексту неподписанного приложения
прочитать или расшифровать cookies Chrome. Добавьте `/Applications/JiveFetch.app` в
**Системные настройки → Конфиденциальность и безопасность → Полный доступ к диску**,
полностью закройте JiveFetch и откройте снова. Если macOS запросит доступ JiveFetch к
**Chrome Safe Storage** в Связке ключей, разрешайте его только для копии с проверенным
checksum.

Для публичных медиа оставьте cookies отключёнными. Для медиа с авторизацией временной
альтернативой может быть Firefox с той же активной учётной записью. Устойчивое решение
для macOS — JiveFetch с Developer ID-подписью и notarization; текущая preview-сборка их
ещё не имеет.

## Обновление движков

- Homebrew: `brew upgrade yt-dlp ffmpeg deno`
- Windows Package Manager: `winget upgrade --id yt-dlp.yt-dlp --exact` и
  `winget upgrade --id Gyan.FFmpeg --exact`
- Deno через Windows Package Manager: `winget upgrade --id DenoLand.Deno --exact`
- Официальный standalone `yt-dlp`: `sudo yt-dlp -U`
- Пакеты дистрибутива: обновляйте их через пакетный менеджер дистрибутива.

После обновления движка перезапустите JiveFetch, чтобы приложение увидело новую версию.

## Официальные источники

- [Установка `yt-dlp`](https://github.com/yt-dlp/yt-dlp/wiki/Installation)
- [Инструкция `yt-dlp` по внешнему JavaScript runtime](https://github.com/yt-dlp/yt-dlp/wiki/EJS)
- [Установка Deno](https://docs.deno.com/runtime/getting_started/installation/)
- [Загрузки FFmpeg и ссылки на пакеты](https://ffmpeg.org/download.html)
- [Формула Homebrew для `yt-dlp`](https://formulae.brew.sh/formula/yt-dlp)
- [Формула Homebrew для FFmpeg](https://formulae.brew.sh/formula/ffmpeg)
- [Документация Windows Package Manager](https://learn.microsoft.com/windows/package-manager/winget/)
