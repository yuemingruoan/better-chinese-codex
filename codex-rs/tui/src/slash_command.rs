use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model,
    Approvals,
    Review,
    New,
    Init,
    Checkpoint,
    Compact,
    Undo,
    Diff,
    Mention,
    Status,
    Mcp,
    Logout,
    Quit,
    #[cfg(debug_assertions)]
    TestApproval,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::New => "在对话中开始新的聊天",
            SlashCommand::Init => "创建包含 Codex 指南的 AGENTS.md 文件",
            SlashCommand::Checkpoint => "阶段性记录 AI 所执行的操作",
            SlashCommand::Compact => "总结当前对话以避免上下文超限",
            SlashCommand::Review => "审查当前改动并查找问题",
            SlashCommand::Undo => "恢复到上一次 Codex 快照",
            SlashCommand::Quit => "退出 Codex",
            SlashCommand::Diff => "显示 git diff（包含未跟踪文件）",
            SlashCommand::Mention => "在消息中提及文件",
            SlashCommand::Status => "显示会话配置与令牌使用情况",
            SlashCommand::Model => "选择模型及推理强度",
            SlashCommand::Approvals => "配置 Codex 无需审批即可执行的操作",
            SlashCommand::Mcp => "列出已配置的 MCP 工具",
            SlashCommand::Logout => "注销 Codex 登录",
            #[cfg(debug_assertions)]
            SlashCommand::TestApproval => "测试审批请求",
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::New
            | SlashCommand::Init
            | SlashCommand::Checkpoint
            | SlashCommand::Compact
            | SlashCommand::Undo
            | SlashCommand::Model
            | SlashCommand::Approvals
            | SlashCommand::Review
            | SlashCommand::Logout => false,
            SlashCommand::Diff
            | SlashCommand::Mention
            | SlashCommand::Status
            | SlashCommand::Mcp
            | SlashCommand::Quit => true,

            #[cfg(debug_assertions)]
            SlashCommand::TestApproval => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    let show_beta_features = beta_features_enabled();

    SlashCommand::iter()
        .filter(|cmd| {
            if *cmd == SlashCommand::Undo {
                show_beta_features
            } else {
                true
            }
        })
        .map(|c| (c.command(), c))
        .collect()
}

fn beta_features_enabled() -> bool {
    std::env::var_os("BETA_FEATURE").is_some()
}
