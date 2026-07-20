use lexflex_engine::{
    api::{input::TextInput, request::EngineRequest},
    runtime::{LexFlexRuntime, RuntimeInitError},
};
use lexflex_model::LanguageId;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn roots(stamp: u128) -> (String, PathBuf, PathBuf, PathBuf) {
    let session_id = format!("integrity-{stamp}");
    let state_root = std::env::temp_dir().join(format!("lexflex-engine-integrity-{stamp}"));
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let model_root = repo_root.join("data/model");
    let language_root = repo_root.join("data/languages");
    (session_id, state_root, model_root, language_root)
}

fn seed_session(
    session_id: &str,
    state_root: &PathBuf,
    model_root: &PathBuf,
    language_root: &PathBuf,
) {
    let mut runtime = LexFlexRuntime::with_session_and_roots(
        session_id.to_string(),
        state_root,
        model_root,
        language_root,
    )
    .expect("runtime");

    let response = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:integrity:test".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Paris is the capital of France.".into(),
        },
    });

    assert!(matches!(
        response,
        lexflex_engine::api::response::EngineResponse::TextIngested { .. }
    ));
}

fn load_session_json(path: &PathBuf) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read session")).expect("parse session json")
}

#[test]
fn corrupted_snapshot_hash_is_rejected_without_repair() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let (session_id, state_root, model_root, language_root) = roots(stamp);
    seed_session(&session_id, &state_root, &model_root, &language_root);

    let session_path = state_root.join(format!("{session_id}.json"));
    let mut json = load_session_json(&session_path);
    json["payload"]["knowledge"]["snapshot_hash"] =
        Value::String("0000000000000000000000000000000000000000000000000000000000000000".into());
    fs::write(
        &session_path,
        serde_json::to_vec_pretty(&json).expect("serialize corrupted json"),
    )
    .expect("write corrupted session");

    let stored = fs::read_to_string(&session_path).expect("read back");
    let result = LexFlexRuntime::with_session_and_roots(
        &session_id,
        &state_root,
        &model_root,
        &language_root,
    );

    assert!(matches!(result, Err(RuntimeInitError::Integrity(_))));
    assert_eq!(
        fs::read_to_string(&session_path).expect("read after"),
        stored
    );
}

#[test]
fn corrupted_assertion_id_is_rejected_without_repair() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let (session_id, state_root, model_root, language_root) = roots(stamp + 1);
    seed_session(&session_id, &state_root, &model_root, &language_root);

    let session_path = state_root.join(format!("{session_id}.json"));
    let mut json = load_session_json(&session_path);
    let assertions = json["payload"]["knowledge"]["assertions"]
        .as_object_mut()
        .expect("assertion object");
    let first_key = assertions.keys().next().cloned().expect("one assertion");
    assertions[&first_key]["id"] = Value::String("assertion:broken".into());
    fs::write(
        &session_path,
        serde_json::to_vec_pretty(&json).expect("serialize corrupted json"),
    )
    .expect("write corrupted session");

    let stored = fs::read_to_string(&session_path).expect("read back");
    let result = LexFlexRuntime::with_session_and_roots(
        &session_id,
        &state_root,
        &model_root,
        &language_root,
    );

    assert!(matches!(result, Err(RuntimeInitError::Integrity(_))));
    assert_eq!(
        fs::read_to_string(&session_path).expect("read after"),
        stored
    );
}
