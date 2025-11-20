# 配置（Config）

Codex 的配置系统可以精细控制模型、执行环境与 CLI 可用的各类集成。使用本文档时，可结合 [`codex exec`](./exec.md) 中的工作流示例、[Sandbox & approvals](./sandbox.md) 介绍的安全限制，以及 [AGENTS.md 指南](./agents_md.md) 中的项目协作建议。

## 快速导航

- [特性开关](#feature-flags)
- [模型选择](#model-selection)
- [执行环境](#execution-environment)
- [MCP 集成](#mcp-integration)
- [可观测性与遥测](#observability-and-telemetry)
- [配置档与覆盖顺序](#profiles-and-overrides)
- [参考表](#config-reference)

设置配置项有多种途径：

- 针对单个配置的命令行参数，例如优先级最高的 `--model o3`。
- 通用的 `-c` / `--config` 语法，可传入 `key=value` 键值对，例如 `--config model="o3"`。
  - `key` 支持使用点号表示层级，如 `--config model_providers.openai.wire_api="chat"`。
  - 与 `config.toml` 保持一致，值使用 TOML 字面量而非 JSON，例如 `key='{a = 1, b = 2}'`、而不是 `key='{"a": 1, "b": 2}'`。
    - 外层引号必不可少，否则 shell 会按空格拆分，导致 `codex` 只收到 `-c key={a` 以及额外的无效参数 `=、1,、b, …`。
  - 值可以是任意 TOML 对象，例如 `--config shell_environment_policy.include_only='["PATH", "HOME", "USER"]'`。
  - 若传入的内容无法解析为合法 TOML，会被视作普通字符串；因此 `-c model='"o3"'` 与 `-c model=o3` 等价。
    - 第一种写法直接得到字符串 `"o3"`；第二种写法虽然缺少引号，但由于 `o3` 不是合法 TOML，将被自动视作字符串 `"o3"`。
    - 类似地，`-c key="true"` 会正确解析为布尔值 `true` 而非字符串；若确实需要字面文本 `"true"`，请写作 `-c key='"true"'`（注意双重引号）。
- `$CODEX_HOME/config.toml` 文件；`CODEX_HOME` 默认为 `~/.codex`，同样也是 Codex 生成日志与缓存的根目录。

`--config` 参数与 `config.toml` 文件均支持下列选项：

## Feature flags

Optional and experimental capabilities are toggled via the `[features]` table in `$CODEX_HOME/config.toml`. If you see a deprecation notice mentioning a legacy key (for example `experimental_use_exec_command_tool`), move the setting into `[features]` or pass `--enable <feature>`.

```toml
[features]
streamable_shell = true          # enable the streamable exec tool
web_search_request = true        # allow the model to request web searches
# view_image_tool defaults to true; omit to keep defaults
```

支持的特性如下：

| Key                                       | 默认值 | 阶段        | 描述                                                         |
| ----------------------------------------- | :----: | ----------- | ------------------------------------------------------------ |
| `unified_exec`                            | false  | Experimental | 启用统一的 PTY 执行器                                        |
| `streamable_shell`                        | false  | Experimental | 使用可流式传输的 `exec-command`/`write-stdin` 组合           |
| `rmcp_client`                             | false  | Experimental | 允许通过 HTTP 流式 MCP 服务器使用 OAuth                      |
| `apply_patch_freeform`                    | false  | Beta        | 暴露自由格式的 `apply_patch` 工具                           |
| `view_image_tool`                         |  true  | Stable      | 暴露 `view_image` 工具                                       |
| `web_search_request`                      | false  | Stable      | 允许模型主动发起 Web 搜索                                    |
| `experimental_sandbox_command_assessment` | false  | Experimental | 启用模型辅助的沙箱风险评估                                   |
| `ghost_commit`                            | false  | Experimental | 为每个回合创建一次“幽灵提交”                                 |
| `enable_experimental_windows_sandbox`     | false  | Experimental | 使用受限令牌版的 Windows 沙箱                                |

提示：

- 未显式配置的键会沿用默认值。
- 旧版布尔开关（例如 `experimental_use_exec_command_tool`、`experimental_use_unified_exec_tool`、`include_apply_patch_tool` 以及类似的 `experimental_use_*`）已废弃；请改用 `[features].<key>`，以免不断看到弃用警告。

## 模型选择

### model

指定 Codex 要使用的模型：

```toml
model = "gpt-5.1"  # 覆盖默认值（默认是跨平台的 "gpt-5.1-codex-max"）
```

### model_providers

用于扩展 Codex 内置的模型提供者列表。你在此表中使用的键名，也是 `model_provider` 字段可选的值。

> [!NOTE]
> 如果键名与内置提供者相同，原有定义不会被覆盖；只有全新的键才会生效。比如你声明 `[model_providers.openai]`，也只是再添加一个同名条目，Codex 会继续使用默认的 OpenAI 配置。若想调整官方 OpenAI 提供者，优先考虑相关环境变量（如 `OPENAI_BASE_URL`），或是注册一个新键然后在 `model_provider` 中引用它。

例如想通过 Chat Completions API 使用 OpenAI 4o，可以写成：

```toml
# TOML 根级别必须先写键再写表。
model = "gpt-4o"
model_provider = "openai-chat-completions"

[model_providers.openai-chat-completions]
# 将显示在 Codex UI 中的名称。
name = "OpenAI using Chat Completions"
# Codex 会在 base_url 后附加 `/chat/completions` 并发送 POST 请求。
base_url = "https://api.openai.com/v1"
# env_key 指定调用该提供者时需要设置的环境变量，其值会作为 Bearer Token 写入 HTTP 头。
env_key = "OPENAI_API_KEY"
# wire_api 可选 "chat" 或 "responses"，默认 "chat"。
wire_api = "chat"
# 若有需要，可附加额外的查询参数，Azure 示例见下文。
query_params = {}
```

借助这种机制，可以让 Codex CLI 连接第三方模型，只要它们兼容 OpenAI Chat Completions 协议即可。比如使用本地 Ollama：

```toml
[model_providers.ollama]
name = "Ollama"
base_url = "http://localhost:11434/v1"
```

或接入 Mistral（使用独立的 API Key 环境变量）：

```toml
[model_providers.mistral]
name = "Mistral"
base_url = "https://api.mistral.ai/v1"
env_key = "MISTRAL_API_KEY"
```

也可以为某个 provider 附加额外的 HTTP 请求头，可直接写常量（`http_headers`），也可引用环境变量（`env_http_headers`）：

```toml
[model_providers.example]
# name, base_url, ...

# 为每个请求附加固定的 HTTP 头。
http_headers = { "X-Example-Header" = "example-value" }

# 或者从环境变量读取要注入的头。
env_http_headers = { "X-Example-Features" = "EXAMPLE_FEATURES" }
```

#### Azure model provider example

Azure 需要在查询参数中带上 `api-version`，因此在配置 Azure provider 时必须写到 `query_params` 中：

```toml
[model_providers.azure]
name = "Azure"
# Make sure you set the appropriate subdomain for this URL.
base_url = "https://YOUR_PROJECT_NAME.openai.azure.com/openai"
env_key = "AZURE_OPENAI_API_KEY"  # Or "OPENAI_API_KEY", whichever you use.
query_params = { api-version = "2025-04-01-preview" }
wire_api = "responses"
```

启动 Codex 前请先设置密钥：`export AZURE_OPENAI_API_KEY=…`

#### 针对不同提供者的网络调优

下面这些可选项可以 **按 provider 维度** 控制重试策略与流式超时，必须写在对应的 `[model_providers.<id>]` 表中。（旧版本曾支持顶层键，现已忽略。）示例：

```toml
[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
# network tuning overrides (all optional; falls back to built‑in defaults)
request_max_retries = 4            # retry failed HTTP requests
stream_max_retries = 10            # retry dropped SSE streams
stream_idle_timeout_ms = 300000    # 5m idle timeout
```

##### request_max_retries

HTTP 请求失败后的最大重试次数，默认 `4`。

##### stream_max_retries

流式响应中断后，Codex 会尝试重连的次数，默认 `5`。

##### stream_idle_timeout_ms

流式响应在被视为断开前允许的最大空闲时间，单位毫秒；默认 `300_000`（5 分钟）。

### model_provider

指定要使用的 provider 键名，默认 `"openai"`。如果想替换内置 OpenAI 的 `base_url`，可以设置 `OPENAI_BASE_URL` 环境变量。

通常在切换 `model_provider` 的同时，也要调整 `model`。例如你在本地的 ollama 里运行 Mistral，除了提供者条目外还需要：

```toml
model_provider = "ollama"
model = "mistral"
```

### model_reasoning_effort

当所选模型支持推理（例如 `o3`、`o4-mini`、`codex-*`、`gpt-5.1-codex-max`、`gpt-5.1`、`gpt-5.1-codex`）时，使用 Responses API 会默认开启推理能力。参考 [OpenAI Platform 文档](https://platform.openai.com/docs/guides/reasoning?api-mode=responses#get-started-with-reasoning)，可选值包括：

- `"minimal"`
- `"low"`
- `"medium"`（默认）
- `"high"`

若希望尽可能降低推理成本，可设置为 `"minimal"`。

### model_reasoning_summary

If the model name starts with `"o"` (as in `"o3"` or `"o4-mini"`) or `"codex"`, reasoning is enabled by default when using the Responses API. As explained in the [OpenAI Platform documentation](https://platform.openai.com/docs/guides/reasoning?api-mode=responses#reasoning-summaries), this can be set to:

- `"auto"` (default)
- `"concise"`
- `"detailed"`

如需关闭推理摘要，在配置中写 `model_reasoning_summary = "none"`：

```toml
model_reasoning_summary = "none"  # 关闭推理摘要
```

### model_verbosity

使用 Responses API 调用 GPT‑5 系列模型时，可通过该项控制输出长度/细节。可选值：

- `"low"`
- `"medium"`（默认）
- `"high"`

设置后，请求体中会包含形如 `"text": { "verbosity": "low" }` 的字段。例如：

```toml
model = "gpt-5.1"
model_verbosity = "low"
```

注意：仅对使用 Responses API 的 provider 生效，Chat Completions 不受影响。

### model_supports_reasoning_summaries

默认情况下，只有已知支持推理的 OpenAI 模型才会自动携带 `reasoning` 字段。如果你希望对当前模型强制启用，可写：

```toml
model_supports_reasoning_summaries = true
```

### model_context_window

指定模型的上下文窗口大小（token 数）。大部分 OpenAI 模型 Codex 都内置了该值；若你在旧版 CLI 中使用了新模型，可通过此项告知 Codex 剩余上下文的计算方式。

### model_max_output_tokens

与 `model_context_window` 类似，但用于限制单次输出 token 数。

> 参见 [`codex exec`](./exec.md)，了解这些模型配置如何影响非交互式运行。

### oss_provider

指定 `--oss` 不带参数时的默认本地模型提供者：

- `"lmstudio"`：使用 LM Studio
- `"ollama"`：使用 Ollama

```toml
# 例：默认走 LM Studio
oss_provider = "lmstudio"
```

## 执行环境

### approval_policy

控制何时提示用户批准命令：

```toml
# Codex 内置了一组“可信”命令。
# 当 approval_policy=untrusted 时，凡不在该列表内的命令都会弹窗确认。
#
# 未来可自定义信任列表，详见 https://github.com/openai/codex/issues/1260
approval_policy = "untrusted"
```

若希望命令失败时再提醒，可用 `"on-failure"`：

```toml
# 命令在沙箱失败后，Codex 会询问是否在沙箱外重试。
approval_policy = "on-failure"
```

若希望模型自己决定何时申请提权，可用 `"on-request"`：

```toml
# 由模型自行判断是否升级权限
approval_policy = "on-request"
```

若完全不想被打扰，可选择 `"never"`：

```toml
# 从不弹窗；命令失败时 Codex 会尝试其他方案。`codex exec` 始终使用该模式。
approval_policy = "never"
```

### sandbox_mode

Codex 默认在操作系统级沙箱中执行模型产生的命令。常见场景只需一个选项即可：

```toml
# 等同于 `--sandbox read-only`
sandbox_mode = "read-only"
```

`read-only` 允许读取任意文件，但禁止写入和联网。

更宽松的 `workspace-write` 允许写入当前工作目录（以及 macOS 的 `$TMPDIR`）。CLI 默认使用启动目录作为 `cwd`，可通过 `--cwd/-C` 覆盖。

在 macOS（以及即将支持的 Linux）上，只要可写根目录的直接子级存在 `.git/`，就会将 `.git/` 设为只读，其余文件仍可写。因此 `git commit` 默认会失败，需要提权。

```toml
# 等同于 `--sandbox workspace-write`
sandbox_mode = "workspace-write"

[sandbox_workspace_write]
# 这些选项仅在 workspace-write 模式下生效，用于调整默认的可写目录。
exclude_tmpdir_env_var = false
exclude_slash_tmp = false

# 除 `/tmp` 与 `$TMPDIR` 外的额外可写根目录。
writable_roots = ["/Users/YOU/.pyenv/shims"]

# 是否允许沙箱内的命令访问网络，默认 false。
network_access = false
```

若完全不需要沙箱（例如运行在 Docker 等受限环境中），可设置：

```toml
# 等同于 `--sandbox danger-full-access`
sandbox_mode = "danger-full-access"
```

在旧内核或 Windows 等不支持原生沙箱的环境里，也可能需要该选项。

### tools.*

可用 `[tools]` 表控制哪些内置工具可被代理调用。`web_search` 默认关闭、`view_image` 默认开启：

```toml
[tools]
web_search = true   # 允许 Codex 主动发起一方 Web 搜索（已废弃）
view_image = false  # 关闭截图/示意图上传
```

`web_search` 已废弃，请改用 `web_search_request` 特性开关。

`view_image` 有助于直接引用仓库中的截图或示意图；沙箱仍会生效，文件必须位于允许的工作区内。

### approval_presets

Codex 预设了三种审批模式：

- Read Only：只能读文件、回答问题；写入/执行/联网需批准。
- Auto：工作区内可自由读写与执行；跨目录或联网时再询问。
- Full Access：完全访问磁盘与网络，不建议在生产环境使用。

你可以通过命令行的 `--ask-for-approval` 与 `--sandbox` 进一步微调。

> 参阅 [Sandbox & approvals](./sandbox.md) 了解平台差异与示例。

### shell_environment_policy

Codex 运行 `local_shell` 等工具时会启动子进程。默认会传递 **完整环境变量**。可在 `config.toml` 中通过 `shell_environment_policy` 进行细化：

```toml
[shell_environment_policy]
inherit = "core"             # 可选 all/core/none
ignore_default_excludes = false
exclude = ["AWS_*", "AZURE_*"]
set = { CI = "1" }
include_only = ["PATH", "HOME"]
```

| 字段                     | 类型                  | 默认值 | 说明                                                                                                   |
| ------------------------ | --------------------- | ------ | ------------------------------------------------------------------------------------------------------ |
| `inherit`                | string                | `all`  | 环境变量模板：`all` 继承全部、`core` 仅保留 `HOME/PATH/USER` 等核心项、`none` 从空环境开始。           |
| `ignore_default_excludes`| boolean               | `false`| 为 `false` 时，Codex 会先移除名字包含 `KEY`/`SECRET`/`TOKEN` 的变量（不区分大小写）。                  |
| `exclude`                | array<string>         | `[]`   | 自定义的忽略模式（大小写无关的 glob），例如 `"AWS_*"`、`"AZURE_*"`。                                   |
| `set`                    | table<string,string>  | `{}`   | 明确写入或覆盖的键值，优先级最高。                                                                     |
| `include_only`           | array<string>         | `[]`   | 白名单模式，仅当不为空时生效；只有匹配任意一个模式的变量才会保留，常与 `inherit = "all"` 搭配。        |

模式采用 **glob** 语法（`*` 任意长度，`?` 单字符，支持 `[A-Z]` / `[^0-9]` 等），且**不区分大小写**。实现细节可参考 `core/src/config_types.rs` 中的 `EnvironmentVariablePattern`。

如果想完全手动指定环境，可这样写：

```toml
[shell_environment_policy]
inherit = "none"
set = { PATH = "/usr/bin", MY_FLAG = "1" }
```

此外，当沙箱禁网时，Codex 会自动注入 `CODEX_SANDBOX_NETWORK_DISABLED=1`，不可关闭。

## MCP 集成

### mcp_servers

你可以通过 [MCP](https://modelcontextprotocol.io/about) 让 Codex 访问外部应用、资源或服务。

#### 服务器配置

##### STDIO

[STDIO 服务器](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports#stdio) 通过本地命令直接启动：

```toml
[mcp_servers.server_name]
command = "npx"
args = ["-y", "mcp-server"]          # 可选
env = { "API_KEY" = "value" }        # 额外传递的环境变量（可改用子表写法）
env_vars = ["API_KEY2"]               # 允许透传的环境变量白名单
cwd = "/Users/<user>/code/my-server"  # 可选工作目录
```

##### Streamable HTTP

[Streamable HTTP](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports#streamable-http) 允许 Codex 连接本地或远程的 HTTP 端点：

```toml
[mcp_servers.figma]
url = "https://mcp.figma.com/mcp"
bearer_token_env_var = "ENV_VAR"      # 可选，Bearer Token 所在的环境变量
http_headers = { "HEADER_NAME" = "HEADER_VALUE" }
env_http_headers = { "HEADER_NAME" = "ENV_VAR" }
```

流式 HTTP 通道使用实验性的 Rust MCP 客户端，OAuth 登录需打开 `experimental_use_rmcp_client = true`：

```toml
experimental_use_rmcp_client = true
```

启用后，可运行 `codex mcp login <server-name>` 完成授权。

#### 其它选项

```toml
startup_timeout_sec = 20  # 默认 10s
tool_timeout_sec = 30     # 默认 60s
enabled = false           # 禁用该 server
# 只暴露部分工具
enabled_tools = ["search", "summarize"]
# 在白名单基础上进一步排除
disabled_tools = ["search"]
```

若同时设置 `enabled_tools` 与 `disabled_tools`，Codex 会先套用白名单，再剔除黑名单。

#### Experimental RMCP client

This flag enables OAuth support for streamable HTTP servers.

```toml
experimental_use_rmcp_client = true

[mcp_servers.server_name]
…
```

#### MCP CLI commands

```shell
# List all available commands
codex mcp --help

# Add a server (env can be repeated; `--` separates the launcher command)
codex mcp add docs -- docs-server --port 4000

# List configured servers (pretty table or JSON)
codex mcp list
codex mcp list --json

# Show one server (table or JSON)
codex mcp get docs
codex mcp get docs --json

# Remove a server
codex mcp remove docs

# Log in to a streamable HTTP server that supports oauth
codex mcp login SERVER_NAME

# Log out from a streamable HTTP server that supports oauth
codex mcp logout SERVER_NAME
```

### 常用 MCP 例子

以下 MCP 服务器在实践中非常常见：

- [Context7](https://github.com/upstash/context7)：连接最新的开发者文档
- Figma [本地](https://developers.figma.com/docs/figma-mcp-server/local-server-installation/) / [远程](https://developers.figma.com/docs/figma-mcp-server/remote-server-installation/)：直接访问 Figma 设计
- [Playwright](https://www.npmjs.com/package/@playwright/mcp)：借助 Playwright 控制浏览器
- [Chrome Developer Tools](https://github.com/ChromeDevTools/chrome-devtools-mcp/)：调试 Chrome
- [Sentry](https://docs.sentry.io/product/sentry-mcp/#codex)：获取 Sentry 日志
- [GitHub](https://github.com/github/github-mcp-server)：管理 PR、Issue 等

## Observability and telemetry

### otel

Codex 可以输出 [OpenTelemetry](https://opentelemetry.io/) **日志事件**，记录 API 请求、流式响应、用户输入、工具审批与执行结果。默认 **不开启导出**，如需上报请添加 `[otel]` 表并配置 exporter。

```toml
[otel]
environment = "staging"   # defaults to "dev"
exporter = "none"          # 可设为 otlp-http / otlp-grpc
log_user_prompt = false    # 是否上报用户 prompt
```

所有导出的事件都会携带 `service.name = $ORIGINATOR`（与 `originator` 头相同，默认 `codex_cli_rs`）、CLI 版本以及 `env` 字段，以便区分 dev/staging/prod。只有 `codex_otel` crate 产生的事件会被转发。

### 事件目录

所有事件都有一组共同元数据：`event.timestamp`、`conversation.id`、`app.version`、`auth_mode`（如有）、`user.account_id`（如有）、`user.email`（如有）、`terminal.type`、`model`、`slug`。在此基础上还会额外输出：

- `codex.conversation_starts`
  - `provider_name`
  - `reasoning_effort` (optional)
  - `reasoning_summary`
  - `context_window` (optional)
  - `max_output_tokens` (optional)
  - `auto_compact_token_limit` (optional)
  - `approval_policy`
  - `sandbox_policy`
  - `mcp_servers` (comma-separated list)
  - `active_profile` (optional)
- `codex.api_request`
  - `attempt`
  - `duration_ms`
  - `http.response.status_code` (optional)
  - `error.message` (failures)
- `codex.sse_event`
  - `event.kind`
  - `duration_ms`
  - `error.message` (failures)
  - `input_token_count` (responses only)
  - `output_token_count` (responses only)
  - `cached_token_count` (responses only, optional)
  - `reasoning_token_count` (responses only, optional)
  - `tool_token_count` (responses only)
- `codex.user_prompt`
  - `prompt_length`
  - `prompt` (redacted unless `log_user_prompt = true`)
- `codex.tool_decision`
  - `tool_name`
  - `call_id`
  - `decision` (`approved`, `approved_for_session`, `denied`, or `abort`)
  - `source` (`config` or `user`)
- `codex.tool_result`
  - `tool_name`
  - `call_id` (optional)
  - `arguments` (optional)
  - `duration_ms` (execution time for the tool)
  - `success` (`"true"` or `"false"`)
  - `output`

These event shapes may change as we iterate.

### Choosing an exporter

Set `otel.exporter` to control where events go:

- `none` – leaves instrumentation active but skips exporting. This is the
  default.
- `otlp-http` – posts OTLP log records to an OTLP/HTTP collector. Specify the
  endpoint, protocol, and headers your collector expects:

  ```toml
  [otel]
  exporter = { otlp-http = {
    endpoint = "https://otel.example.com/v1/logs",
    protocol = "binary",
    headers = { "x-otlp-api-key" = "${OTLP_TOKEN}" }
  }}
  ```

- `otlp-grpc` – 通过 gRPC 推送 OTLP 日志，请提供 endpoint 及需要的元数据头：

  ```toml
  [otel]
  exporter = { otlp-grpc = {
    endpoint = "https://otel.example.com:4317",
    headers = { "x-otlp-meta" = "abc123" }
  }}
  ```

两个 OTLP exporter 都支持可选的 `tls` 配置，可自定义 CA 或启用双向 TLS；相对路径相对于 `~/.codex/`：

```toml
[otel]
exporter = { otlp-http = {
  endpoint = "https://otel.example.com/v1/logs",
  protocol = "binary",
  headers = { "x-otlp-api-key" = "${OTLP_TOKEN}" },
  tls = {
    ca-certificate = "certs/otel-ca.pem",
    client-certificate = "/etc/codex/certs/client.pem",
    client-private-key = "/etc/codex/certs/client-key.pem",
  }
}}
```

若 exporter 为 `none`，不会写入任何地方；否则须准备 OTLP collector。导出在后台批任务执行，退出前会 flush。

若自行从源码构建，`codex_otel` crate 仍受 `otel` feature 控制；官方发布版默认开启。禁用该 feature 时相关钩子为空实现，CLI 可在无附加依赖的情况下运行。

### notify

配置一个可执行程序，在 Codex 产生关键事件时调用。该程序会收到 JSON 字符串参数，例如：

```json
{
  "type": "agent-turn-complete",
  "thread-id": "b5f6c1c2-1111-2222-3333-444455556666",
  "turn-id": "12345",
  "cwd": "/Users/alice/projects/example",
  "input-messages": ["Rename `foo` to `bar` and update the callsites."],
  "last-assistant-message": "Rename complete and verified `cargo build` succeeds."
}
```

`"type"` 字段始终存在，目前仅支持 `"agent-turn-complete"`（回合完成）。

`"thread-id"` 标识触发通知的 Codex 会话，可用来对应同一任务下的多次回合。

`"cwd"` 给出绝对工作目录，方便脚本判断是哪个项目触发。

下面是一个示例脚本，解析 JSON 并在 macOS 上调用 [terminal-notifier](https://github.com/julienXX/terminal-notifier) 推送桌面通知：

```python
#!/usr/bin/env python3

import json
import subprocess
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: notify.py <NOTIFICATION_JSON>")
        return 1

    try:
        notification = json.loads(sys.argv[1])
    except json.JSONDecodeError:
        return 1

    match notification_type := notification.get("type"):
        case "agent-turn-complete":
            assistant_message = notification.get("last-assistant-message")
            if assistant_message:
                title = f"Codex: {assistant_message}"
            else:
                title = "Codex: Turn Complete!"
            input_messages = notification.get("input-messages", [])
            message = " ".join(input_messages)
            title += message
        case _:
            print(f"not sending a push notification for: {notification_type}")
            return 0

    thread_id = notification.get("thread-id", "")

    subprocess.check_output(
        [
            "terminal-notifier",
            "-title",
            title,
            "-message",
            message,
            "-group",
            "codex-" + thread_id,
            "-ignoreDnD",
            "-activate",
            "com.googlecode.iterm2",
        ]
    )

    return 0


if __name__ == "__main__":
    sys.exit(main())
```

若要让 Codex 调用此脚本，可在 `~/.codex/config.toml` 中配置：

```toml
notify = ["python3", "/Users/mbolin/.codex/notify.py"]
```

> [!NOTE]
> `notify` 适用于自动化/集成：每次触发事件都会向外部程序传递一段 JSON，和 TUI 无关。如果只需要轻量级桌面提醒，可用 `tui.notifications`（依赖终端转义序列）。两者可以同时启用：`tui.notifications` 负责 TUI 内的提醒（如审批），`notify` 更适合系统级 Hook 或自定义通知。当前 `notify` 仅发出 `agent-turn-complete`，而 `tui.notifications` 还能处理 `approval-requested` 并支持过滤。

### hide_agent_reasoning

Codex intermittently emits "reasoning" events that show the model's internal "thinking" before it produces a final answer. Some users may find these events distracting, especially in CI logs or minimal terminal output.

将 `hide_agent_reasoning` 置为 `true` 可同时在 TUI 与 `codex exec` 中隐藏这些“思考过程”事件：

```toml
hide_agent_reasoning = true   # defaults to false
```

### show_raw_agent_reasoning

当模型提供原始链路推理内容时，可通过 `show_raw_agent_reasoning` 直接展示：

提示：

- 仅对会返回 raw reasoning 的模型生效，其他模型设置后也不会有输出。
- 原始推理可能包含中间想法或敏感上下文，请按需启用。

Example:

```toml
show_raw_agent_reasoning = true  # defaults to false
```

## Profiles and overrides

### profiles

**Profile** 是一组可同时生效的配置。可以在 `config.toml` 中定义多个，并通过 `--profile` 选择其一。

示例：

```toml
model = "o3"
approval_policy = "untrusted"

# Setting `profile` is equivalent to specifying `--profile o3` on the command
# line, though the `--profile` flag can still be used to override this value.
profile = "o3"

[model_providers.openai-chat-completions]
name = "OpenAI using Chat Completions"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
wire_api = "chat"

[profiles.o3]
model = "o3"
model_provider = "openai"
approval_policy = "never"
model_reasoning_effort = "high"
model_reasoning_summary = "detailed"

[profiles.gpt3]
model = "gpt-3.5-turbo"
model_provider = "openai-chat-completions"

[profiles.zdr]
model = "o3"
model_provider = "openai"
approval_policy = "on-failure"
```

配置可在多层定义，优先级依次为：

1. 命令行参数（如 `--model o3`）
2. `--profile` 指定的配置档
3. `config.toml` 顶层值（如 `model = "o3"`）
4. Codex CLI 内置默认值（默认模型为 `gpt-5.1-codex-max`）

### history

Codex CLI 默认将会话记录到 `$CODEX_HOME/history.jsonl`，在类 UNIX 系统下文件权限为 `0600`，仅拥有者可读写。

要关闭持久化，可配置：

```toml
[history]
persistence = "none"  # "save-all" is the default value
```

### file_opener

在模型输出中对引用的文件添加超链接。支持的 URI scheme 固定，便于在终端中 cmd/ctrl+点击直接打开。

例如输出 `【F:/home/user/project/main.py†L42-L50】`，若 `file_opener = "vscode"`，则会指向 `vscode://file/home/user/project/main.py:42`。

注意这 **不是** `$EDITOR` 那种通用环境变量，只接受以下值：

- `"vscode"` (default)
- `"vscode-insiders"`
- `"windsurf"`
- `"cursor"`
- `"none"` to explicitly disable this feature

目前默认 `"vscode"`，但 Codex 不会检查 VS Code 是否存在；未来可能改为 `"none"`。

### project_doc_max_bytes

读取 `AGENTS.md` 时允许的最大字节数（默认 32 KiB），用于回合开始时注入提示。

### project_doc_fallback_filenames

当某层目录缺少 `AGENTS.md` 时，按顺序尝试的备用文件名。CLI 始终先找 `AGENTS.md`，再依次查找这里列出的文件，可帮助已有 `CLAUDE.md` 等文件的项目逐步迁移。

```toml
project_doc_fallback_filenames = ["CLAUDE.md", ".exampleagentrules.md"]
```

建议尽快统一到 AGENTS.md，否则可能影响模型效果。参见 [AGENTS.md discovery](./agents_md.md) 了解查找规则。

### tui

TUI 相关配置：

```toml
[tui]
# Send desktop notifications when approvals are required or a turn completes.
# Defaults to true.
notifications = true

# You can optionally filter to specific notification types.
# Available types are "agent-turn-complete" and "approval-requested".
notifications = [ "agent-turn-complete", "approval-requested" ]
```

> [!NOTE]
> 桌面通知依赖终端转义序列，macOS Terminal.app 与 VS Code 内置终端不支持，而 iTerm2 / Ghostty / WezTerm 支持。

> [!NOTE]
> `tui.notifications` 仅作用于 TUI；若需要跨平台脚本或系统级通知，请使用 `notify` 运行外部程序。两者可并行启用。

## 身份验证与授权

### 登录方式限制

若需在某台机器上强制指定登录方式或工作区，可结合[受管配置](https://developers.openai.com/codex/security#managed-configuration)与以下字段：

```toml
# Force the user to log in with ChatGPT or via an api key.
forced_login_method = "chatgpt" or "api"
# When logging in with ChatGPT, only the specified workspace ID will be presented during the login
# flow and the id will be validated during the oauth callback as well as every time Codex starts.
forced_chatgpt_workspace_id = "00000000-0000-0000-0000-000000000000"
```

若当前凭据不符合要求，用户会被登出且 Codex 立即退出。如果仅设置 `forced_chatgpt_workspace_id` 而未设置 `forced_login_method`，API Key 登录仍可用。

### 登录凭据存储位置

```toml
cli_auth_credentials_store = "keyring"
```

取值：

- `file`（默认）：保存在 `$CODEX_HOME/auth.json`。
- `keyring`：使用操作系统的凭据存储（[`keyring` crate](https://crates.io/crates/keyring)）；若安全存储不可用会报错。
  - macOS：Keychain
  - Windows：Credential Manager
  - Linux：DBus Secret Service / keyutils 等
  - FreeBSD/OpenBSD：DBus Secret Service
- `auto`：优先尝试 keyring，不可用时退回到 `auth.json`。

## 配置速查表

| 键                                              | 类型/取值                                                         | 说明                                                                                             |
| ------------------------------------------------ | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `model`                                          | string                                                           | 使用的模型（如 `gpt-5.1-codex-max`）。                                                                     |
| `model_provider`                                 | string                                                           | `model_providers` 中的 provider ID，默认 `openai`。                                               |
| `model_context_window`                           | number                                                           | 上下文窗口大小（token 数）。                                                                |
| `model_max_output_tokens`                        | number                                                           | 输出 token 上限。                                                                            |
| `tool_output_token_limit`                        | number                                                           | 历史记录中保存工具输出的 token 预算（默认 2560）。                                          |
| `approval_policy`                                | `untrusted` / `on-failure` / `on-request` / `never`            | 何时提示用户审批。                                                                                 |
| `sandbox_mode`                                   | `read-only` / `workspace-write` / `danger-full-access`          | OS 沙箱策略。                                                                             |
| `sandbox_workspace_write.writable_roots`         | array<string>                                                    | workspace-write 模式下额外可写根目录。                                                   |
| `sandbox_workspace_write.network_access`         | boolean                                                          | workspace-write 模式是否允许联网（默认 false）。                      |
| `sandbox_workspace_write.exclude_tmpdir_env_var` | boolean                                                          | 将 `$TMPDIR` 排除在可写列表之外（默认 false）。                           |
| `sandbox_workspace_write.exclude_slash_tmp`      | boolean                                                          | 将 `/tmp` 排除在可写列表之外（默认 false）。                             |
| `notify`                                         | array<string>                                                    | 外部通知程序。                                                                         |
| `instructions`                                   | string                                                           | 已废弃；请使用 `experimental_instructions_file` 或 `AGENTS.md`。         |
| `features.<feature-flag>`                        | boolean                                                          | 详见 [特性开关](#feature-flags)。                                           |
| `mcp_servers.<id>.command`                       | string                                                           | STDIO MCP 服务器的启动命令。                                          |
| `mcp_servers.<id>.args`                          | array<string>                                                    | STDIO MCP 服务器的参数。                                            |
| `mcp_servers.<id>.env`                           | map<string,string>                                               | STDIO MCP 服务器的环境变量。                                           |
| `mcp_servers.<id>.url`                           | string                                                           | Streamable HTTP MCP 的 URL。                                        |
| `mcp_servers.<id>.bearer_token_env_var`          | string                                                           | Streamable HTTP MCP 使用的 Bearer Token 环境变量。                    |
| `mcp_servers.<id>.enabled`                       | boolean                                                           | When false, Codex skips starting the server (default: true).                                                               |
| `mcp_servers.<id>.startup_timeout_sec`           | number                                                            | Startup timeout in seconds (default: 10). Timeout is applied both for initializing MCP server and initially listing tools. |
| `mcp_servers.<id>.tool_timeout_sec`              | number                                                            | Per-tool timeout in seconds (default: 60). Accepts fractional values; omit to use the default.                             |
| `mcp_servers.<id>.enabled_tools`                 | array<string>                                                     | Restrict the server to the listed tool names.                                                                              |
| `mcp_servers.<id>.disabled_tools`                | array<string>                                                     | Remove the listed tool names after applying `enabled_tools`, if any.                                                       |
| `model_providers.<id>.name`                      | string                                                            | Display name.                                                                                                              |
| `model_providers.<id>.base_url`                  | string                                                            | API base URL.                                                                                                              |
| `model_providers.<id>.env_key`                   | string                                                            | Env var for API key.                                                                                                       |
| `model_providers.<id>.wire_api`                  | `chat` \| `responses`                                             | Protocol used (default: `chat`).                                                                                           |
| `model_providers.<id>.query_params`              | map<string,string>                                                | Extra query params (e.g., Azure `api-version`).                                                                            |
| `model_providers.<id>.http_headers`              | map<string,string>                                                | Additional static headers.                                                                                                 |
| `model_providers.<id>.env_http_headers`          | map<string,string>                                                | Headers sourced from env vars.                                                                                             |
| `model_providers.<id>.request_max_retries`       | number                                                            | Per‑provider HTTP retry count (default: 4).                                                                                |
| `model_providers.<id>.stream_max_retries`        | number                                                            | SSE stream retry count (default: 5).                                                                                       |
| `model_providers.<id>.stream_idle_timeout_ms`    | number                                                            | SSE idle timeout (ms) (default: 300000).                                                                                   |
| `project_doc_max_bytes`                          | number                                                            | Max bytes to read from `AGENTS.md`.                                                                                        |
| `profile`                                        | string                                                            | Active profile name.                                                                                                       |
| `profiles.<name>.*`                              | various                                                           | Profile‑scoped overrides of the same keys.                                                                                 |
| `history.persistence`                            | `save-all` \| `none`                                              | History file persistence (default: `save-all`).                                                                            |
| `history.max_bytes`                              | number                                                            | Currently ignored (not enforced).                                                                                          |
| `file_opener`                                    | `vscode` \| `vscode-insiders` \| `windsurf` \| `cursor` \| `none` | URI scheme for clickable citations (default: `vscode`).                                                                    |
| `tui`                                            | table                                                             | TUI‑specific options.                                                                                                      |
| `tui.notifications`                              | boolean \| array<string>                                          | Enable desktop notifications in the tui (default: true).                                                                   |
| `hide_agent_reasoning`                           | boolean                                                          | 是否隐藏模型推理事件。                                                   |
| `show_raw_agent_reasoning`                       | boolean                                                          | 是否展示原始推理内容（若模型支持）。                              |
| `model_reasoning_effort`                         | `minimal` / `low` / `medium` / `high`                          | Responses API 的推理强度。                                       |
| `model_reasoning_summary`                        | `auto` / `concise` / `detailed` / `none`                       | 推理摘要的粒度。                                               |
| `model_verbosity`                                | `low` / `medium` / `high`                                       | GPT‑5 Responses 文本详略。                                       |
| `model_supports_reasoning_summaries`             | boolean                                                          | 强制开启推理摘要。                                                |
| `model_reasoning_summary_format`                 | `none` / `experimental`                                          | 指定推理摘要格式。                                               |
| `chatgpt_base_url`                               | string                                                           | ChatGPT 登录流程使用的基础 URL。                                |
| `experimental_instructions_file`                 | string (path)                                                    | 指定额外的指令文件（实验特性）。                              |
| `experimental_use_exec_command_tool`             | boolean                                                          | 启用实验版 exec-command 工具。                                |
| `projects.<path>.trust_level`                    | string                                                           | 标记某个路径为可信（目前仅识别 `"trusted"`）。                                  |
| `tools.web_search`                               | boolean                                                          | 启用 web_search 工具（已废弃，默认 false）。                                          |
| `tools.view_image`                               | boolean                                                          | 控制是否允许 `view_image` 工具上传工作区内的图片（默认 true）。      |
| `forced_login_method`                            | `chatgpt` / `api`                                                | 强制只允许 ChatGPT 或 API Key 登录。                                |
| `forced_chatgpt_workspace_id`                    | string (uuid)                                                    | 仅允许连接到指定的 ChatGPT Workspace。                             |
| `cli_auth_credentials_store`                     | `file` / `keyring` / `auto`                                     | CLI 凭据存储位置（默认 `file`）。                                                        |
