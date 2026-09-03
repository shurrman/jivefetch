[English](security.md) | [Русский](security.ru.md) | [简体中文](security.zh-CN.md)

# Безопасность и аутентификация

## 1. Цели безопасности

JiveFetch запускает внешние tools на untrusted URLs и временно использует browser auth.

Цель — минимизировать authority каждого компонента и не допускать утечки auth material.

## 2. Threat model

Защита нужна от malicious metadata/URL/output, shell injection, path traversal,
secret leakage, чтения temp другим local user, tampered update, remote IPC и убийства
unrelated process. Полностью compromised user account/kernel/browser вне MVP model.

## 3. Режимы authentication

### 3.1 Browser reference

Browser mode хранит browser kind и validated profile reference, передавая движку
`--cookies-from-browser`; cookie values не копируются.
В `0.2.0` сохраняется только один из документированных yt-dlp identifiers: Brave, Chrome,
Chromium, Edge, Firefox, Opera, Safari, Vivaldi или Whale. Profile/keyring пока не вводятся.

### 3.2 Импортированный cookie file

Imported Netscape cookies: validation, encryption random DEK, encrypted blob в app
data, DEK в OS credential service, user-only temp lease, cleanup после process tree и
при startup. Если secure store отсутствует, persistent import fail-closed; plaintext
или machine-derived fallback запрещён.

### 3.3 Будущие credentials

Новый password/token flow требует scoped broker contract, secure-store policy и
redaction tests; raw credential fields не добавляются.

## 4. Правила secret data

Cookies, passwords, tokens, auth headers, signed query values и decrypted temp data
секретны. Redaction выполняется до persistence. Support bundle строится allowlist и не
включает DB целиком, blobs, profiles, raw args/env или media.

## 5. Граница выполнения процессов

- Direct executable + argument vector, verified registry, minimal env, no shell.
- Bounded untrusted stdout/stderr, escaped rendering, no arbitrary extra executable/env.

## 6. Граница filesystem
- Canonical destination, sanitized template, запрет traversal/device names/ADS/control
  chars и symlink escape; explicit collision policy.
- Remove-with-files удаляет только tracked canonical artifacts, не directory/glob.

## 7. Граница Tauri/webview

Минимальные capabilities per window, Rust validation size/scheme, packaged local UI,
strict CSP без remote scripts, constrained thumbnail path, запрет privileged navigation,
external links через OS confirmation, никаких secrets в React/storage/clipboard/logs.

## 8. Обновления engine и приложения

Pinned compatibility, declared upstream, signed manifest, hash, staging, bounded health
check, atomic activation и rollback. Не выполнять partial/unverified binary. App update
и engine update имеют разные trust paths.

## 9. Privacy defaults

Telemetry отсутствует; history можно очистить без media; clipboard opt-in и URL-only;
deep links открывают review; auth scope видим пользователю.

## 10. Проверка безопасности

Injection, fuzz path/redaction, temp ACL/permissions на каждой ОС, sentinel scans DB/logs/
support bundle, Tauri remote-capability denial, tampered update/downgrade/rollback и
owned-process tests. Insecure fallback блокирует release.

## 11. Отслеживаемый upstream advisory

Linux dependency graph получает `glib 0.18.5` через Tauri и GTK 0.18. Версия затронута
`GHSA-wrw7-89jp-8q8g` в `VariantStrIter`; JiveFetch этот API не вызывает. Исправленный
`glib 0.20` несовместим с текущим ограничением GTK `^0.18`, поэтому исключение временно
разрешено для приватной preview-линейки до `v0.2.0` включительно. Обновление цепочки Tauri/GTK или
иное устранение advisory остаётся gate для stable-релиза.
