[English](licensing.md) | [Русский](licensing.ru.md) | [简体中文](licensing.zh-CN.md)

# 许可与第三方引擎

## JiveFetch 源代码

JiveFetch 源代码和项目文档采用 [Apache License 2.0](../LICENSE)。
`package.json` 和 `src-tauri/Cargo.toml` 使用对应的 SPDX 标识符 `Apache-2.0`。

选择 Apache-2.0 是因为它允许私人及商业使用、修改和再分发，提供明确的专利授权，
同时要求保留许可证与署名通知。本文只是工程决策记录，不构成法律意见。

## 当前引擎模式

`0.2.0` 通过绝对路径发现并调用用户安装的 `yt-dlp` 和 FFmpeg。目前 JiveFetch
不再分发这些可执行文件。进程监管器直接传递类型化参数列表，不经过 shell。

- `yt-dlp` 源代码采用 [The Unlicense](https://github.com/yt-dlp/yt-dlp/blob/master/LICENSE)。
- FFmpeg 通常采用 LGPL 2.1-or-later；如果启用了可选 GPL 组件，特定构建将适用
  GPL 2-or-later。必须依据 [FFmpeg 官方法律说明](https://ffmpeg.org/legal.html)
  审查准确的构建配置。

本机验证的 Homebrew FFmpeg 报告启用了 GPL 组件。它只是开发机依赖，不包含在
JiveFetch 产物中。

## 分发门禁

任何 installer 捆绑或下载引擎之前，发布负责人必须：

1. 记录准确的引擎版本、来源、hash、构建配置和许可证；
2. 确认其与预定分发形式兼容；
3. 包含所有必需的许可证、署名、对应源代码提供方式和通知；
4. 在应用中显示应用及引擎的准确版本和许可证；
5. 验证最终产物，不能以另一构建的许可证代替。

签名 installer、受管理的引擎更新、SBOM 和完整第三方通知仍属于发布阶段工作。
仓库中的 native CI 验证源码构建与进程所有权，但本身不构成再分发许可。
