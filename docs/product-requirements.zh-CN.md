[English](product-requirements.md) | [Русский](product-requirements.ru.md) | [简体中文](product-requirements.zh-CN.md)

# 产品需求

## 1. 产品定位

JiveFetch 是本地优先的桌面应用，用于添加、探测、排队和控制合法媒体下载。
外部操作应简单，内部必须像可靠的 job system。

## 2. 设计原则

原则：界面简单但语义精确；持久性优先于动画；实时探测而非静态清晰度列表；最小化
秘密接触；只终止自有进程；错误可解释；初始版本不依赖云端或遥测。

## 3. 核心用户场景

### 3.1 快速下载

粘贴 URL、读取 metadata/formats、选择策略并加入队列。

### 3.2 Batch queue

审核 batch/playlist 展开结果、应用默认值、覆盖单项、排序并独立控制。

### 3.3 认证来源

选择 browser/profile 或导入 Netscape Cookie，值不进入 UI。

### 3.4 重启恢复

崩溃或重启后，先协调 transient state 和 partial artifact，再启动新任务。

## 4. 功能需求

### FR-1 URL intake 与 probing

- 支持单个/多个 URL、可选 clipboard 和 deep link；外部输入必须先 review。
- 独立 probe 返回 metadata、playlist、subtitle、chapter 和实际 format。
- 输入变化时取消旧 probe；缓存含时效和 engine version；启动前按需 re-probe。

### FR-2 动态格式选择

- 简单 preset 与高级清单：分辨率、FPS、codec、container、HDR、audio、估算大小、
  是否需要 merge/transcode。
- 用户选择 intent；Rust 编译类型化 `yt-dlp` 参数，不使用 shell。
- 清楚解释 fallback；remux 与 transcode 必须显式区分。

### FR-3 持久队列与 concurrency

- SQLite 保存 task、优先级/顺序、policy、attempt、progress 和 artifact。
- 网络和后处理独立并发限制，输出路径锁，priority aging。
- 每个 task 最多一个 live attempt；仅 transient error 进入有界 backoff retry。
- 单个失败不能阻塞其他任务。

### FR-4 单任务控制

- Pause 保留可用 partial，并最终处于无活进程的 paused 状态。
- Resume 创建新的 resumable attempt；Stop 禁止自动恢复。
- Retry 保留历史；Remove 与删除 tracked file 是两个独立选项。
- 命令使用 revision/idempotency key，重复调用安全。

### FR-5 进程监管

- Attempt 拥有完整进程树：Unix session/process group 或 Windows Job Object。
- 先 graceful deadline，再强制终止 owned tree 并验证；绝不按名称 kill。

### FR-6 认证与 Cookie

- Browser 模式不保存 Cookie；导入 Cookie 用 OS credential store 中的随机密钥加密，
  明文只存在于 user-only 临时 lease。
- Cookie、token、认证头和签名 query 不得进入日志、数据库、UI 或 Git。

### FR-7 带宽限制

- 支持 per-task ceiling 与公平的全局 ceiling；不支持动态限速的 engine 通过
  debounced resumable restart 重平衡；稳定后总流量不超过设定值的 110%。

### FR-8 历史、artifacts 与 diagnostics

- 脱敏 attempt history、类型化错误、canonical artifact 和安全 Show in Folder。

### FR-9 依赖管理

- 签名 engine manifest、hash、health check、不可变版本与 rollback。

### FR-10 跨平台交付

- 每个支持 OS 都必须通过原生签名/公证打包及 process/path/secret/migration tests。

## 5. 非功能需求

### Reliability

Hard kill 不得损坏队列或产生重复 attempt。

### Performance

10,000 条历史通过索引和分页保持可用；progress coalescing 避免 UI/SQLite 写放大。

### Security 与 privacy

Tauri capability 最小化，禁止 remote script，redaction 在持久化前完成。

### Accessibility 与 UX

所有控制支持键盘和无障碍，不只用颜色表达状态。

## 6. MVP 边界

MVP 包含 URL/batch、dynamic probe、persistent concurrency/recovery、controls、
browser/imported cookies、rate limits、history 和验证过的打包。Player、library、
subscriptions、extensions、remote/cloud、plugins、raw expert args 延后。

## 7. 发布验收场景

发布验收包括：崩溃恢复无重复；每个 OS 暂停/继续；停止 owned tree 而不影响同名无关
进程；全局限速重平衡；sentinel secret 扫描；format 消失时 re-probe；engine rollback；
数据库 migration/recovery。
