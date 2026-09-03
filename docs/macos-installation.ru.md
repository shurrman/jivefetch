[English](macos-installation.md) | [Русский](macos-installation.ru.md) | [简体中文](macos-installation.zh-CN.md)

# Установка неподписанной сборки на macOS

## Область применения

JiveFetch `0.2.0` собран для Apple Silicon (`arm64`), но пока не подписан сертификатом
Apple Developer ID и не прошёл notarization. GitHub `Latest` означает самый новый
опубликованный выпуск, а не проверку приложения компанией Apple.

До появления подписи и notarization Gatekeeper может сообщать, что приложение
повреждено. Используйте следующую процедуру только для DMG JiveFetch, скачанного из
официального репозитория.

## Проверка загрузки

Скачайте `JiveFetch_0.2.0_aarch64.dmg` и `SHA256SUMS-macOS-ARM64.txt` из одного
релиза GitHub. Выполните в Terminal:

```bash
cd ~/Downloads
shasum -a 256 -c SHA256SUMS-macOS-ARM64.txt
```

Продолжайте, только если получен результат:

```text
JiveFetch_0.2.0_aarch64.dmg: OK
```

Для `0.2.0` ожидаемый SHA-256:
`7377fa07f247124ecd81388982d9360ba46db4f1e56ac419e7185384c73d9530`.

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

## Оставшееся ограничение

Обычная установка и запуск двойным щелчком без Terminal требуют Developer ID signing
и Apple notarization. Они намеренно отложены; в примечаниях к релизу текущие artifacts
явно обозначены как unsigned и unnotarized.
