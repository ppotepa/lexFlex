use lexflex::engine::{
    source_snapshot_from_text, ClearTarget, ConversationRequest, DebugCommand, DebugFactRole,
    DebugInspectTarget, DebugSelectorOutcome, EngineRequest, EngineResponse, EvidenceDebugQuery,
    FactsDebugQuery, LanguageMode, LexFlexEngine, SessionRequest, TranslationRequest,
    SourceFetchPolicy, SourceKind, SourceProvider, SourceRequest,
};

fn translation_turn(text: &str, from: &str, to: &str) -> EngineRequest {
    EngineRequest::Translation(TranslationRequest::Turn {
        text: text.into(),
        from: Some(LanguageMode::Explicit(from.into())),
        to: Some(lexflex::core::interlingua::LanguageId::new(to)),
    })
}

fn facts_request(selector: &str, limit: Option<usize>) -> EngineRequest {
    EngineRequest::Debug {
        command: DebugCommand::Facts(FactsDebugQuery {
            selector: selector.into(),
            role: DebugFactRole::Any,
            limit,
            page: 1,
        }),
    }
}

fn evidence_request(claim_id: &str) -> EngineRequest {
    EngineRequest::Debug {
        command: DebugCommand::Evidence(EvidenceDebugQuery {
            claim_id: claim_id.into(),
        }),
    }
}

#[derive(Clone)]
struct TestSource { request: SourceRequest, text: String }

impl SourceProvider for TestSource {
    fn resolve(&self, request: &SourceRequest) -> Result<lexflex::engine::SourceSnapshot, lexflex::engine::EngineError> {
        if request != &self.request { return Err(lexflex::engine::EngineError::SourceUnavailable("unexpected source".into())); }
        Ok(source_snapshot_from_text(request, self.text.clone()))
    }
}

fn engine() -> LexFlexEngine {
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    LexFlexEngine::new("data", "contract-test", true).unwrap()
        .with_provider(Box::new(TestSource { request, text: "Paris is the capital of France.".into() }))
}

#[test]
fn engine_returns_typed_deterministic_responses() {
    let mut first = engine();
    let mut second = engine();
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let a = first.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source: request.clone() }));
    let b = second.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source: request }));
    assert_eq!(a, b);
    assert!(matches!(a, EngineResponse::Ingest(_)));
}

#[test]
fn debug_facts_are_typed_evidence_backed_and_deterministic() {
    let mut first = engine();
    let mut second = engine();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let _ = first.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source: source.clone() }));
    let _ = second.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source }));
    let request = facts_request("PARIS", None);
    let encoded = serde_json::to_string(&request).unwrap();
    assert_eq!(serde_json::from_str::<EngineRequest>(&encoded).unwrap(), request);
    let left = first.handle(request.clone());
    let right = second.handle(request);
    assert_eq!(left, right);
    match left {
        EngineResponse::Debug(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Ok);
            assert!(matches!(value.selector_outcome, DebugSelectorOutcome::Matched { .. }));
            assert!(!value.facts.is_empty());
            assert_eq!(value.total_facts, value.returned_facts);
            assert_eq!(value.presentation.title, "Facts · PARIS");
            assert_eq!(value.ui.title, "Facts · PARIS");
            assert!(!value.ui.sections.is_empty());
            assert!(!value.presentation.cards.is_empty());
            assert!(value.facts.iter().all(|fact| !fact.evidence.is_empty()));
            assert!(value.facts.iter().flat_map(|fact| &fact.evidence).all(|evidence| {
                !evidence.source_sha256.is_empty()
                    && !evidence.sentence_text.is_empty()
                    && !evidence.source_spans.is_empty()
                    && evidence.source_spans.iter().all(|span| span.start <= span.end)
            }));
        }
        other => panic!("unexpected debug response: {other:?}"),
    }
}

#[test]
fn debug_facts_do_not_mutate_or_substring_match() {
    let mut engine = engine();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let _ = engine.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source }));
    let snapshot = engine.session.snapshot_id.clone();
    let source_count = engine.session.sources.len();
    match engine.handle(facts_request("Par", Some(50))) {
        EngineResponse::Debug(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Unknown);
            assert_eq!(value.selector_outcome, DebugSelectorOutcome::NotFound);
            assert!(value.facts.is_empty());
        }
        other => panic!("unexpected debug response: {other:?}"),
    }
    assert_eq!(engine.session.snapshot_id, snapshot);
    assert_eq!(engine.session.sources.len(), source_count);
}

#[test]
fn debug_evidence_returns_humanized_presentation() {
    let mut engine = engine();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let _ = engine.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source }));
    let fact = match engine.handle(facts_request("Paris", Some(1))) {
        EngineResponse::Debug(value) => value.facts.into_iter().next().expect("fact"),
        other => panic!("unexpected facts response: {other:?}"),
    };
    match engine.handle(evidence_request(&fact.claim_id)) {
        EngineResponse::Debug(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Ok);
            assert_eq!(value.presentation.category, lexflex::engine::DebugPresentationCategory::Evidence);
            assert!(!value.presentation.summary.trim().is_empty());
            assert!(!value.ui.summary.trim().is_empty());
            assert_eq!(value.facts.len(), 1);
        }
        other => panic!("unexpected evidence response: {other:?}"),
    }
}

#[test]
fn translation_publishes_original_knowledge_and_clear_removes_it() {
    let mut engine = LexFlexEngine::new("data", "translation-knowledge", true).unwrap();
    let response = engine.handle(translation_turn("Paris is the capital of France.", "en", "pl"));
    match response {
        EngineResponse::Translation(value) => {
            assert!(value.knowledge_committed);
            assert!(value.turn_id.is_some());
            assert_eq!(value.source_language.0, "en");
            assert_eq!(value.target_language.0, "pl");
            assert_eq!(value.presentation.title, "Translation");
            assert!(!value.presentation.sections.is_empty());
        }
        other => panic!("unexpected translation response: {other:?}"),
    }
    assert_eq!(engine.session.translation.turn_order.len(), 1);
    assert_eq!(engine.session.conversation.sources.len(), 1);
    let trace = engine.store.as_ref().unwrap().load_trace("translation-knowledge", "run:00000001").unwrap();
    assert!(trace.contains("translation.knowledge.publish"));
    assert!(!trace.contains("source.discovery"));
    assert!(!trace.contains("source.resolve.started"));
    assert!(matches!(engine.handle(EngineRequest::Debug { command: DebugCommand::Inspect(DebugInspectTarget::Trace { run_id: None }) }), EngineResponse::Debug(value) if value.presentation.category == lexflex::engine::DebugPresentationCategory::Trace));
    assert!(matches!(engine.handle(facts_request("Paris", None)), EngineResponse::Debug(value) if !value.facts.is_empty()));
    let _ = engine.handle(EngineRequest::Session(SessionRequest::Clear { target: ClearTarget::Translation }));
    assert!(engine.session.translation.turns.is_empty());
    assert!(engine.session.conversation.sources.is_empty());
    assert!(matches!(engine.handle(facts_request("Paris", None)), EngineResponse::Debug(value) if value.selector_outcome == DebugSelectorOutcome::NotFound));
    let _ = std::fs::remove_dir_all("data/sessions/translation-knowledge");
}

#[test]
fn translation_workspace_persists_with_conversation_knowledge() {
    let session = "translation-persistence";
    let mut engine = LexFlexEngine::new("data", session, true).unwrap();
    let _ = engine.handle(translation_turn("Paris is the capital of France.", "en", "pl"));
    engine.save_session().unwrap();
    let expected = engine.session.snapshot_sha256.clone();
    let mut loaded = LexFlexEngine::new("data", session, true).unwrap();
    loaded.load_session().unwrap();
    assert_eq!(loaded.session.snapshot_sha256, expected);
    assert_eq!(loaded.session.translation.turns.len(), 1);
    assert_eq!(loaded.session.conversation.sources.len(), 1);
    assert!(matches!(loaded.handle(facts_request("Paris", None)), EngineResponse::Debug(value) if !value.facts.is_empty()));
    let _ = std::fs::remove_dir_all(format!("data/sessions/{session}"));
}

#[test]
fn unsupported_generation_still_commits_translation_knowledge() {
    let mut engine = LexFlexEngine::new("data", "translation-unsupported", true).unwrap();
    let response = engine.handle(translation_turn("Paris is the capital of France.", "en", "xx"));
    match response {
        EngineResponse::Translation(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Unsupported);
            assert!(value.text.is_none());
            assert!(value.knowledge_committed);
            assert_eq!(value.presentation.title, "Translation");
        }
        other => panic!("unexpected translation response: {other:?}"),
    }
    assert_eq!(engine.session.translation.turns.len(), 1);
    assert_eq!(engine.session.conversation.sources.len(), 1);
    let _ = std::fs::remove_dir_all("data/sessions/translation-unsupported");
}

#[test]
fn translation_direction_changes_share_one_multilingual_context() {
    let mut engine = LexFlexEngine::new("data", "translation-context", true).unwrap();
    let _ = engine.handle(translation_turn("Tomek has a cat.", "en", "pl"));
    let first_snapshot = engine.session.translation.snapshot_id.clone();
    let _ = engine.handle(translation_turn("Tomek ma psa.", "pl", "en"));
    assert_eq!(engine.session.translation.turn_order.len(), 2);
    assert_ne!(engine.session.translation.snapshot_id, first_snapshot);
    let languages = engine.session.translation.turn_order.iter()
        .filter_map(|id| engine.session.translation.turns.get(id))
        .map(|turn| turn.source_language.0.as_str())
        .collect::<Vec<_>>();
    assert_eq!(languages, vec!["en", "pl"]);
    let _ = std::fs::remove_dir_all("data/sessions/translation-context");
}

#[test]
fn engine_answers_are_deterministic_across_fresh_instances() {
    let mut first = engine();
    let mut second = engine();
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let _ = first.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source: request.clone() }));
    let _ = second.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source: request }));
    let a = first.handle(EngineRequest::Conversation(ConversationRequest::Turn { text: "What is the capital of France?".into(), language: LanguageMode::Explicit("en".into()) }));
    let b = second.handle(EngineRequest::Conversation(ConversationRequest::Turn { text: "What is the capital of France?".into(), language: LanguageMode::Explicit("en".into()) }));
    assert_eq!(a, b);
}

#[test]
fn user_turn_without_evidence_is_explicitly_unknown() {
    let mut engine = engine();
    let response = engine.handle(EngineRequest::Conversation(ConversationRequest::Turn { text: "What is the capital of France?".into(), language: LanguageMode::Auto }));
    assert!(matches!(response, EngineResponse::Conversation(ref value) if value.meta.status == lexflex::engine::EngineStatus::Unknown));
    match response {
        EngineResponse::Conversation(value) => {
            assert!(value.meta.diagnostics.iter().any(|diagnostic| diagnostic == "no_active_session_bundle"));
            assert!(value.meta.diagnostics.iter().any(|diagnostic| diagnostic == "no_ingested_evidence"));
            assert!(value.text.as_deref().is_some_and(|text| text.contains("Use /ingest")));
            assert_eq!(value.presentation.title, "Conversation");
        }
        other => panic!("unexpected engine response: {other:?}"),
    }
}

#[test]
fn clear_session_changes_immutable_snapshot() {
    let mut engine = engine();
    let before = engine.session.snapshot_id.clone();
    let response = engine.handle(EngineRequest::Session(SessionRequest::Clear { target: ClearTarget::All }));
    assert!(matches!(response, EngineResponse::Conversation(_)));
    assert_ne!(before, engine.session.snapshot_id);
}

#[test]
fn session_store_round_trips_and_rejects_tampering() {
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let mut engine = LexFlexEngine::new("data", "contract-store", true).unwrap()
        .with_provider(Box::new(TestSource { request: request.clone(), text: "Paris is the capital of France.".into() }));
    assert!(matches!(engine.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source: request })), EngineResponse::Ingest(_)));
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
    let mut engine = LexFlexEngine::new("data", "paris-corpus", true).unwrap();
    let request = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    let response = engine.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source: request }));
    assert!(matches!(response, EngineResponse::Ingest(_)));
    assert_eq!(engine.session.sources.len(), 1);
    assert!(engine.session.validate().is_ok());
}

#[test]
fn wikipedia_snapshot_auto_language_resolves_local_cache() {
    let mut engine = LexFlexEngine::new("data", "paris-auto", true).unwrap();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("auto"), policy: SourceFetchPolicy::SnapshotOnly };
    let response = engine.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source }));
    assert!(matches!(response, EngineResponse::Ingest(_)));
    assert_eq!(engine.session.sources.len(), 1);
    let _ = std::fs::remove_dir_all("data/sessions/paris-auto");
}

#[test]
fn controlled_paris_snapshot_answers_with_evidence() {
    let mut engine = LexFlexEngine::new("data", "paris-evidence", true).unwrap();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    assert!(matches!(engine.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source })), EngineResponse::Ingest(_)));
    let response = engine.handle(EngineRequest::Conversation(ConversationRequest::Turn { text: "What is the capital of France?".into(), language: LanguageMode::Explicit("en".into()) }));
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
    let mut engine = LexFlexEngine::new("data", "paris-definition", true).unwrap();
    let source = SourceRequest { source_kind: SourceKind::Wikipedia, title: "Paris".into(), language: lexflex::core::interlingua::LanguageId::new("en"), policy: SourceFetchPolicy::SnapshotOnly };
    assert!(matches!(engine.handle(EngineRequest::Conversation(ConversationRequest::IngestSource { source })), EngineResponse::Ingest(_)));
    let response = engine.handle(EngineRequest::Conversation(ConversationRequest::Turn { text: "What is Paris?".into(), language: LanguageMode::Auto }));
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
    let mut engine = LexFlexEngine::new("data", "auto-user-turn", true).unwrap()
        .with_provider(Box::new(TestSource { request, text: "Paris is the capital of France.".into() }));
    let response = engine.handle(EngineRequest::Conversation(ConversationRequest::Turn { text: "What is Paris?".into(), language: LanguageMode::Auto }));
    match response {
        EngineResponse::Conversation(value) => {
            assert_eq!(value.meta.status, lexflex::engine::EngineStatus::Ok);
            assert_eq!(value.meta.run_id, "run:00000001");
            assert_eq!(engine.session.sources.len(), 1);
            assert!(value.answer.as_ref().is_some_and(|answer| !answer.evidence.is_empty()));
        }
        other => panic!("unexpected response: {other:?}"),
    }
    let repeated = engine.handle(EngineRequest::Conversation(ConversationRequest::Turn { text: "What is Paris?".into(), language: LanguageMode::Auto }));
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
    let mut engine = LexFlexEngine::new("data", "explicit-parse-failure", true).unwrap();
    let response = engine.handle(EngineRequest::Conversation(ConversationRequest::Turn {
        text: "Paris?".into(),
        language: LanguageMode::Explicit("en".into()),
    }));
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
    let mut engine = LexFlexEngine::new("data", "trace-contract", true).unwrap().with_trace_sink(Box::new(collector.clone()));
    let _ = engine.handle(EngineRequest::Session(SessionRequest::Clear { target: ClearTarget::All }));
    let events = collector.events();
    assert!(events.iter().any(|event| event.stage == "request"));
    assert!(events.iter().any(|event| event.payload.is_some()));
    let current_run = events.last().expect("trace event").run_id.clone();
    let path = engine.store.as_ref().unwrap().save_trace("trace-contract", "turn-1", &collector.to_jsonl_for(&current_run)).unwrap();
    assert!(path.ends_with("turn-1.jsonl"));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir_all("data/sessions/trace-contract").unwrap();
}
