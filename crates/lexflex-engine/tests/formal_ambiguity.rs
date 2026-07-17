use lexflex_engine::{
    api::{input::TextInput, request::EngineRequest, response::EngineResponse},
    runtime::LexFlexRuntime,
};
use lexflex_model::LanguageId;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_runtime() -> LexFlexRuntime {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let state_root = std::env::temp_dir().join(format!("lexflex-formal-ambiguity-{stamp}"));
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    LexFlexRuntime::with_session_and_roots(
        format!("test-{stamp}"),
        state_root,
        repo_root.join("data/model"),
        repo_root.join("data/languages"),
    )
    .expect("runtime")
}

#[test]
fn capital_analysis_is_not_reported_as_ambiguous() {
    let mut runtime = test_runtime();
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input: TextInput {
            source_id: "source:formal:ambiguity".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Paris is the capital of France.".into(),
        },
        include_derivation: true,
    });
    assert!(matches!(response, EngineResponse::TextAnalyzed { .. }));
}
