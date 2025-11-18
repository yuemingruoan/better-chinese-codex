use clap::Parser;
use clap::ValueHint;
use codex_common::ApprovalModeCliArg;
use codex_common::CliConfigOverrides;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    /// 可选，启动会话时的初始提示词。
    #[arg(value_name = "PROMPT", value_hint = clap::ValueHint::Other)]
    pub prompt: Option<String>,

    /// 可选，附加到初始提示中的图片。
    #[arg(long = "image", short = 'i', value_name = "FILE", value_delimiter = ',', num_args = 1..)]
    pub images: Vec<PathBuf>,

    // Internal controls set by the top-level `codex resume` subcommand.
    // These are not exposed as user flags on the base `codex` command.
    #[clap(skip)]
    pub resume_picker: bool,

    #[clap(skip)]
    pub resume_last: bool,

    /// Internal: resume a specific recorded session by id (UUID). Set by the
    /// top-level `codex resume <SESSION_ID>` wrapper; not exposed as a public flag.
    #[clap(skip)]
    pub resume_session_id: Option<String>,

    /// 指定代理要使用的模型。
    #[arg(long, short = 'm')]
    pub model: Option<String>,

    /// 选择本地开源模型提供者的便捷开关。
    /// 等效于 -c model_provider=oss，并会校验本地 Ollama 服务是否运行。
    #[arg(long = "oss", default_value_t = false)]
    pub oss: bool,

    /// 使用 config.toml 中的配置档来指定默认选项。
    #[arg(long = "profile", short = 'p')]
    pub config_profile: Option<String>,

    /// 指定在执行模型生成的 shell 命令时使用的沙箱策略。
    #[arg(long = "sandbox", short = 's')]
    pub sandbox_mode: Option<codex_common::SandboxModeCliArg>,

    /// 配置模型在执行命令前需要人工审批的时机。
    #[arg(long = "ask-for-approval", short = 'a')]
    pub approval_policy: Option<ApprovalModeCliArg>,

    /// 低阻力沙箱自动执行的便捷别名（-a on-request，--sandbox workspace-write）。
    #[arg(long = "full-auto", default_value_t = false)]
    pub full_auto: bool,

    /// 跳过所有确认提示并在无沙箱下执行命令。
    /// 极度危险，仅适用于外部环境已提供沙箱保护的情况。
    #[arg(
        long = "dangerously-bypass-approvals-and-sandbox",
        alias = "yolo",
        default_value_t = false,
        conflicts_with_all = ["approval_policy", "full_auto"]
    )]
    pub dangerously_bypass_approvals_and_sandbox: bool,

    /// 将代理的工作根目录切换到指定路径。
    #[clap(long = "cd", short = 'C', value_name = "DIR")]
    pub cwd: Option<PathBuf>,

    /// 启用联网搜索（默认关闭）。启用后模型可直接使用 Responses 的 `web_search` 工具（无需逐次审批）。
    #[arg(long = "search", default_value_t = false)]
    pub web_search: bool,

    /// 需要与主工作区一同放开的额外可写目录。
    #[arg(long = "add-dir", value_name = "DIR", value_hint = ValueHint::DirPath)]
    pub add_dir: Vec<PathBuf>,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,
}
