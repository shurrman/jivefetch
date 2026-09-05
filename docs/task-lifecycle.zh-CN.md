[English](task-lifecycle.md) | [Русский](task-lifecycle.ru.md) | [简体中文](task-lifecycle.zh-CN.md)

# 任务生命周期与恢复

## 1. 为什么需要正式 lifecycle

UI 命令、crash recovery 和进程副作用必须收敛到一个持久状态；否则 Pause/Stop 只是
界面标志。

## 2. Task 与 attempt

Task 是持久 intent；attempt 是一次不可变 execution plan 及 owned process tree；
artifact 是归属于 task/attempt 的 partial 或 final file。一个 task 可以有多个历史
attempt，但最多一个 live attempt。

## 3. States

状态包括 `probing`、`queued`、`starting`、`downloading`、`postprocessing`、
`pausing`、`paused`、`stopping`、`stopped`、`waiting_retry`、`completed`、
`failed`、`interrupted`、`removed`。Foundation `0.2.0` 实现了 `queued/paused/stopped`
稳定子集，并为 runtime state 保留模型。

## 4. 转换概览

```text
probe -> queued -> starting -> downloading -> postprocessing -> completed
                    │             ├ pause -> pausing -> paused -> queued
                    │             ├ stop  -> stopping -> stopped
                    │             └ error -> waiting_retry / failed
startup old transient -> interrupted -> reconcile -> paused/queued/failed/completed
```

## 5. 命令语义

### Pause

先 commit intent，再 graceful interrupt，超时后终止 owned tree，检查 partial，
  最终进入无活进程的 `paused`。

### Resume

从 `paused` 或可恢复 `interrupted` 状态开始，必要时 re-probe，然后回到队列。

### Stop

关闭自动恢复，默认保留文件并进入 `stopped`。

### Retry

新建 attempt，保留旧历史。

### Remove

先达到稳定无进程状态，再 tombstone；删除文件是另一个有预览的明确操作，
  只能作用于 tracked canonical path，不能递归删除 destination。

所有命令幂等并检查 revision；非法 transition 由 backend 拒绝。

## 6. 持久 transition protocol

Scheduler transaction 验证 revision/slot，创建 attempt/plan，写入 `starting` 和 event，
commit 后再创建 ownership container 并 spawn。commit 后的 spawn failure 成为可见错误，
不会偷偷 rollback。Pause/Stop/Remove 同样遵守“副作用前持久 intent，副作用后协调结果”。

## 7. 启动恢复

启动时先打开/migrate DB、获取 single-instance scheduler lock、创建 `run_id`、一次性把旧
transient state 变为 `interrupted`、清理严格验证的 app-owned plaintext lease、检查
attempt/artifact、分类 partial、应用 recovery policy，最后才开始正常 dispatch。

不凭旧 PID attach。Windows 依靠 Job Object close，Unix orphan 只有在 ownership metadata
完整验证后处理。

## 8. Partial file 兼容性

Partial 重用要求 destination/template/source/format/engine fingerprint 兼容；不兼容数据
不能静默使用。

## 9. Progress 与 checkpoints

UI progress 合并，checkpoint 只在有界间隔和 phase boundary 持久化；单个组件的
100% progress 不等于完成。`0.4.0` UI 每五秒请求一次 authoritative snapshot。
`yt-dlp` 的独立视频与音频组件事件会聚合为单调递增的任务总字节数、progress 与 ETA；
副标题说明当前正在下载视频、音频或合并最终文件。下载与合并期间的 progress 保持在
100% 以下；只有经过验证的最终文件才会显示 100% 和绿色。
完成还要求配置目录内存在经过验证的非空普通文件。每次 snapshot 都会重新检查文件，
只有存在时才显示绿色并启用“打开”。Open 命令只接收任务 ID，由 Rust 解析并验证已记录
路径，再交给操作系统默认应用；webview 不能提供任意路径。Startup 仍会用同一个验证
文件修复旧 completed 任务的零字节指标。右键任务操作继续映射到 revision 检查的
Start/Stop/Pause/Remove 转换；Copy URL 不改变状态。

## 10. 必须测试的 concurrency races

必须测试 Pause+Stop、Pause 中 Remove、重复 Resume、commit/spawn 之间崩溃、process exit/
completed commit 之间崩溃、retry deadline 与 Stop、output conflict、带宽重平衡与完成竞态。
