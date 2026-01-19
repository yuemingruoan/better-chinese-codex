use codex_core::protocol::SandboxPolicy;
use crate::i18n::tr_args;
use codex_protocol::config_types::Language;
use std::path::PathBuf;

/// Returns a warning describing why `--add-dir` entries will be ignored for the
/// resolved sandbox policy. The caller is responsible for presenting the
/// warning to the user (for example, printing to stderr).
pub fn add_dir_warning_message(
    additional_dirs: &[PathBuf],
    sandbox_policy: &SandboxPolicy,
    language: Language,
) -> Option<String> {
    if additional_dirs.is_empty() {
        return None;
    }

    match sandbox_policy {
        SandboxPolicy::WorkspaceWrite { .. }
        | SandboxPolicy::DangerFullAccess
        | SandboxPolicy::ExternalSandbox { .. } => None,
        SandboxPolicy::ReadOnly => Some(format_warning(additional_dirs, language)),
    }
}

fn format_warning(additional_dirs: &[PathBuf], language: Language) -> String {
    let joined_paths = additional_dirs
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(", ");
    tr_args(
        language,
        "additional_dirs.read_only_warning",
        &[("paths", &joined_paths)],
    )
}

#[cfg(test)]
mod tests {
    use super::add_dir_warning_message;
    use codex_core::protocol::NetworkAccess;
    use codex_core::protocol::SandboxPolicy;
    use codex_protocol::config_types::Language;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn returns_none_for_workspace_write() {
        let sandbox = SandboxPolicy::new_workspace_write_policy();
        let dirs = vec![PathBuf::from("/tmp/example")];
        assert_eq!(
            add_dir_warning_message(&dirs, &sandbox, Language::ZhCn),
            None
        );
    }

    #[test]
    fn returns_none_for_danger_full_access() {
        let sandbox = SandboxPolicy::DangerFullAccess;
        let dirs = vec![PathBuf::from("/tmp/example")];
        assert_eq!(
            add_dir_warning_message(&dirs, &sandbox, Language::ZhCn),
            None
        );
    }

    #[test]
    fn returns_none_for_external_sandbox() {
        let sandbox = SandboxPolicy::ExternalSandbox {
            network_access: NetworkAccess::Enabled,
        };
        let dirs = vec![PathBuf::from("/tmp/example")];
        assert_eq!(
            add_dir_warning_message(&dirs, &sandbox, Language::ZhCn),
            None
        );
    }

    #[test]
    fn warns_for_read_only() {
        let sandbox = SandboxPolicy::ReadOnly;
        let dirs = vec![PathBuf::from("relative"), PathBuf::from("/abs")];
        let message = add_dir_warning_message(&dirs, &sandbox, Language::ZhCn)
            .expect("expected warning for read-only sandbox");
        assert_eq!(
            message,
            "由于当前沙箱模式为只读，--add-dir (relative, /abs) 将被忽略。请切换到 workspace-write 或 danger-full-access 以允许额外的可写根目录。"
        );
    }

    #[test]
    fn returns_none_when_no_additional_dirs() {
        let sandbox = SandboxPolicy::ReadOnly;
        let dirs: Vec<PathBuf> = Vec::new();
        assert_eq!(
            add_dir_warning_message(&dirs, &sandbox, Language::ZhCn),
            None
        );
    }
}
