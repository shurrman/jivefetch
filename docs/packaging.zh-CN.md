[English](packaging.md) | [Русский](packaging.ru.md) | [简体中文](packaging.zh-CN.md)

# 跨平台打包与引擎分发

## 1. 交付模型

JiveFetch 有两个版本化层：已签名 Tauri 应用，以及受管理的 `yt-dlp`、FFmpeg/ffprobe 和
可选 aria2c。Installer 在许可证和体积允许时包含固定 baseline；已验证的新版本存放在
app data，通过签名 compatibility manifest 选择，不修改密封 app bundle。

## 2. Engine registry

Engine registry 按顺序选择：健康的 managed override、bundled baseline、用户明确允许且
验证过的 system executable。记录 version、target、path identity、hash/source、capability、
license 和 health time；未知 `$PATH` 工具不会静默优先。

## 3. Layout

App bundle 保存不可变 baseline、notice 和 manifest；app data 保存 versioned managed
engine、staging 和 last-known-good metadata；用户配置仅保存 settings/reference；cache
只保存可安全删除的 probe/thumbnail。Mutable engine 不写入 signed/sealed bundle。

## 4. Target matrix

| OS | 架构 | 候选软件包 | 进程所有权 |
| --- | --- | --- | --- |
| Windows | 初始 x86_64，评估 arm64 | NSIS/MSI | Job Object kill-on-close |
| macOS | arm64/x86_64 | signed/notarized app/DMG | session/process group |
| Linux | 初始 x86_64，评估 arm64 | AppImage、按需 deb/rpm | session/process group |

只有 native CI 和 smoke 通过后才能宣称支持；仅 cross-compile 不足以证明。

## 5. 平台要求

### Windows

需要 WebView2 policy、code signing、native path、早期 Job assignment、user ACL、
long/non-ASCII path、杀毒延迟和安全卸载测试。

### macOS

需要 native/universal build、nested signing、hardened runtime、notarization、bundle 外
可变 engine、Gatekeeper/keychain/sleep 测试。

### Linux

需要 WebKitGTK、Secret Service fail-closed、明确 glibc target、Wayland/X11、AppImage
和 distro upgrade 测试。

## 6. Engine 获取与更新

选择 allowed target/version；带限制下载到 staging；验证 manifest signature 与 hash；
安全解包且防 traversal/symlink escape；检查名称和权限；运行有界 `--version` probe；
原子移动到不可变版本目录；激活并保留 previous；失败自动 rollback。不得越过 compatibility。

## 7. 许可证 gate

JiveFetch 源代码采用 Apache-2.0。`0.1.1` 只调用验证过的系统 executable，不再分发引擎。
Bundled/managed 引擎交付仍需依据[许可文档](licensing.zh-CN.md)审核准确二进制文件。

## 8. 可复现性与供应链

固定 toolchain/
lockfile/Actions SHA，最小 CI 权限，生成 SBOM、checksum 和 provenance，签名秘密不进仓库。

## 9. 发布 gate

在 production signing 就绪前，可以通过 native CI 构建私有 preview，但必须标记为
pre-release，明确说明 artifact 未签名/未 notarize，且不得宣称 production support。
Stable release 必须通过以下全部 gate。

每个 target 必须通过 frontend/Rust、migration/crash、owned process tree、Cookie secret、
engine rollback、installer lifecycle、signature/notarization、SBOM/license 和 clean-machine
probe/download/postprocess gate。
