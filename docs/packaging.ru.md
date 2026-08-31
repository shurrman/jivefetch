[English](packaging.md) | [Русский](packaging.ru.md) | [简体中文](packaging.zh-CN.md)

# Кроссплатформенная упаковка и движки

## 1. Модель поставки

Два versioned слоя: подписанное Tauri-приложение и managed engines (`yt-dlp`, FFmpeg,
ffprobe, optional aria2c). Installer по возможности содержит pinned baseline;
verified overrides живут в app data и выбираются signed compatibility manifest.

## 2. Реестр движков

Registry выбирает: healthy managed override, bundled baseline, затем явно разрешённый
validated system executable. Для каждого хранит version/target/path identity/hash/
source/capabilities/licenses. Не предпочитать неизвестный `$PATH` tool молча.

## 3. Layout

App bundle содержит неизменяемый baseline, notices и manifest. App data содержит
versioned managed engines, staging и last-known-good metadata; user config хранит только
settings/references, а cache — безопасно удаляемые probes/thumbnails. Mutable engine не
устанавливается внутрь signed/sealed bundle.

## 4. Target matrix

| ОС | Архитектуры | Пакеты | Владение процессом |
| --- | --- | --- | --- |
| Windows | x86_64, arm64 после оценки | NSIS/MSI | Job Object kill-on-close |
| macOS | arm64/x86_64 | signed/notarized app/DMG | session/process group |
| Linux | x86_64, arm64 после оценки | AppImage, deb/rpm по необходимости | session/process group |

Поддержка заявляется только после native CI/smoke. Cross-compile не является доказательством.

## 5. Требования платформ

### Windows

WebView2 policy, code signing, native paths, early Job assignment, user ACL,
long/non-ASCII paths, AV latency и safe uninstall.

### macOS

Native/universal builds, nested signing, hardened runtime, notarization, mutable engines
вне sealed bundle, Gatekeeper/quarantine/keychain/sleep tests.

### Linux

WebKitGTK, Secret Service fail-closed, glibc targets, Wayland/X11, AppImage paths и
package upgrade tests.

## 6. Получение и обновление движков

Выбрать allowed target/version; скачать в staging с limits; проверить manifest signature
и hash; безопасно unpack без traversal/symlink escape; проверить names/permissions;
bounded `--version` probe; atomic move в immutable version; activate; сохранить previous;
rollback при failure. Не ставить вне compatibility range.

## 7. Licensing gate

До public build: inventory exact source/binary licenses; различать license исходников и
standalone bundles; записать FFmpeg configure; включить notices/source obligations;
проверить совместимость лицензии JiveFetch.

## 8. Reproducibility и supply chain

Pin toolchains/lockfiles/actions SHA,
минимальные CI permissions, SBOM, checksums, provenance; signing secrets вне repo/logs.

## 9. Release gates

Frontend/Rust checks; migration/crash suite; owned process tree; cookie secret tests;
engine discovery/rollback; installer lifecycle; signing/notarization; SBOM/licenses;
clean-machine probe/download/postprocess. Gate проходит отдельно на каждой заявленной ОС.
