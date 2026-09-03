[English](README.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md)

# JiveFetch

> 你的媒体，你的队列，你的规则。

JiveFetch 是一款本地优先、跨平台的桌面媒体下载管理器，用于合法的本地下载。
目标技术栈为 Tauri 2、Rust、React/TypeScript，以及受管理的 `yt-dlp`、
`ffmpeg`/`ffprobe` 和可选的 `aria2`。

`0.2.0` 是可运行的设置与实时队列预览版：它包含 Tauri 桌面外壳、英语、俄语和
简体中文界面，以及由 Rust 管理的 SQLite 队列。用户可以添加经过验证的 HTTP(S)
链接，通过本机安装的 `yt-dlp` 和 FFmpeg 下载，选择 1–10 或自定义的有界并发数，
设置全局速度预算与输出目录，并切换系统、深色或浅色主题。队列会显示进度、速度、
ETA、当前与总大小；默认目录仍为系统 Downloads 下的 `JiveFetch`。
浏览器下拉框会把 yt-dlp 支持的浏览器标识传给 `--cookies-from-browser`；JiveFetch
不会复制 Cookie 值。

## 产品目标

- 有界并发、可在崩溃和重启后恢复的持久队列。
- 根据实时探测结果选择格式、清晰度、编解码器、容器和字幕。
- 浏览器认证和 Cookie 导入，且不以明文保存秘密。
- 全局以及单任务限速。
- 语义明确的暂停、继续、停止、重试和移除。
- 在 Windows、macOS 和 Linux 上正确拥有并终止整个进程树。
- 使用固定、可验证的 sidecar 工具生成可复现、已签名的软件包。

## 非目标

- 绕过 DRM、付费墙或访问控制。
- 初始版本中的云端中继、账户系统或遥测。
- 在 MVP 中把任意 `yt-dlp` shell 参数作为普通功能。
- 复制 MediaHarbor、FlowGrab 的代码、界面、资源或内部结构。

用户必须自行确保下载行为符合授权、版权和服务条款。

## 架构概览

```text
React UI
   │ 类型化 Tauri 命令 + 版本化事件
   ▼
Tauri adapter
   ▼
Rust application services ─► scheduler ─► process supervisor
          │                      │                 │
          ▼                      ▼                 ▼
    format policy            SQLite/WAL   yt-dlp / ffmpeg / aria2
          │                                        │
          └──────── secure credential broker ◄─────┘
```

SQLite 是持久事实来源。React 从快照和有序事件重建状态。只有 Rust scheduler
可以预留队列槽位并变更任务状态。

## 仓库结构

```text
src/              React/TypeScript UI 与三种语言字典
src-tauri/        Tauri 外壳、scheduler、supervisor 与 Rust/SQLite
.github/workflows/ Windows/macOS/Linux 原生源码验证
docs/             EN/RU/简体中文需求、架构和路线图
package.json      前端和 Tauri CLI 依赖
README.*.md       多语言项目入口
```

首个纵向功能在 `src-tauri` 中以独立 Rust 模块实现 scheduler、storage、engine adapter
和 process supervisor；目标 crate 边界继续记录在架构文档中。

## 文档

- [产品需求](docs/product-requirements.zh-CN.md)
- [架构](docs/architecture.zh-CN.md)
- [任务生命周期](docs/task-lifecycle.zh-CN.md)
- [安全与认证](docs/security.zh-CN.md)
- [跨平台打包](docs/packaging.zh-CN.md)
- [许可与第三方引擎](docs/licensing.zh-CN.md)
- [路线图](docs/roadmap.zh-CN.md)
- [研究参考](docs/research-references.zh-CN.md)
- [本地化策略](docs/localization.zh-CN.md)
- [变更记录](CHANGES.zh-CN.md)
- [v0.1.1 发布说明](docs/releases/v0.1.1.zh-CN.md)
- [v0.2.0 发布说明](docs/releases/v0.2.0.zh-CN.md)
- [项目记忆](MEMORY.zh-CN.md)

## 在 macOS 本地运行

已验证要求：Node.js 22+、npm 10+、Rust 1.88+、`yt-dlp` 和 FFmpeg。

```bash
npm install
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --test real_engine_smoke -- --ignored
npm run tauri dev
```

队列数据库保存在系统应用数据目录，而不是仓库中。首次运行默认英语；用户明确选择的
语言会保存在本机。

被忽略的 smoke 测试使用 FFmpeg 创建短 fixture，仅通过 loopback 提供，随后由真实
Rust scheduler 和 `yt-dlp` 下载并验证输出路径；不需要公共媒体或外部网络。

## 项目标识

- 产品：**JiveFetch**
- 仓库：`jivefetch`
- 私有仓库：[github.com/shurrman/jivefetch](https://github.com/shurrman/jivefetch)
- Application ID：`top.jivejournal.jivefetch`
- 计划网站：`fetch.jivejournal.top`

- 源代码许可证：[Apache-2.0](LICENSE)
- 当前引擎模式：验证系统 executable，不再分发引擎二进制文件
