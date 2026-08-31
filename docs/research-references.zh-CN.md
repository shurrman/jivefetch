[English](research-references.md) | [Русский](research-references.ru.md) | [简体中文](research-references.zh-CN.md)

# 研究参考与差异化

## 1. 使用政策

最后检查：2026-09-01。MediaHarbor 和 FlowGrab 是公开 product/repository reference，
不是 upstream。JiveFetch 可以学习公开工作流和创意，但不复制代码、架构、资源、UI 文案、
品牌或文档。其他 README 的功能声明只是研究线索，不是行为证明。

## 2. 来源

来源：[MediaHarbor](https://github.com/MediaHarbor/mediaharbor)、
[FlowGrab](https://github.com/HrshD1eux/Flowgrab_Downloader)、
[yt-dlp](https://github.com/yt-dlp/yt-dlp)、[Tauri](https://v2.tauri.app/)、
[FFmpeg legal](https://ffmpeg.org/legal.html)、[aria2](https://github.com/aria2/aria2)。

## 3. Idea-to-contract 矩阵

| 公开好创意 | JiveFetch 更严格的合同 |
| --- | --- |
| 粘贴并预览 | 可取消 typed probe、旧结果抑制、normalized/expiring snapshot |
| preset/format | intent + 实时清单、兼容说明、显式 remux/transcode |
| clipboard/deep link | opt-in、URL-only、review、dedupe、绝不自动启动 |
| batch queue | SQLite 持久化、有界并发、aging、lock、retry/recovery |
| progress/history | sequenced event、coalescing、checkpoint、attempt、reconnect snapshot |
| Pause/Resume/Stop | 正式跨平台 lifecycle 与验证过的 owned-tree stop |
| Browser Cookie | reference，或使用 OS-held random key 的加密导入 |
| Settings | 版本化非秘密值、secret reference、per-task override |
| Engine updater | signed manifest、hash、compatibility、immutable version、rollback/SBOM |
| 并行与限速 | central slot/postprocess pool/fairness，加 per-task 与 global budget |
| 安装包 | native test、签名/公证、clean-machine gate |

## 4. 有意的差异化

### 持久队列，而非 frontend list

Queue、attempt 和 recovery 属于 SQLite/Rust，因此 restart 后 UI 不是事实来源。

### 诚实的 controls

Pause/Resume/Stop 有正式跨平台语义，并验证 owned process tree 已结束。

### 全局资源 policy

Concurrency、post-processing slot 和 bandwidth 都由中央 scheduler 分配。

### 秘密不进入普通 state

Browser reference 或加密导入保证 Cookie 值不进入 UI、DB 和日志。

### 可验证分发

使用 signed manifest、hash、immutable version、rollback、SBOM 和 native release gate。

## 5. 有意延后的创意

Player/library/lyrics、extension、subscription、cloud 和 raw flag 在 core 成熟前延后，
避免削弱 process、durability 和 security contract。

## 6. 审查周期

每个重大 UX milestone 前可以重新检查公开事实，但不得把参考代码导入仓库作为捷径。
