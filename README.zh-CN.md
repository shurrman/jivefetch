[English](README.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md)

# JiveFetch

> 你的媒体，你的队列，你的规则。

JiveFetch 是一款本地优先、跨平台的桌面媒体下载管理器，用于合法的本地下载。
目标技术栈为 Tauri 2、Rust、React/TypeScript，以及受管理的 `yt-dlp`、
`ffmpeg`/`ffprobe` 和可选的 `aria2`。

`0.0.1` 已是可在 macOS 上运行的基础版本：它包含 Tauri 桌面外壳、英语、俄语和
简体中文界面，以及由 Rust 管理的 SQLite 队列。用户可以添加经过验证的 HTTP(S)
链接，并持久执行暂停、继续、停止和移除。此版本不会假装已经下载媒体；
`yt-dlp`/FFmpeg 集成是下一个纵向功能切片。

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
src-tauri/        Tauri 外壳及 Rust/SQLite 队列命令
docs/             EN/RU/简体中文需求、架构和路线图
package.json      前端和 Tauri CLI 依赖
README.*.md       多语言项目入口
```

加入进程监管和下载引擎后，Rust 代码将按 core/storage/process/engines 边界拆分。

## 文档

- [产品需求](docs/product-requirements.zh-CN.md)
- [架构](docs/architecture.zh-CN.md)
- [任务生命周期](docs/task-lifecycle.zh-CN.md)
- [安全与认证](docs/security.zh-CN.md)
- [跨平台打包](docs/packaging.zh-CN.md)
- [路线图](docs/roadmap.zh-CN.md)
- [研究参考](docs/research-references.zh-CN.md)
- [本地化策略](docs/localization.zh-CN.md)
- [变更记录](CHANGES.zh-CN.md)
- [项目记忆](MEMORY.zh-CN.md)

## 在 macOS 本地运行

已验证要求：Node.js 22+、npm 10+、Rust 1.88+。

```bash
npm install
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

队列数据库保存在系统应用数据目录，而不是仓库中。首次运行默认英语；用户明确选择的
语言会保存在本机。

下一步是 fake engine 和进程树监管，然后加入真实 `yt-dlp` 探测与单次下载闭环。

## 项目标识

- 产品：**JiveFetch**
- 仓库：`jivefetch`
- Application ID：`top.jivejournal.jivefetch`
- 计划网站：`fetch.jivejournal.top`

项目许可证将在完成 sidecar 二进制分发义务审查后确定。
