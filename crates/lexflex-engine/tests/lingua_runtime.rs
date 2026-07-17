use lexflex_engine::{
    api::{input::TextInput, request::EngineRequest, response::EngineResponse},
    runtime::LexFlexRuntime,
};
use lexflex_lingua::solve::EvidencePolicy;
use lexflex_lingua::{LinguaExpression, LinguaGoal, LinguaProgram, ProgramId};
use lexflex_model::{
    ConceptId, EntityId, Evidence, LanguageId, SemanticExpression, SemanticType, VariableId,
};
use lexflex_parser::ParseError;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_runtime() -> LexFlexRuntime {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let state_root = std::env::temp_dir().join(format!("lexflex-engine-{stamp}"));
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let model_root = repo_root.join("data/model");
    let language_root = repo_root.join("data/languages");
    LexFlexRuntime::with_session_and_roots(
        format!("test-{stamp}"),
        state_root,
        model_root,
        language_root,
    )
    .expect("runtime")
}

#[test]
fn evaluate_lingua_request_uses_new_runtime() {
    let mut runtime = test_runtime();
    let response = runtime.handle(EngineRequest::EvaluateLingua {
        program: LinguaProgram {
            id: ProgramId::new_unchecked("test:runtime-wrapper"),
            declarations: Vec::new(),
            entry: LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
        },
        policy: lexflex_lingua::ExecutionPolicy {
            expansion: lexflex_lingua::ExpansionMode::PreserveApplications,
        },
        include_trace: false,
    });

    match response {
        EngineResponse::LinguaEvaluated { result } => {
            assert_eq!(
                result.value,
                SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))
            );
            assert!(result.trace.events.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn text_analysis_uses_parser() {
    let mut runtime = test_runtime();
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input: TextInput {
            source_id: "test:text".into(),
            language: LanguageId::new("en").expect("valid language"),
            text: "Paris is the capital of France.".into(),
        },
        include_derivation: true,
    });
    match response {
        EngineResponse::TextAnalyzed { analysis, .. } => {
            assert!(analysis.derivation.is_some());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn ambiguous_text_analysis_carries_derivation() {
    let mut runtime = test_runtime();
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input: TextInput {
            source_id: "test:ambiguous".into(),
            language: LanguageId::new("en").expect("valid language"),
            text: "Paris is the capital of France.".into(),
        },
        include_derivation: true,
    });

    match response {
        EngineResponse::TextAnalyzed { analysis, .. } => {
            assert!(analysis.derivation.is_some());
        }
        EngineResponse::TextAmbiguous { alternatives, .. } => {
            assert!(alternatives
                .iter()
                .all(|alternative| alternative.derivation.is_some()));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn ingest_english_and_ask_polish_returns_paris() {
    let mut runtime = test_runtime();
    let ingest = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:en:capital".into(),
            language: LanguageId::new("en").expect("valid language"),
            text: "Paris is the capital of France.".into(),
        },
    });
    assert!(matches!(ingest, EngineResponse::TextIngested { .. }));

    let answer = runtime.handle(EngineRequest::AskText {
        input: TextInput {
            source_id: "source:pl:question".into(),
            language: LanguageId::new("pl").expect("valid language"),
            text: "Jaka jest stolica Francji?".into(),
        },
        evidence_policy: EvidencePolicy::Required,
        limit: Some(10),
    });

    match answer {
        EngineResponse::TextAnswer {
            goal, solutions, ..
        } => {
            assert_eq!(solutions.len(), 1);
            let answer_variable = goal.projection.first().expect("projection");
            assert_eq!(
                goal.variables.get(answer_variable),
                Some(&SemanticType::EntityOf(ConceptId::new_unchecked("CITY")))
            );
            assert_eq!(
                solutions[0].substitution.get(answer_variable),
                Some(&SemanticExpression::Entity(EntityId::new_unchecked(
                    "PARIS"
                )))
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn ingest_polish_and_ask_english_returns_paris() {
    let mut runtime = test_runtime();
    let ingest = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:pl:capital".into(),
            language: LanguageId::new("pl").expect("valid language"),
            text: "Paryż jest stolicą Francji.".into(),
        },
    });
    match ingest {
        EngineResponse::TextIngested { .. } => {}
        other => panic!("unexpected response: {other:?}"),
    }

    let answer = runtime.handle(EngineRequest::AskText {
        input: TextInput {
            source_id: "source:en:question".into(),
            language: LanguageId::new("en").expect("valid language"),
            text: "What is the capital of France?".into(),
        },
        evidence_policy: EvidencePolicy::Required,
        limit: Some(10),
    });

    match answer {
        EngineResponse::TextAnswer {
            goal, solutions, ..
        } => {
            assert_eq!(solutions.len(), 1);
            let answer_variable = goal.projection.first().expect("projection");
            assert_eq!(
                goal.variables.get(answer_variable),
                Some(&SemanticType::EntityOf(ConceptId::new_unchecked("CITY")))
            );
            assert_eq!(
                solutions[0].substitution.get(answer_variable),
                Some(&SemanticExpression::Entity(EntityId::new_unchecked(
                    "PARIS"
                )))
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn declarative_query_words_are_rejected() {
    let mut runtime = test_runtime();
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input: TextInput {
            source_id: "test:declarative-query-word".into(),
            language: LanguageId::new("en").expect("valid language"),
            text: "What is the capital of France.".into(),
        },
        include_derivation: false,
    });

    match response {
        EngineResponse::TextNotParsed { diagnostics } => {
            assert!(diagnostics
                .iter()
                .any(|diagnostic| matches!(diagnostic, ParseError::UnexpectedQueryVariable)));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn questions_without_projection_are_rejected() {
    let mut runtime = test_runtime();
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input: TextInput {
            source_id: "test:question-without-projection".into(),
            language: LanguageId::new("en").expect("valid language"),
            text: "Paris is the capital of France?".into(),
        },
        include_derivation: false,
    });

    match response {
        EngineResponse::TextNotParsed { diagnostics } => {
            assert!(diagnostics
                .iter()
                .any(|diagnostic| matches!(diagnostic, ParseError::QuestionWithoutProjection)));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn ingest_query_and_session_round_trip() {
    let mut runtime = test_runtime();
    let ingest = runtime.handle(EngineRequest::IngestLingua {
        program: LinguaProgram {
            id: ProgramId::new_unchecked("test:ingest"),
            declarations: Vec::new(),
            entry: LinguaExpression::Satisfies {
                subject: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
                concept: Box::new(LinguaExpression::ApplyConcept {
                    concept: ConceptId::new_unchecked("CAPITAL"),
                    bindings: std::collections::BTreeMap::from([(
                        lexflex_model::ParameterId::new_unchecked("scope"),
                        LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
                    )]),
                }),
            },
        },
        evidence: vec![Evidence::create("source:test", None, None).expect("valid evidence")],
    });
    assert!(matches!(
        ingest,
        EngineResponse::LinguaIngested {
            outcome: lexflex_engine::AssertionWriteOutcome::Inserted { .. },
            ..
        }
    ));

    let city = VariableId::new_unchecked("city");
    let query = runtime.handle(EngineRequest::QueryLingua {
        goal: LinguaGoal {
            expression: SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(city.clone())),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("CAPITAL"),
                    bindings: std::collections::BTreeMap::from([(
                        lexflex_model::ParameterId::new_unchecked("scope"),
                        SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                    )]),
                }),
            },
            variables: std::collections::BTreeMap::from([(
                city.clone(),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![city.clone()],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: None,
        },
    });

    match query {
        EngineResponse::LinguaQueryResult { solutions, .. } => {
            assert_eq!(solutions.len(), 1);
            assert_eq!(
                solutions[0].substitution.get(&city),
                Some(&SemanticExpression::Entity(EntityId::new_unchecked(
                    "PARIS"
                )))
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let inspect = runtime.handle(EngineRequest::InspectSession);
    match inspect {
        EngineResponse::SessionInspection {
            assertion_count,
            language_hash,
            ..
        } => {
            assert_eq!(assertion_count, 1);
            assert!(!language_hash.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let cleared = runtime.handle(EngineRequest::ClearSession);
    assert!(matches!(cleared, EngineResponse::SessionCleared { .. }));
}

#[test]
fn duplicate_assertions_merge_evidence() {
    let mut runtime = test_runtime();
    let program = LinguaProgram {
        id: ProgramId::new_unchecked("test:merge"),
        declarations: Vec::new(),
        entry: LinguaExpression::Equals {
            left: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
            right: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
        },
    };

    let first = runtime.handle(EngineRequest::IngestLingua {
        program: program.clone(),
        evidence: vec![Evidence::create("source:1", None, None).expect("valid evidence")],
    });
    assert!(matches!(
        first,
        EngineResponse::TextIngested {
            outcome: lexflex_engine::AssertionWriteOutcome::Inserted { .. },
            ..
        } | EngineResponse::LinguaIngested {
            outcome: lexflex_engine::AssertionWriteOutcome::Inserted { .. },
            ..
        }
    ));

    let second = runtime.handle(EngineRequest::IngestLingua {
        program,
        evidence: vec![Evidence::create("source:2", None, None).expect("valid evidence")],
    });

    match second {
        EngineResponse::LinguaIngested {
            outcome, assertion, ..
        } => {
            assert!(matches!(
                outcome,
                lexflex_engine::AssertionWriteOutcome::EvidenceMerged { .. }
            ));
            assert_eq!(assertion.evidence.len(), 2);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn corrupted_session_is_rejected_on_runtime_init() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let state_root = std::env::temp_dir().join(format!("lexflex-engine-corrupt-{stamp}"));
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let model_root = repo_root.join("data/model");
    let language_root = repo_root.join("data/languages");
    let session_id = format!("corrupt-{stamp}");
    fs::create_dir_all(&state_root).expect("create state root");
    fs::write(state_root.join(format!("{session_id}.json")), b"not-json")
        .expect("write corrupted session");

    match LexFlexRuntime::with_session_and_roots(session_id, state_root, model_root, language_root)
    {
        Err(lexflex_engine::runtime::RuntimeInitError::Store(_)) => {}
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("corrupted session must fail"),
    }
}
