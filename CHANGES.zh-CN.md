[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# 变更记录

## 0.1.0 - 2026-09-03

### 新增

- 架构基线、需求、安全、生命周期、打包和路线图文档。
- 可在 macOS 运行的 Tauri 2 + React/TypeScript + Rust 原型。
- 带修订号、HTTP(S) URL 验证及暂停/继续/停止/移除操作的 SQLite 队列。
- 英语、俄语、简体中文应用和文档；首次运行默认英语。
- 初始 JiveFetch 图标以及可锁定依赖的 npm/Cargo lockfile。
- 使用类型化参数真实启动 `yt-dlp`，验证 FFmpeg 发现、限制进度解析并校验输出路径。
- 双执行槽 Rust scheduler、事务 attempt、启动恢复和可移植 Pause/Resume/Stop。
- 使用 Unix process group 与 Windows Job Object 监管所属进程树，并通过保留无关进程的跨平台 helper 测试。
- 本地真实引擎 smoke、secret scan、Windows/macOS/Linux native CI，以及三种语言实时进度 UI。
- 为 JiveFetch 选择 Apache-2.0，并记录系统引擎许可门禁。
- 已创建私有 GitHub 仓库 `shurrman/jivefetch` 并配置为 `origin`。
- 新增 tag-driven native release workflow，用于生成 macOS DMG、Linux AppImage/deb、
  Windows NSIS/MSI 预览包及各平台 SHA-256 checksum。
- 首次远程 native CI matrix 已在 macOS、Linux 和 Windows 上通过：frontend、三语文档、
  secret scan、rustfmt、clippy、Rust 测试和 desktop compilation。

### 指标

- 之前：2 个原始文件、1 个通用 Markdown、0 个应用源码文件和 0 个项目文档。
- 现在：132 个仓库文件，其中包括 14 组结构一致的 EN/RU/简体中文文档（共 42 个
  Markdown）、3,262 行应用/检查源码、可运行 desktop build、8 个常规通过的 Rust 测试、
  1 个通过的 opt-in 真实引擎 smoke，以及持久 SQLite task/attempt 状态。

### 待完成

- Installer signing identity、managed sidecar、FFmpeg 再分发审查、SBOM/provenance 和 clean-machine release test 尚未完成。
- Linux 通过 Tauri/GTK 间接使用 `glib 0.18.5`，仍受 `GHSA-wrw7-89jp-8q8g`
  标记；受影响 API 未被使用，但该 advisory 会阻止 stable release。
