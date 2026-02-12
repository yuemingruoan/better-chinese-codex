use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use serde_json::json;

use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::SearchToolBm25Handler;
use crate::tools::handlers::collab::CollabHandler;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::ToolHandler;
use crate::tools::registry::ToolKind;

pub struct ClaudeToolAdapterHandler;

const TASK_TOOL_NAME: &str = "Task";
const TASK_OUTPUT_TOOL_NAME: &str = "TaskOutput";
const TASK_STOP_TOOL_NAME: &str = "TaskStop";
const TOOL_SEARCH_TOOL_NAME: &str = "ToolSearch";
const SKILL_TOOL_NAME: &str = "Skill";

fn default_block() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct TaskArgs {
    description: Option<String>,
    prompt: Option<String>,
    subagent_type: Option<String>,
    max_turns: Option<u32>,
    mode: Option<String>,
    model: Option<String>,
    name: Option<String>,
    resume: Option<String>,
    run_in_background: Option<bool>,
    team_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TaskOutputArgs {
    task_id: Option<String>,
    #[serde(default = "default_block")]
    block: bool,
    timeout: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TaskStopArgs {
    task_id: Option<String>,
    shell_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ToolSearchArgs {
    query: Option<String>,
    max_results: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct SkillArgs {
    skill: Option<String>,
    args: Option<String>,
}

#[async_trait]
impl ToolHandler for ClaudeToolAdapterHandler {
    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<ToolOutput, FunctionCallError> {
        let arguments = match invocation.payload.clone() {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "Claude tool adapter only supports function payloads / Claude 工具适配仅支持函数负载"
                        .to_string(),
                ));
            }
        };

        match invocation.tool_name.as_str() {
            TASK_TOOL_NAME => {
                let args: TaskArgs = parse_arguments(&arguments)?;
                let mapped_args = map_task_to_spawn_payload(args)?;
                dispatch_to_collab(invocation, "spawn_agent", mapped_args).await
            }
            TASK_OUTPUT_TOOL_NAME => {
                let args: TaskOutputArgs = parse_arguments(&arguments)?;
                let mapped_args = map_task_output_to_wait_payload(args)?;
                dispatch_to_collab(invocation, "wait", mapped_args).await
            }
            TASK_STOP_TOOL_NAME => {
                let args: TaskStopArgs = parse_arguments(&arguments)?;
                let mapped_args = map_task_stop_to_close_payload(args)?;
                dispatch_to_collab(invocation, "close_agent", mapped_args).await
            }
            TOOL_SEARCH_TOOL_NAME => {
                let args: ToolSearchArgs = parse_arguments(&arguments)?;
                let mapped_args = map_tool_search_payload(args)?;
                dispatch_to_search_tool(invocation, mapped_args).await
            }
            SKILL_TOOL_NAME => {
                let args: SkillArgs = parse_arguments(&arguments)?;
                let mapped_args = map_skill_to_spawn_payload(args)?;
                dispatch_to_collab(invocation, "spawn_agent", mapped_args).await
            }
            other => Err(FunctionCallError::RespondToModel(format!(
                "unsupported Claude tool alias {other} / 不支持的 Claude 工具别名: {other}"
            ))),
        }
    }
}

fn map_task_to_spawn_payload(args: TaskArgs) -> Result<JsonValue, FunctionCallError> {
    let TaskArgs {
        description,
        prompt,
        subagent_type,
        max_turns,
        mode,
        model,
        name,
        resume,
        run_in_background,
        team_name,
    } = args;

    let _ignored = (max_turns, mode, resume, run_in_background, team_name);
    let prompt = required_non_empty_text(
        prompt.as_deref(),
        "prompt must not be empty / prompt 不能为空",
    )?;

    let mut payload = JsonMap::new();
    payload.insert(
        "items".to_string(),
        json!([{ "type": "text", "text": prompt }]),
    );

    if let Some(agent_type) = normalize_text(subagent_type.as_deref())
        && is_supported_agent_type(&agent_type)
    {
        payload.insert("agent_type".to_string(), JsonValue::String(agent_type));
    }

    if let Some(label) =
        normalize_text(name.as_deref()).or_else(|| normalize_text(description.as_deref()))
    {
        payload.insert("label".to_string(), JsonValue::String(label));
    }

    if let Some(model) = normalize_text(model.as_deref()) {
        payload.insert("model".to_string(), JsonValue::String(model));
    }

    Ok(JsonValue::Object(payload))
}

fn map_task_output_to_wait_payload(args: TaskOutputArgs) -> Result<JsonValue, FunctionCallError> {
    let task_id = required_non_empty_text(
        args.task_id.as_deref(),
        "task_id must not be empty / task_id 不能为空",
    )?;
    let mut payload = JsonMap::new();
    payload.insert("ids".to_string(), json!([task_id]));
    let timeout_ms = if args.block { args.timeout } else { Some(0) };
    if let Some(timeout_ms) = timeout_ms {
        if args.block && timeout_ms < 0 {
            return Err(FunctionCallError::RespondToModel(
                "timeout must be greater than or equal to zero / timeout 必须大于等于 0"
                    .to_string(),
            ));
        }
        payload.insert("timeout_ms".to_string(), json!(timeout_ms));
    }
    Ok(JsonValue::Object(payload))
}

fn map_task_stop_to_close_payload(args: TaskStopArgs) -> Result<JsonValue, FunctionCallError> {
    let target_id = normalize_text(args.task_id.as_deref())
        .or_else(|| normalize_text(args.shell_id.as_deref()))
        .ok_or_else(|| {
            FunctionCallError::RespondToModel(
                "task_id or shell_id must not be empty / task_id 或 shell_id 不能为空".to_string(),
            )
        })?;
    Ok(json!({ "id": target_id }))
}

fn map_tool_search_payload(args: ToolSearchArgs) -> Result<JsonValue, FunctionCallError> {
    let query = required_non_empty_text(
        args.query.as_deref(),
        "query must not be empty / query 不能为空",
    )?;
    let mut payload = JsonMap::new();
    payload.insert("query".to_string(), JsonValue::String(query));
    if let Some(limit) = args.max_results {
        if limit == 0 {
            return Err(FunctionCallError::RespondToModel(
                "max_results must be greater than zero / max_results 必须大于 0".to_string(),
            ));
        }
        payload.insert("limit".to_string(), json!(limit));
    }
    Ok(JsonValue::Object(payload))
}

fn map_skill_to_spawn_payload(args: SkillArgs) -> Result<JsonValue, FunctionCallError> {
    let skill = required_non_empty_text(
        args.skill.as_deref(),
        "skill must not be empty / skill 不能为空",
    )?;

    let mut items = vec![json!({
        "type": "skill",
        "name": skill,
        "path": format!("skill://{skill}"),
    })];

    if let Some(text) = normalize_text(args.args.as_deref()) {
        items.push(json!({ "type": "text", "text": text }));
    }

    Ok(json!({
        "items": items,
        "label": format!("skill:{skill}"),
    }))
}

fn required_non_empty_text(
    value: Option<&str>,
    error_message: &'static str,
) -> Result<String, FunctionCallError> {
    normalize_text(value)
        .ok_or_else(|| FunctionCallError::RespondToModel(error_message.to_string()))
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn is_supported_agent_type(value: &str) -> bool {
    matches!(value, "default" | "worker" | "explorer" | "orchestrator")
}

async fn dispatch_to_collab(
    invocation: ToolInvocation,
    target_tool_name: &str,
    mapped_arguments: JsonValue,
) -> Result<ToolOutput, FunctionCallError> {
    let ToolInvocation {
        session,
        turn,
        tracker,
        call_id,
        ..
    } = invocation;
    let arguments = serde_json::to_string(&mapped_arguments).map_err(|err| {
        FunctionCallError::Fatal(format!(
            "failed to serialize {target_tool_name} alias arguments: {err}"
        ))
    })?;

    CollabHandler
        .handle(ToolInvocation {
            session,
            turn,
            tracker,
            call_id,
            tool_name: target_tool_name.to_string(),
            payload: ToolPayload::Function { arguments },
        })
        .await
}

async fn dispatch_to_search_tool(
    invocation: ToolInvocation,
    mapped_arguments: JsonValue,
) -> Result<ToolOutput, FunctionCallError> {
    let ToolInvocation {
        session,
        turn,
        tracker,
        call_id,
        ..
    } = invocation;
    let arguments = serde_json::to_string(&mapped_arguments).map_err(|err| {
        FunctionCallError::Fatal(format!(
            "failed to serialize search_tool_bm25 alias arguments: {err}"
        ))
    })?;

    SearchToolBm25Handler
        .handle(ToolInvocation {
            session,
            turn,
            tracker,
            call_id,
            tool_name: "search_tool_bm25".to_string(),
            payload: ToolPayload::Function { arguments },
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CodexAuth;
    use crate::ThreadManager;
    use crate::built_in_model_providers;
    use crate::codex::make_session_and_context;
    use crate::protocol::Op;
    use crate::turn_diff_tracker::TurnDiffTracker;
    use codex_protocol::models::FunctionCallOutputBody;
    use pretty_assertions::assert_eq;
    use serde_json::Value as JsonValue;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;
    use tokio::time::timeout;

    fn invocation(
        session: Arc<crate::codex::Session>,
        turn: Arc<crate::codex::TurnContext>,
        tool_name: &str,
        arguments: JsonValue,
    ) -> ToolInvocation {
        ToolInvocation {
            session,
            turn,
            tracker: Arc::new(Mutex::new(TurnDiffTracker::default())),
            call_id: "call-1".to_string(),
            tool_name: tool_name.to_string(),
            payload: ToolPayload::Function {
                arguments: arguments.to_string(),
            },
        }
    }

    fn thread_manager() -> ThreadManager {
        ThreadManager::with_models_provider(
            CodexAuth::from_api_key("dummy"),
            built_in_model_providers()["openai"].clone(),
        )
    }

    #[test]
    fn task_output_non_blocking_maps_to_zero_timeout() {
        let payload = map_task_output_to_wait_payload(TaskOutputArgs {
            task_id: Some("agent-1".to_string()),
            block: false,
            timeout: Some(5000),
        })
        .expect("payload should be valid");

        assert_eq!(
            payload,
            json!({
                "ids": ["agent-1"],
                "timeout_ms": 0,
            })
        );
    }

    #[test]
    fn task_maps_supported_agent_type_and_label() {
        let payload = map_task_to_spawn_payload(TaskArgs {
            description: Some("Investigate failing test".to_string()),
            prompt: Some("Check latest regression".to_string()),
            subagent_type: Some("explorer".to_string()),
            max_turns: None,
            mode: None,
            model: Some("gpt-5.1-codex-mini".to_string()),
            name: None,
            resume: None,
            run_in_background: None,
            team_name: None,
        })
        .expect("payload should be valid");

        assert_eq!(
            payload,
            json!({
                "items": [{"type": "text", "text": "Check latest regression"}],
                "agent_type": "explorer",
                "label": "Investigate failing test",
                "model": "gpt-5.1-codex-mini",
            })
        );
    }

    #[test]
    fn tool_search_maps_max_results_to_limit() {
        let payload = map_tool_search_payload(ToolSearchArgs {
            query: Some("slack send".to_string()),
            max_results: Some(3),
        })
        .expect("payload should be valid");

        assert_eq!(
            payload,
            json!({
                "query": "slack send",
                "limit": 3,
            })
        );
    }

    #[test]
    fn skill_maps_to_spawnable_skill_item() {
        let payload = map_skill_to_spawn_payload(SkillArgs {
            skill: Some("review-pr".to_string()),
            args: Some("123".to_string()),
        })
        .expect("skill payload should be valid");

        assert_eq!(
            payload,
            json!({
                "items": [
                    {
                        "type": "skill",
                        "name": "review-pr",
                        "path": "skill://review-pr",
                    },
                    {
                        "type": "text",
                        "text": "123",
                    }
                ],
                "label": "skill:review-pr",
            })
        );
    }

    #[test]
    fn task_output_rejects_missing_task_id() {
        let err = map_task_output_to_wait_payload(TaskOutputArgs {
            task_id: None,
            block: true,
            timeout: None,
        })
        .expect_err("missing task_id should fail");

        assert_eq!(
            err,
            FunctionCallError::RespondToModel(
                "task_id must not be empty / task_id 不能为空".to_string()
            )
        );
    }

    #[test]
    fn task_output_rejects_negative_blocking_timeout() {
        let err = map_task_output_to_wait_payload(TaskOutputArgs {
            task_id: Some("agent-1".to_string()),
            block: true,
            timeout: Some(-1),
        })
        .expect_err("negative timeout should fail");

        assert_eq!(
            err,
            FunctionCallError::RespondToModel(
                "timeout must be greater than or equal to zero / timeout 必须大于等于 0"
                    .to_string()
            )
        );
    }

    #[test]
    fn task_stop_accepts_shell_id_fallback() {
        let payload = map_task_stop_to_close_payload(TaskStopArgs {
            task_id: None,
            shell_id: Some("agent-2".to_string()),
        })
        .expect("shell_id fallback should map");

        assert_eq!(payload, json!({ "id": "agent-2" }));
    }

    #[test]
    fn tool_search_rejects_zero_max_results() {
        let err = map_tool_search_payload(ToolSearchArgs {
            query: Some("list".to_string()),
            max_results: Some(0),
        })
        .expect_err("max_results=0 should fail");

        assert_eq!(
            err,
            FunctionCallError::RespondToModel(
                "max_results must be greater than zero / max_results 必须大于 0".to_string()
            )
        );
    }

    #[tokio::test]
    async fn task_output_block_false_is_non_blocking_end_to_end() {
        let (mut session, turn) = make_session_and_context().await;
        let manager = thread_manager();
        session.services.agent_control = manager.agent_control();
        let config = turn.config.as_ref().clone();
        let thread = manager.start_thread(config).await.expect("start thread");
        let agent_id = thread.thread_id.to_string();

        let invocation = invocation(
            Arc::new(session),
            Arc::new(turn),
            TASK_OUTPUT_TOOL_NAME,
            json!({
                "task_id": agent_id,
                "block": false,
                "timeout": 5000
            }),
        );

        let output = timeout(
            Duration::from_millis(500),
            ClaudeToolAdapterHandler.handle(invocation),
        )
        .await
        .expect("TaskOutput block=false should return quickly")
        .expect("TaskOutput alias should succeed");

        let ToolOutput::Function {
            body: FunctionCallOutputBody::Text(content),
            success,
            ..
        } = output
        else {
            panic!("expected function output");
        };
        assert_eq!(success, None);

        let result: JsonValue =
            serde_json::from_str(&content).expect("TaskOutput result should be valid json");
        assert_eq!(result.get("timed_out"), Some(&JsonValue::Bool(true)));
        assert_eq!(
            result.get("wakeup_reason"),
            Some(&JsonValue::String("timeout".to_string()))
        );

        let _ = thread
            .thread
            .submit(Op::Shutdown {})
            .await
            .expect("shutdown should submit");
    }
}
