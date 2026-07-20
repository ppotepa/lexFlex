use serde_json::Value;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn file(name: &str, contents: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("lexflex-learning-{name}-{stamp}"));
    fs::write(&path, contents).expect("write");
    path
}

#[test]
fn learning_cli_requires_explicit_approval_before_promotion() {
    let observation = file(
        "observation",
        r#"{"source_id":"obs-1","surface":"Paris","context":null}"#,
    );
    let proposal = file(
        "proposal",
        r#"{"proposal_id":"p-1","observation_id":"obs-1","summary":"candidate"}"#,
    );
    let observed = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .args(["learning-observe", observation.to_str().expect("path")])
        .output()
        .expect("launch");
    assert!(observed.status.success());
    let observed: Value = serde_json::from_slice(&observed.stdout).expect("observation JSON");
    assert_eq!(observed["source_id"], "obs-1");
    let output = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .args([
            "learning-propose",
            observation.to_str().expect("path"),
            proposal.to_str().expect("path"),
            "--expected-behavior",
            "resolve to PARIS",
        ])
        .output()
        .expect("launch");
    assert!(output.status.success());
    let overlay: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(overlay["proposals"][0]["status"], "Pending");
    let overlay_path = file("overlay", "");
    fs::write(
        &overlay_path,
        serde_json::to_vec(&overlay).expect("serialize"),
    )
    .expect("write");
    let pending = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .args([
            "learning-promote",
            overlay_path.to_str().expect("path"),
            "p-1",
        ])
        .output()
        .expect("launch");
    assert_eq!(pending.status.code(), Some(4));

    let approved = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .args([
            "learning-approve",
            overlay_path.to_str().expect("path"),
            "p-1",
        ])
        .output()
        .expect("launch");
    assert!(approved.status.success());
    fs::write(&overlay_path, &approved.stdout).expect("persist artifact");

    let promoted = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .args([
            "learning-promote",
            overlay_path.to_str().expect("path"),
            "p-1",
        ])
        .output()
        .expect("launch");
    assert!(promoted.status.success());
    let promoted: Value = serde_json::from_slice(&promoted.stdout).expect("json");
    assert_eq!(promoted["proposal_id"], "p-1");
    let _ = fs::remove_file(observation);
    let _ = fs::remove_file(proposal);
    let _ = fs::remove_file(overlay_path);
}
