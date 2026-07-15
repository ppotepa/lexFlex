use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_lexflex"))
}

#[test]
fn cli_auto_discovers_data_root_from_src_directory() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(binary())
        .current_dir(repo.join("src"))
        .args([
            "answer",
            "What is Paris?",
            "--lang",
            "en",
            "--session",
            "cli-nested-root",
            "--source-policy",
            "snapshot-only",
            "--format",
            "summary",
        ])
        .output()
        .expect("command should run");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status=Ok"), "stdout: {stdout}");
    let _ = std::fs::remove_dir_all(repo.join("data/sessions/cli-nested-root"));
}

#[test]
fn cli_rejects_explicit_invalid_relative_data_root() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(binary())
        .current_dir(repo.join("src"))
        .args([
            "answer",
            "What is Paris?",
            "--lang",
            "en",
            "--data",
            "data",
            "--session",
            "cli-invalid-root",
            "--source-policy",
            "snapshot-only",
            "--format",
            "summary",
        ])
        .output()
        .expect("command should run");
    assert!(!output.status.success(), "stdout: {}", String::from_utf8_lossy(&output.stdout));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Runtime data root error"), "stderr: {stderr}");
    assert!(stderr.contains("Missing required runtime asset"), "stderr: {stderr}");
}

#[test]
fn cli_engine_facts_reads_persisted_session_with_evidence() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let session = "cli-engine-facts";
    let ingest = Command::new(binary())
        .current_dir(&repo)
        .args(["ingest", "Paris", "--lang", "en", "--session", session, "--format", "summary"])
        .output()
        .expect("ingest should run");
    assert!(ingest.status.success(), "stderr: {}", String::from_utf8_lossy(&ingest.stderr));

    let output = Command::new(binary())
        .current_dir(&repo)
        .args(["engine", "facts", "Paris", "--session", session, "--limit", "1", "--format", "table"])
        .output()
        .expect("engine facts should run");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status=Ok"), "stdout: {stdout}");
    assert!(stdout.contains("Facts · Paris"), "stdout: {stdout}");
    assert!(stdout.contains("Source: Paris"), "stdout: {stdout}");
    assert!(stdout.contains("Sentence:"), "stdout: {stdout}");
    let _ = std::fs::remove_dir_all(repo.join(format!("data/sessions/{session}")));
}
