use lexflex_engine::{
    api::{input::TextInput, request::EngineRequest, response::EngineResponse},
    runtime::LexFlexRuntime,
};
use lexflex_lingua::solve::EvidencePolicy;
use lexflex_model::{EntityId, LanguageId, SemanticExpression};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_runtime() -> LexFlexRuntime {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let state_root = std::env::temp_dir().join(format!("lexflex-engine-event-{stamp}"));
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
fn ingest_english_and_ask_polish_returns_tom() {
    let mut runtime = test_runtime();
    let ingest = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:event:en".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Tom sees Iza.".into(),
        },
    });
    assert!(matches!(ingest, EngineResponse::TextIngested { .. }));

    let answer = runtime.handle(EngineRequest::AskText {
        input: TextInput {
            source_id: "source:event:pl".into(),
            language: LanguageId::new("pl").expect("language"),
            text: "Kto widzi Izę?".into(),
        },
        evidence_policy: EvidencePolicy::Required,
        limit: Some(10),
    });

    match answer {
        EngineResponse::TextAnswer { goal, solutions, .. } => {
            let variable = goal.projection.first().expect("projection");
            assert_eq!(
                solutions[0].substitution.get(variable),
                Some(&SemanticExpression::Entity(EntityId::new_unchecked("TOM")))
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
