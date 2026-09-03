[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# JiveFetch 项目记忆

更新日期：2026-09-03

## 当前状态

- `0.1.0` 是可运行的首个下载预览版：Tauri 2、React、Rust/SQLite，以及本机
  `yt-dlp` 和 FFmpeg。
- 双槽 Rust scheduler 管理类型化 process plan、attempt、progress 与使用 task revision
  的暂停、继续、停止和移除。
- UI 和文档支持 EN/RU/简体中文；首次启动默认 EN，明确选择会保存在本机。
- Supervisor 管理 Unix process group 或 Windows Job Object，限制输出并只终止自己创建的树。
- macOS 上真实引擎 loopback smoke 已通过，版本为 yt-dlp `2026.07.04` 和 FFmpeg `8.1.2`；
  保留无关进程的 supervisor 测试也已通过。
- Windows/macOS/Linux native source CI 已定义，但尚未远程执行。
- GitHub CLI 已以 `shurrman` 身份完成认证；私有仓库
  `https://github.com/shurrman/jivefetch` 已配置为 `origin`。`v0.1.0` 是面向 macOS、
  Linux 和 Windows 的首个 native release candidate。
- JiveFetch 源代码采用 Apache-2.0；不再分发系统引擎。

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
- JiveFetch 代码与文档采用 Apache-2.0；bundled/managed 引擎分发仍需按准确二进制许可证单独审核。
- 与 package version 一致的 tag 会触发 macOS/Linux/Windows native preview 构建；
  只有整个 matrix 上传 package 和 checksum 后才发布。

## 安全不变量

- 不在数据库、日志或 Git 中保存 Cookie、token、认证头或解密秘密。
- PID 不是所有权证明；不得接管未知进程。
- 移除任务记录和删除下载文件是两个独立的明确操作。
- 在 Windows/macOS/Linux 原生测试通过前，不宣称跨平台支持。
- 新增或修改文档必须同时具备三种语言及互链，才算完成。

## 下一步

1. 观察首次 Windows/macOS/Linux native CI 和 release workflow；在确认 `v0.1.0`
   通过验证前修复所有 platform-specific 失败。
2. 加入 metadata probe、格式选择和 ffprobe capability 报告。
3. 扩展 storage：artifacts/events、idempotency、retry policy 和 pagination。
4. 发布前加入 credential-store Cookie 和可验证的 managed engine。
