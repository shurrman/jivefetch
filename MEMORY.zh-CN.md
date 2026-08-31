[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# JiveFetch 项目记忆

更新日期：2026-09-01

## 当前状态

- `0.0.1` 是可运行的 macOS 基础版本：Tauri 2、React 和 Rust/SQLite。
- 可以把 URL 加入持久队列；暂停、继续、停止和移除使用 revision 做乐观并发控制。
- UI 和文档支持 EN/RU/简体中文；首次启动默认 EN，明确选择会保存在本机。
- 系统中已找到 `yt-dlp` 和 FFmpeg，但应用尚未调用它们。
- 当前仅本地工作；没有 remote，也不 push。

## 长期决定

- Product ID 为 `top.jivejournal.jivefetch`，计划站点为 `fetch.jivejournal.top`。
- SQLite WAL 是事实来源；React 只是投影。
- 单一 Rust scheduler 将拥有状态迁移、并发、重试和带宽策略。
- 跨平台暂停表示受控停止，再创建可恢复的新 attempt。
- 进程所有权使用 Unix process group/session 或 Windows Job Object；绝不按名称 kill。
- 浏览器模式只保存 browser/profile 引用；导入 Cookie 使用 OS credential store 中的密钥加密。
- 格式来自实时探测，并由 Rust 编译为参数，不使用 shell 字符串。
- 文档和用户可见字符串必须同步提供 EN/RU/简体中文版本。
- MediaHarbor 和 FlowGrab 仅作为创意参考，不复制代码或结构。
- 在完成 sidecar 二进制审查前不决定项目许可证。

## 安全不变量

- 不在数据库、日志或 Git 中保存 Cookie、token、认证头或解密秘密。
- PID 不是所有权证明；不得接管未知进程。
- 移除任务记录和删除下载文件是两个独立的明确操作。
- 在 Windows/macOS/Linux 原生测试通过前，不宣称跨平台支持。
- 新增或修改文档必须同时具备三种语言及互链，才算完成。

## 下一步

1. Fake engine 与进程树 helper。
2. 完整 tasks/attempts/artifacts/events 模型的首次迁移。
3. 真实 `yt-dlp` 探测和单次下载纵向闭环。
4. 公开打包前确定许可证和 sidecar 策略。
