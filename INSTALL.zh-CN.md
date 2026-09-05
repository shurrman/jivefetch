[English](INSTALL.md) | [Русский](INSTALL.ru.md) | [简体中文](INSTALL.zh-CN.md)

# 安装 JiveFetch

JiveFetch 可在 macOS、Windows 与 Linux 上运行，但当前预览版不内置下载引擎。
请先安装 `yt-dlp`、FFmpeg 和 Deno，再安装 JiveFetch。Deno 是 `yt-dlp` 为当前
YouTube 支持推荐的 JavaScript 运行时。受支持的网站会随时间变化，因此请及时更新这些工具。

请仅使用 JiveFetch 下载你有权下载的媒体。应用不会绕过 DRM 或其他访问控制。

## 支持的发布包

当前 release pipeline 会生成以下预览包：

| 操作系统 | 架构 | 软件包 |
| --- | --- | --- |
| macOS | Apple Silicon（`arm64`） | DMG |
| Windows | 64 位（`x64`） | NSIS EXE 或 MSI |
| Linux | 64 位（`x86_64`/`amd64`） | AppImage 或 DEB |

JiveFetch 暂未发布其他架构的可安装软件包。

## 1. 安装 yt-dlp、FFmpeg 与 Deno

### macOS

如果尚未安装，请先安装 [Homebrew](https://brew.sh/)，然后打开 Terminal：

```bash
brew install yt-dlp ffmpeg deno
```

### Windows

打开 PowerShell，并使用 Windows Package Manager：

```powershell
winget install --id yt-dlp.yt-dlp --exact
winget install --id Gyan.FFmpeg --exact
winget install --id DenoLand.Deno --exact
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
curl -fsSL https://deno.land/install.sh | sh
```

### Fedora

```bash
sudo dnf install ffmpeg-free curl
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /tmp/jivefetch-yt-dlp
sudo install -m 0755 /tmp/jivefetch-yt-dlp /usr/local/bin/yt-dlp
curl -fsSL https://deno.land/install.sh | sh
```

### Arch Linux

```bash
sudo pacman -Syu yt-dlp ffmpeg deno
```

对于其他 Linux 发行版，请使用其软件包管理器安装 FFmpeg，按照官方说明安装 `yt-dlp`，
并按 Deno 官方说明安装 Deno。这些工具必须位于桌面会话的 `PATH`、标准系统路径，
或 Deno 的默认目录 `$HOME/.deno/bin`。

## 2. 验证下载引擎

打开新的 Terminal 或 PowerShell 窗口并运行：

```text
yt-dlp --version
ffmpeg -version
deno --version
```

三个命令都必须输出版本号。如果有命令未找到，请完成对应工具的安装，然后同时重启
终端和 JiveFetch。

## 3. 下载并验证 JiveFetch

打开 [JiveFetch 版本页面](https://github.com/shurrman/jivefetch/releases)，在
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

在 JiveFetch 顶部确认已显示应用版本，并且 `yt-dlp` 与 FFmpeg 均显示真实版本。如果应用提示未找到某个
引擎，请完全关闭 JiveFetch，在新终端中验证该引擎，再重新启动应用。默认下载位置是
操作系统 Downloads 目录下的 `JiveFetch` 子目录，可在顶部控件中修改。首次启动默认
使用英语，也可在同一区域切换语言。

### 当前未签名 macOS 构建中的浏览器 Cookie

如果公开 URL 在选择**不使用浏览器 Cookie**时正常，但选择 Chrome 后失败，则表示
macOS 阻止未签名应用的上下文读取或解密 Chrome Cookie。请在**系统设置 → 隐私与安全性
→ 完全磁盘访问权限**中添加 `/Applications/JiveFetch.app`，完全退出 JiveFetch 后重新
打开。如果 macOS 请求允许 JiveFetch 访问钥匙串中的 **Chrome Safe Storage**，请仅对
已验证 checksum 的应用副本授权。

下载公开媒体时请停用浏览器 Cookie。对于需要认证的媒体，可暂时改用已登录同一账号的
Firefox。长期解决方案是使用 Developer ID 签名并经过 notarization 的 JiveFetch；当前
预览构建尚未提供此项。

## 保持下载引擎为最新版本

- Homebrew：`brew upgrade yt-dlp ffmpeg deno`
- Windows Package Manager：`winget upgrade --id yt-dlp.yt-dlp --exact` 和
  `winget upgrade --id Gyan.FFmpeg --exact`
- 通过 Windows Package Manager 安装的 Deno：`winget upgrade --id DenoLand.Deno --exact`
- 官方 standalone `yt-dlp`：`sudo yt-dlp -U`
- 发行版软件包：通过发行版的软件包管理器进行更新。

更新下载引擎后请重启 JiveFetch，以便应用检测新版本。

## 官方参考资料

- [`yt-dlp` 安装说明](https://github.com/yt-dlp/yt-dlp/wiki/Installation)
- [`yt-dlp` 外部 JavaScript 运行时指南](https://github.com/yt-dlp/yt-dlp/wiki/EJS)
- [Deno 安装说明](https://docs.deno.com/runtime/getting_started/installation/)
- [FFmpeg 下载与软件包链接](https://ffmpeg.org/download.html)
- [Homebrew `yt-dlp` formula](https://formulae.brew.sh/formula/yt-dlp)
- [Homebrew FFmpeg formula](https://formulae.brew.sh/formula/ffmpeg)
- [Windows Package Manager 文档](https://learn.microsoft.com/windows/package-manager/winget/)
