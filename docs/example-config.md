# 示例 config.toml

可以把这份示例配置当成起点。若想了解每个字段含义与额外背景，请参阅 [Configuration](./config.md)。将下面的片段复制到 `~/.codex/config.toml`，然后按需调整数值。

```toml
# Codex 示例配置（config.toml）
#
# 该文件罗列出 Codex 会从 config.toml 读取的所有键、它们的默认值以及简短说明。
# 这些值与 CLI 中编译的实际默认值一致，可按需覆盖。
#
# 说明
# - 在 TOML 中根级键必须写在所有表之前。
# - 默认值为“未设置”的可选键会以注释方式展示，并附带说明。
# - MCP 服务器、配置档与模型提供方仅供展示，可自行删除或修改。

################################################################################
# 核心模型选择
################################################################################

# Codex 主要使用的模型。所有平台默认值："gpt-5.1-codex-max"。
model = "gpt-5.1-codex-max"

# /review 功能（代码评审）所用模型。默认："gpt-5.1-codex-max"。
review_model = "gpt-5.1-codex-max"

# 在 [model_providers] 中选择的 provider id。默认："openai"。
model_provider = "openai"

# 可选的模型元数据。若未设置，Codex 会依据模型自动识别。
# 取消注释即可强制覆盖。
# model_context_window = 128000       # token 数；默认：按模型自动决定
# model_auto_compact_token_limit = 0  # 0 表示禁用/覆盖自动压缩；默认：依模型家族而定
# tool_output_token_limit = 10000  # 每个工具输出保存的 token 数；默认：gpt-5.1-codex-max 为 10000

################################################################################
# 推理与冗长度（支持 Responses API 的模型）
################################################################################

# 推理力度：minimal | low | medium | high | xhigh（默认：medium；xhigh 对应“极高”，部分模型可用）
model_reasoning_effort = "medium"

# 推理摘要：auto | concise | detailed | none（默认：auto）
model_reasoning_summary = "auto"

# GPT-5 系列（Responses API）的文字冗长度：low | medium | high（默认：medium）
model_verbosity = "medium"

# 强制开启推理摘要（默认：false）
model_supports_reasoning_summaries = false

# 强制指定推理摘要格式：none | experimental（默认：none）
model_reasoning_summary_format = "none"

################################################################################
# 指令覆盖
################################################################################

# 附加到 AGENTS.md 之后的用户指令。默认：未设置。
# developer_instructions = ""

# 可选的旧版基础指令覆盖（推荐使用 AGENTS.md）。默认：未设置。
# instructions = ""

# 在行内覆盖历史压缩提示词。默认：未设置。
# compact_prompt = ""

# 使用文件路径覆盖内置基础指令。默认：未设置。
# experimental_instructions_file = "/absolute/or/relative/path/to/instructions.txt"

# 从文件加载压缩提示词覆盖。默认：未设置。
# experimental_compact_prompt_file = "/absolute/or/relative/path/to/compact_prompt.txt"

################################################################################
# 审批与沙箱
################################################################################

# 何时请求命令审批：
# - untrusted：仅自动运行已知安全的只读命令，其他命令会提示
# - on-failure：默认在沙箱运行，仅在失败时提示升级
# - on-request：由模型决定何时询问（默认）
# - never：从不提示（风险极高）
approval_policy = "on-request"

# 工具调用的文件系统/网络沙箱策略：
# - read-only（默认）
# - workspace-write
# - danger-full-access（无遮罩，风险极高）
sandbox_mode = "read-only"

# 仅在 sandbox_mode = "workspace-write" 时生效的附加参数。
[sandbox_workspace_write]
# 除工作区（cwd）以外的额外可写根路径。默认：[]
writable_roots = []
# 是否允许沙箱内访问外网。默认：false
network_access = false
# 将 $TMPDIR 排除在可写目录之外。默认：false
exclude_tmpdir_env_var = false
# 将 /tmp 排除在可写目录之外。默认：false
exclude_slash_tmp = false

################################################################################
# 子进程的 Shell 环境策略
################################################################################

[shell_environment_policy]
# inherit：all（默认）| core | none
inherit = "all"
# 忽略名称中包含 KEY/TOKEN（大小写不敏感）的默认排除项。默认：true
ignore_default_excludes = true
# 需移除的大小写不敏感通配符（例如 "AWS_*"、"AZURE_*")。默认：[]
exclude = []
# 明确设置的键值（始终优先生效）。默认：{}
set = {}
# 白名单；非空时仅保留匹配的变量。默认：[]
include_only = []
# 实验特性：通过用户 shell profile 启动。默认：false
experimental_use_profile = false

################################################################################
# 历史记录与文件打开方式
################################################################################

[history]
# save-all（默认）| none
persistence = "save-all"
# 历史文件的最大字节数，超过后会裁剪最旧条目。示例：5242880
# max_bytes = 0

# 点击引用时使用的 URI scheme：vscode（默认）| vscode-insiders | windsurf | cursor | none
file_opener = "vscode"

################################################################################
# UI、通知与其他
################################################################################

[tui]
# TUI 桌面通知：布尔值或过滤列表。默认：true
# 示例：false | ["agent-turn-complete", "approval-requested"]
notifications = false

# 终端动画（欢迎页、状态 shimmer、spinner）。默认：true
animations = true

# 隐藏输出中的内部推理事件（默认：false）
hide_agent_reasoning = false

# 在可用时显示原始推理内容（默认：false）
show_raw_agent_reasoning = false

# 在 TUI 中禁用快速粘贴检测（默认：false）
disable_paste_burst = false

# 记录 Windows 引导提示已阅读（仅 Windows）。默认：false
windows_wsl_setup_acknowledged = false

# 外部通知程序（argv 数组）。未设置时禁用。
# 示例：notify = ["notify-send", "Codex"]
# notify = [ ]

# 产品内提示（多由 Codex 自动填充）。
[notice]
# hide_full_access_warning = true
# hide_rate_limit_model_nudge = true

################################################################################
# 鉴权与登录
################################################################################

# CLI 登录凭据的保存方式：file（默认）| keyring | auto
cli_auth_credentials_store = "file"

# ChatGPT 鉴权流程的基础 URL（非 OpenAI API）。默认：
chatgpt_base_url = "https://chatgpt.com/backend-api/"

# 将 ChatGPT 登录限制在指定 workspace id。默认：未设置。
# forced_chatgpt_workspace_id = ""

# 在 Codex 会自动选择登录方式时强制指定机制。默认：未设置。
# 允许值：chatgpt | api
# forced_login_method = "chatgpt"

################################################################################
# 项目文档控制
################################################################################

# 嵌入首轮指令的 AGENTS.md 最大字节数。默认：32768
project_doc_max_bytes = 32768

# 当某级目录缺少 AGENTS.md 时按序查找的后备文件名。默认：[]
project_doc_fallback_filenames = []

################################################################################
# 工具（保留的旧版开关）
################################################################################

[tools]
# 启用网页搜索工具（别名：web_search_request）。默认：false
web_search = false

# 允许代理使用 view_image 工具附加本地图片。默认：true
view_image = true

# （别名）也可以写成：
# web_search_request = false

################################################################################
# 集中式特性开关（推荐）
################################################################################

[features]
# 如果接受默认值，可以保持该表为空。设置布尔值即可明确选择加入或退出。
unified_exec = false
streamable_shell = false
rmcp_client = false
apply_patch_freeform = false
view_image_tool = true
web_search_request = false
experimental_sandbox_command_assessment = false
ghost_commit = false
enable_experimental_windows_sandbox = false

################################################################################
# 实验性切换（旧式，优先使用 [features]）
################################################################################

# 启用实验版统一 exec 工具。默认：false
experimental_use_unified_exec_tool = false

# 启用实验版 Rust MCP 客户端（允许 HTTP MCP 走 OAuth）。默认：false
experimental_use_rmcp_client = false

# 通过自由编辑路径包含 apply_patch（影响默认工具集）。默认：false
experimental_use_freeform_apply_patch = false

# 启用基于模型的沙箱命令评估。默认：false
experimental_sandbox_command_assessment = false

################################################################################
# MCP（Model Context Protocol）服务器
################################################################################

# MCP OAuth 凭据的首选存储：auto（默认）| file | keyring
mcp_oauth_credentials_store = "auto"

# 在此表中定义 MCP 服务器。保持为空即表示禁用。
[mcp_servers]

# --- 示例：STDIO 传输 ---
# [mcp_servers.docs]
# command = "docs-server"                 # 必填
# args = ["--port", "4000"]               # 可选
# env = { "API_KEY" = "value" }           # 可选的键值对，原样传递
# env_vars = ["ANOTHER_SECRET"]            # 可选：从父进程环境转发这些变量
# cwd = "/path/to/server"                 # 可选的工作目录覆盖
# startup_timeout_sec = 10.0               # 可选；默认 10.0 秒
# # startup_timeout_ms = 10000              # 可选别名，单位毫秒
# tool_timeout_sec = 60.0                  # 可选；默认 60.0 秒
# enabled_tools = ["search", "summarize"]  # 可选允许列表
# disabled_tools = ["slow-tool"]           # 可选拒绝列表（在允许列表之后应用）

# --- 示例：可流式的 HTTP 传输 ---
# [mcp_servers.github]
# url = "https://github-mcp.example.com/mcp"  # 必填
# bearer_token_env_var = "GITHUB_TOKEN"        # 可选；发送 Authorization: Bearer <token>
# http_headers = { "X-Example" = "value" }    # 可选静态请求头
# env_http_headers = { "X-Auth" = "AUTH_ENV" } # 可选：由环境变量填充的请求头
# startup_timeout_sec = 10.0                   # 可选
# tool_timeout_sec = 60.0                      # 可选
# enabled_tools = ["list_issues"]             # 可选允许列表

################################################################################
# 模型提供方（扩展/覆盖内置设置）
################################################################################

# 内置提供方包括：
# - openai（Responses API；需要登录或通过鉴权流程获取 OPENAI_API_KEY）
# - oss（Chat Completions API；默认指向 http://localhost:11434/v1）

[model_providers]

# --- 示例：覆盖 OpenAI 的基础 URL 或请求头 ---
# [model_providers.openai]
# name = "OpenAI"
# base_url = "https://api.openai.com/v1"         # 未设置时的默认值
# wire_api = "responses"                         # "responses" | "chat"（默认为模型决定）
# # requires_openai_auth = true                    # 内置 OpenAI 默认即为 true
# # request_max_retries = 4                        # 默认 4；最多 100
# # stream_max_retries = 5                         # 默认 5；最多 100
# # stream_idle_timeout_ms = 300000                # 默认 300_000（5 分钟）
# # experimental_bearer_token = "sk-example"      # 可选，仅供本地开发使用的直接 token
# # http_headers = { "X-Example" = "value" }
# # env_http_headers = { "OpenAI-Organization" = "OPENAI_ORGANIZATION", "OpenAI-Project" = "OPENAI_PROJECT" }

# --- 示例：Azure（根据 endpoint 使用 Chat 或 Responses） ---
# [model_providers.azure]
# name = "Azure"
# base_url = "https://YOUR_PROJECT_NAME.openai.azure.com/openai"
# wire_api = "responses"                          # 或 "chat"，视 endpoint 而定
# query_params = { api-version = "2025-04-01-preview" }
# env_key = "AZURE_OPENAI_API_KEY"
# # env_key_instructions = "在环境变量中设置 AZURE_OPENAI_API_KEY"

# --- 示例：本地 OSS（例如兼容 Ollama） ---
# [model_providers.ollama]
# name = "Ollama"
# base_url = "http://localhost:11434/v1"
# wire_api = "chat"

################################################################################
# 配置档（命名预设）
################################################################################

# 当前启用的配置档名称。未设置时不应用任何 profile。
# profile = "default"

[profiles]

# [profiles.default]
# model = "gpt-5.1-codex-max"
# model_provider = "openai"
# approval_policy = "on-request"
# sandbox_mode = "read-only"
# model_reasoning_effort = "medium"
# model_reasoning_summary = "auto"
# model_verbosity = "medium"
# chatgpt_base_url = "https://chatgpt.com/backend-api/"
# experimental_compact_prompt_file = "compact_prompt.txt"
# include_apply_patch_tool = false
# experimental_use_unified_exec_tool = false
# experimental_use_rmcp_client = false
# experimental_use_freeform_apply_patch = false
# experimental_sandbox_command_assessment = false
# tools_web_search = false
# tools_view_image = true
# features = { unified_exec = false }

################################################################################
# 项目（信任级别）
################################################################################

# 将特定工作树标记为 trusted。目前仅支持 "trusted"。
[projects]
# [projects."/absolute/path/to/project"]
# trust_level = "trusted"

################################################################################
# OpenTelemetry（OTEL）——默认禁用
################################################################################

[otel]
# 是否把用户提问文本写入日志。默认：false
log_user_prompt = false
# 写入遥测时附加的 environment 标签。默认："dev"
environment = "dev"
# 导出方式：none（默认）| otlp-http | otlp-grpc
exporter = "none"

# OTLP/HTTP 导出示例
# [otel]
# exporter = { otlp-http = {
#   endpoint = "https://otel.example.com/v1/logs",
#   protocol = "binary",                      # "binary" | "json"
#   headers = { "x-otlp-api-key" = "${OTLP_TOKEN}" }
# }}

# OTLP/gRPC 导出示例
# [otel]
# exporter = { otlp-grpc = {
#   endpoint = "https://otel.example.com:4317",
#   headers = { "x-otlp-meta" = "abc123" }
# }}

# 启用双向 TLS 的 OTLP 导出示例
# [otel]
# exporter = { otlp-http = {
#   endpoint = "https://otel.example.com/v1/logs",
#   protocol = "binary",
#   headers = { "x-otlp-api-key" = "${OTLP_TOKEN}" },
#   tls = {
#     ca-certificate = "certs/otel-ca.pem",
#     client-certificate = "/etc/codex/certs/client.pem",
#     client-private-key = "/etc/codex/certs/client-key.pem",
#   }
# }}
```
