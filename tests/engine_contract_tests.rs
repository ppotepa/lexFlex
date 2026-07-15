use lexflex::engine::{
    source_snapshot_from_text, ConversationEngine, EngineRequest, EngineResponse, LanguageMode,
    SourceProvider, SourceRequest, SourceFetchPolicy, SourceKind,
};

#[derive(Clone)]
struct TestSource { request: SourceRequest, text: String }

impl SourceProvider for TestSource {
    fn resolve(&self, request: &SourceRequest) -> Result<lexflex::engine::SourceSnapshot, lexflex::engine::EngineError> {
        if request != &self.request { return Err(lexflex::engine::EngineError::SourceUnavailable("unexpected source".into())); }
        Ok(source_snapshot_from_text(request, self.text.clone()))
    }
}

fn engine() -> ConversationEngine {
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    ConversationEngine::new("data", "contract-test", true).unwrap()
        .with_provider(Box::new(TestSource { request, text: "Paris is the capital of France.".into() }))
}

#[test]
fn engine_returns_typed_deterministic_responses() {
    let mut first = engine();
    let mut second = engine();
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let a = first.handle(EngineRequest::IngestSource { source: request.clone() });
    let b = second.handle(EngineRequest::IngestSource { source: request });
    assert_eq!(a, b);
    assert!(matches!(a, EngineResponse::Ingest(_)));
}

#[test]
fn engine_answers_are_deterministic_across_fresh_instances() {
    let mut first = engine();
    let mut second = engine();
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let _ = first.handle(EngineRequest::IngestSource { source: request.clone() });
    let _ = second.handle(EngineRequest::IngestSource { source: request });
    let a = first.handle(EngineRequest::UserTurn { text: "What is the capital of France?".into(), language: LanguageMode::Explicit("en".into()) });
    let b = second.handle(EngineRequest::UserTurn { text: "What is the capital of France?".into(), language: LanguageMode::Explicit("en".into()) });
    assert_eq!(a, b);
}

#[test]
fn user_turn_without_evidence_is_explicitly_unknown() {
    let mut engine = engine();
    let response = engine.handle(EngineRequest::UserTurn { text: "What is the capital of France?".into(), language: LanguageMode::Auto });
    assert!(matches!(response, EngineResponse::Conversation(ref value) if value.meta.status == lexflex::engine::EngineStatus::Unknown));
    match response {
        EngineResponse::Conversation(value) => {
            assert!(value.meta.diagnostics.iter().any(|diagnostic| diagnostic == "no_active_session_bundle"));
            assert!(value.meta.diagnostics.iter().any(|diagnostic| diagnostic == "no_ingested_evidence"));
            assert!(value.text.as_deref().is_some_and(|text| text.contains("Use /ingest")));
        }
        other => panic!("unexpected engine response: {other:?}"),
    }
}

#[test]
fn clear_session_changes_immutable_snapshot() {
    let mut engine = engine();
    let before = engine.session.snapshot_id.clone();
    let response = engine.handle(EngineRequest::ClearSession);
    assert!(matches!(response, EngineResponse::Conversation(_)));
    assert_ne!(before, engine.session.snapshot_id);
}

#[test]
fn session_store_round_trips_and_rejects_tampering() {
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let mut engine = ConversationEngine::new("data", "contract-store", true).unwrap()
        .with_provider(Box::new(TestSource { request: request.clone(), text: "Paris is the capital of France.".into() }));
    assert!(matches!(engine.handle(EngineRequest::IngestSource { source: request }), EngineResponse::Ingest(_)));
    let path = engine.save_session().unwrap();
    let mut restored = engine.session.clone();
    restored.snapshot_sha256 = "tampered".into();
    assert!(restored.validate().is_err());
    let store = lexflex::engine::SessionStore::new("data");
    let loaded = store.load("contract-store").unwrap();
    assert_eq!(loaded.snapshot_id, engine.session.snapshot_id);
    assert!(std::path::Path::new("data/sessions/contract-store/snapshots").join(format!("{}.json", engine.session.snapshot_id)).exists());
    assert!(std::fs::read_dir("data/sessions/contract-store/sources").unwrap().next().is_some());
    assert!(std::fs::read_dir("data/sessions/contract-store/bundles").unwrap().next().is_some());
    let source_artifact = std::fs::read_dir("data/sessions/contract-store/sources").unwrap().next().unwrap().unwrap().path();
    std::fs::write(&source_artifact, b"{}").unwrap();
    assert!(store.load("contract-store").is_err());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir_all("data/sessions/contract-store").unwrap();
}

#[test]
fn wikipedia_snapshot_is_local_and_offline_first() {
    let mut engine = ConversationEngine::new("data", "paris-corpus", true).unwrap();
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let response = engine.handle(EngineRequest::IngestSource { source: request });
    assert!(matches!(response, EngineResponse::Ingest(_)));
    assert_eq!(engine.session.sources.len(), 1);
    assert!(engine.session.validate().is_ok());
}

#[test]
fn wikipedia_snapshot_auto_language_resolves_local_cache() {
    let mut engine = ConversationEngine::new("data", "paris-auto", true).unwrap();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("auto"), policy: SourceFetchPolicy::SnapshotOnly };
    let response = engine.handle(EngineRequest::IngestSource { source });
    assert!(matches!(response, EngineResponse::Ingest(_)));
    assert_eq!(engine.session.sources.len(), 1);
    let _ = std::fs::remove_dir_all("data/sessions/paris-auto");
}

#[test]
fn controlled_paris_snapshot_answers_with_evidence() {
    let mut engine = ConversationEngine::new("data", "paris-evidence", true).unwrap();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    assert!(matches!(engine.handle(EngineRequest::IngestSource { source }), EngineResponse::Ingest(_)));
    let response = engine.handle(EngineRequest::UserTurn { text: "What is the capital of France?".into(), language: LanguageMode::Explicit("en".into()) });
    match response {
        EngineResponse::Conversation(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Ok);
            let answer = value.answer.expect("question must produce an answer");
            assert!(!answer.rows.is_empty());
            assert!(answer.rows.iter().all(|row| !row.evidence.is_empty()));
            assert!(answer.text.as_deref().is_some_and(|text| text.to_lowercase().contains("paris")));
        }
        other => panic!("unexpected engine response: {other:?}"),
    }
    let _ = std::fs::remove_dir_all("data/sessions/paris-evidence");
}

#[test]
fn controlled_paris_definition_question_produces_evidence_backed_answer() {
    let mut engine = ConversationEngine::new("data", "paris-definition", true).unwrap();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    assert!(matches!(engine.handle(EngineRequest::IngestSource { source }), EngineResponse::Ingest(_)));
    let response = engine.handle(EngineRequest::UserTurn { text: "What is Paris?".into(), language: LanguageMode::Auto });
    match response {
        EngineResponse::Conversation(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Ok);
            let answer = value.answer.expect("question must produce an answer");
            assert!(!answer.evidence.is_empty());
            assert!(!answer.rows.is_empty());
            assert_ne!(answer.status, lexflex::query::AnswerStatus::Unknown);
        }
        other => panic!("unexpected engine response: {other:?}"),
    }
    let _ = std::fs::remove_dir_all("data/sessions/paris-definition");
}

#[test]
fn ordinary_user_turn_automatically_ingests_matching_source() {
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let mut engine = ConversationEngine::new("data", "auto-user-turn", true).unwrap()
        .with_provider(Box::new(TestSource { request, text: "Paris is the capital of France.".into() }));
    let response = engine.handle(EngineRequest::UserTurn { text: "What is Paris?".into(), language: LanguageMode::Auto });
    match response {
        EngineResponse::Conversation(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Ok);
            assert_eq!(value.meta.run_id, "run:00000001");
            assert_eq!(engine.session.sources.len(), 1);
            assert!(value.answer.as_ref().is_some_and(|answer| !answer.evidence.is_empty()));
        }
        other => panic!("unexpected response: {other:?}"),
    }
    let repeated = engine.handle(EngineRequest::UserTurn { text: "What is Paris?".into(), language: LanguageMode::Auto });
    let run_id = match repeated {
        EngineResponse::Conversation(value) => value.meta.run_id,
        other => panic!("unexpected response: {other:?}"),
    };
    assert_eq!(run_id, "run:00000002");
    assert!(engine.store.as_ref().unwrap().load_trace("auto-user-turn", "run:00000001").is_ok());
    assert!(engine.store.as_ref().unwrap().load_trace("auto-user-turn", "run:00000002").is_ok());
    let _ = std::fs::remove_dir_all("data/sessions/auto-user-turn");
}

#[test]
fn explicit_parse_failure_is_reported_as_unknown_conversation() {
    let mut engine = ConversationEngine::new("data", "explicit-parse-failure", true).unwrap();
    let response = engine.handle(EngineRequest::UserTurn {
        text: "Paris?".into(),
        language: LanguageMode::Explicit("en".into()),
    });
    match response {
        EngineResponse::Conversation(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Unknown);
            assert!(value.meta.diagnostics.iter().any(|diagnostic| diagnostic == "language_parse_failed"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
    let _ = std::fs::remove_dir_all("data/sessions/explicit-parse-failure");
}

#[test]
fn trace_sink_is_deterministic_and_persistable() {
    let collector = std::sync::Arc::new(lexflex::engine::TraceCollector::default());
    let mut engine = ConversationEngine::new("data", "trace-contract", true).unwrap().with_trace_sink(Box::new(collector.clone()));
    let _ = engine.handle(EngineRequest::ClearSession);
    let events = collector.events();
    assert!(events.iter().any(|event| event.stage == "request"));
    assert!(events.iter().any(|event| event.payload.is_some()));
    let current_run = events.last().expect("trace event").run_id.clone();
    let path = engine.store.as_ref().unwrap().save_trace("trace-contract", "turn-1", &collector.to_jsonl_for(&current_run)).unwrap();
    assert!(path.ends_with("turn-1.jsonl"));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir_all("data/sessions/trace-contract").unwrap();
}
