# Slash 指令

关于 Codex CLI 的 slash 指令概览，请参考官方文档：
https://developers.openai.com/codex/cli/slash-commands

补充：`/lang` 用于切换界面语言（回车后弹出选择列表）。

## 本仓库新增命令

- `/spec`：打开规范配置弹窗（当前仅 `Parallel Priority`）。
- 开启 `Parallel Priority` 后，Codex 会在每次请求时动态注入内置提示词（按当前语言选择中/英文）。
- 关闭后，后续请求不再携带该提示词；不会创建 `.codex/spec/AGENTS.md` 等外部文件。
