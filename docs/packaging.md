[English](packaging.md) | [Русский](packaging.ru.md) | [简体中文](packaging.zh-CN.md)

# Cross-platform packaging and engine distribution

## 1. Delivery model

JiveFetch has two versioned layers:

- the signed Tauri application;
- managed external engines (`yt-dlp`, `ffmpeg`/`ffprobe`, optional `aria2c`).

An installer contains a pinned baseline sufficient for first use where licensing and
package size permit. Newer verified engine versions may be installed in the user's app
data and selected through a signed compatibility manifest. The application bundle is
not modified in place.

## 2. Engine registry

The Rust engine registry resolves each tool by this order:

1. healthy managed override explicitly activated by JiveFetch;
2. bundled baseline resource/sidecar;
3. optional user-selected system executable, only after validation and explicit
   policy allows it.

For every selected tool, record name, version, target, path identity, hash/source,
capabilities, license notice, and health-check time. A path from `$PATH` is never
silently preferred over a verified bundled tool.

## 3. Layout

Conceptual app-data layout:

```text
JiveFetch/
├── jivefetch.sqlite3
├── engines/
│   ├── manifests/
│   ├── staging/
│   ├── yt-dlp/<version>/<target>/
│   ├── ffmpeg/<version>/<target>/
│   └── aria2/<version>/<target>/
├── secrets/                    # encrypted blobs only
├── temp/                       # app-owned, startup-cleaned leases
└── logs/                       # bounded and redacted
```

Managed versions are immutable directories. Activation updates a small atomic pointer
or database setting; rollback selects the previous healthy directory.

## 4. Target matrix

The exact minimum versions are a Phase 0 decision. The intended packaging matrix is:

| OS | Architectures | Package candidates | Process ownership |
| --- | --- | --- | --- |
| Windows | x86_64 initially; arm64 evaluated | NSIS and/or MSI | Job Object, kill-on-close |
| macOS | arm64 and x86_64 | signed/notarized DMG or app bundle | new session/process group |
| Linux | x86_64 initially; arm64 evaluated | AppImage plus deb/rpm as justified | new session/process group |

Support is claimed only for a row that passes native CI and release smoke tests. A
cross-compiled binary alone is not sufficient evidence.

## 5. Platform requirements

### Windows

- Decide whether to bootstrap or require the supported WebView2 runtime.
- Sign installer and binaries with the release certificate.
- Quote paths through native APIs, not shell rules.
- Assign children to a Job Object before they can spawn unowned descendants.
- Apply user-only ACLs to app data and credential temp files.
- Test long paths, reserved names, non-ASCII paths, antivirus latency, installer
  upgrade, and uninstall-with-user-data-preserved behavior.

### macOS

- Build arm64 and x86_64 artifacts natively or produce a verified universal bundle.
- Document the temporary, checksum-first procedure for
  [installing an unsigned macOS release](macos-installation.md).
- Sign nested sidecars in the correct order, enable hardened runtime, and notarize the
  final deliverable.
- Keep mutable engine overrides outside the sealed application bundle.
- Test quarantine, Gatekeeper, spaces/non-ASCII paths, sandbox/keychain prompts, sleep
  and wake, and update rollback.

### Linux

- Document WebKitGTK and desktop integration requirements per package.
- Support Secret Service/libsecret for persistent credential keys; fail closed when
  no secure store is available.
- Test glibc compatibility for declared targets, Wayland/X11 behavior, headless
  credential-service absence, AppImage mount paths, and distro package upgrades.
- Avoid claiming one binary supports musl/glibc or every distribution without tests.

## 6. Engine acquisition and updates

The release pipeline produces or consumes a signed engine manifest. Activation flow:

1. choose an allowed target/version;
2. download with size and timeout limits to staging;
3. verify manifest signature and artifact hash;
4. unpack without path traversal or symlink escape;
5. verify expected executable names and permissions;
6. run `--version`/capability probes in a restricted, bounded subprocess;
7. atomically move to an immutable version directory;
8. activate and retain the prior version;
9. roll back automatically if the first real health check fails.

Automatic engine updates are configurable and never install a version outside the
application's declared compatibility range.

## 7. Licensing gate

Before any public build:

- inventory exact source and binary licenses for Tauri dependencies and each sidecar;
- distinguish source-code license from the license obligations of distributed
  standalone binaries and their bundled dependencies;
- record the FFmpeg configuration used, because enabled codecs/libraries can change
  LGPL/GPL obligations;
- include required notices, source offers, and license texts;
- verify that the chosen JiveFetch license is compatible with the distribution model;
- document update sources and attribution in the application.

JiveFetch source uses Apache-2.0. Version `0.2.0` only invokes validated system
executables and does not redistribute engines. Bundled or managed engine delivery
still requires the exact-binary review in [Licensing](licensing.md).

## 8. Reproducibility and supply chain

- Pin Rust and Node toolchains plus lockfiles after scaffold creation.
- Pin GitHub Actions by commit SHA and minimize workflow permissions.
- Generate an SBOM and checksums for each release artifact.
- Record build provenance and exact sidecar hashes.
- Keep signing credentials outside the repository and CI logs.
- Build release candidates in clean native runners and compare expected contents.

## 9. Release gates

The newest successfully published release is marked as GitHub `Latest`, including
while production signing is not yet in place. `Latest` identifies release ordering,
not certification or production readiness. Unsigned/unnotarized assets must be called
out explicitly and must not claim production support. A fully supported production
release must satisfy every gate below.

A target artifact is releasable only when:

- frontend and Rust formatting/lint/tests pass;
- database migration and crash-recovery suites pass;
- owned process-tree termination passes with child and grandchild helpers;
- browser/imported-cookie storage tests pass without sentinel leakage;
- baseline and managed engine discovery/rollback tests pass;
- installer install, launch, update, and uninstall smoke tests pass;
- signature/notarization verification passes where applicable;
- SBOM, checksums, license notices, and provenance are attached;
- a clean machine completes one probe/download/post-process flow.

Failures are reported per target; success on one OS does not waive another target's
gate.
