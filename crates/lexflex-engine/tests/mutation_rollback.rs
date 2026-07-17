use lexflex_engine::{
    api::{input::TextInput, request::EngineRequest, response::EngineResponse},
    runtime::LexFlexRuntime,
    EngineErrorCode,
};
use lexflex_model::LanguageId;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn blocked_store_runtime(stamp: u128) -> LexFlexRuntime {
    let state_root = std::env::temp_dir().join(format!("lexflex-engine-store-file-{stamp}"));
    fs::create_dir_all(&state_root).expect("create state dir");
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let model_root = repo_root.join("data/model");
    let language_root = repo_root.join("data/languages");
    let runtime = LexFlexRuntime::with_session_and_roots(
        format!("rollback-{stamp}"),
        &state_root,
        model_root,
        language_root,
    )
    .expect("runtime");
    let mut permissions = fs::metadata(&state_root).expect("metadata").permissions();
    permissions.set_mode(0o555);
    fs::set_permissions(&state_root, permissions).expect("set readonly");
    runtime
}

fn inspect(runtime: &mut LexFlexRuntime) -> (usize, usize, String) {
    match runtime.handle(EngineRequest::InspectSession) {
        EngineResponse::SessionInspection {
            assertion_count,
            evidence_count,
            snapshot_hash,
            ..
        } => (assertion_count, evidence_count, snapshot_hash),
        other => panic!("unexpected inspect response: {other:?}"),
    }
}

#[test]
fn store_error_during_ingest_restores_session_state() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let mut runtime = blocked_store_runtime(stamp);
    let before = inspect(&mut runtime);

    let response = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:rollback:ingest".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Paris is the capital of France.".into(),
        },
    });

    match response {
        EngineResponse::Error { code, .. } => {
            assert_eq!(code, EngineErrorCode::StoreError);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let after = inspect(&mut runtime);
    assert_eq!(before, after);
}

#[test]
fn store_error_during_clear_restores_session_state() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let mut runtime = blocked_store_runtime(stamp + 1);
    let before = inspect(&mut runtime);

    let response = runtime.handle(EngineRequest::ClearSession);

    match response {
        EngineResponse::Error { code, .. } => {
            assert_eq!(code, EngineErrorCode::StoreError);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let after = inspect(&mut runtime);
    assert_eq!(before, after);
}
