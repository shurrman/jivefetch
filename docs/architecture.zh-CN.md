[English](architecture.md) | [Русский](architecture.ru.md) | [简体中文](architecture.zh-CN.md)

# 架构

## 1. 架构驱动因素

JiveFetch 是长时间运行且可能失败的外部进程之上的桌面 UI。核心难点是持久调度、
进程所有权、秘密处理，以及崩溃后仍能真实恢复 UI 状态。

设计分为四个边界：Domain 管理任务、尝试、状态、格式、限制和错误；Application
管理 scheduler、恢复、探测、带宽和命令；Infrastructure 包含 SQLite、文件系统、
keychain、时钟和平台进程；Adapters 提供 Tauri IPC/events 与 engine 集成。

Domain/Application 必须能脱离 webview 和真实下载引擎测试。`0.1.0` 在 `src-tauri`
内保留明确的 `model`、`storage`、`scheduler`、`engine` 和 `process_supervisor` 模块；
后续 crate 拆分将遵循这些已测试接口，而不是创建空边界。

## 2. 目标组件模型

```text
React UI ─ typed commands/events ─ Tauri adapter
                                      │
                         application services/query projection
                                      │
                            persistent Rust scheduler
                         ┌────────────┼────────────┐
                      SQLite   process supervisor  credential broker
                                      │
                               yt-dlp/ffmpeg/aria2
```

## 3. 规划中的 workspace

```text
src/                               React/TypeScript 应用
src-tauri/                         Tauri bootstrap、commands、events、capabilities
crates/jivefetch-core/             纯 domain model 和 scheduler policies
crates/jivefetch-storage/          SQLite migrations 和 repositories
crates/jivefetch-process/          平台进程所有权与监管
crates/jivefetch-engines/          engine plans、parsers 和 capability detection
tests/helpers/process-tree/        child/grandchild 测试程序
tests/fixtures/                     已脱敏 engine output 和 migration fixtures
```

依赖向内：`src-tauri` 依赖 adapters/core，infrastructure crates 实现 core traits；
`jivefetch-core` 不依赖 Tauri、SQLite 或 process API。

## 4. 命令与事件契约

Tauri 命令表达用户意图：probe、单个/批量 enqueue、pause/resume/stop/retry/remove、
reorder、limits 和 snapshot。每个写命令包含 idempotency key 与预期 revision。
Rust service 校验并提交转换后才发送 versioned event；冲突返回当前 revision。

事件在数据库实例内有单调 sequence number。发现缺口或 webview 重连时 React 重新
请求 snapshot，不猜状态。高频 progress 会合并，durable checkpoint 仅按有限频率
和 phase 边界写入。

## 5. 持久化模型

SQLite 启用 foreign keys、WAL、busy timeout、migrations 和明确 durability policy，
且只有应用拥有写路径。

| 表 | 用途 |
| --- | --- |
| `tasks` | 用户意图、state、revision、priority 和时间戳 |
| `attempts` | 一次执行、engine plan、owner run、结果和 diagnostics |
| `probes` | 已脱敏 metadata、formats、engine version 和 expiry |
| `artifacts` | Canonical paths、角色、完整性、大小和 attempt owner |
| `task_events` | 带单调序号的 durable state/audit events |
| `settings` | 非秘密设置和 credential references |
| `idempotency_keys` | 命令去重及既有结果 |
| `schema_migrations` | 已执行 migrations 和 checksums |

原始 cookies/credentials 永不写入 SQLite；probe payload 在保存前移除敏感 header
和 signed URL。Task state 与 event 在同一 transaction 内写入。Attempt ownership
使用 `run_id` 和平台 metadata；PID 单独不能证明所有权。

## 6. Scheduler

Scheduler 是 dispatch/task state 的唯一权威，通过 channel 接收命令和进程事件，
应用 policy、提交变化并安排下一次 tick。

### 6.1 Dispatch 不变量

- 任务只能从稳定且允许的状态 dispatch。
- Slot reservation 与 `starting` 转换原子完成。
- Engine plan 在 spawn 前保存，一次 attempt 内不可变。
- 每个任务最多一个 live attempt。
- 输出路径冲突的任务不能并行。
- Download 与 post-processing 使用独立 pool。
- Priority 与 queue age 组合，防止 starvation。

### 6.2 重试策略

错误被分类。认证、无效输入、不支持格式、权限和磁盘已满需要用户操作；timeout、
指定网络错误、transient extractor failure 和受管 engine crash 才以有界指数退避和
jitter 重试。每次 retry 都是新 attempt，绝不覆盖历史。

### 6.3 带宽策略

```text
effective(task) = min(task_limit_or_infinity, fair_share_of_global_limit)
```

首版向 engine 传固定 cap。活动成员或设置明显变化时，scheduler 通过
pause/checkpoint 和可续传的新 attempt 重新平衡；变化需 debounce。以后可增加本地
transport broker，但正确 task semantics 不依赖它。

## 7. Engine 抽象

Scheduler 不拼 shell command。Adapter 实现 `probe`、`plan`、`spawn`、`parse` 和
`classify`。`ExecutionPlan` 包含 verified executable、参数向量、脱敏显示、environment
allowlist、工作目录、预期 artifacts 和 capabilities，不含 shell syntax。

`yt-dlp` adapter 管理 extractor/format；直接本地处理以后可有独立 `ffmpeg` adapter。
`aria2` 仅在能力和协议匹配时启用。优先解析 versioned JSON/progress template；文本
解析被隔离、以 fixtures 测试，并视为不可信输入。

## 8. 进程监管

有效工作前，macOS/Linux attempt 创建新 session/process group；Windows 使用
terminate-on-close Job Object，并防止 descendants 在分配前逃逸。

Supervisor 异步排空 stdout/stderr、限制内存、记录脱敏 diagnostics，并发送
heartbeat/progress。Stop 依次持久写入 `stopping`、请求 graceful termination、有限
等待、终止 owned group/job、验证无成员、reconcile artifacts、提交稳定结果。
崩溃后旧 attempt 视为 uncertain；JiveFetch 不会只凭复用的 PID 接管进程。

## 9. Credential broker

Scheduler 请求短时 `CredentialLease`。Browser mode 返回非秘密 selector；imported
cookie mode 从 OS credential store 取密钥，把 blob 解密到权限严格的临时文件，只
提供路径和 cleanup guard。Process tree 退出后释放 lease。启动时仅清理路径和
ownership metadata 已验证的应用临时明文文件。

完整边界见[安全](security.zh-CN.md)。

## 10. Frontend 架构

React 保存 intake、queue、task details 和 settings 的 normalized read model。命令可先
显示 pending，但只有 backend 确认 durable revision 后才完成状态。

- typed IPC bindings 与 Rust DTO 一致；
- history 分页，大列表虚拟化；
- 控件支持键盘和辅助技术；
- 明确显示 phase 与 pending command；
- thumbnail 使用无 Tauri privilege 的受限 fetch/cache path；
- webview state、browser storage 和 developer log 中没有秘密。

## 11. 可观测性

日志只保存在本地，结构化、有界，并在 ingestion 时脱敏。关联字段含 task ID、
attempt ID、run ID、event sequence、engine/version 和 phase；排除 cookie、authorization、
原始 signed URL 和明文 secret path。

MVP metrics 也只在本地：queue depth、dispatch latency、active count、throughput、
retry、stop latency 和 recovery outcome。未来 telemetry 必须另行设计并 opt-in。

## 12. 测试架构

- 状态机合法转换和 command idempotency 的 property tests。
- 使用 fake clock/engine 和确定性 bandwidth policy 的 scheduler tests。
- 每个已发布 schema fixture 的 migration tests。
- 每个 transition boundary 的 crash tests。
- 包含 children/grandchildren 与 unrelated same-name process 的 native helper。
- 支持 engine versions 的 probe/progress/error golden fixtures。
- Redaction、path traversal、temp permissions 和 support bundle 安全测试。
- Windows、macOS、Linux 原生 packaging smoke tests。

## 13. 开放决定

- 最终 Rust 数据库库和 migration tool。
- Typed IPC 生成库，或小型手工 schema。
- 首批支持的 OS versions 和 CPU architectures。
- 严格 aggregate rate limiting 是否需要 transport broker。
- License 与 sidecar build 的 redistribution strategy。

这些决定在对应 roadmap gate 前完成，并记录为 ADR。
