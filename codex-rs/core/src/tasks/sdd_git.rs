use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use codex_async_utils::CancelErr;
use codex_async_utils::OrCancelExt;
use codex_protocol::user_input::UserInput;
use tokio_util::sync::CancellationToken;
use tracing::error;
use uuid::Uuid;

use crate::codex::TurnContext;
use crate::exec::ExecToolCallOutput;
use crate::exec::SandboxType;
use crate::exec::StreamOutput;
use crate::exec::execute_exec_env;
use crate::exec_env::create_env;
use crate::parse_command::parse_command;
use crate::protocol::ErrorEvent;
use crate::protocol::EventMsg;
use crate::protocol::ExecCommandBeginEvent;
use crate::protocol::ExecCommandEndEvent;
use crate::protocol::ExecCommandSource;
use crate::protocol::SandboxPolicy;
use crate::protocol::SddGitAction;
use crate::protocol::TaskStartedEvent;
use crate::protocol::WarningEvent;
use crate::sandboxing::ExecEnv;
use crate::state::TaskKind;
use crate::tools::format_exec_output_str;

use super::SessionTask;
use super::SessionTaskContext;

const SDD_GIT_TIMEOUT_MS: u64 = 5 * 60 * 1000;
const SDD_BRANCH_PREFIX: &str = "sdd/";
const SDD_BASE_BRANCH: &str = "develop-main";

#[derive(Clone)]
pub(crate) struct SddGitTask {
    action: SddGitAction,
}

impl SddGitTask {
    pub(crate) fn new(action: SddGitAction) -> Self {
        Self { action }
    }
}

#[async_trait]
impl SessionTask for SddGitTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        turn_context: Arc<TurnContext>,
        _input: Vec<UserInput>,
        cancellation_token: CancellationToken,
    ) -> Option<String> {
        let event = EventMsg::TaskStarted(TaskStartedEvent {
            model_context_window: turn_context.client.get_model_context_window(),
        });
        let session = session.clone_session();
        session.send_event(turn_context.as_ref(), event).await;

        let result =
            run_sdd_git_action(&session, &turn_context, &self.action, &cancellation_token).await;

        if let Err(message) = result {
            session
                .send_event(
                    turn_context.as_ref(),
                    EventMsg::Error(ErrorEvent {
                        message,
                        codex_error_info: None,
                    }),
                )
                .await;
        }

        None
    }
}

async fn run_sdd_git_action(
    session: &Arc<crate::codex::Session>,
    turn_context: &Arc<TurnContext>,
    action: &SddGitAction,
    cancellation_token: &CancellationToken,
) -> Result<(), String> {
    ensure_git_repository(&turn_context.cwd)?;

    match action {
        SddGitAction::CreateBranch { name, base } => {
            ensure_base_branch(base)?;
            ensure_sdd_branch(name)?;
            ensure_clean_repo(&turn_context.cwd)?;

            let current = current_branch(&turn_context.cwd)?;
            if current != *base {
                run_git_logged(
                    session,
                    turn_context,
                    vec!["checkout", base],
                    cancellation_token,
                )
                .await?;
            }

            run_git_logged(
                session,
                turn_context,
                vec!["checkout", "-b", name],
                cancellation_token,
            )
            .await?;
        }
        SddGitAction::SwitchBranch { name } => {
            ensure_sdd_branch(name)?;

            let current = current_branch(&turn_context.cwd)?;
            if current != *name {
                run_git_logged(
                    session,
                    turn_context,
                    vec!["checkout", name],
                    cancellation_token,
                )
                .await?;
            }
        }
        SddGitAction::FinalizeMerge {
            name,
            base,
            commit_message,
        } => {
            ensure_base_branch(base)?;
            ensure_sdd_branch(name)?;

            let current = current_branch(&turn_context.cwd)?;
            let dirty = is_repo_dirty(&turn_context.cwd)?;
            if dirty && current != *name {
                return Err("工作区有未提交修改，且当前不在 SDD 分支，无法继续合并。".to_string());
            }
            if current != *name {
                run_git_logged(
                    session,
                    turn_context,
                    vec!["checkout", name],
                    cancellation_token,
                )
                .await?;
            }

            let dirty = is_repo_dirty(&turn_context.cwd)?;
            if dirty {
                run_git_logged(session, turn_context, vec!["add", "-A"], cancellation_token)
                    .await?;
                run_git_logged(
                    session,
                    turn_context,
                    vec!["commit", "-m", commit_message],
                    cancellation_token,
                )
                .await?;
            } else {
                session
                    .send_event(
                        turn_context.as_ref(),
                        EventMsg::Warning(WarningEvent {
                            message: "工作区无变更，跳过提交。".to_string(),
                        }),
                    )
                    .await;
            }

            let current = current_branch(&turn_context.cwd)?;
            if current != *base {
                run_git_logged(
                    session,
                    turn_context,
                    vec!["checkout", base],
                    cancellation_token,
                )
                .await?;
            }

            run_git_logged(
                session,
                turn_context,
                vec!["merge", "--no-ff", name],
                cancellation_token,
            )
            .await?;
        }
        SddGitAction::AbandonBranch { name, base } => {
            ensure_base_branch(base)?;
            ensure_sdd_branch(name)?;
            ensure_clean_repo(&turn_context.cwd)?;

            let current = current_branch(&turn_context.cwd)?;
            if current != *base {
                run_git_logged(
                    session,
                    turn_context,
                    vec!["checkout", base],
                    cancellation_token,
                )
                .await?;
            }

            run_git_logged(
                session,
                turn_context,
                vec!["branch", "-D", name],
                cancellation_token,
            )
            .await?;
        }
    }

    Ok(())
}

fn ensure_base_branch(base: &str) -> Result<(), String> {
    if base != SDD_BASE_BRANCH {
        return Err(format!(
            "仅允许使用基线分支 `{SDD_BASE_BRANCH}`，收到 `{base}`。"
        ));
    }
    Ok(())
}

fn ensure_sdd_branch(name: &str) -> Result<(), String> {
    if !name.starts_with(SDD_BRANCH_PREFIX) {
        return Err(format!(
            "仅允许操作 `{SDD_BRANCH_PREFIX}` 前缀分支，收到 `{name}`。"
        ));
    }
    if name.len() <= SDD_BRANCH_PREFIX.len() {
        return Err(format!("分支名无效：`{name}`。"));
    }
    if name.chars().any(char::is_whitespace) {
        return Err(format!("分支名包含空白字符：`{name}`。"));
    }
    if name.contains("..") {
        return Err(format!("分支名包含非法片段：`{name}`。"));
    }
    Ok(())
}

fn ensure_git_repository(repo: &Path) -> Result<(), String> {
    let output = run_git_silent(repo, &["rev-parse", "--is-inside-work-tree"])?;
    if output.trim() != "true" {
        return Err("当前目录不是 Git 仓库，无法执行 SDD Git 操作。".to_string());
    }
    Ok(())
}

fn current_branch(repo: &Path) -> Result<String, String> {
    let name = run_git_silent(repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if name.is_empty() {
        return Err("无法确定当前分支。".to_string());
    }
    Ok(name)
}

fn is_repo_dirty(repo: &Path) -> Result<bool, String> {
    let status = run_git_silent(repo, &["status", "--porcelain"])?;
    Ok(!status.trim().is_empty())
}

fn ensure_clean_repo(repo: &Path) -> Result<(), String> {
    if is_repo_dirty(repo)? {
        return Err("工作区存在未提交修改，请先清理后再操作。".to_string());
    }
    Ok(())
}

fn run_git_silent(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .map_err(|err| format!("执行 git 失败: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            return Err("git 命令执行失败。".to_string());
        }
        return Err(stderr);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn run_git_logged(
    session: &Arc<crate::codex::Session>,
    turn_context: &Arc<TurnContext>,
    args: Vec<&str>,
    cancellation_token: &CancellationToken,
) -> Result<(), String> {
    let mut command = Vec::with_capacity(args.len() + 1);
    command.push("git".to_string());
    for arg in args {
        command.push(arg.to_string());
    }

    let parsed_cmd = parse_command(&command);
    let call_id = Uuid::new_v4().to_string();

    session
        .send_event(
            turn_context.as_ref(),
            EventMsg::ExecCommandBegin(ExecCommandBeginEvent {
                call_id: call_id.clone(),
                process_id: None,
                turn_id: turn_context.sub_id.clone(),
                command: command.clone(),
                cwd: turn_context.cwd.clone(),
                parsed_cmd: parsed_cmd.clone(),
                source: ExecCommandSource::SddGit,
                interaction_input: None,
            }),
        )
        .await;

    let exec_env = ExecEnv {
        command: command.clone(),
        cwd: turn_context.cwd.clone(),
        env: create_env(&turn_context.shell_environment_policy),
        expiration: SDD_GIT_TIMEOUT_MS.into(),
        sandbox: SandboxType::None,
        sandbox_permissions: crate::sandboxing::SandboxPermissions::UseDefault,
        justification: None,
        arg0: None,
    };

    let exec_result = execute_exec_env(exec_env, &SandboxPolicy::DangerFullAccess, None)
        .or_cancel(cancellation_token)
        .await;

    match exec_result {
        Err(CancelErr::Cancelled) => {
            let aborted_message = "command aborted by user".to_string();
            session
                .send_event(
                    turn_context.as_ref(),
                    EventMsg::ExecCommandEnd(ExecCommandEndEvent {
                        call_id,
                        process_id: None,
                        turn_id: turn_context.sub_id.clone(),
                        command,
                        cwd: turn_context.cwd.clone(),
                        parsed_cmd,
                        source: ExecCommandSource::SddGit,
                        interaction_input: None,
                        stdout: String::new(),
                        stderr: aborted_message.clone(),
                        aggregated_output: aborted_message.clone(),
                        exit_code: -1,
                        duration: Duration::ZERO,
                        formatted_output: aborted_message,
                    }),
                )
                .await;
            Err("SDD Git 命令已取消。".to_string())
        }
        Ok(Ok(output)) => {
            session
                .send_event(
                    turn_context.as_ref(),
                    EventMsg::ExecCommandEnd(ExecCommandEndEvent {
                        call_id,
                        process_id: None,
                        turn_id: turn_context.sub_id.clone(),
                        command,
                        cwd: turn_context.cwd.clone(),
                        parsed_cmd,
                        source: ExecCommandSource::SddGit,
                        interaction_input: None,
                        stdout: output.stdout.text.clone(),
                        stderr: output.stderr.text.clone(),
                        aggregated_output: output.aggregated_output.text.clone(),
                        exit_code: output.exit_code,
                        duration: output.duration,
                        formatted_output: format_exec_output_str(
                            &output,
                            turn_context.truncation_policy,
                        ),
                    }),
                )
                .await;
            if output.exit_code == 0 {
                Ok(())
            } else {
                let message = if output.aggregated_output.text.trim().is_empty() {
                    "SDD Git 命令执行失败。".to_string()
                } else {
                    output.aggregated_output.text.clone()
                };
                Err(message)
            }
        }
        Ok(Err(err)) => {
            error!("sdd git command failed: {err:?}");
            let message = format!("SDD Git 执行失败: {err:?}");
            let exec_output = ExecToolCallOutput {
                exit_code: -1,
                stdout: StreamOutput::new(String::new()),
                stderr: StreamOutput::new(message.clone()),
                aggregated_output: StreamOutput::new(message.clone()),
                duration: Duration::ZERO,
                timed_out: false,
            };
            session
                .send_event(
                    turn_context.as_ref(),
                    EventMsg::ExecCommandEnd(ExecCommandEndEvent {
                        call_id,
                        process_id: None,
                        turn_id: turn_context.sub_id.clone(),
                        command,
                        cwd: turn_context.cwd.clone(),
                        parsed_cmd,
                        source: ExecCommandSource::SddGit,
                        interaction_input: None,
                        stdout: exec_output.stdout.text.clone(),
                        stderr: exec_output.stderr.text.clone(),
                        aggregated_output: exec_output.aggregated_output.text.clone(),
                        exit_code: exec_output.exit_code,
                        duration: exec_output.duration,
                        formatted_output: exec_output.aggregated_output.text.clone(),
                    }),
                )
                .await;
            Err(message)
        }
    }
}
