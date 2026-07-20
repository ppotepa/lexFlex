mod command_support;

use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(prefix: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("lexflex-{prefix}-{stamp}.ron"))
}

#[test]
fn unknown_surface_exits_4_and_prints_json() {
    let output = command_support::run(&[
        "text-ask",
        "--language",
        "en",
        "--source-id",
        "source:test:unknown",
        "--text",
        "Nonsenseword",
    ]);

    assert_eq!(output.code, Some(4));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("text not parsed"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout JSON");
    assert!(
        json.get("TextNotParsed").is_some(),
        "unexpected JSON: {json}"
    );
}

#[test]
fn invalid_ron_input_exits_9() {
    let path = temp_path("invalid-ron");
    std::fs::write(&path, "not valid ron").expect("write invalid ron");
    let path_arg = path.to_str().expect("utf-8 path");

    let output = command_support::run(&["lingua-eval", path_arg]);

    assert_eq!(output.code, Some(9));
    assert!(output.stdout.is_empty());
    assert!(
        !output.stderr.is_empty(),
        "stderr should explain local input failure"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn empty_text_exits_4() {
    let output = command_support::run(&[
        "text-analyze",
        "--language",
        "en",
        "--source-id",
        "source:test:empty",
        "--text",
        "   ",
    ]);

    assert_eq!(output.code, Some(4));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("text input is empty"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn invalid_language_exits_4() {
    let output = command_support::run(&[
        "text-analyze",
        "--language",
        "invalid language",
        "--source-id",
        "source:test:language",
        "--text",
        "Paris is the capital of France.",
    ]);

    assert_eq!(output.code, Some(4));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid language"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn clap_usage_exits_2() {
    let output = command_support::run(&[]);
    assert_eq!(output.code, Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn valid_text_analysis_exits_0() {
    let output = command_support::run(&[
        "text-analyze",
        "--language",
        "en",
        "--source-id",
        "source:test:success",
        "--text",
        "Paris is the capital of France.",
    ]);
    assert_eq!(output.code, Some(0));
    assert!(!output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn oversized_text_input_exits_4_without_parsing() {
    let text = "x".repeat(8 * 1024 * 1024 + 1);
    let output = command_support::run_with_stdin(
        &[
            "text-analyze",
            "--language",
            "en",
            "--source-id",
            "source:test:oversized",
        ],
        &text,
    );
    assert_eq!(output.code, Some(4));
    assert!(String::from_utf8_lossy(&output.stderr).contains("byte limit"));
}

#[test]
fn oversized_conversation_expression_exits_4_before_json_parse() {
    let path = temp_path("oversized-expression");
    std::fs::write(&path, "{".repeat(8 * 1024 * 1024 + 1)).expect("write oversized expression");
    let path_arg = path.to_str().expect("utf-8 path");
    let output = command_support::run(&["conversation-turn", "turn-large", path_arg]);
    assert_eq!(output.code, Some(4));
    assert!(String::from_utf8_lossy(&output.stderr).contains("byte limit"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn oversized_generation_expression_exits_4_before_json_parse() {
    let path = temp_path("oversized-generation-expression");
    std::fs::write(&path, "{".repeat(8 * 1024 * 1024 + 1)).expect("write oversized expression");
    let path_arg = path.to_str().expect("utf-8 path");
    let output = command_support::run(&["text-generate", path_arg, "--language", "en"]);
    assert_eq!(output.code, Some(4));
    assert!(String::from_utf8_lossy(&output.stderr).contains("byte limit"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn oversized_lingua_program_exits_4_before_ron_parse() {
    let path = temp_path("oversized-lingua-program");
    std::fs::write(&path, "(".repeat(8 * 1024 * 1024 + 1)).expect("write oversized program");
    let path_arg = path.to_str().expect("utf-8 path");
    let output = command_support::run(&["lingua-eval", path_arg]);
    assert_eq!(output.code, Some(4));
    assert!(String::from_utf8_lossy(&output.stderr).contains("byte limit"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn session_migrate_is_explicit_and_reports_artifact() {
    let root = std::env::temp_dir().join(format!(
        "lexflex-migrate-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("create state root");
    std::fs::write(
        root.join("migration-test.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema": 2,
            "session_id": "migration-test",
            "payload": {"schema": 3}
        }))
        .expect("serialize"),
    )
    .expect("write");
    let state_dir = root.to_str().expect("utf-8 path");
    let output = command_support::run(&[
        "--session",
        "migration-test",
        "--state-dir",
        state_dir,
        "session-migrate",
        "--from-schema",
        "2",
    ]);
    assert_eq!(output.code, Some(0));
    let artifact: Value = serde_json::from_slice(&output.stdout).expect("artifact JSON");
    assert_eq!(
        artifact.get("kind").and_then(Value::as_str),
        Some("session")
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn semantic_generation_command_returns_json_text() {
    let path = temp_path("semantic-expression");
    std::fs::write(&path, r#"{"Entity":"PARIS"}"#).expect("write expression");
    let expression = path.to_str().expect("utf-8 path");
    let output =
        command_support::run(&["text-generate", expression, "--language", "en", "--trace"]);
    assert_eq!(output.code, Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("generation JSON");
    assert_eq!(json.get("text").and_then(Value::as_str), Some("Paris"));
    assert!(json.get("trace").is_some());
    let _ = std::fs::remove_file(path);
}
