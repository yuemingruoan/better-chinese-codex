## 非交互模式

在非交互模式下使用 Codex，可以自动化常见工作流。

```shell
codex exec "统计这个项目的代码总行数"
```

非交互模式不会询问命令或编辑审批。默认情况下它在 `read-only` 模式运行，因此无法修改文件，也不能执行需要联网的命令。

使用 `codex exec --full-auto` 允许编辑文件；使用 `codex exec --sandbox danger-full-access` 可同时允许编辑和联网命令。

### 默认输出模式

默认情况下，Codex 会把执行过程流式写到 stderr，而只把代理的最终回复写到 stdout。这样可以轻松把 `codex exec` 的输出传递给其他工具，而无需额外过滤。

若想把 `codex exec` 的最终输出写入文件，除了常规的 shell 重定向（例如 `>`）之外，还可以使用专门的 `-o`/`--output-last-message` 标志指定输出文件。

### JSON 输出模式

`codex exec` 支持 `--json` 模式，会在代理运行时以 JSON Lines（JSONL）格式把事件流式写到 stdout。

支持的事件类型：

- `thread.started`：线程被创建或恢复时触发。
- `turn.started`：轮次开始时触发。一次轮次包含用户消息与助手回复之间的全部事件。
- `turn.completed`：轮次结束时触发，包含 token 用量。
- `turn.failed`：轮次失败时触发，包含错误详情。
- `item.started`/`item.updated`/`item.completed`：线程条目被添加、更新或完成时触发。
- `error`：流中出现无法恢复的错误时触发，包含错误信息。

支持的条目类型：

- `agent_message`：助手消息。
- `reasoning`：助手思考过程的摘要。
- `command_execution`：助手执行命令。
- `file_change`：助手修改文件。
- `mcp_tool_call`：助手调用 MCP 工具。
- `web_search`：助手执行网页搜索。
- `todo_list`：使用计划工具时的 TODO 列表，步骤变化时会持续更新。

通常 `agent_message` 会在轮次结束时出现。

示例输出：

```jsonl
{"type":"thread.started","thread_id":"0199a213-81c0-7800-8aa1-bbab2a035a53"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"reasoning","text":"**正在查找 README 文件**"}}
{"type":"item.started","item":{"id":"item_1","type":"command_execution","command":"bash -lc ls","aggregated_output":"","status":"in_progress"}}
{"type":"item.completed","item":{"id":"item_1","type":"command_execution","command":"bash -lc ls","aggregated_output":"2025-09-11\nAGENTS.md\nCHANGELOG.md\ncliff.toml\ncodex-cli\ncodex-rs\ndocs\nexamples\nflake.lock\nflake.nix\nLICENSE\nnode_modules\nNOTICE\npackage.json\npnpm-lock.yaml\npnpm-workspace.yaml\nPNPM.md\nREADME.md\nscripts\nsdk\ntmp\n","exit_code":0,"status":"completed"}}
{"type":"item.completed","item":{"id":"item_2","type":"reasoning","text":"**在仓库根目录检查 README**"}}
{"type":"item.completed","item":{"id":"item_3","type":"agent_message","text":"仓库根目录里确实有 `README.md`。"}}
{"type":"turn.completed","usage":{"input_tokens":24763,"cached_input_tokens":24448,"output_tokens":122}}
```

### 结构化输出

默认情况下，代理使用自然语言回复。通过 `--output-schema` 传入 JSON Schema，可让代理输出符合该 Schema 的 JSON。

JSON Schema 必须遵循 [严格 Schema 规则](https://platform.openai.com/docs/guides/structured-outputs)。

示例 Schema：

```json
{
  "type": "object",
  "properties": {
    "project_name": { "type": "string" },
    "programming_languages": { "type": "array", "items": { "type": "string" } }
  },
  "required": ["project_name", "programming_languages"],
  "additionalProperties": false
}
```

```shell
codex exec "提取项目的关键信息" --output-schema ~/schema.json
...

{"project_name":"Codex CLI","programming_languages":["Rust","TypeScript","Shell"]}
```

把 `--output-schema` 与 `-o` 配合，可以只输出最终 JSON。`-o` 同样可以接受文件路径，将 JSON 写入指定文件。

### Git 仓库要求

Codex 需要在 Git 仓库中运行以避免破坏性更改。若想跳过该检查，可执行 `codex exec --skip-git-repo-check`。

### 恢复非交互会话

使用 `codex exec resume <SESSION_ID>` 或 `codex exec resume --last` 恢复先前的非交互会话。这样可以保留会话上下文，继续追问或布置新任务。

```shell
codex exec "Review 这次提交，排查 use-after-free 风险"
codex exec resume --last "修复 use-after-free 问题"
```

仅会保存对话上下文；若需自定义 Codex 行为，仍需重新提供相关标志。

```shell
codex exec --model gpt-5.1-codex-max --json "Review 这次提交，排查 use-after-free 风险"
codex exec --model gpt-5.1 --json resume --last "修复 use-after-free 问题"
```

## 鉴权

默认情况下，`codex exec` 会沿用 Codex CLI 与 VS Code 扩展的鉴权方式。你可以通过设置 `CODEX_API_KEY` 环境变量来覆盖 API key。

```shell
CODEX_API_KEY=your-api-key-here codex exec "解决合并冲突"
```

注意：`CODEX_API_KEY` 仅在 `codex exec` 中受支持。
