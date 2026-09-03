[English](macos-installation.md) | [Русский](macos-installation.ru.md) | [简体中文](macos-installation.zh-CN.md)

# Installing an unsigned build on macOS

## Scope

JiveFetch `0.2.0` is built for Apple Silicon (`arm64`) but is not signed with an
Apple Developer ID certificate or notarized. GitHub `Latest` means that it is the
newest published build; it does not mean that Apple has verified it.

Until signing and notarization are enabled, Gatekeeper may report that the app is
damaged. Use the following procedure only for a JiveFetch DMG downloaded from the
official repository.

## Verify the download

Download both `JiveFetch_0.2.0_aarch64.dmg` and
`SHA256SUMS-macOS-ARM64.txt` from the same GitHub release. In Terminal, run:

```bash
cd ~/Downloads
shasum -a 256 -c SHA256SUMS-macOS-ARM64.txt
```

Continue only when the result is:

```text
JiveFetch_0.2.0_aarch64.dmg: OK
```

For `0.2.0`, the expected SHA-256 is
`7377fa07f247124ecd81388982d9360ba46db4f1e56ac419e7185384c73d9530`.

## Install and prepare

1. Open the DMG and drag `JiveFetch.app` into `Applications`.
2. Eject the DMG.
3. Run the following commands in Terminal:

```bash
codesign --force --deep --sign - /Applications/JiveFetch.app
xattr -dr com.apple.quarantine /Applications/JiveFetch.app
open /Applications/JiveFetch.app
```

These commands should not normally require `sudo`. If macOS reports a permissions
error, remove that copy and copy the application again through Finder as the current
user instead of changing system-wide permissions.

## What the commands do

- `codesign ... --sign -` creates a local ad-hoc signature for the complete app bundle.
  It is not an Apple identity and does not prove publisher authenticity.
- `xattr ... com.apple.quarantine` removes the download quarantine attribute only from
  this application. Do not use this command on broad directories.
- `open` launches the prepared application.

Repeat the verification and preparation after replacing JiveFetch with a newer unsigned
build. Do not use these commands for an app from another source or when its checksum does
not match the release manifest.

## Remaining limitation

Normal double-click installation without Terminal preparation requires Developer ID
signing and Apple notarization. These are intentionally deferred; the published release
notes identify the current artifacts as unsigned and unnotarized.
