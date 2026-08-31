[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# 变更记录

## 未发布

### 新增

- 架构基线、需求、安全、生命周期、打包和路线图文档。
- 可在 macOS 运行的 Tauri 2 + React/TypeScript + Rust 原型。
- 带修订号、HTTP(S) URL 验证及暂停/继续/停止/移除操作的 SQLite 队列。
- 英语、俄语、简体中文应用和文档；首次运行默认英语。
- 初始 JiveFetch 图标以及可锁定依赖的 npm/Cargo lockfile。

### 指标

- 之前：2 个原始文件、1 个通用 Markdown、0 个应用源码文件和 0 个项目文档。
- 现在：113 个版本化文件，其中包括 12 组结构一致的 EN/RU/简体中文文档（共 36 个
  Markdown）、1,341 行应用/检查源码、可运行 `.app`、4 个通过的 Rust 测试和持久
  SQLite 队列。

### 待完成

- 应用尚未调用 `yt-dlp`/FFmpeg。
- 尚无进程监管、引擎探测、真实下载和原生 CI。
- 尚未配置 remote，也未选择项目许可证。
