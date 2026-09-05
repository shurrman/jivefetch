[English](macos-installation.md) | [Русский](macos-installation.ru.md) | [简体中文](macos-installation.zh-CN.md)

# Установка неподписанной сборки на macOS

## Область применения

Текущие preview-релизы JiveFetch собраны для Apple Silicon (`arm64`), но пока не
подписаны сертификатом Apple Developer ID и не прошли notarization. GitHub `Latest`
означает самый новый опубликованный выпуск, а не проверку приложения компанией Apple.

До появления подписи и notarization Gatekeeper может сообщать, что приложение
повреждено. Используйте следующую процедуру только для DMG JiveFetch, скачанного из
официального репозитория.

## Проверка загрузки

Скачайте `JiveFetch_<version>_aarch64.dmg` и `SHA256SUMS-macOS-ARM64.txt` из одного
релиза GitHub. Выполните в Terminal:

```bash
cd ~/Downloads
shasum -a 256 -c SHA256SUMS-macOS-ARM64.txt
```

Продолжайте, только если получен результат:

```text
JiveFetch_<version>_aarch64.dmg: OK
```

## Установка и подготовка

1. Откройте DMG и перетащите `JiveFetch.app` в `Программы` (`Applications`).
2. Извлеките DMG.
3. Выполните в Terminal:

```bash
codesign --force --deep --sign - /Applications/JiveFetch.app
xattr -dr com.apple.quarantine /Applications/JiveFetch.app
open /Applications/JiveFetch.app
```

Обычно этим командам не нужен `sudo`. Если macOS сообщает об ошибке доступа, удалите
эту копию и снова скопируйте приложение через Finder от имени текущего пользователя,
не меняя системные права целиком.

## Что делают команды

- `codesign ... --sign -` создаёт локальную ad-hoc подпись всего app bundle. Это не
  Apple identity и не доказательство подлинности издателя.
- `xattr ... com.apple.quarantine` снимает атрибут карантина только с этого приложения.
  Не применяйте эту команду к широким каталогам.
- `open` запускает подготовленное приложение.

После замены JiveFetch на новую неподписанную версию повторите проверку и подготовку.
Не выполняйте эти команды для приложения из другого источника или при несовпадении
checksum с manifest релиза.

## Разрешения конфиденциальности после замены

У каждой ad-hoc пересборки другая code identity. Поэтому после замены приложения macOS
может перестать связывать с ним прежнее решение для Downloads или Full Disk Access. Если
доступ всё ещё не работает, удалите старую запись JiveFetch из **Системные настройки →
Конфиденциальность и безопасность → Полный доступ к диску**, снова добавьте
`/Applications/JiveFetch.app`, полностью закройте приложение и откройте его заново. Если
macOS отдельно запросит доступ к папке Downloads, разрешите его.

Запрос Keychain к **Chrome Safe Storage** появляется только тогда, когда `yt-dlp`
действительно начинает читать и расшифровывать cookies Chrome. Его не будет при
отключённых cookies или если попытка раньше остановилась из-за движка, JavaScript runtime,
сети либо доступа к папке загрузок.

## Оставшееся ограничение

Обычная установка и запуск двойным щелчком без Terminal требуют Developer ID signing
и Apple notarization. Они намеренно отложены; в примечаниях к релизу текущие artifacts
явно обозначены как unsigned и unnotarized.
