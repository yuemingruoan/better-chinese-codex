#![cfg(not(target_os = "windows"))]

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use codex_core::protocol::EventMsg;
use codex_core::protocol::Op;
use codex_core::protocol::SddGitAction;
use core_test_support::test_codex::TestCodexHarness;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

fn git(path: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(path)
        .status()
        .with_context(|| format!("failed to run git {args:?}"))?;
    if status.success() {
        return Ok(());
    }
    let exit_status = status;
    bail!("git {args:?} exited with {exit_status}");
}

fn git_output(path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .with_context(|| format!("failed to run git {args:?}"))?;
    if !output.status.success() {
        let exit_status = output.status;
        bail!("git {args:?} exited with {exit_status}");
    }
    String::from_utf8(output.stdout).context("stdout was not valid utf8")
}

fn init_git_repo(path: &Path) -> Result<()> {
    git(path, &["init", "--initial-branch=develop-main"])?;
    git(path, &["config", "core.autocrlf", "false"])?;
    git(path, &["config", "user.name", "Codex Tests"])?;
    git(path, &["config", "user.email", "codex-tests@example.com"])?;
    fs::write(path.join("README.txt"), "SDD test repo\n")?;
    git(path, &["add", "README.txt"])?;
    git(path, &["commit", "-m", "init sdd repo"])?;
    Ok(())
}

#[tokio::test]
async fn sdd_git_action_create_branch_dispatches() -> Result<()> {
    let harness = TestCodexHarness::new().await?;
    let repo = TempDir::new()?;
    init_git_repo(repo.path())?;

    harness
        .test()
        .codex
        .submit(Op::OverrideTurnContext {
            cwd: Some(repo.path().to_path_buf()),
            approval_policy: None,
            sandbox_policy: None,
            model: None,
            effort: None,
            summary: None,
        })
        .await?;

    let branch_name = "sdd/test-create-branch";
    harness
        .test()
        .codex
        .submit(Op::SddGitAction {
            action: SddGitAction::CreateBranch {
                name: branch_name.to_string(),
                base: "develop-main".to_string(),
            },
        })
        .await?;

    wait_for_event(&harness.test().codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let current = git_output(repo.path(), &["rev-parse", "--abbrev-ref", "HEAD"])?;
    assert_eq!(current.trim(), branch_name);
    Ok(())
}
