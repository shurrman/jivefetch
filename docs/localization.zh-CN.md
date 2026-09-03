[English](localization.md) | [Русский](localization.ru.md) | [简体中文](localization.zh-CN.md)

# 本地化策略

## 支持语言

- `en`：英语，首次运行强制默认，也是 source locale。
- `ru`：俄语。
- `zh-CN`：简体中文。

应用不会用操作系统语言静默替换首次 EN 默认值。用户明确选择语言后，该选择会保存在
本机，并在下次启动时恢复。

## 应用字符串

- 所有用户可见字符串在 `src/i18n.tsx` 中使用稳定 key；component 不内嵌三语正文。
- 三个字典必须拥有相同 key，TypeScript 负责类型检查。
- 日期和数字按所选 locale 格式化；持久化 domain value 保持中立。
- 任务状态、错误、通知、无障碍标签、菜单、installer 和 updater 都必须本地化。
- 不把原始引擎输出伪装成翻译。已知结构化错误映射为本地化说明；未知且已脱敏的输出
  明确标记为 engine diagnostics。
- Runtime 缺失时回退 EN，但任何已提交的缺失翻译都会阻止发布。

## 文档

英语文件不加后缀，俄语使用 `.ru.md`，简体中文使用 `.zh-CN.md`。本地 agent 指令和
memory 不属于版本化产品文档，因此不参与仓库 translation set 校验。

每个 Markdown 顶部必须直接链接三个语言版本。翻译中的内部链接应尽量指向同语言文件。

翻译应保持语义一致，而不是机械逐字翻译。命令、标识符、代码、状态名、路径和安全
不变量必须精确；不得削弱安全规则、宣称未实现功能或省略发布 gate。

## 修改流程

1. 修改英语/source 语义和 UI key。
2. 在同一逻辑阶段更新 RU 和简体中文。
3. 运行 key parity、翻译集合及 Markdown 链接检查。
4. 使用较长俄语标签和 CJK 字形检查布局。
5. 三种语言全部完成后才能 commit。

## 验收

- 清空本地存储后应用以 EN 打开。
- 三种语言都可选择并在重启后保留。
- 所有字典 key 完整。
- HTML `lang` 与所选语言一致。
- 较长标签不会破坏队列控制。
- 每个 Markdown base 都有 EN/RU/zh-CN 三个互链文件。
- 三种 README 对命令和功能状态的含义一致。
