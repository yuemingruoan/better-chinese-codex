use codex_git::merge_base_with_head;
use codex_protocol::config_types::Language;
use codex_protocol::protocol::ReviewRequest;
use codex_protocol::protocol::ReviewTarget;
use std::path::Path;

use crate::i18n::tr;
use crate::i18n::tr_args;

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedReviewRequest {
    pub target: ReviewTarget,
    pub prompt: String,
    pub user_facing_hint: String,
}

pub fn resolve_review_request(
    request: ReviewRequest,
    cwd: &Path,
    language: Language,
) -> anyhow::Result<ResolvedReviewRequest> {
    let target = request.target;
    let prompt = review_prompt(&target, cwd, language)?;
    let user_facing_hint = request
        .user_facing_hint
        .unwrap_or_else(|| user_facing_hint(&target, language));

    Ok(ResolvedReviewRequest {
        target,
        prompt,
        user_facing_hint,
    })
}

pub fn review_prompt(
    target: &ReviewTarget,
    cwd: &Path,
    language: Language,
) -> anyhow::Result<String> {
    match target {
        ReviewTarget::UncommittedChanges => {
            Ok(tr(language, "review.prompt.uncommitted").to_string())
        }
        ReviewTarget::BaseBranch { branch } => {
            if let Some(commit) = merge_base_with_head(cwd, branch)? {
                Ok(tr_args(
                    language,
                    "review.prompt.base_branch",
                    &[("baseBranch", branch.as_str()), ("mergeBaseSha", &commit)],
                ))
            } else {
                Ok(tr_args(
                    language,
                    "review.prompt.base_branch_backup",
                    &[("branch", branch.as_str())],
                ))
            }
        }
        ReviewTarget::Commit { sha, title } => {
            if let Some(title) = title {
                Ok(tr_args(
                    language,
                    "review.prompt.commit_with_title",
                    &[("sha", sha.as_str()), ("title", title.as_str())],
                ))
            } else {
                Ok(tr_args(
                    language,
                    "review.prompt.commit",
                    &[("sha", sha.as_str())],
                ))
            }
        }
        ReviewTarget::Custom { instructions } => {
            let prompt = instructions.trim();
            if prompt.is_empty() {
                anyhow::bail!(tr(language, "review.error.empty_prompt"));
            }
            Ok(prompt.to_string())
        }
    }
}

pub fn user_facing_hint(target: &ReviewTarget, language: Language) -> String {
    match target {
        ReviewTarget::UncommittedChanges => tr(language, "review.hint.uncommitted").to_string(),
        ReviewTarget::BaseBranch { branch } => tr_args(
            language,
            "review.hint.base_branch",
            &[("branch", branch.as_str())],
        ),
        ReviewTarget::Commit { sha, title } => {
            let short_sha: String = sha.chars().take(7).collect();
            if let Some(title) = title {
                tr_args(
                    language,
                    "review.hint.commit_with_title",
                    &[("sha", &short_sha), ("title", title.as_str())],
                )
            } else {
                tr_args(language, "review.hint.commit", &[("sha", &short_sha)])
            }
        }
        ReviewTarget::Custom { instructions } => instructions.trim().to_string(),
    }
}

impl From<ResolvedReviewRequest> for ReviewRequest {
    fn from(resolved: ResolvedReviewRequest) -> Self {
        ReviewRequest {
            target: resolved.target,
            user_facing_hint: Some(resolved.user_facing_hint),
        }
    }
}
