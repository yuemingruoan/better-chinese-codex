use crate::i18n::tr;
use codex_protocol::config_types::Language;
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
    Lang,
    Approvals,
    #[strum(serialize = "setup-elevated-sandbox")]
    ElevateSandbox,
    Experimental,
    Skills,
    Review,
    New,
    Resume,
    Fork,
    Init,
    Checkpoint,
    Compact,
    // Undo,
    Diff,
    Mention,
    Status,
    SddDevelop,
    Mcp,
    Logout,
    Quit,
    Exit,
    Feedback,
    Rollout,
    Ps,
    TestApproval,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self, language: Language) -> &'static str {
        match self {
            SlashCommand::Feedback => tr(language, "slash_command.description.feedback"),
            SlashCommand::New => tr(language, "slash_command.description.new"),
            SlashCommand::Init => tr(language, "slash_command.description.init"),
            SlashCommand::Compact => tr(language, "slash_command.description.compact"),
            SlashCommand::Review => tr(language, "slash_command.description.review"),
            SlashCommand::Resume => tr(language, "slash_command.description.resume"),
            SlashCommand::Fork => tr(language, "slash_command.description.fork"),
            SlashCommand::Quit | SlashCommand::Exit => {
                tr(language, "slash_command.description.exit")
            }
            SlashCommand::Diff => tr(language, "slash_command.description.diff"),
            SlashCommand::Mention => tr(language, "slash_command.description.mention"),
            SlashCommand::Skills => tr(language, "slash_command.description.skills"),
            SlashCommand::Status => tr(language, "slash_command.description.status"),
            SlashCommand::Ps => tr(language, "slash_command.description.ps"),
            SlashCommand::Model => tr(language, "slash_command.description.model"),
            SlashCommand::Lang => tr(language, "slash_command.description.lang"),
            SlashCommand::Approvals => tr(language, "slash_command.description.approvals"),
            SlashCommand::ElevateSandbox => {
                tr(language, "slash_command.description.elevate_sandbox")
            }
            SlashCommand::Experimental => tr(language, "slash_command.description.experimental"),
            SlashCommand::Mcp => tr(language, "slash_command.description.mcp"),
            SlashCommand::Logout => tr(language, "slash_command.description.logout"),
            SlashCommand::Rollout => tr(language, "slash_command.description.rollout"),
            SlashCommand::TestApproval => tr(language, "slash_command.description.test_approval"),
            SlashCommand::Checkpoint => tr(language, "slash_command.description.checkpoint"),
            SlashCommand::SddDevelop => tr(language, "slash_command.description.sdd_develop"),
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
            | SlashCommand::Resume
            | SlashCommand::Fork
            | SlashCommand::Init
            | SlashCommand::Checkpoint
            | SlashCommand::Compact
            | SlashCommand::SddDevelop
            | SlashCommand::Model
            | SlashCommand::Lang
            | SlashCommand::Approvals
            | SlashCommand::ElevateSandbox
            | SlashCommand::Experimental
            | SlashCommand::Review
            | SlashCommand::Logout => false,
            SlashCommand::Diff
            | SlashCommand::Mention
            | SlashCommand::Skills
            | SlashCommand::Status
            | SlashCommand::Ps
            | SlashCommand::Mcp
            | SlashCommand::Feedback
            | SlashCommand::Quit
            | SlashCommand::Exit => true,
            SlashCommand::Rollout => true,
            SlashCommand::TestApproval => true,
        }
    }

    fn is_visible(self) -> bool {
        match self {
            SlashCommand::Rollout | SlashCommand::TestApproval => cfg!(debug_assertions),
            _ => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter()
        .filter(|command| command.is_visible())
        .map(|c| (c.command(), c))
        .collect()
}
