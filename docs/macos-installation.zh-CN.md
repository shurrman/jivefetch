[English](macos-installation.md) | [Русский](macos-installation.ru.md) | [简体中文](macos-installation.zh-CN.md)

# 在 macOS 上安装未签名构建

## 适用范围

当前 JiveFetch 预览版面向 Apple Silicon (`arm64`) 构建，但尚未使用 Apple Developer
ID 证书签名，也未经过 notarization。它们在 GitHub 上标记为 pre-release，且未经
Apple 验证。

在签名和 notarization 完成前，Gatekeeper 可能提示应用已损坏。仅对从官方仓库下载的
JiveFetch DMG 使用以下步骤。

## 验证下载

从同一个 GitHub release 下载 `JiveFetch_<version>_aarch64.dmg` 和
`SHA256SUMS-macOS-ARM64.txt`。在 Terminal 中运行：

```bash
cd ~/Downloads
shasum -a 256 -c SHA256SUMS-macOS-ARM64.txt
```

仅在得到以下结果时继续：

```text
JiveFetch_<version>_aarch64.dmg: OK
```

## 安装与准备

1. 打开 DMG，将 `JiveFetch.app` 拖入 `Applications`。
2. 弹出 DMG。
3. 在 Terminal 中运行：

```bash
codesign --force --deep --sign - /Applications/JiveFetch.app
xattr -dr com.apple.quarantine /Applications/JiveFetch.app
open /Applications/JiveFetch.app
```

这些命令通常不需要 `sudo`。如果 macOS 报告权限错误，请删除该副本，并以当前用户
通过 Finder 重新复制应用，而不是修改系统级权限。

## 命令作用

- `codesign ... --sign -` 为完整 app bundle 创建本地 ad-hoc 签名。它不是 Apple
  identity，也不能证明发布者真实性。
- `xattr ... com.apple.quarantine` 仅移除此应用的下载隔离属性。不要对大范围目录运行
  该命令。
- `open` 启动准备好的应用。

用新的未签名版本替换 JiveFetch 后，请重新执行验证和准备。若应用来自其他来源，或
checksum 与 release manifest 不一致，请勿运行这些命令。

## 替换应用后的隐私权限

每次 ad-hoc 重新构建都会产生不同的代码身份。替换应用后，macOS 可能不再把先前的
Downloads 或完全磁盘访问权限决定与新副本匹配。如果仍无法访问，请在**系统设置 →
隐私与安全性 → 完全磁盘访问权限**中移除旧的 JiveFetch 条目，再次添加
`/Applications/JiveFetch.app`，完全退出应用后重新打开。如果 macOS 另行询问是否允许
访问 Downloads 文件夹，请允许。

只有当 `yt-dlp` 实际开始读取并解密 Chrome Cookie 时，钥匙串才会显示
**Chrome Safe Storage** 提示。停用 Cookie，或尝试在此之前因下载引擎、JavaScript
运行时、网络或下载目录错误而停止时，都不会出现该提示。

## 剩余限制

无需 Terminal 准备即可正常双击安装和启动，需要 Developer ID signing 与 Apple
notarization。这些工作目前有意推迟；发布说明已明确当前 artifact 未签名且未 notarize。
