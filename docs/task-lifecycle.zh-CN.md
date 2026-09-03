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
`failed`、`interrupted`、`removed`。Foundation `0.1.0` 实现了 `queued/paused/stopped`
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

UI progress 合并，checkpoint 只在有界间隔和 phase boundary 持久化；
100% progress 不等于完成。

## 10. 必须测试的 concurrency races

必须测试 Pause+Stop、Pause 中 Remove、重复 Resume、commit/spawn 之间崩溃、process exit/
completed commit 之间崩溃、retry deadline 与 Stop、output conflict、带宽重平衡与完成竞态。
