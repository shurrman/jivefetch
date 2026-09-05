[English](INSTALL.md) | [Русский](INSTALL.ru.md) | [简体中文](INSTALL.zh-CN.md)

# Install JiveFetch

JiveFetch runs on macOS, Windows, and Linux, but the current preview does not bundle
its download engines. Install `yt-dlp`, FFmpeg, and Deno first, then install JiveFetch.
Deno is the JavaScript runtime recommended by `yt-dlp` for current YouTube support.
Keep these tools current because supported sites change over time.

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

## 1. Install yt-dlp, FFmpeg, and Deno

### macOS

Install [Homebrew](https://brew.sh/) if it is not already available, then open Terminal:

```bash
brew install yt-dlp ffmpeg deno
```

### Windows

Open PowerShell and use Windows Package Manager:

```powershell
winget install --id yt-dlp.yt-dlp --exact
winget install --id Gyan.FFmpeg --exact
winget install --id DenoLand.Deno --exact
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

For another Linux distribution, install FFmpeg with its package manager, follow the
official `yt-dlp` binary instructions, and install Deno from its official instructions.
Make sure the tools are in the desktop session's `PATH`, standard system paths, or Deno's
default `$HOME/.deno/bin` directory.

## 2. Verify the engines

Open a new Terminal or PowerShell window and run:

```text
yt-dlp --version
ffmpeg -version
deno --version
```

All three commands must print a version. If a command is not found, finish that tool's
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

At the top of JiveFetch, confirm that the application version and real versions for
`yt-dlp` and FFmpeg are shown.
If the app says an engine is not found, close JiveFetch completely, verify the engine in
a new terminal, and launch the app again. Downloads are saved to the operating system's
Downloads directory under `JiveFetch` by default; the folder can be changed in the top
controls. English is selected on first launch, and the language can be changed there too.

### Browser cookies in the current unsigned macOS build

If a public URL works with **Do not use browser cookies** but fails when Chrome is
selected, macOS is preventing the unsigned application context from reading or
decrypting Chrome cookies. Add `/Applications/JiveFetch.app` to **System Settings →
Privacy & Security → Full Disk Access**, quit JiveFetch completely, and open it again.
If macOS asks whether JiveFetch may access **Chrome Safe Storage** in Keychain, allow
access only for the copy whose checksum you verified.

For public media, leave browser cookies disabled. For authenticated media, Firefox with
the same signed-in session can be used as a temporary alternative. A Developer ID-signed
and notarized JiveFetch build is the durable macOS solution; the current preview does not
yet provide it.

## Keep the engines current

- Homebrew: `brew upgrade yt-dlp ffmpeg deno`
- Windows Package Manager: `winget upgrade --id yt-dlp.yt-dlp --exact` and
  `winget upgrade --id Gyan.FFmpeg --exact`
- Deno with Windows Package Manager: `winget upgrade --id DenoLand.Deno --exact`
- Official standalone `yt-dlp`: `sudo yt-dlp -U`
- Distribution packages: update them through the distribution package manager.

Restart JiveFetch after an engine update so it detects the new version.

## Official references

- [`yt-dlp` installation](https://github.com/yt-dlp/yt-dlp/wiki/Installation)
- [`yt-dlp` external JavaScript runtime guide](https://github.com/yt-dlp/yt-dlp/wiki/EJS)
- [Deno installation](https://docs.deno.com/runtime/getting_started/installation/)
- [FFmpeg downloads and package links](https://ffmpeg.org/download.html)
- [Homebrew `yt-dlp` formula](https://formulae.brew.sh/formula/yt-dlp)
- [Homebrew FFmpeg formula](https://formulae.brew.sh/formula/ffmpeg)
- [Windows Package Manager documentation](https://learn.microsoft.com/windows/package-manager/winget/)
