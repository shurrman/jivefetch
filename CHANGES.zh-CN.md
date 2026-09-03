[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# 变更记录

## 0.2.0 - 2026-09-03

### 新增

- 持久 Rust/SQLite 设置：1–10 并发预设及已校验的自定义值、可选总速度预算、
  browser-cookie 认证，以及默认为 `Downloads/JiveFetch` 的原生目录选择器。
- 跟随系统、深色、浅色主题，以及覆盖所有生成平台尺寸的新高对比度 JiveFetch 图标。
- 每五秒 authoritative 刷新队列：蓝色 active、绿色 completed、红色 failed/interrupted
  进度，并显示 speed、ETA、当前与总大小。
- 将 `yt-dlp`/FFmpeg 版本移到顶部，移除 SQLite/engine 实现细节卡片和多余默认语言提示。
- 类型化 `--limit-rate` 与 `--cookies-from-browser` 参数，并加入 allowlist、migration、
  persistence、allocation 和 argument-vector 测试。
- `v0.1.1` 已作为首个跨平台 prerelease 发布，包含 DMG、AppImage、DEB、NSIS EXE、
  MSI 以及各平台 SHA-256 manifest。
- `v0.2.0` 已在 macOS、Linux、Windows 的 Native CI 与 Native Release 全部成功后
  发布，包含八个 installer/checksum artifact。
- 本地 `AGENTS*` 与 `MEMORY*` context 已移出 version control，并通过明确 ignore
  规则保留工作副本。
- 修复了已发布 `v0.2.0` 的翻译/文档链接；release workflow 现在会把相对链接转换为
  对应 tag 的稳定绝对 URL。
- 删除了不完整的 `v0.1.0` draft release，且未重新构建 artifact 就将 `v0.2.0`
  设为 GitHub `Latest`；后续成功发布的版本也会自动成为 `Latest`。签名与 notarization
  仍是独立的 production gate。

### 指标

- 之前：135 个文件、15 组共 45 个 Markdown、3,262 行应用/检查源码、8 个常规 Rust 测试。
- 现在：135 个文件、14 组共 42 个 Markdown、4,075 行源码、14 个常规 Rust 测试，
  以及 opt-in 真实引擎 smoke。

## 0.1.1 - 2026-09-03

### 修复

- 明确配置完整 PNG/ICNS/ICO bundle icon，使 Linux AppImage 和 Windows MSI 能找到
  所需的方形图标和 `.ico` asset。
- 将 Native CI 的 push 触发限制为 branch，避免 release tag 重复运行 Native Release
  已包含的源码验证 matrix。

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
- 现在：135 个仓库文件，其中包括 15 组结构一致的 EN/RU/简体中文文档（共 45 个
  Markdown）、3,262 行应用/检查源码、可运行 desktop build、8 个常规通过的 Rust 测试、
  1 个通过的 opt-in 真实引擎 smoke，以及持久 SQLite task/attempt 状态。

### 待完成

- Installer signing identity、managed sidecar、FFmpeg 再分发审查、SBOM/provenance 和 clean-machine release test 尚未完成。
- Linux 通过 Tauri/GTK 间接使用 `glib 0.18.5`，仍受 `GHSA-wrw7-89jp-8q8g`
  标记；受影响 API 未被使用，但该 advisory 会阻止完整支持的 production release。
