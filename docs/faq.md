## 常见问题（FAQ）

本 FAQ 汇总了最常见的问题，并指向 `docs/` 中的详细指南。

### OpenAI 不是在 2021 年发布过一个叫 Codex 的模型吗？

2021 年，OpenAI 发布了 Codex —— 一套可以根据自然语言提示生成代码的 AI 系统。那个最初的 Codex 模型已在 2023 年 3 月停止服务，与本 CLI 工具并无关系。

### 目前推荐使用哪些模型？

推荐在 Codex 中使用 GPT-5.1 Codex Max，这是我们最强的编码模型。默认推理级别为 medium，可在复杂任务中通过 `/model` 切换至 high 或 xhigh（当模型支持时，例如 `gpt-5.1-codex-max` 与 `gpt-5.2`）。

若想使用旧版模型，可改用基于 API 的鉴权方式，并在启动 codex 时通过 `--model` 标志指定模型。

### 审批与沙箱模式如何协同？

审批机制用于在 Codex 需要提升权限运行工具调用前先征求同意——通常是离开沙箱或在无隔离的情况下重试失败命令。沙箱模式则提供默认隔离（`Read Only`、`Workspace Write` 或 `Danger Full Access`，详见 [Sandbox & approvals](./sandbox.md)）。

### 不开 TUI 能自动化任务吗？

可以。[`codex exec`](./exec.md) 可在非交互模式下运行 Codex，提供流式日志、JSONL 输出与结构化 Schema 支持。该命令会遵循你在 [配置指南](./config.md) 中设置的沙箱与审批策略。

### 如何阻止 Codex 修改我的文件？

默认情况下，Codex 在当前工作目录具备修改权限（Auto 模式）。如需禁止编辑，可在 CLI 中使用 `--sandbox read-only`。也可以在对话过程中通过 `/approvals` 调整审批级别。

### 如何把 Codex 接入 MCP 服务器？

在 `config.toml` 中配置 MCP 服务器，可参考 [Config -> Connecting to MCP servers](./config.md#connecting-to-mcp-servers) 提供的示例。

### 登录遇到问题，该检查什么？

按以下步骤排查：

1. 参考 [Authentication](./authentication.md) 中的流程，确认 `~/.codex/auth.json` 内存在正确的凭据。
2. 如果你在无头或远程机器上使用 Codex，确保按照 [Authentication -> Connecting on a "Headless" Machine](./authentication.md#connecting-on-a-headless-machine) 配置端口转发。

### 可以在 Windows 上直接使用吗？

在 Windows 上直接运行 Codex 可能可行，但暂未正式支持。我们建议通过 [Windows Subsystem for Linux (WSL2)](https://learn.microsoft.com/en-us/windows/wsl/install) 使用。

### 安装完成后应该先做什么？

先完成 [Install & build](./install.md) 中的快速设置，然后阅读 [Getting started](./getting-started.md)，了解交互式使用技巧、提示例子与 AGENTS.md 的写法。

### `brew upgrade codex` 没有把我升级到最新版

如果你正在使用 v0.46.0 或更早版本，`brew upgrade codex` 不会升级到最新版本，因为我们已将 Homebrew 配方迁移到 cask。升级步骤如下：

```bash
brew uninstall --formula codex
brew install --cask codex
```

重新安装后，执行 `brew upgrade --cask codex` 即可保持后续版本更新。
