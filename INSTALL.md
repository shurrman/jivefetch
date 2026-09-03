[English](INSTALL.md) | [Русский](INSTALL.ru.md) | [简体中文](INSTALL.zh-CN.md)

# Install JiveFetch

JiveFetch runs on macOS, Windows, and Linux, but the current preview does not bundle
its download engines. Install both `yt-dlp` and FFmpeg first, then install JiveFetch.
Keep the engines current because supported sites change over time.

Use JiveFetch only for media you are permitted to download. It does not bypass DRM or
other access controls.

## Supported release packages

The current release pipeline produces these preview packages:

| Operating system | Architecture | Package |
| --- | --- | --- |
| macOS | Apple Silicon (`arm64`) | DMG |
| Windows | 64-bit (`x64`) | NSIS EXE or MSI |
| Linux | 64-bit (`x86_64`/`amd64`) | AppImage or DEB |

Other architectures are not yet published as installable JiveFetch packages.

## 1. Install yt-dlp and FFmpeg

### macOS

Install [Homebrew](https://brew.sh/) if it is not already available, then open Terminal:

```bash
brew install yt-dlp ffmpeg
```

### Windows

Open PowerShell and use Windows Package Manager:

```powershell
winget install --id yt-dlp.yt-dlp --exact
winget install --id Gyan.FFmpeg --exact
```

Close PowerShell and open it again so the updated `PATH` is applied.

### Ubuntu or Debian

Install FFmpeg from the distribution and the official `yt-dlp` release binary in a
system path that desktop applications can discover:

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

For another Linux distribution, install FFmpeg with its package manager and follow the
official `yt-dlp` binary instructions linked below. Make sure both executables are in
the desktop session's `PATH` or in `/usr/local/bin` or `/usr/bin`.

## 2. Verify the engines

Open a new Terminal or PowerShell window and run:

```text
yt-dlp --version
ffmpeg -version
```

Both commands must print a version. If either command is not found, finish that engine's
installation and restart both the terminal and JiveFetch.

## 3. Download and verify JiveFetch

Open the [latest JiveFetch release](https://github.com/shurrman/jivefetch/releases/latest)
and download the package for your OS plus its matching `SHA256SUMS-*.txt` file from
**Assets**.

- macOS: run `shasum -a 256 -c SHA256SUMS-macOS-ARM64.txt` in the download directory.
- Linux: run `sha256sum JiveFetch_*_amd64.deb` or
  `sha256sum JiveFetch_*_amd64.AppImage` and compare the result with the matching line
  in `SHA256SUMS-Linux-X64.txt`.
- Windows: run
  `Get-FileHash (Get-ChildItem .\JiveFetch_*_x64-setup.exe).FullName -Algorithm SHA256`
  or the equivalent `*_x64_en-US.msi` command in PowerShell, then compare the result
  with the matching line in `SHA256SUMS-Windows-X64.txt`.

Do not continue if the checksum differs.

## 4. Install the application

### macOS

Open the DMG and drag `JiveFetch.app` into `Applications`. The current DMG is unsigned
and unnotarized, so Gatekeeper may call it damaged. After verifying the checksum, follow
the [narrow unsigned-macOS procedure](docs/macos-installation.md).

### Windows

Run either the `x64-setup.exe` installer or the `x64_en-US.msi` package. Current preview
installers are unsigned, so Windows may show a publisher or SmartScreen warning. Continue
only for a package from this repository whose checksum you verified.

### Linux

For the DEB package:

```bash
sudo apt install ./JiveFetch_*_amd64.deb
```

For the AppImage:

```bash
chmod +x JiveFetch_*_amd64.AppImage
./JiveFetch_*_amd64.AppImage
```

## 5. First launch

At the top of JiveFetch, confirm that real versions are shown for `yt-dlp` and FFmpeg.
If the app says an engine is not found, close JiveFetch completely, verify the engine in
a new terminal, and launch the app again. Downloads are saved to the operating system's
Downloads directory under `JiveFetch` by default; the folder can be changed in the top
controls. English is selected on first launch, and the language can be changed there too.

## Keep the engines current

- Homebrew: `brew upgrade yt-dlp ffmpeg`
- Windows Package Manager: `winget upgrade --id yt-dlp.yt-dlp --exact` and
  `winget upgrade --id Gyan.FFmpeg --exact`
- Official standalone `yt-dlp`: `sudo yt-dlp -U`
- Distribution packages: update them through the distribution package manager.

Restart JiveFetch after an engine update so it detects the new version.

## Official references

- [`yt-dlp` installation](https://github.com/yt-dlp/yt-dlp/wiki/Installation)
- [FFmpeg downloads and package links](https://ffmpeg.org/download.html)
- [Homebrew `yt-dlp` formula](https://formulae.brew.sh/formula/yt-dlp)
- [Homebrew FFmpeg formula](https://formulae.brew.sh/formula/ffmpeg)
- [Windows Package Manager documentation](https://learn.microsoft.com/windows/package-manager/winget/)
