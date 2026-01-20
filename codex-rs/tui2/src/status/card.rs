use crate::history_cell::CompositeHistoryCell;
use crate::history_cell::HistoryCell;
use crate::history_cell::PlainHistoryCell;
use crate::history_cell::with_border_with_inner_width;
use crate::i18n::tr;
use crate::i18n::tr_args;
use crate::version::CODEX_CLI_VERSION;
use chrono::DateTime;
use chrono::Local;
use codex_common::create_config_summary_entries;
use codex_core::config::Config;
use codex_core::protocol::NetworkAccess;
use codex_core::protocol::SandboxPolicy;
use codex_core::protocol::TokenUsage;
use codex_core::protocol::TokenUsageInfo;
use codex_protocol::ThreadId;
use codex_protocol::account::PlanType;
use codex_protocol::config_types::Language;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use std::collections::BTreeSet;
use std::path::PathBuf;
use url::Url;

use super::account::StatusAccountDisplay;
use super::format::FieldFormatter;
use super::format::line_display_width;
use super::format::push_label;
use super::format::truncate_line_to_width;
use super::helpers::compose_account_display;
use super::helpers::compose_agents_summary;
use super::helpers::compose_model_display;
use super::helpers::format_directory_display;
use super::helpers::format_tokens_compact;
use super::rate_limits::RateLimitSnapshotDisplay;
use super::rate_limits::StatusRateLimitData;
use super::rate_limits::StatusRateLimitRow;
use super::rate_limits::StatusRateLimitValue;
use super::rate_limits::compose_rate_limit_data;
use super::rate_limits::format_status_limit_summary;
use super::rate_limits::render_status_limit_progress_bar;
use crate::wrapping::RtOptions;
use crate::wrapping::word_wrap_lines;
use codex_core::AuthManager;

#[derive(Debug, Clone)]
struct StatusContextWindowData {
    percent_remaining: i64,
    tokens_in_context: i64,
    window: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct StatusTokenUsageData {
    total: i64,
    input: i64,
    output: i64,
    context_window: Option<StatusContextWindowData>,
}

#[derive(Debug)]
struct StatusHistoryCell {
    model_name: String,
    model_details: Vec<String>,
    directory: PathBuf,
    approval: String,
    sandbox: String,
    agents_summary: String,
    model_provider: Option<String>,
    account: Option<StatusAccountDisplay>,
    session_id: Option<String>,
    token_usage: StatusTokenUsageData,
    rate_limits: StatusRateLimitData,
    language: Language,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn new_status_output(
    config: &Config,
    auth_manager: &AuthManager,
    token_info: Option<&TokenUsageInfo>,
    total_usage: &TokenUsage,
    session_id: &Option<ThreadId>,
    rate_limits: Option<&RateLimitSnapshotDisplay>,
    plan_type: Option<PlanType>,
    now: DateTime<Local>,
    model_name: &str,
) -> CompositeHistoryCell {
    let command = PlainHistoryCell::new(vec!["/status".magenta().into()]);
    let card = StatusHistoryCell::new(
        config,
        auth_manager,
        token_info,
        total_usage,
        session_id,
        rate_limits,
        plan_type,
        now,
        model_name,
    );

    CompositeHistoryCell::new(vec![Box::new(command), Box::new(card)])
}

impl StatusHistoryCell {
    #[allow(clippy::too_many_arguments)]
    fn new(
        config: &Config,
        auth_manager: &AuthManager,
        token_info: Option<&TokenUsageInfo>,
        total_usage: &TokenUsage,
        session_id: &Option<ThreadId>,
        rate_limits: Option<&RateLimitSnapshotDisplay>,
        plan_type: Option<PlanType>,
        now: DateTime<Local>,
        model_name: &str,
    ) -> Self {
        let config_entries = create_config_summary_entries(config, model_name);
        let (model_name, model_details) =
            compose_model_display(model_name, &config_entries, config.language);
        let approval = config_entries
            .iter()
            .find(|(k, _)| *k == "approval")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| tr(config.language, "status.approval.unknown").to_string());
        let sandbox = match config.sandbox_policy.get() {
            SandboxPolicy::DangerFullAccess => "danger-full-access".to_string(),
            SandboxPolicy::ReadOnly => "read-only".to_string(),
            SandboxPolicy::WorkspaceWrite { .. } => "workspace-write".to_string(),
            SandboxPolicy::ExternalSandbox { network_access } => {
                if matches!(network_access, NetworkAccess::Enabled) {
                    "external-sandbox (network access enabled)".to_string()
                } else {
                    "external-sandbox".to_string()
                }
            }
        };
        let agents_summary = compose_agents_summary(config, config.language);
        let model_provider = format_model_provider(config);
        let account = compose_account_display(auth_manager, plan_type, config.language);
        let session_id = session_id.as_ref().map(std::string::ToString::to_string);
        let default_usage = TokenUsage::default();
        let (context_usage, context_window) = match token_info {
            Some(info) => (&info.last_token_usage, info.model_context_window),
            None => (&default_usage, config.model_context_window),
        };
        let context_window = context_window.map(|window| StatusContextWindowData {
            percent_remaining: context_usage.percent_of_context_window_remaining(window),
            tokens_in_context: context_usage.tokens_in_context_window(),
            window,
        });

        let token_usage = StatusTokenUsageData {
            total: total_usage.blended_total(),
            input: total_usage.non_cached_input(),
            output: total_usage.output_tokens,
            context_window,
        };
        let rate_limits = compose_rate_limit_data(rate_limits, now, config.language);

        Self {
            model_name,
            model_details,
            directory: config.cwd.clone(),
            approval,
            sandbox,
            agents_summary,
            model_provider,
            account,
            session_id,
            token_usage,
            rate_limits,
            language: config.language,
        }
    }

    fn token_usage_spans(&self) -> Vec<Span<'static>> {
        let total_fmt = format_tokens_compact(self.token_usage.total);
        let input_fmt = format_tokens_compact(self.token_usage.input);
        let output_fmt = format_tokens_compact(self.token_usage.output);

        vec![
            Span::from(total_fmt),
            Span::from(tr(self.language, "status.token_usage.total_label")),
            Span::from(tr(self.language, "status.token_usage.paren_open")).dim(),
            Span::from(input_fmt).dim(),
            Span::from(tr(self.language, "status.token_usage.input_label")).dim(),
            Span::from(" + ").dim(),
            Span::from(output_fmt).dim(),
            Span::from(tr(self.language, "status.token_usage.output_label")).dim(),
            Span::from(tr(self.language, "status.token_usage.paren_close")).dim(),
        ]
    }

    fn context_window_spans(&self) -> Option<Vec<Span<'static>>> {
        let context = self.token_usage.context_window.as_ref()?;
        let percent = context.percent_remaining;
        let used_fmt = format_tokens_compact(context.tokens_in_context);
        let window_fmt = format_tokens_compact(context.window);

        let percent_value = percent.to_string();
        Some(vec![
            Span::from(tr_args(
                self.language,
                "status.context_window.remaining",
                &[("percent", percent_value.as_str())],
            )),
            Span::from(tr(self.language, "status.context_window.paren_open")).dim(),
            Span::from(used_fmt).dim(),
            Span::from(tr(self.language, "status.context_window.used_ratio")).dim(),
            Span::from(window_fmt).dim(),
            Span::from(tr(self.language, "status.context_window.paren_close")).dim(),
        ])
    }

    fn rate_limit_lines(
        &self,
        available_inner_width: usize,
        formatter: &FieldFormatter,
    ) -> Vec<Line<'static>> {
        match &self.rate_limits {
            StatusRateLimitData::Available(rows_data) => {
                if rows_data.is_empty() {
                    let label = tr(self.language, "status.rate_limits.label");
                    let value = tr(self.language, "status.rate_limits.data_not_available");
                    return vec![formatter.line(label, vec![Span::from(value).dim()])];
                }

                self.rate_limit_row_lines(rows_data, available_inner_width, formatter)
            }
            StatusRateLimitData::Stale(rows_data) => {
                let mut lines =
                    self.rate_limit_row_lines(rows_data, available_inner_width, formatter);
                lines.push(formatter.line(
                    tr(self.language, "status.rate_limits.warning_label"),
                    vec![Span::from(tr(self.language, "status.rate_limits.stale_warning")).dim()],
                ));
                lines
            }
            StatusRateLimitData::Missing => {
                let label = tr(self.language, "status.rate_limits.label");
                let value = tr(self.language, "status.rate_limits.data_not_available");
                vec![formatter.line(label, vec![Span::from(value).dim()])]
            }
        }
    }

    fn rate_limit_row_lines(
        &self,
        rows: &[StatusRateLimitRow],
        available_inner_width: usize,
        formatter: &FieldFormatter,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::with_capacity(rows.len().saturating_mul(2));

        for row in rows {
            match &row.value {
                StatusRateLimitValue::Window {
                    percent_used,
                    resets_at,
                } => {
                    let percent_remaining = (100.0 - percent_used).clamp(0.0, 100.0);
                    let value_spans = vec![
                        Span::from(render_status_limit_progress_bar(percent_remaining)),
                        Span::from(" "),
                        Span::from(format_status_limit_summary(
                            percent_remaining,
                            self.language,
                        )),
                    ];
                    let base_spans = formatter.full_spans(row.label.as_str(), value_spans);
                    let base_line = Line::from(base_spans.clone());

                    if let Some(resets_at) = resets_at.as_ref() {
                        let resets_span = Span::from(tr_args(
                            self.language,
                            "status.rate_limits.reset_at",
                            &[("resets_at", resets_at.as_str())],
                        ))
                        .dim();
                        let mut inline_spans = base_spans.clone();
                        inline_spans.push(Span::from(" ").dim());
                        inline_spans.push(resets_span.clone());

                        if line_display_width(&Line::from(inline_spans.clone()))
                            <= available_inner_width
                        {
                            lines.push(Line::from(inline_spans));
                        } else {
                            lines.push(base_line);
                            lines.push(formatter.continuation(vec![resets_span]));
                        }
                    } else {
                        lines.push(base_line);
                    }
                }
                StatusRateLimitValue::Text(text) => {
                    let label = row.label.clone();
                    let spans =
                        formatter.full_spans(label.as_str(), vec![Span::from(text.clone())]);
                    lines.push(Line::from(spans));
                }
            }
        }

        lines
    }

    fn collect_rate_limit_labels(&self, seen: &mut BTreeSet<String>, labels: &mut Vec<String>) {
        match &self.rate_limits {
            StatusRateLimitData::Available(rows) => {
                if rows.is_empty() {
                    push_label(labels, seen, tr(self.language, "status.rate_limits.label"));
                } else {
                    for row in rows {
                        push_label(labels, seen, row.label.as_str());
                    }
                }
            }
            StatusRateLimitData::Stale(rows) => {
                for row in rows {
                    push_label(labels, seen, row.label.as_str());
                }
                push_label(
                    labels,
                    seen,
                    tr(self.language, "status.rate_limits.warning_label"),
                );
            }
            StatusRateLimitData::Missing => {
                push_label(labels, seen, tr(self.language, "status.rate_limits.label"));
            }
        }
    }
}

impl HistoryCell for StatusHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let language = self.language;
        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(vec![
            Span::from(format!("{}>_ ", FieldFormatter::INDENT)).dim(),
            Span::from("OpenAI Codex").bold(),
            Span::from(" ").dim(),
            Span::from(format!("(v{CODEX_CLI_VERSION})")).dim(),
        ]));
        lines.push(Line::from(Vec::<Span<'static>>::new()));

        let available_inner_width = usize::from(width.saturating_sub(4));
        if available_inner_width == 0 {
            return Vec::new();
        }

        let account_value = self.account.as_ref().map(|account| match account {
            StatusAccountDisplay::ChatGpt { email, plan } => match (email, plan) {
                (Some(email), Some(plan)) => format!("{email} ({plan})"),
                (Some(email), None) => email.clone(),
                (None, Some(plan)) => plan.clone(),
                (None, None) => "ChatGPT".to_string(),
            },
            StatusAccountDisplay::ApiKey => {
                tr(language, "status.account.api_key_configured").to_string()
            }
        });

        let label_model = tr(language, "status.fields.model");
        let label_directory = tr(language, "status.fields.directory");
        let label_model_provider = tr(language, "status.fields.model_provider");
        let label_approval = tr(language, "status.fields.approval");
        let label_sandbox = tr(language, "status.fields.sandbox");
        let label_account = tr(language, "status.fields.account");
        let label_session = tr(language, "status.fields.session");
        let label_token_usage = tr(language, "status.fields.token_usage");
        let label_context_window = tr(language, "status.fields.context_window");

        let mut labels: Vec<String> = vec![
            label_model,
            label_directory,
            label_approval,
            label_sandbox,
            "Agents.md",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        let mut seen: BTreeSet<String> = labels.iter().cloned().collect();

        if self.model_provider.is_some() {
            push_label(&mut labels, &mut seen, label_model_provider);
        }
        if account_value.is_some() {
            push_label(&mut labels, &mut seen, label_account);
        }
        if self.session_id.is_some() {
            push_label(&mut labels, &mut seen, label_session);
        }
        push_label(&mut labels, &mut seen, label_token_usage);
        if self.token_usage.context_window.is_some() {
            push_label(&mut labels, &mut seen, label_context_window);
        }
        self.collect_rate_limit_labels(&mut seen, &mut labels);

        let formatter = FieldFormatter::from_labels(labels.iter().map(String::as_str));
        let value_width = formatter.value_width(available_inner_width);

        let note_first_line = Line::from(vec![
            Span::from(tr(language, "status.note.visit_prefix")).cyan(),
            "https://chatgpt.com/codex/settings/usage"
                .cyan()
                .underlined(),
            Span::from(tr(language, "status.note.up_to_date_prefix")).cyan(),
        ]);
        let note_second_line = Line::from(vec![
            Span::from(tr(language, "status.note.rate_limits_and_credits")).cyan(),
        ]);
        let note_lines = word_wrap_lines(
            [note_first_line, note_second_line],
            RtOptions::new(available_inner_width),
        );
        lines.extend(note_lines);
        lines.push(Line::from(Vec::<Span<'static>>::new()));

        let mut model_spans = vec![Span::from(self.model_name.clone())];
        if !self.model_details.is_empty() {
            model_spans.push(Span::from(" (").dim());
            model_spans.push(Span::from(self.model_details.join(", ")).dim());
            model_spans.push(Span::from(")").dim());
        }

        let directory_value = format_directory_display(&self.directory, Some(value_width));

        lines.push(formatter.line(label_model, model_spans));
        if let Some(model_provider) = self.model_provider.as_ref() {
            lines.push(formatter.line(
                label_model_provider,
                vec![Span::from(model_provider.clone())],
            ));
        }
        lines.push(formatter.line(label_directory, vec![Span::from(directory_value)]));
        lines.push(formatter.line(label_approval, vec![Span::from(self.approval.clone())]));
        lines.push(formatter.line(label_sandbox, vec![Span::from(self.sandbox.clone())]));
        lines.push(formatter.line("Agents.md", vec![Span::from(self.agents_summary.clone())]));

        if let Some(account_value) = account_value {
            lines.push(formatter.line(label_account, vec![Span::from(account_value)]));
        }

        if let Some(session) = self.session_id.as_ref() {
            lines.push(formatter.line(label_session, vec![Span::from(session.clone())]));
        }

        lines.push(Line::from(Vec::<Span<'static>>::new()));
        // Hide token usage only for ChatGPT subscribers
        if !matches!(self.account, Some(StatusAccountDisplay::ChatGpt { .. })) {
            lines.push(formatter.line(label_token_usage, self.token_usage_spans()));
        }

        if let Some(spans) = self.context_window_spans() {
            lines.push(formatter.line(label_context_window, spans));
        }

        lines.extend(self.rate_limit_lines(available_inner_width, &formatter));

        let content_width = lines.iter().map(line_display_width).max().unwrap_or(0);
        let inner_width = content_width.min(available_inner_width);
        let truncated_lines: Vec<Line<'static>> = lines
            .into_iter()
            .map(|line| truncate_line_to_width(line, inner_width))
            .collect();

        with_border_with_inner_width(truncated_lines, inner_width)
    }
}

fn format_model_provider(config: &Config) -> Option<String> {
    let provider = &config.model_provider;
    let name = provider.name.trim();
    let provider_name = if name.is_empty() {
        config.model_provider_id.as_str()
    } else {
        name
    };
    let base_url = provider.base_url.as_deref().and_then(sanitize_base_url);
    let is_default_openai = provider.is_openai() && base_url.is_none();
    if is_default_openai {
        return None;
    }

    Some(match base_url {
        Some(base_url) => format!("{provider_name} - {base_url}"),
        None => provider_name.to_string(),
    })
}

fn sanitize_base_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let Ok(mut url) = Url::parse(trimmed) else {
        return None;
    };
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    Some(url.to_string().trim_end_matches('/').to_string()).filter(|value| !value.is_empty())
}
