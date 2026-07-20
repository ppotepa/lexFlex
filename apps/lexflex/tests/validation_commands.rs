use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../")
}

fn temp_state_dir(prefix: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    std::env::temp_dir().join(format!("lexflex-{prefix}-{stamp}"))
}

fn temp_text_file(prefix: &str, contents: &str) -> std::path::PathBuf {
    let path = temp_state_dir(prefix).with_extension("txt");
    std::fs::write(&path, contents).expect("write temp text file");
    path
}

fn app_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lexflex-app"));
    command.current_dir(workspace_root());
    command
}

fn run_command(args: &[&str]) -> Value {
    run_command_with_stdin(args, None)
}

fn run_command_with_stdin(args: &[&str], stdin: Option<&str>) -> Value {
    if let Some(stdin) = stdin {
        let mut command = app_command();
        let mut child = command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("command should launch");
        child
            .stdin
            .as_mut()
            .expect("stdin should be piped")
            .write_all(stdin.as_bytes())
            .expect("write stdin");
        let output = child.wait_with_output().expect("wait for command");
        assert!(
            output.status.success(),
            "command failed: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    }
    let output = app_command()
        .args(args)
        .stdin(Stdio::piped())
        .output()
        .expect("command should launch");
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn run_stateful_command(state_dir: &std::path::Path, args: &[&str]) -> Value {
    let output = app_command()
        .args(["--session", "cli-stateful", "--state-dir"])
        .arg(state_dir)
        .args(args)
        .output()
        .expect("command should launch");
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

#[test]
fn model_validation_command_emits_json() {
    let json = run_command(&["model-validate"]);
    assert_eq!(json["package_id"], "lexflex:model:core");
    assert_eq!(json["validation_issues"], 0);
    assert_eq!(json["languages"], 2);
}

#[test]
fn language_validation_command_emits_json() {
    let json = run_command(&["language-validate"]);
    assert_eq!(json["languages"], 2);
    assert!(!json["registry_hash"]
        .as_str()
        .unwrap_or_default()
        .is_empty());
    assert!(!json["model_hash"].as_str().unwrap_or_default().is_empty());
}

#[test]
fn language_validation_command_accepts_explicit_language_root() {
    let json = run_command(&["language-validate", "data/languages"]);
    assert_eq!(json["languages"], 2);
}

#[test]
fn text_analyze_command_emits_json() {
    let json = run_command(&[
        "text-analyze",
        "--language",
        "en",
        "--text",
        "Paris is the capital of France.",
    ]);
    assert!(
        json.get("TextAnalyzed").is_some(),
        "unexpected JSON: {json}"
    );
}

#[test]
fn text_analyze_command_reads_stdin() {
    let json = run_command_with_stdin(
        &["text-analyze", "--language", "en"],
        Some("Paris is the capital of France."),
    );
    let analysis = json
        .get("TextAnalyzed")
        .and_then(|value| value.get("analysis"))
        .expect("analysis payload");
    assert!(
        analysis["source_id"]
            .as_str()
            .unwrap_or_default()
            .starts_with("stdin:"),
        "unexpected source id: {analysis}"
    );
}

#[test]
fn text_analyze_command_reads_file() {
    let file = temp_text_file("cli-file-analyze", "Paris is the capital of France.");
    let file_arg = file
        .to_str()
        .expect("temp path should be utf-8")
        .to_string();
    let json = run_command(&["text-analyze", "--language", "en", "--file", &file_arg]);
    let analysis = json
        .get("TextAnalyzed")
        .and_then(|value| value.get("analysis"))
        .expect("analysis payload");
    assert!(
        analysis["source_id"]
            .as_str()
            .unwrap_or_default()
            .starts_with("file:"),
        "unexpected source id: {analysis}"
    );
    let _ = std::fs::remove_file(file);
}

#[test]
fn text_translate_text_uses_source_analysis_and_target_generation() {
    let json = run_command(&[
        "text-translate-text",
        "--language",
        "en",
        "--target-language",
        "pl",
        "--text",
        "Paris is the capital of France.",
    ]);
    assert!(json["text"].as_str().is_some(), "unexpected JSON: {json}");
    assert!(!json["text"].as_str().unwrap_or_default().is_empty());
}

#[test]
fn text_translate_text_supports_polish_source() {
    let json = run_command(&[
        "text-translate-text",
        "--language",
        "pl",
        "--target-language",
        "en",
        "--text",
        "Paryż jest stolicą Francji.",
    ]);
    let text = json["text"].as_str().expect("translated text");
    assert!(!text.is_empty(), "translation must not be empty");
    assert!(text.contains("Paris"), "unexpected translation: {text}");
    assert!(text.contains("France"), "unexpected translation: {text}");
}

#[test]
fn text_ingest_and_ask_commands_emit_json() {
    let state_dir = temp_state_dir("cli-text");

    let ingest = app_command()
        .args([
            "--session",
            "cli-text",
            "--state-dir",
            state_dir.to_str().expect("temp path should be valid utf-8"),
            "text-ingest",
            "--language",
            "en",
            "--source-id",
            "source:test:en",
            "--text",
            "Paris is the capital of France.",
        ])
        .output()
        .expect("text ingest command should launch");
    assert!(
        ingest.status.success(),
        "{}",
        String::from_utf8_lossy(&ingest.stderr)
    );
    let ingest_json: Value =
        serde_json::from_slice(&ingest.stdout).expect("ingest stdout should be JSON");
    assert!(
        ingest_json.get("TextIngested").is_some(),
        "unexpected JSON: {ingest_json}"
    );

    let ask = app_command()
        .args([
            "--session",
            "cli-text",
            "--state-dir",
            state_dir.to_str().expect("temp path should be valid utf-8"),
            "text-ask",
            "--language",
            "en",
            "--source-id",
            "source:test:question",
            "--text",
            "What is the capital of France?",
        ])
        .output()
        .expect("text ask command should launch");
    assert!(
        ask.status.success(),
        "{}",
        String::from_utf8_lossy(&ask.stderr)
    );
    let ask_json: Value = serde_json::from_slice(&ask.stdout).expect("ask stdout should be JSON");
    assert!(
        ask_json.get("TextAnswer").is_some(),
        "unexpected JSON: {ask_json}"
    );

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn text_ingest_and_ask_commands_read_file_input() {
    let state_dir = temp_state_dir("cli-text-file");
    let ingest_file = temp_text_file(
        "cli-text-file-input-ingest",
        "Paris is the capital of France.",
    );
    let ingest_file_arg = ingest_file
        .to_str()
        .expect("temp path should be utf-8")
        .to_string();
    let question_file = temp_text_file(
        "cli-text-file-input-question",
        "What is the capital of France?",
    );
    let question_file_arg = question_file
        .to_str()
        .expect("temp path should be utf-8")
        .to_string();

    let ingest = app_command()
        .args([
            "--session",
            "cli-text-file",
            "--state-dir",
            state_dir.to_str().expect("temp path should be valid utf-8"),
            "text-ingest",
            "--language",
            "en",
            "--file",
            &ingest_file_arg,
        ])
        .output()
        .expect("text ingest command should launch");
    assert!(
        ingest.status.success(),
        "{}",
        String::from_utf8_lossy(&ingest.stderr)
    );
    let ingest_json: Value =
        serde_json::from_slice(&ingest.stdout).expect("ingest stdout should be JSON");
    let ingest_analysis = ingest_json
        .get("TextIngested")
        .and_then(|value| value.get("analysis"))
        .expect("ingest analysis payload");
    assert!(
        ingest_analysis["source_id"]
            .as_str()
            .unwrap_or_default()
            .starts_with("file:"),
        "unexpected source id: {ingest_analysis}"
    );

    let ask = app_command()
        .args([
            "--session",
            "cli-text-file",
            "--state-dir",
            state_dir.to_str().expect("temp path should be valid utf-8"),
            "text-ask",
            "--language",
            "en",
            "--file",
            &question_file_arg,
        ])
        .output()
        .expect("text ask command should launch");
    assert!(
        ask.status.success(),
        "{}",
        String::from_utf8_lossy(&ask.stderr)
    );
    let ask_json: Value = serde_json::from_slice(&ask.stdout).expect("ask stdout should be JSON");
    let ask_analysis = ask_json
        .get("TextAnswer")
        .and_then(|value| value.get("analysis"))
        .expect("ask analysis payload");
    assert!(
        ask_analysis["source_id"]
            .as_str()
            .unwrap_or_default()
            .starts_with("file:"),
        "unexpected source id: {ask_analysis}"
    );

    let _ = std::fs::remove_file(ingest_file);
    let _ = std::fs::remove_file(question_file);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn session_inspect_and_clear_commands_emit_json() {
    let state_dir = temp_state_dir("cli-session");

    let ingest = run_stateful_command(
        &state_dir,
        &[
            "text-ingest",
            "--language",
            "en",
            "--source-id",
            "source:test:en",
            "--text",
            "Paris is the capital of France.",
        ],
    );
    assert!(
        ingest.get("TextIngested").is_some(),
        "unexpected JSON: {ingest}"
    );

    let inspect_before = run_stateful_command(&state_dir, &["session-inspect"]);
    let session_before = inspect_before
        .get("SessionInspection")
        .expect("session inspect payload");
    assert_eq!(session_before["assertion_count"], 1);
    assert_eq!(session_before["evidence_count"], 1);

    let cleared = run_stateful_command(&state_dir, &["session-clear"]);
    assert!(
        cleared.get("SessionCleared").is_some(),
        "unexpected JSON: {cleared}"
    );

    let inspect_after = run_stateful_command(&state_dir, &["session-inspect"]);
    let session_after = inspect_after
        .get("SessionInspection")
        .expect("session inspect payload");
    assert_eq!(session_after["assertion_count"], 0);
    assert_eq!(session_after["evidence_count"], 0);

    let _ = std::fs::remove_dir_all(state_dir);
}
