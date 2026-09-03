[English](MEMORY.md) | [Русский](MEMORY.ru.md) | [简体中文](MEMORY.zh-CN.md)

# JiveFetch 项目记忆

更新日期：2026-09-03

## 当前状态

- `0.2.0` 是当前的下一预览版：Tauri 2、React、Rust/SQLite，以及本机 `yt-dlp` 和 FFmpeg。
- 可配置 Rust scheduler 管理类型化 process plan、attempt、progress、总速度分配，
  以及使用 task revision 的暂停、继续、停止和移除。
- 持久非秘密设置包括并发数（默认 2、1–10 预设、最大 64 的已校验自定义值）、
  总速度限制、yt-dlp 浏览器 Cookie 选择器，以及默认为 `Downloads/JiveFetch` 的目录。
- UI 每五秒读取权威队列，显示每项彩色进度、速度、ETA 和大小，支持系统/深色/浅色
  主题，并使用新的高对比度图标。
- UI 和文档支持 EN/RU/简体中文；首次启动默认 EN，明确选择会保存在本机。
- Supervisor 管理 Unix process group 或 Windows Job Object，限制输出并只终止自己创建的树。
- 常规 Rust suite 有 14 个测试，包括 v2/v3 设置 migration、类型化 engine 参数和
  无关进程所有权。Opt-in 真实引擎 loopback smoke 是 `v0.2.0` 最后的本机 runtime gate。
- 首次远程 native source CI 已在 Windows、macOS 和 Linux 上通过；GitHub Actions
  run 为 `33779282453`。
- GitHub CLI 已以 `shurrman` 身份完成认证；私有仓库
  `https://github.com/shurrman/jivefetch` 已配置为 `origin`。Candidate `v0.1.0`
  暴露了 Linux/Windows bundle icon 问题。Native CI run `33783089736` 和 Native
  Release run `33784331491` 已为 `v0.1.1` 成功完成；该版本发布了八个
  macOS/Linux/Windows artifact。
- Commit `a4792bb` 在 macOS、Linux 和 Windows 上通过 Native CI run `33788132275`。
  Native Release run `33789236621` 随后发布 `v0.2.0` prerelease，其中包含八个
  已验证的 DMG/AppImage/DEB/NSIS/MSI/checksum artifact。
- JiveFetch 源代码采用 Apache-2.0；不再分发系统引擎。
- Linux 通过 Tauri/GTK 引入 `glib 0.18.5`。`GHSA-wrw7-89jp-8q8g` 影响 JiveFetch
  未调用的 API；已修复的 `glib 0.20` 与 GTK `^0.18` 不兼容。该例外只适用于私有
  preview，并阻止 stable release。

## 长期决定

- Product ID 为 `top.jivejournal.jivefetch`，计划站点为 `fetch.jivejournal.top`。
- SQLite WAL 是事实来源；React 只是投影。
- 单一 Rust scheduler 将拥有状态迁移、并发、重试和带宽策略。
- 跨平台暂停表示受控停止，再创建可恢复的新 attempt。
- 进程所有权使用 Unix process group/session 或 Windows Job Object；绝不按名称 kill。
- Browser-cookie 模式只保存 allowlist 中的 browser identifier，并以类型化
  `yt-dlp --cookies-from-browser` 参数传递；绝不复制 Cookie 值。未来导入 Cookie
  文件时必须使用 OS credential store 中的密钥加密。
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

1. 加入 metadata probe、格式选择和 ffprobe capability 报告。
2. 扩展 storage：artifacts/events、idempotency、retry policy 和 pagination。
3. 加入 profile/keyring 细节和不暴露值的加密 Cookie 文件导入。
4. 在 stable release 前加入 signing/notarization 和经过验证的 managed engine。
