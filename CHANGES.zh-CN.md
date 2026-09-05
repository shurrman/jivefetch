[English](CHANGES.md) | [Русский](CHANGES.ru.md) | [简体中文](CHANGES.zh-CN.md)

# 变更记录

## 尚未发布

## 0.4.0 - 2026-09-05

### 新增

- 新增任务级总 progress 模型，合并计划的视频/音频组件大小、已下载 bytes、speed 与 ETA，
  并在每种语言中显示“视频/音频/合并”当前阶段。
- Completed 任务新增“打开”。Rust 根据 task ID 解析路径，重新验证它是所选目录内的非空
  canonical output，再交给操作系统默认应用；webview 不传入 filesystem path。
- 新增本地 structured JSON diagnostics，包含稳定 task/attempt/error field，当前文件上限
  2 MiB 并保留一次 rotation，不记录原始 engine line、URL、Cookie 或 path。
- 新增窄 engine/process-spawner trait 与 scheduler test double。

### 变更

- Rust 内部的 `Result<T, String>` 改为 typed validation/storage/engine/scheduler/input
  error；稳定本地化字符串只在 Tauri IPC boundary 映射。`AppSettings` 自行验证，
  browser Cookie 使用 typed allowlist。
- Binary discovery 与 `yt-dlp` execution 分离，同时保留单一 scheduler 和每个 supervised
  process tree 的单一 RAII owner。
- 本地控制与引擎就绪信息合并为 header 中的一行，并使用绿色/红色 indicator；URL、
  format picker 和加入队列的表单完整保留。
- 在其余 production release gate 完成前，未签名且未 notarize 的构建会作为 GitHub
  pre-release 发布。

### 修复

- 单独视频流达到 100% 后开始音频流时，不再看似完成又重新开始。Component progress
  单调聚合，active work 保持低于 100%，只有验证完成才达到 100%。
- Completed 任务只有在最终文件仍真实存在时才显示绿色并提供“打开”；文件被删除或为空
  时，会在下一次 queue refresh 显示 output error。

### 指标

- SQLite schema：版本 5 -> 6，并有经过测试的 in-place `download_stage` migration。
- 常规 Rust suite：22 -> 26 个测试，另有通过的 opt-in real-engine loopback smoke。
- Error boundary：从 Rust 全程 string code 变为 5 个 typed error family，仅在 Tauri IPC
  进行一次稳定 string mapping。
- Progress model：从一个会被覆盖的 per-component snapshot 变为计划组件的单调 aggregate，
  每种应用语言包含 4 个本地化 stage label。

## 0.3.1 - 2026-09-05

### 新增

- 新增英语、俄语、简体中文完整安装指南，覆盖 macOS、Windows、Ubuntu/Debian、
  Fedora、Arch Linux、AppImage 与 DEB，并说明如何安装和验证 `yt-dlp`、FFmpeg、Deno，
  核对 JiveFetch checksum，完成首次启动及更新下载引擎。
- 每个 README 译本的开头现在都会醒目说明跨平台应用的引擎前置要求，并提供安装指南链接。
- 新增语言无关的 README hero，展示面向用户的下载队列、进度状态、媒体格式与跨设备
  使用，不包含第三方品牌。
- 所有 README 改为面向用户的顺序：先说明可获得的结果与功能，再列出前置要求、平台
  软件包和安装链接。
- 在顶部状态区域的 `yt-dlp` 与 FFmpeg 版本旁新增 JiveFetch 应用版本。

### 修复

- Release 语言导航现在指向具体 tag 页面和 section anchor，例如
  `/releases/tag/v0.3.0#russian`，而不是相对于多版本列表的 anchor；语言跳转可以正常
  工作，同时仍能访问 Assets。
- 未签名 macOS 安装指南改为与版本无关，因此新 Latest 发布后不会继续显示过时的 DMG
  文件名或 checksum。
- 将通用 `yt-dlp` 失败替换为本地化且可操作的错误类别：浏览器 Cookie 访问、认证、
  媒体不可用、请求频率限制、格式、网络与文件系统权限；不会持久化原始引擎输出。
- 记录当前未签名 macOS 构建读取 Chrome Cookie 所需的完全磁盘访问权限与钥匙串步骤，
  并提供公开媒体的安全模式及 Firefox 临时替代方案。
- 将引擎版本探测从受保护的 Downloads 文件夹移至应用自己的数据目录，避免 macOS TCC
  启动竞态导致 Homebrew `yt-dlp` 显示缺失而 FFmpeg 同时可用。
- 在下载器的精简 `PATH` 中加入标准软件包管理器目录和 Deno 用户目录。通过 Finder
  启动的 macOS 构建现在能让 `yt-dlp` 找到 Homebrew Deno，避免元数据探测成功后因
  缺少 JavaScript 运行时而在 YouTube 媒体请求中收到 HTTP 403。
- HTTP 403 现在有独立的本地化错误，提示更新 `yt-dlp`、安装 Deno，或为受限媒体启用
  浏览器 Cookie。

### 指标

- 文档：从 48 个增加到 54 个受跟踪 Markdown 文件，从 16 组增加到 18 组已验证的
  EN/RU/简体中文文档。
- 三个 README 合计从 389 行缩减为 93 行；架构与开发细节保留在独立文档中，不再放在
  产品入口页面。
- Release 导航：3 个含义不明确的列表相对 anchor 改为 3 个明确的 tag 页面 anchor，
  并由 release-note checker 验证。
- 顶部版本指示：从 2 个引擎版本增加为 1 个应用版本加 2 个引擎版本。
- 面向用户的引擎失败类别：从 1 个通用错误增加为 8 个可操作的本地化类别并保留通用
  fallback；常规 Rust 测试从 19 个增加到 22 个。

## 0.3.0 - 2026-09-03

### 新增

- 新增 cookie-aware `yt-dlp` 实时 JSON probe，并可为每个任务从来源实际视频格式中
  选择。Picker 显示分辨率、FPS、来源码率、编解码器、容器和估算大小，默认选择最高
  质量，同时隐藏内部 ID 和没有大小估算的重复标题式行。
- 新增任务右键菜单：启动、停止、暂停、复制 URL 和移除；当前状态不可用的转换仍显示
  但处于禁用状态。
- 所选的已验证 format selector 持久保存在 SQLite schema version 5，并仅通过类型化
  参数向量传给引擎。

### 修复

- 显式启用 `yt-dlp --progress`，因此最终路径输出不再抑制当前字节、总大小、速度、
  ETA 与进度事件。
- 队列卡片保留非秘密 URL 查询参数，同时对类似秘密的值脱敏，使不同的 YouTube
  `/watch?v=…` 任务可以区分。
- 完成任务前必须验证存在非空输出普通文件，持久保存其实际大小，并在启动时修复旧
  completed 任务的零字节指标。
- 快速版本进程退出后会等待输出排空，避免桌面冷启动时偶发错误显示“未找到 yt-dlp”。

### 指标

- 之前（`v0.2.0`）：140 个文件、48 个 Markdown 文档、4,029 行应用/检查源码、
  14 个常规 Rust 测试，以及 opt-in 真实引擎 smoke。
- 现在：141 个文件、16 组 EN/RU/简体中文共 48 个 Markdown、4,975 行源码、
  19 个常规 Rust 测试，以及通过的真实引擎 smoke。

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
- 新增 EN/RU/简体中文 macOS 安装说明：先验证 checksum，再为当前未签名应用创建本地
  ad-hoc 签名，并仅移除该应用的 quarantine 属性。
- 三种 release notes 翻译现在嵌入同一个 GitHub Release 页面，并使用页面内语言导航；
  切换语言后仍可看到共用的 Assets 区域。

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
