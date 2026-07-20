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
        EngineResponse::TextAnswer {
            goal, solutions, ..
        } => {
            let variable = goal.projection.first().expect("projection");
            assert_eq!(
                solutions[0].substitution.get(variable),
                Some(&SemanticExpression::Entity(EntityId::new_unchecked("TOM")))
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn ingest_polish_and_ask_english_returns_tom() {
    let mut runtime = test_runtime();
    let ingest = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:event:pl:statement".into(),
            language: LanguageId::new("pl").expect("language"),
            text: "Tomek widzi Izę.".into(),
        },
    });
    assert!(matches!(ingest, EngineResponse::TextIngested { .. }));

    let answer = runtime.handle(EngineRequest::AskText {
        input: TextInput {
            source_id: "source:event:en:question".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Who sees Iza?".into(),
        },
        evidence_policy: EvidencePolicy::Required,
        limit: Some(10),
    });

    match answer {
        EngineResponse::TextAnswer {
            goal, solutions, ..
        } => {
            assert_eq!(solutions.len(), 1);
            let variable = goal.projection.first().expect("projection");
            assert_eq!(
                solutions[0].substitution.get(variable),
                Some(&SemanticExpression::Entity(EntityId::new_unchecked("TOM")))
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn multiple_event_answers_are_deterministic() {
    let mut runtime = test_runtime();
    for (source_id, text) in [
        ("source:event:tom", "Tom sees Iza."),
        ("source:event:iza", "Iza sees Iza."),
    ] {
        let ingest = runtime.handle(EngineRequest::IngestText {
            input: TextInput {
                source_id: source_id.into(),
                language: LanguageId::new("en").expect("language"),
                text: text.into(),
            },
        });
        assert!(matches!(ingest, EngineResponse::TextIngested { .. }));
    }

    let answer = runtime.handle(EngineRequest::AskText {
        input: TextInput {
            source_id: "source:event:multiple:question".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Who sees Iza?".into(),
        },
        evidence_policy: EvidencePolicy::Required,
        limit: Some(10),
    });

    match answer {
        EngineResponse::TextAnswer {
            goal, solutions, ..
        } => {
            assert_eq!(solutions.len(), 2);
            let variable = goal.projection.first().expect("projection");
            let values = solutions
                .iter()
                .map(|solution| solution.substitution.get(variable).cloned())
                .collect::<Vec<_>>();
            assert_eq!(
                values,
                vec![
                    Some(SemanticExpression::Entity(EntityId::new_unchecked("IZA"))),
                    Some(SemanticExpression::Entity(EntityId::new_unchecked("TOM"))),
                ]
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn wrong_polish_case_does_not_mutate_session() {
    let mut runtime = test_runtime();
    let before = match runtime.handle(EngineRequest::InspectSession) {
        EngineResponse::SessionInspection { snapshot_hash, .. } => snapshot_hash,
        other => panic!("unexpected response: {other:?}"),
    };

    let response = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:event:wrong-case".into(),
            language: LanguageId::new("pl").expect("language"),
            text: "Tomek widzi Iza.".into(),
        },
    });
    assert!(matches!(response, EngineResponse::TextNotParsed { .. }));

    let after = match runtime.handle(EngineRequest::InspectSession) {
        EngineResponse::SessionInspection { snapshot_hash, .. } => snapshot_hash,
        other => panic!("unexpected response: {other:?}"),
    };
    assert_eq!(before, after);
}

#[test]
fn answer_evidence_uses_statement_source_not_question_source() {
    let mut runtime = test_runtime();
    let statement_source = "source:event:evidence:statement";
    let question_source = "source:event:evidence:question";
    let ingest = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: statement_source.into(),
            language: LanguageId::new("en").expect("language"),
            text: "Tom sees Iza.".into(),
        },
    });
    assert!(matches!(ingest, EngineResponse::TextIngested { .. }));

    let answer = runtime.handle(EngineRequest::AskText {
        input: TextInput {
            source_id: question_source.into(),
            language: LanguageId::new("pl").expect("language"),
            text: "Kto widzi Izę?".into(),
        },
        evidence_policy: EvidencePolicy::Required,
        limit: Some(10),
    });

    match answer {
        EngineResponse::TextAnswer { solutions, .. } => {
            let source_ids = solutions[0]
                .evidence
                .values()
                .map(|evidence| evidence.source_id())
                .collect::<Vec<_>>();
            assert!(source_ids.contains(&statement_source));
            assert!(!source_ids.contains(&question_source));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
