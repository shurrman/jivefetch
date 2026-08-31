[English](research-references.md) | [Русский](research-references.ru.md) | [简体中文](research-references.zh-CN.md)

# Исследовательские ориентиры и отличия

## 1. Политика использования

Проверено: 2026-09-01. MediaHarbor и FlowGrab — публичные product/repository references,
не upstream. JiveFetch использует видимые идеи, но не копирует code, architecture,
assets, UI text, branding или docs. README-claims референса — leads, а не доказательства.

## 2. Источники

Источники: [MediaHarbor](https://github.com/MediaHarbor/mediaharbor),
[FlowGrab](https://github.com/HrshD1eux/Flowgrab_Downloader),
[yt-dlp](https://github.com/yt-dlp/yt-dlp), [Tauri](https://v2.tauri.app/),
[FFmpeg legal](https://ffmpeg.org/legal.html), [aria2](https://github.com/aria2/aria2).

## 3. Матрица idea-to-contract

| Идея | Контракт JiveFetch |
| --- | --- |
| Paste + preview | cancellable typed probe, stale suppression, normalized/expiring snapshot |
| Presets/formats | intent + live inventory, compatibility, explicit remux/transcode |
| Clipboard/deep link | opt-in, URL-only, review, dedupe, no auto-start |
| Batch queue | SQLite durability, bounded concurrency, aging, locks, retry/recovery |
| Progress/history | sequenced events, coalescing, checkpoints, attempts, reconnect snapshot |
| Pause/Resume/Stop | formal cross-platform lifecycle и verified owned-tree stop |
| Browser cookies | references или encrypted import с OS-held random key |
| Settings | versioned non-secret DB values, secret references, per-task overrides |
| Engine updater | signed manifest, hash, compatibility, immutable versions, rollback/SBOM |
| Parallel downloads | central slots/postprocess pool/fairness/idempotent leases |
| Rate limits | per-task + fair global budget с measurable ceiling |
| Installers | native tests/signing/notarization/clean-machine gates |

## 4. Намеренные отличия

### Durable queue, а не frontend list

Очередь, attempts и recovery живут в SQLite/Rust, поэтому restart не превращает UI в
источник истины.

### Честные controls

Pause/Resume/Stop имеют формальную portable semantics и проверяют завершение owned tree.

### Глобальная resource policy

Concurrency, post-processing slots и bandwidth распределяются центральным scheduler.

### Secrets вне обычного state

Browser references или encrypted import не допускают cookie values в UI/DB/logs.

### Проверяемая дистрибуция

Signed manifests, hashes, immutable versions, rollback, SBOM и native release gates.

## 5. Намеренно отложенные идеи

Player/library/lyrics, extensions, subscriptions, cloud и raw flags отложены до зрелости
core, чтобы не размывать process, durability и security contracts.

## 6. Периодичность пересмотра

Перед крупным UX milestone референсы проверяются заново только по публичным фактам.
Их код не импортируется в этот repo «для быстрого изучения».
