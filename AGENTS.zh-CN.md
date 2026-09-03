[Русский](AGENTS.md) | [English](AGENTS.en.md) | [简体中文](AGENTS.zh-CN.md)

# Agent 工作说明

这是规范俄语 `AGENTS.md` 的简体中文译本。若含义不一致，应保留更严格的安全规则，
并同步更新三种语言。

## 入口文件

- `README.md`：仓库概览与已验证命令。
- `CHANGES.md`：变更、前后指标和已知问题。
- `MEMORY.md`：长期、无秘密的决定与继续工作的位置。
- `docs/`：需求、架构、生命周期、安全、打包、许可、路线图、研究和本地化策略。

## 工作方式

- 开始前阅读 `MEMORY.md`；自主工作并尽量减少打断。
- 如果在重复结构中发现问题，也检查类似 role/component。
- 执行 `rm`、删除 playbook、重写 inventory 或大范围不可逆修改前必须询问。
- 用户要求 commit 时按逻辑阶段拆分；验证成功后才能 commit/push。
- 重构后按需更新 README、AGENTS、CHANGES 和 MEMORY。
- 大任务总结必须包含修改前/后的指标和既有问题。

## 旧仓库禁令

- 不得提交 vault 密码文件，例如 `.ansible.vault`、`vault_pass*`。
- 不得引入弃用的 Ansible 语法：`with_items`、含糊的 `include:`、扁平模块参数、
  `become: yes/no`。
- 不得机械地把现有 `shell:` 改为 native module。

## JiveFetch 规则

JiveFetch 是本地优先的 Tauri 桌面应用；Rust 拥有队列、持久状态和进程树，React
只展示投影并发送类型化意图。

## 开发规则

- 队列、任务生命周期和进程所有权属于 Rust；React 只是投影。
- scheduler 状态变更必须先事务写入 SQLite，再发布 UI 事件。
- 用户并发数、全局速度限制和输出目录必须保存在 SQLite 中并由 Rust scheduler 执行；
  UI 控件不是第二事实来源。
- PID 不是所有权。只能停止本应用拥有的 Unix process group/session 或 Windows
  Job Object，绝不按可执行文件名批量 kill。
- 跨平台暂停是可恢复的受控停止加新 attempt；OS suspend 只能作为优化。
- UI 输入必须变成类型化 Rust 参数，绝不能拼接 shell 命令。
- 不记录或提交 Cookie、token、认证头、浏览器 profile、解密临时文件或签名 URL。
- 密钥存入 OS credential service；大型 Cookie blob 使用其中保存的随机密钥加密。
- SQLite migration 必须向前安全并有恢复测试。
- 队列功能必须有状态迁移和崩溃/重启测试；进程测试使用确定性 helper。
- 只有 Windows、macOS、Linux 原生测试通过后才能宣称打包支持。
- Release tag 必须与应用和 package 版本一致。在 signing、notarization、sidecar
  许可证和 release gate 完成前，发布必须标记为 pre-release。
- MediaHarbor 和 FlowGrab 仅为研究参考；不复制代码、资源、UI 文案或内部结构。
- 私有 `origin` 已配置。只有用户明确要求且相关检查通过后，才可以 commit 或 push。

## 本地化与文档

- 每个 Markdown 文档都有完整 EN/RU/简体中文版本，顶部互相链接。
- 所有用户可见字符串必须经过 i18n。首次运行默认 EN，明确选择会保存在本机。
- 三种语言必须在同一阶段更新；缺少翻译即表示工作未完成。
- README 只记录已存在并验证的命令；计划必须明确标为未来工作。
- 交付前执行相关 format、lint、测试、`git diff --check`、相对链接和秘密检查。
