# AGENTS.md

`AGENTS.md` 用于为 Codex 提供项目级指导与约束。扫描与加载规则如下：

1. 从仓库根目录到当前工作目录（含）逐级查找，每个目录最多取一个文件。
2. 每个目录内优先级为：`AGENTS.override.md` → `AGENTS.md` → 配置的 fallback 文件名。
3. 对当前工作目录，若同时存在 `AGENTS.md` 与 `.codex/AGENTS.md`，只取修改时间较新的一个（时间相同默认取 `AGENTS.md`）。

更多信息请参考官方文档：[AGENTS.md 指南](https://developers.openai.com/codex/guides/agents-md)。
