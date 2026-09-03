[English](INSTALL.md) | [Русский](INSTALL.ru.md) | [简体中文](INSTALL.zh-CN.md)

# 安装 JiveFetch

JiveFetch 可在 macOS、Windows 与 Linux 上运行，但当前预览版不内置下载引擎。
请先安装 `yt-dlp` 和 FFmpeg，再安装 JiveFetch。受支持的网站会随时间变化，因此请
及时更新下载引擎。

请仅使用 JiveFetch 下载你有权下载的媒体。应用不会绕过 DRM 或其他访问控制。

## 支持的发布包

当前 release pipeline 会生成以下预览包：

| 操作系统 | 架构 | 软件包 |
| --- | --- | --- |
| macOS | Apple Silicon（`arm64`） | DMG |
| Windows | 64 位（`x64`） | NSIS EXE 或 MSI |
| Linux | 64 位（`x86_64`/`amd64`） | AppImage 或 DEB |

JiveFetch 暂未发布其他架构的可安装软件包。

## 1. 安装 yt-dlp 与 FFmpeg

### macOS

如果尚未安装，请先安装 [Homebrew](https://brew.sh/)，然后打开 Terminal：

```bash
brew install yt-dlp ffmpeg
```

### Windows

打开 PowerShell，并使用 Windows Package Manager：

```powershell
winget install --id yt-dlp.yt-dlp --exact
winget install --id Gyan.FFmpeg --exact
```

关闭并重新打开 PowerShell，使更新后的 `PATH` 生效。

### Ubuntu 或 Debian

从发行版安装 FFmpeg，并把官方 `yt-dlp` 发布二进制文件安装到桌面应用可发现的
系统路径：

```bash
sudo apt update
sudo apt install ffmpeg curl
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /tmp/jivefetch-yt-dlp
sudo install -m 0755 /tmp/jivefetch-yt-dlp /usr/local/bin/yt-dlp
```

### Fedora

```bash
sudo dnf install ffmpeg-free curl
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /tmp/jivefetch-yt-dlp
sudo install -m 0755 /tmp/jivefetch-yt-dlp /usr/local/bin/yt-dlp
```

### Arch Linux

```bash
sudo pacman -Syu yt-dlp ffmpeg
```

对于其他 Linux 发行版，请使用其软件包管理器安装 FFmpeg，并按照下方官方说明安装
`yt-dlp` 二进制文件。两个可执行文件必须位于桌面会话的 `PATH`，或位于
`/usr/local/bin`、`/usr/bin`。

## 2. 验证下载引擎

打开新的 Terminal 或 PowerShell 窗口并运行：

```text
yt-dlp --version
ffmpeg -version
```

两个命令都必须输出版本号。如果有命令未找到，请完成对应引擎的安装，然后同时重启
终端和 JiveFetch。

## 3. 下载并验证 JiveFetch

打开 [JiveFetch 最新版本](https://github.com/shurrman/jivefetch/releases/latest)，在
**Assets** 中下载适用于你的操作系统的软件包，以及对应的 `SHA256SUMS-*.txt` 文件。

- macOS：在下载目录运行 `shasum -a 256 -c SHA256SUMS-macOS-ARM64.txt`。
- Linux：运行 `sha256sum JiveFetch_*_amd64.deb` 或
  `sha256sum JiveFetch_*_amd64.AppImage`，并与 `SHA256SUMS-Linux-X64.txt` 中对应行比较。
- Windows：在 PowerShell 中运行
  `Get-FileHash (Get-ChildItem .\JiveFetch_*_x64-setup.exe).FullName -Algorithm SHA256`，
  或针对 `*_x64_en-US.msi` 运行同等命令，并与 `SHA256SUMS-Windows-X64.txt` 中对应行比较。

如果 checksum 不一致，请勿继续安装。

## 4. 安装应用

### macOS

打开 DMG，将 `JiveFetch.app` 拖入 `Applications`。当前 DMG 尚未签名或 notarize，
因此 Gatekeeper 可能会提示应用已损坏。验证 checksum 后，请按照
[未签名 macOS 应用的限定操作说明](docs/macos-installation.zh-CN.md)处理。

### Windows

运行 `x64-setup.exe` 安装程序或 `x64_en-US.msi` 软件包。当前预览版安装程序未签名，
因此 Windows 可能显示发布者或 SmartScreen 警告。仅当软件包来自本仓库且 checksum
验证成功时才继续。

### Linux

对于 DEB 软件包：

```bash
sudo apt install ./JiveFetch_*_amd64.deb
```

对于 AppImage：

```bash
chmod +x JiveFetch_*_amd64.AppImage
./JiveFetch_*_amd64.AppImage
```

## 5. 首次启动

在 JiveFetch 顶部确认 `yt-dlp` 与 FFmpeg 均显示真实版本。如果应用提示未找到某个
引擎，请完全关闭 JiveFetch，在新终端中验证该引擎，再重新启动应用。默认下载位置是
操作系统 Downloads 目录下的 `JiveFetch` 子目录，可在顶部控件中修改。首次启动默认
使用英语，也可在同一区域切换语言。

## 保持下载引擎为最新版本

- Homebrew：`brew upgrade yt-dlp ffmpeg`
- Windows Package Manager：`winget upgrade --id yt-dlp.yt-dlp --exact` 和
  `winget upgrade --id Gyan.FFmpeg --exact`
- 官方 standalone `yt-dlp`：`sudo yt-dlp -U`
- 发行版软件包：通过发行版的软件包管理器进行更新。

更新下载引擎后请重启 JiveFetch，以便应用检测新版本。

## 官方参考资料

- [`yt-dlp` 安装说明](https://github.com/yt-dlp/yt-dlp/wiki/Installation)
- [FFmpeg 下载与软件包链接](https://ffmpeg.org/download.html)
- [Homebrew `yt-dlp` formula](https://formulae.brew.sh/formula/yt-dlp)
- [Homebrew FFmpeg formula](https://formulae.brew.sh/formula/ffmpeg)
- [Windows Package Manager 文档](https://learn.microsoft.com/windows/package-manager/winget/)
