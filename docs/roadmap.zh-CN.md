[English](roadmap.md) | [Русский](roadmap.ru.md) | [简体中文](roadmap.zh-CN.md)

# 路线图

阶段按证据而非日期完成；后续 UI polish 不能跳过前面的 durability/security gate。

## Phase 0 — 可复现基础

### Deliverables

Tauri 2 + React/TS + Rust、固定 lockfile/toolchain、format/lint/test/link/secret check、
原生 CI 骨架和 ADR。`0.1.0` 已有可运行 macOS UI、SQLite 队列、三种语言、Apache-2.0、
unit/build 检查和 native CI matrix；首次 remote matrix 已在 macOS/Linux/Windows
通过，ADR 和 crate 拆分仍待完成。

### Exit criteria

Clean checkout 可重复构建并运行，决策和检查均已记录。

## Phase 1 — Domain 与存储

### Deliverables

纯 Rust task/attempt/artifact/error transition、完整 migration/event/idempotency、snapshot/
pagination。Exit：property test、重复命令安全、migration fixture、10k history pagination。

### Exit criteria

上述 property、idempotency、migration 和 pagination 检查全部通过。

## Phase 2 — Fake engine 与进程所有权

### Deliverables

平台 supervisor、child/grandchild helper、graceful/forced stop、scheduler slot/aging/lock、
全部 control。Exit：完整 owned tree 消失，同名无关进程存活，竞态收敛，崩溃边界可恢复。

现已实现 process-group/Job-Object supervisor、bounded output、双 scheduler slot、durable control
和 unrelated-process test；priority、aging、output lock 与完整 crash matrix 仍待完成。

### Exit criteria

Owned tree 全部消失、无关进程存活，竞态和 crash recovery 收敛。

## Phase 3 — yt-dlp 纵向闭环

### Deliverables

Verified engine registry、probe/normalized format、format compiler、intake/picker、一次真实
queued download、FFmpeg capability error。Exit：fixture suite、clean-machine 合法测试下载、
无 shell 字符串。

现已实现 system engine discovery、typed real plan、progress parser、output verification、UI
progress 和 loopback smoke；metadata probe、format selection 与 ffprobe capability 仍待完成。

### Exit criteria

Fixture suite 和 clean-machine test 通过，不存在 shell string。

## Phase 4 — 并行队列与恢复

### Deliverables

网络/后处理 pool、checkpoint/retry/artifact fingerprint、startup reconcile、batch/reorder/
history、磁盘和 collision policy。Exit：hard-kill matrix、无重复/path conflict、单项失败不
阻塞队列。

### Exit criteria

Hard-kill matrix 不产生 duplicate/path conflict，失败任务不阻塞队列。

## Phase 5 — 认证与带宽

### Deliverables

Browser reference、加密导入 Cookie、per-task/global 公平限速、resumable rebalance、
sentinel redaction。Exit：各 target auth、无秘密泄漏、aggregate ceiling 达标、partial 不丢失。

### Exit criteria

各 target auth 可用、无秘密泄漏、ceiling 达标、partial 不丢失。

## Phase 6 — 产品级桌面 UX

### Deliverables

可访问页面、opt-in clipboard/deep link、notification/folder、清晰 phase/error、virtualization/
thumbnail/theme。Exit：keyboard/screen-reader smoke、外部输入不自动启动、10k history 流畅。

### Exit criteria

Accessibility smoke 通过，外部输入不自动启动，10k history 保持流畅。

## Phase 7 — Engine 管理与打包

### Deliverables

Signed manifest/hash/health/rollback、license/SBOM/provenance、签名 Windows、已公证 macOS、
Linux package、installer test。Exit：篡改拒绝、自动 rollback、每 target gate、显示精确版本许可。

### Exit criteria

篡改拒绝和 rollback 已验证，每个 target gate 通过，版本与许可证可见。

## Phase 8 — MVP 加固

### Deliverables

长队列、弱网络、低磁盘、sleep/wake、fuzz、capability/security review、冻结合同与用户文档。
Exit：需求场景通过，无 critical/high security 或 data-loss 问题，无 orphan tree，发布 artifact+
signature+SBOM。

### Exit criteria

需求场景通过，无高风险问题或 orphan tree，release 产物完整。

## Post-MVP candidates

Post-MVP：extension、library/player、subscription、plugin、remote/cloud、transport broker、
受约束的 expert policy language。

## 每个 phase 的 Definition of done

每阶段都要求 EN/RU/简体中文同步、相关测试、failure/restart path、无秘密 fixture、diff/link
check 和 CHANGES 指标。
