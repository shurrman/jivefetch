[English](security.md) | [Русский](security.ru.md) | [简体中文](security.zh-CN.md)

# 安全与认证

## 1. 安全目标

目标是让每个组件只拥有必要权限，并避免认证材料泄漏。

## 2. Threat model

JiveFetch 会对不可信 URL 运行第三方工具，并可能短暂访问浏览器认证。需要防御恶意 URL、
metadata、filename、engine output，shell injection、path traversal、秘密泄漏、其他本地用户
读取临时 Cookie、篡改更新、remote IPC 和误杀无关进程。完全被攻陷的用户账户、kernel、
browser 或 OS credential store 不在 MVP threat model 内。

## 3. 认证模式

### 3.1 Browser reference

Browser 模式只保存 browser kind 和经过验证的 profile reference，并使用类似
`--cookies-from-browser` 的参数；不复制 Cookie 值。

### 3.2 导入 Cookie file

导入 Netscape Cookie 时：本地验证；使用随机 DEK 加密；加密 blob 存在 app data；DEK
存在 OS credential service；只为短期 CredentialLease 解密为 user-only 临时文件；process
tree 退出后及 startup 时清理。没有 secure store 时 persistent import 必须 fail-closed，
不得回退明文或机器派生密钥。

### 3.3 未来 credentials

新 password/token flow 必须有 scoped broker contract、secure-store policy 和 redaction
tests，不能直接增加 raw credential field。

## 4. Secret data 规则

Cookie、password、token、认证头、签名 URL query 和解密 temp data 全部视为秘密。
Redaction 在持久化前进行。Support bundle 使用 allowlist，不包含完整 DB、secret blob、
browser profile、raw args/env 或媒体文件。

## 5. 进程执行边界

- 直接 executable + 参数向量；verified registry；最小 env；无 shell。
- stdout/stderr 不可信且有大小限制；渲染前 escape；普通 UI 不允许任意 executable/env。

## 6. 文件系统边界
- Canonical destination 与 sanitized template；拒绝 traversal、device name、ADS、control
  character 和 symlink escape；collision policy 必须明确。
- Remove-with-files 只能删除 tracked canonical artifact，不能删除目录或展开 glob。

## 7. Tauri/webview 边界

每 window 最小 capability；Rust 校验 payload 大小和 scheme；只加载 packaged local UI；
严格 CSP 禁止 remote script；thumbnail 通道受限；remote page 无 privileged IPC；secret 不
进入 React/storage/clipboard/log。

## 8. Engine 与应用更新

Engine 更新使用 fixed compatibility、declared upstream、signed manifest、hash、staging、
bounded health check、atomic activation 和 rollback；未完整验证的 binary 绝不执行。

应用更新与 engine 更新使用独立 trust path。

## 9. 默认隐私策略

初始版本无 telemetry；history 可在不删除 media 的情况下清理；clipboard opt-in 且只看
URL；deep link 必须 review；认证作用域对用户可见。

## 10. 安全验证

Injection、path/redaction fuzz、各 OS temp ACL/permission、sentinel secret 扫描、remote
capability denial、tampered update/downgrade/rollback、owned process test。任何不安全
fallback 都会阻止发布。

## 11. 已跟踪的 upstream advisory

Linux dependency graph 通过 Tauri 和 GTK 0.18 引入 `glib 0.18.5`，其中
`VariantStrIter` 受 `GHSA-wrw7-89jp-8q8g` 影响；JiveFetch 不调用该 API。已修复的
`glib 0.20` 与 GTK 当前的 `^0.18` 约束不兼容，因此只为私有 `v0.1.0` pre-release
临时接受此例外。Stable release 前必须升级 Tauri/GTK 链或以其他方式消除该 advisory。
