#![allow(clippy::expect_used)]
use codex_core::CodexAuth;
use codex_core::ConversationManager;
use codex_core::ModelProviderInfo;
use codex_core::NewConversation;
use codex_core::built_in_model_providers;
use codex_core::compact::SUMMARIZATION_PROMPT;
use codex_core::compact::SUMMARY_PREFIX;
use codex_core::config::Config;
use codex_core::protocol::EventMsg;
use codex_core::protocol::Op;
use codex_core::protocol::RolloutItem;
use codex_core::protocol::RolloutLine;
use codex_core::protocol::WarningEvent;
use codex_protocol::user_input::UserInput;
use core_test_support::load_default_config_for_test;
use core_test_support::skip_if_no_network;
use core_test_support::wait_for_event;
use core_test_support::wait_for_event_match;
use std::collections::VecDeque;
use tempfile::TempDir;

use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_completed_with_tokens;
use core_test_support::responses::get_responses_requests;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::sse_failed;
use core_test_support::responses::start_mock_server;
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::MockServer;
// --- Test helpers -----------------------------------------------------------

pub(super) const FIRST_REPLY: &str = "FIRST_REPLY";
pub(super) const SECOND_LARGE_REPLY: &str = "SECOND_LARGE_REPLY";
pub(super) const FINAL_REPLY: &str = "FINAL_REPLY";
pub(super) const SUMMARY_TEXT: &str = "SUMMARY_ONLY_CONTEXT";
const CONTEXT_LIMIT_MESSAGE: &str = "context_length_exceeded";
const THIRD_USER_MSG: &str = "next turn";

pub(super) const COMPACT_WARNING_MESSAGE: &str = "Heads up: Long conversations and multiple compactions can cause the model to be less accurate. Start a new conversation when possible to keep conversations small and targeted.";

fn summary_with_prefix(summary: &str) -> String {
    format!("{SUMMARY_PREFIX}\n{summary}")
}

fn set_test_compact_prompt(config: &mut Config) {
    config.compact_prompt = Some(SUMMARIZATION_PROMPT.to_string());
}

fn auto_summary(summary: &str) -> String {
    summary.to_string()
}

fn body_contains_text(body: &str, text: &str) -> bool {
    body.contains(&json_fragment(text))
}

fn json_fragment(text: &str) -> String {
    serde_json::to_string(text)
        .expect("serialize text to JSON")
        .trim_matches('"')
        .to_string()
}

fn drop_call_id(value: &mut serde_json::Value) -> &mut serde_json::Value {
    if let Some(object) = value.as_object_mut() {
        object.remove("call_id");
    }
    value
}

fn non_openai_model_provider(server: &MockServer) -> ModelProviderInfo {
    let mut provider = built_in_model_providers()["openai"].clone();
    provider.name = "OpenAI (test)".into();
    provider.base_url = Some(format!("{}/v1", server.uri()));
    provider
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn summarize_context_three_requests_and_instructions() {
    skip_if_no_network!();

    // Set up a mock server that we can inspect after the run.
    let server = start_mock_server().await;

    // SSE 1: assistant replies normally so it is recorded in history.
    let sse1 = sse(vec![
        ev_assistant_message("m1", FIRST_REPLY),
        ev_completed("r1"),
    ]);

    // SSE 2: summarizer returns a summary message.
    let sse2 = sse(vec![
        ev_assistant_message("m2", SUMMARY_TEXT),
        ev_completed("r2"),
    ]);

    // SSE 3: minimal completed; we only need to capture the request body.
    let sse3 = sse(vec![ev_completed("r3")]);

    // Mount the three expected requests in sequence so the assertions below can
    // inspect them without relying on specific prompt markers.
    let request_log = mount_sse_sequence(&server, vec![sse1, sse2, sse3]).await;

    // Build config pointing to the mock server and spawn Codex.
    let model_provider = non_openai_model_provider(&server);
    let home = TempDir::new().unwrap();
    let mut config = load_default_config_for_test(&home).await;
    config.model_provider = model_provider;
    set_test_compact_prompt(&mut config);
    let conversation_manager = ConversationManager::with_models_provider(
        CodexAuth::from_api_key("dummy"),
        config.model_provider.clone(),
    );
    let NewConversation {
        conversation: codex,
        session_configured,
        ..
    } = conversation_manager.new_conversation(config).await.unwrap();
    let rollout_path = session_configured.rollout_path;

    // 1) Normal user input – should hit server once.
    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "hello world".into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    // 2) Summarize – second hit should include the summarization prompt.
    codex.submit(Op::Compact).await.unwrap();
    let warning_event = wait_for_event(&codex, |ev| matches!(ev, EventMsg::Warning(_))).await;
    let EventMsg::Warning(WarningEvent { message }) = warning_event else {
        panic!("expected warning event after compact");
    };
    assert_eq!(message, COMPACT_WARNING_MESSAGE);
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    // 3) Next user input – third hit; history should include only the summary.
    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: THIRD_USER_MSG.into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    // Inspect the three captured requests.
    let requests = request_log.requests();
    assert_eq!(requests.len(), 3, "expected exactly three requests");
    let body1 = requests[0].body_json();
    let body2 = requests[1].body_json();
    let body3 = requests[2].body_json();

    // Manual compact should keep the baseline developer instructions.
    let instr1 = body1.get("instructions").and_then(|v| v.as_str()).unwrap();
    let instr2 = body2.get("instructions").and_then(|v| v.as_str()).unwrap();
    assert_eq!(
        instr1, instr2,
        "manual compact should keep the standard developer instructions"
    );

    // The summarization request should include the injected user input marker.
    let body2_str = body2.to_string();
    let input2 = body2.get("input").and_then(|v| v.as_array()).unwrap();
    let has_compact_prompt = body_contains_text(&body2_str, SUMMARIZATION_PROMPT);
    assert!(
        has_compact_prompt,
        "compaction request should include the summarize trigger"
    );
    // The last item is the user message created from the injected input.
    let last2 = input2.last().unwrap();
    assert_eq!(last2.get("type").unwrap().as_str().unwrap(), "message");
    assert_eq!(last2.get("role").unwrap().as_str().unwrap(), "user");
    let text2 = last2["content"][0]["text"].as_str().unwrap();
    assert_eq!(
        text2, SUMMARIZATION_PROMPT,
        "expected summarize trigger, got `{text2}`"
    );

    // Third request must contain the refreshed instructions, compacted user history, and new user message.
    let input3 = body3.get("input").and_then(|v| v.as_array()).unwrap();

    assert!(
        input3.len() >= 3,
        "expected refreshed context and new user message in third request"
    );

    let mut messages: Vec<(String, String)> = Vec::new();
    let expected_summary_message = summary_with_prefix(SUMMARY_TEXT);

    for item in input3 {
        if let Some("message") = item.get("type").and_then(|v| v.as_str()) {
            let role = item
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let text = item
                .get("content")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|entry| entry.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            messages.push((role, text));
        }
    }

    // No previous assistant messages should remain and the new user message is present.
    let assistant_count = messages.iter().filter(|(r, _)| r == "assistant").count();
    assert_eq!(assistant_count, 0, "assistant history should be cleared");
    assert!(
        messages
            .iter()
            .any(|(r, t)| r == "user" && t == THIRD_USER_MSG),
        "third request should include the new user message"
    );
    assert!(
        messages
            .iter()
            .any(|(r, t)| r == "user" && t == "hello world"),
        "third request should include the original user message"
    );
    assert!(
        messages
            .iter()
            .any(|(r, t)| r == "user" && t == &expected_summary_message),
        "third request should include the summary message"
    );
    assert!(
        !messages
            .iter()
            .any(|(_, text)| text.contains(SUMMARIZATION_PROMPT)),
        "third request should not include the summarize trigger"
    );

    // Shut down Codex to flush rollout entries before inspecting the file.
    codex.submit(Op::Shutdown).await.unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::ShutdownComplete)).await;

    // Verify rollout contains APITurn entries for each API call and a Compacted entry.
    println!("rollout path: {}", rollout_path.display());
    let text = std::fs::read_to_string(&rollout_path).unwrap_or_else(|e| {
        panic!(
            "failed to read rollout file {}: {e}",
            rollout_path.display()
        )
    });
    let mut api_turn_count = 0usize;
    let mut saw_compacted_summary = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(entry): Result<RolloutLine, _> = serde_json::from_str(trimmed) else {
            continue;
        };
        match entry.item {
            RolloutItem::TurnContext(_) => {
                api_turn_count += 1;
            }
            RolloutItem::Compacted(ci) => {
                if ci.message == expected_summary_message {
                    saw_compacted_summary = true;
                }
            }
            _ => {}
        }
    }

    assert!(
        api_turn_count == 3,
        "expected three APITurn entries in rollout"
    );
    assert!(
        saw_compacted_summary,
        "expected a Compacted entry containing the summarizer output"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manual_compact_uses_custom_prompt() {
    skip_if_no_network!();

    let server = start_mock_server().await;
    let sse_stream = sse(vec![ev_completed("r1")]);
    mount_sse_once(&server, sse_stream).await;

    let custom_prompt = "Use this compact prompt instead";

    let model_provider = non_openai_model_provider(&server);
    let home = TempDir::new().unwrap();
    let mut config = load_default_config_for_test(&home).await;
    config.model_provider = model_provider;
    config.compact_prompt = Some(custom_prompt.to_string());

    let conversation_manager = ConversationManager::with_models_provider(
        CodexAuth::from_api_key("dummy"),
        config.model_provider.clone(),
    );
    let codex = conversation_manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;

    codex.submit(Op::Compact).await.expect("trigger compact");
    let warning_event = wait_for_event(&codex, |ev| matches!(ev, EventMsg::Warning(_))).await;
    let EventMsg::Warning(WarningEvent { message }) = warning_event else {
        panic!("expected warning event after compact");
    };
    assert_eq!(message, COMPACT_WARNING_MESSAGE);
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    let requests = get_responses_requests(&server).await;
    let body = requests
        .iter()
        .find_map(|req| req.body_json::<serde_json::Value>().ok())
        .expect("summary request body");

    let input = body
        .get("input")
        .and_then(|v| v.as_array())
        .expect("input array");
    let mut found_custom_prompt = false;
    let mut found_default_prompt = false;

    for item in input {
        if item["type"].as_str() != Some("message") {
            continue;
        }
        let text = item["content"][0]["text"].as_str().unwrap_or_default();
        if text == custom_prompt {
            found_custom_prompt = true;
        }
        if text == SUMMARIZATION_PROMPT {
            found_default_prompt = true;
        }
    }

    let used_prompt = found_custom_prompt || found_default_prompt;
    if used_prompt {
        assert!(found_custom_prompt, "custom prompt should be injected");
        assert!(
            !found_default_prompt,
            "default prompt should be replaced when a compact prompt is used"
        );
    } else {
        assert!(
            !found_default_prompt,
            "summarization prompt should not appear if compaction omits a prompt"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manual_compact_emits_api_and_local_token_usage_events() {
    skip_if_no_network!();

    let server = start_mock_server().await;

    // Compact run where the API reports zero tokens in usage. Our local
    // estimator should still compute a non-zero context size for the compacted
    // history.
    let sse_compact = sse(vec![
        ev_assistant_message("m1", SUMMARY_TEXT),
        ev_completed_with_tokens("r1", 0),
    ]);
    mount_sse_once(&server, sse_compact).await;

    let model_provider = non_openai_model_provider(&server);
    let home = TempDir::new().unwrap();
    let mut config = load_default_config_for_test(&home).await;
    config.model_provider = model_provider;
    set_test_compact_prompt(&mut config);

    let conversation_manager = ConversationManager::with_models_provider(
        CodexAuth::from_api_key("dummy"),
        config.model_provider.clone(),
    );
    let NewConversation {
        conversation: codex,
        ..
    } = conversation_manager.new_conversation(config).await.unwrap();

    // Trigger manual compact and collect TokenCount events for the compact turn.
    codex.submit(Op::Compact).await.unwrap();

    // First TokenCount: from the compact API call (usage.total_tokens = 0).
    let first = wait_for_event_match(&codex, |ev| match ev {
        EventMsg::TokenCount(tc) => tc
            .info
            .as_ref()
            .map(|info| info.last_token_usage.total_tokens),
        _ => None,
    })
    .await;

    // Second TokenCount: from the local post-compaction estimate.
    let last = wait_for_event_match(&codex, |ev| match ev {
        EventMsg::TokenCount(tc) => tc
            .info
            .as_ref()
            .map(|info| info.last_token_usage.total_tokens),
        _ => None,
    })
    .await;

    // Ensure the compact task itself completes.
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    assert_eq!(
        first, 0,
        "expected first TokenCount from compact API usage to be zero"
    );
    assert!(
        last > 0,
        "second TokenCount should reflect a non-zero estimated context size after compaction"
    );
}

// Windows CI only: bump to 4 workers to prevent SSE/event starvation and test timeouts.
#[cfg_attr(windows, tokio::test(flavor = "multi_thread", worker_threads = 4))]
#[cfg_attr(not(windows), tokio::test(flavor = "multi_thread", worker_threads = 2))]
async fn manual_compact_retries_after_context_window_error() {
    skip_if_no_network!();

    let server = start_mock_server().await;

    let user_turn = sse(vec![
        ev_assistant_message("m1", FIRST_REPLY),
        ev_completed("r1"),
    ]);
    let compact_failed = sse_failed(
        "resp-fail",
        "context_length_exceeded",
        CONTEXT_LIMIT_MESSAGE,
    );
    let compact_succeeds = sse(vec![
        ev_assistant_message("m2", SUMMARY_TEXT),
        ev_completed("r2"),
    ]);

    let request_log = mount_sse_sequence(
        &server,
        vec![
            user_turn.clone(),
            compact_failed.clone(),
            compact_succeeds.clone(),
        ],
    )
    .await;

    let model_provider = non_openai_model_provider(&server);

    let home = TempDir::new().unwrap();
    let mut config = load_default_config_for_test(&home).await;
    config.model_provider = model_provider;
    set_test_compact_prompt(&mut config);
    let codex = ConversationManager::with_models_provider(
        CodexAuth::from_api_key("dummy"),
        config.model_provider.clone(),
    )
    .new_conversation(config)
    .await
    .unwrap()
    .conversation;

    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "first turn".into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    codex.submit(Op::Compact).await.unwrap();
    let EventMsg::BackgroundEvent(event) =
        wait_for_event(&codex, |ev| matches!(ev, EventMsg::BackgroundEvent(_))).await
    else {
        panic!("expected background event after compact retry");
    };
    assert!(
        event.message.contains("Trimmed 1 older conversation item"),
        "background event should mention trimmed item count: {}",
        event.message
    );
    let warning_event = wait_for_event(&codex, |ev| matches!(ev, EventMsg::Warning(_))).await;
    let EventMsg::Warning(WarningEvent { message }) = warning_event else {
        panic!("expected warning event after compact retry");
    };
    assert_eq!(message, COMPACT_WARNING_MESSAGE);
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    let requests = request_log.requests();
    assert_eq!(
        requests.len(),
        3,
        "expected user turn and two compact attempts"
    );

    let compact_attempt = requests[1].body_json();
    let retry_attempt = requests[2].body_json();

    let compact_input = compact_attempt["input"]
        .as_array()
        .unwrap_or_else(|| panic!("compact attempt missing input array: {compact_attempt}"));
    let retry_input = retry_attempt["input"]
        .as_array()
        .unwrap_or_else(|| panic!("retry attempt missing input array: {retry_attempt}"));
    let compact_contains_prompt =
        body_contains_text(&compact_attempt.to_string(), SUMMARIZATION_PROMPT);
    let retry_contains_prompt =
        body_contains_text(&retry_attempt.to_string(), SUMMARIZATION_PROMPT);
    assert_eq!(
        compact_contains_prompt, retry_contains_prompt,
        "compact attempts should consistently include or omit the summarization prompt"
    );
    assert_eq!(
        retry_input.len(),
        compact_input.len().saturating_sub(1),
        "retry should drop exactly one history item (before {} vs after {})",
        compact_input.len(),
        retry_input.len()
    );
    if let (Some(first_before), Some(first_after)) = (compact_input.first(), retry_input.first()) {
        assert_ne!(
            first_before, first_after,
            "retry should drop the oldest conversation item"
        );
    } else {
        panic!("expected non-empty compact inputs");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manual_compact_twice_preserves_latest_user_messages() {
    skip_if_no_network!();

    let first_user_message = "first manual turn";
    let second_user_message = "second manual turn";
    let final_user_message = "post compact follow-up";
    let first_summary = "FIRST_MANUAL_SUMMARY";
    let second_summary = "SECOND_MANUAL_SUMMARY";
    let expected_second_summary = summary_with_prefix(second_summary);

    let server = start_mock_server().await;

    let first_turn = sse(vec![
        ev_assistant_message("m1", FIRST_REPLY),
        ev_completed("r1"),
    ]);
    let first_compact_summary = auto_summary(first_summary);
    let first_compact = sse(vec![
        ev_assistant_message("m2", &first_compact_summary),
        ev_completed("r2"),
    ]);
    let second_turn = sse(vec![
        ev_assistant_message("m3", SECOND_LARGE_REPLY),
        ev_completed("r3"),
    ]);
    let second_compact_summary = auto_summary(second_summary);
    let second_compact = sse(vec![
        ev_assistant_message("m4", &second_compact_summary),
        ev_completed("r4"),
    ]);
    let final_turn = sse(vec![
        ev_assistant_message("m5", FINAL_REPLY),
        ev_completed("r5"),
    ]);

    let responses_mock = mount_sse_sequence(
        &server,
        vec![
            first_turn,
            first_compact,
            second_turn,
            second_compact,
            final_turn,
        ],
    )
    .await;

    let model_provider = non_openai_model_provider(&server);

    let home = TempDir::new().unwrap();
    let mut config = load_default_config_for_test(&home).await;
    config.model_provider = model_provider;
    set_test_compact_prompt(&mut config);
    let codex = ConversationManager::with_models_provider(
        CodexAuth::from_api_key("dummy"),
        config.model_provider.clone(),
    )
    .new_conversation(config)
    .await
    .unwrap()
    .conversation;

    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: first_user_message.into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    codex.submit(Op::Compact).await.unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: second_user_message.into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    codex.submit(Op::Compact).await.unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: final_user_message.into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    let requests = responses_mock.requests();
    assert_eq!(
        requests.len(),
        5,
        "expected exactly 5 requests (user turn, compact, user turn, compact, final turn)"
    );
    let contains_user_text = |input: &[serde_json::Value], expected: &str| -> bool {
        input.iter().any(|item| {
            item.get("type").and_then(|v| v.as_str()) == Some("message")
                && item.get("role").and_then(|v| v.as_str()) == Some("user")
                && item
                    .get("content")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter().any(|entry| {
                            entry.get("text").and_then(|v| v.as_str()) == Some(expected)
                        })
                    })
                    .unwrap_or(false)
        })
    };

    let first_turn_input = requests[0].input();
    assert!(
        contains_user_text(&first_turn_input, first_user_message),
        "first turn request missing first user message"
    );
    assert!(
        !contains_user_text(&first_turn_input, SUMMARIZATION_PROMPT),
        "first turn request should not include summarization prompt"
    );

    let first_compact_input = requests[1].input();
    assert!(
        contains_user_text(&first_compact_input, first_user_message),
        "first compact request should include history before compaction"
    );

    let second_turn_input = requests[2].input();
    assert!(
        contains_user_text(&second_turn_input, second_user_message),
        "second turn request missing second user message"
    );
    assert!(
        contains_user_text(&second_turn_input, first_user_message),
        "second turn request should include the compacted user history"
    );

    let second_compact_input = requests[3].input();
    assert!(
        contains_user_text(&second_compact_input, second_user_message),
        "second compact request should include latest history"
    );

    let first_compact_has_prompt = contains_user_text(&first_compact_input, SUMMARIZATION_PROMPT);
    let second_compact_has_prompt = contains_user_text(&second_compact_input, SUMMARIZATION_PROMPT);
    assert_eq!(
        first_compact_has_prompt, second_compact_has_prompt,
        "compact requests should consistently include or omit the summarization prompt"
    );

    let mut final_output = requests
        .last()
        .unwrap_or_else(|| panic!("final turn request missing for {final_user_message}"))
        .input()
        .into_iter()
        .collect::<VecDeque<_>>();

    // System prompt
    final_output.pop_front();
    // Developer instructions
    final_output.pop_front();

    let _ = final_output
        .iter_mut()
        .map(drop_call_id)
        .collect::<Vec<_>>();

    let expected = vec![
        json!({
            "content": vec![json!({
                "text": first_user_message,
                "type": "input_text",
            })],
            "role": "user",
            "type": "message",
        }),
        json!({
            "content": vec![json!({
                "text": second_user_message,
                "type": "input_text",
            })],
            "role": "user",
            "type": "message",
        }),
        json!({
            "content": vec![json!({
                "text": expected_second_summary,
                "type": "input_text",
            })],
            "role": "user",
            "type": "message",
        }),
        json!({
            "content": vec![json!({
                "text": final_user_message,
                "type": "input_text",
            })],
            "role": "user",
            "type": "message",
        }),
    ];
    assert_eq!(final_output, expected);
}
