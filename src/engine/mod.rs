mod source;
mod types;
mod workspace;
mod persistence;

pub use source::{source_snapshot_from_text, LocalSnapshotSourceProvider, SourceProvider};
pub use types::*;
pub use workspace::{SessionClaimRef, SessionIndexes, SessionWorkspace};
pub use persistence::SessionStore;

use crate::api::LexFlexAPI;
use crate::error::LexFlexError;
use crate::core::interlingua::QuestionKind;
use crate::runtime::{LexFlexDocumentEngine, LexFlexRuntimeConfig};
use crate::query::{AnswerKind, AnswerStatus, DocumentAnswer, QueryService};
use sha2::{Digest, Sha256};
use serde_json::json;
use std::sync::Mutex;
use std::sync::Arc;

pub trait TraceSink: Send + Sync {
    fn record(
        &self,
        _run_id: &str,
        _request_id: &str,
        _stage: &str,
        _payload: Option<serde_json::Value>,
    ) {
    }
    fn snapshot_jsonl_for(&self, _run_id: &str) -> Option<String> { None }
}
pub struct NoopTraceSink;
impl TraceSink for NoopTraceSink {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceEvent {
    #[serde(default)]
    pub run_id: String,
    pub request_id: String,
    #[serde(default)]
    pub sequence: u64,
    pub stage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Default)]
pub struct TraceCollector { events: Mutex<Vec<TraceEvent>> }
impl TraceCollector {
    pub fn events(&self) -> Vec<TraceEvent> { self.events.lock().map(|events| events.clone()).unwrap_or_default() }
    pub fn to_jsonl(&self) -> String {
        self.events()
            .into_iter()
            .map(|event| serde_json::to_string(&event).expect("trace events must be serializable"))
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn events_for(&self, run_id: &str) -> Vec<TraceEvent> {
        self.events()
            .into_iter()
            .filter(|event| event.run_id == run_id)
            .collect()
    }
    pub fn to_jsonl_for(&self, run_id: &str) -> String {
        self.events_for(run_id)
            .into_iter()
            .map(|event| serde_json::to_string(&event).expect("trace events must be serializable"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
impl TraceSink for TraceCollector {
    fn record(&self, run_id: &str, request_id: &str, stage: &str, payload: Option<serde_json::Value>) {
        if let Ok(mut events) = self.events.lock() {
            let sequence = events.iter().filter(|event| event.run_id == run_id).count() as u64;
            events.push(TraceEvent {
                run_id: run_id.into(),
                request_id: request_id.into(),
                sequence,
                stage: stage.into(),
                payload,
            });
        }
    }
    fn snapshot_jsonl_for(&self, request_id: &str) -> Option<String> {
        Some(self.to_jsonl_for(request_id))
    }
}
impl<T: TraceSink + ?Sized> TraceSink for Arc<T> {
    fn record(&self, run_id: &str, request_id: &str, stage: &str, payload: Option<serde_json::Value>) {
        (**self).record(run_id, request_id, stage, payload);
    }
    fn snapshot_jsonl_for(&self, request_id: &str) -> Option<String> {
        (**self).snapshot_jsonl_for(request_id)
    }
}

pub struct ConversationEngine {
    pub session: SessionWorkspace,
    document_engine: LexFlexDocumentEngine,
    source_provider: Box<dyn SourceProvider>,
    trace: Box<dyn TraceSink>,
    pub store: Option<SessionStore>,
    auto_source_policy: SourceFetchPolicy,
    observer: Option<Arc<dyn Fn(TraceEvent) + Send + Sync>>,
}

impl ConversationEngine {
    pub fn new(data_dir: &str, session_id: impl Into<String>, offline: bool) -> Result<Self, EngineError> {
        let data_root = std::fs::canonicalize(data_dir)
            .map_err(|error| EngineError::Pipeline(format!("data root error: {error}")))?;
        crate::data::layout::validate_data_root(&data_root)
            .map_err(|error| EngineError::Pipeline(error.to_string()))?;
        let data_root_string = data_root.display().to_string();
        let api = LexFlexAPI::builder()
            .data_dir(&data_root_string)
            .build()
            .map_err(|e| EngineError::Pipeline(e.to_string()))?;
        Ok(Self {
            session: SessionWorkspace::new(session_id),
            document_engine: LexFlexDocumentEngine::with_config(
                api,
                LexFlexRuntimeConfig {
                    data_dir: data_root_string.clone(),
                    offline,
                    ..Default::default()
                },
            ),
            source_provider: Box::new(LocalSnapshotSourceProvider::new(&data_root_string, offline)),
            trace: Box::new(Arc::new(TraceCollector::default())),
            store: Some(SessionStore::new(&data_root_string)),
            auto_source_policy: if offline {
                SourceFetchPolicy::SnapshotOnly
            } else {
                SourceFetchPolicy::Live
            },
            observer: None,
        })
    }
    pub fn with_provider(mut self, provider: Box<dyn SourceProvider>) -> Self { self.source_provider = provider; self }
    pub fn with_trace_sink(mut self, trace: Box<dyn TraceSink>) -> Self { self.trace = trace; self }
    pub fn with_auto_source_policy(mut self, policy: SourceFetchPolicy) -> Self { self.auto_source_policy = policy; self }
    pub fn with_observer(mut self, observer: Arc<dyn Fn(TraceEvent) + Send + Sync>) -> Self { self.observer = Some(observer); self }
    pub fn set_observer(&mut self, observer: Arc<dyn Fn(TraceEvent) + Send + Sync>) { self.observer = Some(observer); }
    pub fn save_session(&self) -> Result<std::path::PathBuf, EngineError> { self.store.as_ref().ok_or_else(|| EngineError::Persistence("session store disabled".into()))?.save(&self.session) }
    pub fn load_session(&mut self) -> Result<(), EngineError> { let loaded = self.store.as_ref().ok_or_else(|| EngineError::Persistence("session store disabled".into()))?.load(&self.session.session_id)?; self.session = loaded; Ok(()) }
    pub fn handle(&mut self, request: EngineRequest) -> EngineResponse {
        let run_id = self.session.next_run_id();
        let request_id = request_id(&self.session.snapshot_id, &request);
        self.record_trace(&run_id, &request_id, "request", Some(json!({ "request": request })));
        self.record_trace(&run_id, &request_id, "runtime.data_root", Some(json!({
            "data_dir": self.document_engine.data_dir(),
            "offline": self.document_engine.config().offline,
            "source_policy": self.auto_source_policy,
        })));
        let result = self.handle_inner(&run_id, &request_id, request);
        if let (Some(store), Some(trace)) = (&self.store, self.trace.snapshot_jsonl_for(&run_id)) {
            if let Err(error) = store.save_trace(&self.session.session_id, &run_id, &trace) {
                self.record_trace(&run_id, &request_id, "trace.persist_error", Some(json!({ "error": format!("{error:?}") })));
            }
        }
        result.unwrap_or_else(|error| {
            let diagnostics = vec![format!("engine_error: {error:?}")];
            EngineResponse::Error {
                meta: self.meta(EngineStatus::Error, request_id, run_id, diagnostics),
                error,
            }
        })
    }
    fn record_trace(&self, run_id: &str, request_id: &str, stage: &str, payload: Option<serde_json::Value>) {
        self.trace.record(run_id, request_id, stage, payload.clone());
        if let Some(observer) = &self.observer {
            let event = TraceEvent { run_id: run_id.into(), request_id: request_id.into(), sequence: 0, stage: stage.into(), payload };
            observer(event);
        }
    }
    fn meta(&self, status: EngineStatus, request_id: RequestId, run_id: RunId, diagnostics: Vec<String>) -> ResponseMeta {
        let mut artifact_hashes = std::collections::BTreeMap::from([("session_snapshot".into(), self.session.snapshot_sha256.clone())]);
        for (bundle_id, bundle) in &self.session.bundles { artifact_hashes.insert(format!("bundle:{bundle_id}"), stable_bundle_hash(bundle_id, &bundle.source_sha256)); }
        ResponseMeta { status, request_id: request_id.clone(), run_id: run_id.clone(), session_snapshot_id: self.session.snapshot_id.clone(), diagnostics, trace_ref: Some(format!("trace:{run_id}")), artifact_hashes }
    }
    fn meta_with_artifacts(&self, status: EngineStatus, request_id: RequestId, run_id: RunId, diagnostics: Vec<String>, artifacts: &[(&str, String)]) -> ResponseMeta {
        let mut meta = self.meta(status, request_id, run_id, diagnostics);
        meta.artifact_hashes.extend(artifacts.iter().map(|(key, value)| ((*key).into(), value.clone())));
        meta
    }
    fn handle_inner(&mut self, run_id: &str, request_id: &str, request: EngineRequest) -> Result<EngineResponse, EngineError> {
        match request {
            EngineRequest::ClearSession => { self.session.clear(); Ok(EngineResponse::Conversation(ConversationResponse { meta: self.meta(EngineStatus::Ok, request_id.into(), run_id.into(), Vec::new()), text: Some("session cleared".into()), answer: None })) }
            EngineRequest::Translate { text, from, to } => { let out = self.document_engine.translate(&text, &from.0, &to.0).map_err(|e| EngineError::Pipeline(e.to_string()))?; Ok(EngineResponse::Translation(TranslationResponse { meta: self.meta(EngineStatus::Ok, request_id.into(), run_id.into(), Vec::new()), text: out })) }
            EngineRequest::IngestSource { source } => self.ingest(run_id, request_id, source),
            EngineRequest::Query { query } => {
                let answer = self.answer_query(run_id, request_id, query)?;
                let status = if answer.evidence.is_empty() { EngineStatus::Unknown } else { EngineStatus::Ok };
                let diagnostics = diagnostics_for_answer(&answer);
                let meta = self.meta_with_artifacts(status, request_id.into(), run_id.into(), diagnostics, &[("answer", answer.answer_sha256.clone())]);
                Ok(EngineResponse::Answer { meta, answer })
            }
            EngineRequest::UserTurn { text, language } => self.user_turn(run_id, request_id, text, language),
            EngineRequest::Inspect { target } => self.inspect(run_id, request_id, target),
        }
    }
    fn ingest(&mut self, run_id: &str, request_id: &str, request: SourceRequest) -> Result<EngineResponse, EngineError> {
        self.record_trace(run_id, request_id, "source.resolve.started", Some(json!({
            "kind": request.source_kind,
            "title": request.title,
            "language": request.language,
            "policy": request.policy,
        })));
        let snapshot = self.source_provider.resolve(&request)?;
        self.record_trace(run_id, request_id, "source.resolved", Some(json!({
            "source_id": snapshot.source_id,
            "language": snapshot.language,
            "sha256": snapshot.content_sha256,
            "revision": snapshot.revision,
        })));
        let metadata = crate::runtime::bundle::SourceMetadata { title: snapshot.title.clone(), language: snapshot.language.0.clone(), uri: snapshot.uri.clone(), revision: snapshot.revision.clone() };
        self.record_trace(run_id, request_id, "pipeline.started", Some(json!({
            "source_sha256": snapshot.content_sha256,
            "language": snapshot.language,
        })));
        let bundle = self.document_engine.ingest_document_bundle_with_metadata(&snapshot.text, &snapshot.language.0, metadata).map_err(|e| EngineError::Pipeline(e.to_string()))?;
        self.record_trace(run_id, request_id, "pipeline.bundle_ready", Some(json!({
            "bundle_sha256": bundle.bundle_sha256,
            "source_sha256": snapshot.content_sha256,
            "stages": bundle.artifact_checksums,
            "summary": bundle.summary,
        })));
        for stage in ["compilation", "graph", "resolution", "temporal_discourse", "knowledge"] {
            if let Some(hash) = bundle.artifact_checksums.get(stage) {
                self.record_trace(run_id, request_id, &format!("pipeline.{stage}"), Some(json!({ "sha256": hash })));
            }
        }
        let id = self.session.add_bundle(&snapshot, bundle);
        let bundle_sha = stable_bundle_hash(&id, &snapshot.content_sha256);
        self.record_trace(run_id, request_id, "session.snapshot", Some(json!({
            "snapshot_id": self.session.snapshot_id,
            "bundle_id": id,
        })));
        let meta = self.meta_with_artifacts(EngineStatus::Ok, request_id.into(), run_id.into(), Vec::new(), &[("source", snapshot.content_sha256.clone()), ("bundle", bundle_sha.clone())]);
        Ok(EngineResponse::Ingest(IngestResponse { meta, source_id: snapshot.source_id, bundle_id: id, source_sha256: snapshot.content_sha256, bundle_sha256: bundle_sha }))
    }
    fn user_turn(&mut self, run_id: &str, request_id: &str, text: String, language: LanguageMode) -> Result<EngineResponse, EngineError> {
        let auto_language = matches!(language, LanguageMode::Auto);
        let mut lang = match language { LanguageMode::Explicit(value) => value, LanguageMode::Auto => {
            let api = LexFlexAPI::builder().data_dir(self.document_engine.data_dir()).build().map_err(|e| EngineError::Pipeline(e.to_string()))?;
            detect_language_with_api(&api, &text).into()
        }};
        self.record_trace(run_id, request_id, "language.detected", Some(json!({ "language": lang })));
        let il = match self.document_engine.parse(&text, &lang) {
            Ok(il) => il,
            Err(LexFlexError::Parse(error)) if auto_language => {
                self.record_trace(run_id, request_id, "language.parse_failed", Some(json!({
                    "language": lang,
                    "error": error.to_string(),
                    "kind": format!("{error:?}"),
                })));
                let mut fallback = None;
                for candidate in ["en", "pl"] {
                    if candidate == lang { continue; }
                    match self.document_engine.parse(&text, candidate) {
                        Ok(parsed) => {
                            fallback = Some((candidate.to_string(), parsed));
                            break;
                        }
                        Err(LexFlexError::Parse(fallback_error)) => {
                            self.record_trace(run_id, request_id, "language.parse_failed", Some(json!({
                                "language": candidate,
                                "error": fallback_error.to_string(),
                                "kind": format!("{fallback_error:?}"),
                            })));
                        }
                        Err(other) => return Err(EngineError::Pipeline(other.to_string())),
                    }
                }
                let Some((fallback_language, parsed)) = fallback else {
                    self.record_trace(run_id, request_id, "answer.unknown", Some(json!({ "reason": "language_parse_failed" })));
                    return Ok(EngineResponse::Conversation(ConversationResponse {
                        meta: self.meta(EngineStatus::Unknown, request_id.into(), run_id.into(), vec!["language_parse_failed".into(), format!("language:{lang}")]),
                        text: Some("I could not parse this turn in the detected languages.".into()),
                        answer: None,
                    }));
                };
                self.record_trace(run_id, request_id, "language.parse_fallback", Some(json!({ "language": fallback_language })));
                lang = fallback_language;
                parsed
            }
            Err(LexFlexError::Parse(error)) => {
                self.record_trace(run_id, request_id, "language.parse_failed", Some(json!({
                    "language": lang,
                    "error": error.to_string(),
                    "kind": format!("{error:?}"),
                })));
                self.record_trace(run_id, request_id, "answer.unknown", Some(json!({ "reason": "language_parse_failed" })));
                return Ok(EngineResponse::Conversation(ConversationResponse {
                    meta: self.meta(
                        EngineStatus::Unknown,
                        request_id.into(),
                        run_id.into(),
                        vec!["language_parse_failed".into(), format!("language:{lang}"), format!("parse:{error:?}")],
                    ),
                    text: Some(format!("I could not parse this turn in language '{lang}'.")),
                    answer: None,
                }));
            }
            Err(error) => return Err(EngineError::Pipeline(error.to_string())),
        };
        let sentence_count = il.as_natural().map(|u| u.sentences.len()).unwrap_or(0);
        let question = il
            .as_natural()
            .and_then(|u| u.sentences.iter().find_map(|s| s.question.as_ref()))
            .cloned();
        self.record_trace(run_id, request_id, "interlingua.parsed", Some(json!({
            "sentences": sentence_count,
            "question": question.as_ref().map(|q| json!({
                "kind": format!("{:?}", q.kind),
                "predicate": q.proposition.predicate.0,
                "projection": format!("{:?}", q.projection),
                "subject": q.proposition.subject,
                "object": q.proposition.object,
            })),
        })));
        if let Some(q) = question {
            let bundle_id = self.ensure_source_for_question(run_id, request_id, &q, &lang)?;
            let Some(bundle_id) = bundle_id else {
                self.record_trace(run_id, request_id, "answer.unknown", Some(json!({
                    "reason": "no_ingested_evidence",
                    "question_predicate": q.proposition.predicate.0,
                })));
                let diagnostics = vec!["no_active_session_bundle".into(), "no_ingested_evidence".into(), "auto_source_unavailable".into()];
                return Ok(EngineResponse::Conversation(ConversationResponse {
                    meta: self.meta(EngineStatus::Unknown, request_id.into(), run_id.into(), diagnostics),
                    text: Some("No evidence source was available for this question. Use /ingest <title> or enable a source policy with live access.".into()),
                    answer: None,
                }));
            };
            let Some(bundle) = self.session.bundles.get(&bundle_id) else {
                return Err(EngineError::SourceUnavailable("automatic source did not produce a bundle".into()));
            };
            self.record_trace(run_id, request_id, "query.interlingua", Some(json!({
                "kind": format!("{:?}", q.kind),
                "predicate": q.proposition.predicate.0,
                "projection": format!("{:?}", q.projection),
            })));
            let query = question_to_query(&q, &lang, &text, bundle)?;
            self.record_trace(run_id, request_id, "query.structured", Some(serde_json::to_value(&query).unwrap_or_else(|_| json!({"id": query.id.0}))));
            let mut answer = self.answer_query(run_id, request_id, query)?;
            if let Some(fallback) = source_sentence_answer(&q, bundle, &answer)? {
                self.record_trace(run_id, request_id, "answer.source_sentence_fallback", Some(json!({
                    "status_before": format!("{:?}", answer.status),
                    "status_after": format!("{:?}", fallback.status),
                    "text": fallback.text,
                })));
                answer = fallback;
            }
            self.record_trace(run_id, request_id, "answer.selected", Some(json!({
                "status": format!("{:?}", answer.status),
                "evidence_count": answer.evidence.len(),
                "row_count": answer.rows.len(),
                "text": answer.text,
            })));
            let status = if answer.evidence.is_empty() { EngineStatus::Unknown } else { EngineStatus::Ok };
            let diagnostics = diagnostics_for_answer(&answer);
            return Ok(EngineResponse::Conversation(ConversationResponse { meta: self.meta(status, request_id.into(), run_id.into(), diagnostics), text: answer.text.clone(), answer: Some(answer) }));
        }
        self.record_trace(run_id, request_id, "answer.unknown", Some(json!({ "reason": "no_question_semantics" })));
        Ok(EngineResponse::Conversation(ConversationResponse {
            meta: self.meta(EngineStatus::Unknown, request_id.into(), run_id.into(), vec!["no_question_semantics".into()]),
            text: None,
            answer: None,
        }))
    }
    fn answer_query(&self, run_id: &str, request_id: &str, query: crate::query::QueryInterlingua) -> Result<DocumentAnswer, EngineError> {
        if self.session.bundles.is_empty() { return Err(EngineError::Query("no ingested evidence".into())); }
        let mut answers = self.session.bundles.iter().map(|(bundle_id, bundle)| {
            let mut traced = QueryService::trace_document_query(query.clone(), &bundle.knowledge)
                .map_err(|error| EngineError::Query(error.to_string()))?;
            self.record_trace(run_id, request_id, "query.plan", Some(json!({
                "bundle_id": bundle_id,
                "plan_id": traced.plan.id,
                "operators": traced.plan.operators,
                "steps": traced.plan.steps,
            })));
            self.record_trace(run_id, request_id, "query.execution", Some(json!({
                "bundle_id": bundle_id,
                "row_count": traced.execution.rows.len(),
                "rows": summarize_rows(&traced.execution.rows),
            })));
            present_entity_names(&mut traced.answer, &bundle.entity_resolution);
            rank_answer_rows(&mut traced.answer, &bundle.source_metadata.title);
            Ok::<_, EngineError>(traced.answer)
        }).collect::<Result<Vec<_>, _>>()?;
        let Some(mut merged) = answers.pop() else { return Err(EngineError::Query("no ingested evidence".into())); };
        for answer in answers {
            merged.rows.extend(answer.rows);
            merged.evidence.extend(answer.evidence);
            merged.conflicts.extend(answer.conflicts);
            merged.status = merge_answer_status(merged.status, answer.status);
        }
        if let Some(limit) = query.limit { merged.rows.sort_by_key(|row| row.columns.get("claim").cloned()); merged.rows.truncate(limit); }
        merged.evidence.sort(); merged.evidence.dedup();
        merged.conflicts.sort(); merged.conflicts.dedup();
        if merged.evidence.is_empty() { merged.status = AnswerStatus::Unknown; }
        merged.answer_sha256 = answer_hash(&merged)?;
        Ok(merged)
    }
    fn inspect(&self, run_id: &str, request_id: &str, target: InspectTarget) -> Result<EngineResponse, EngineError> { let mut values = std::collections::BTreeMap::new(); values.insert("session_id".into(), self.session.session_id.clone()); values.insert("snapshot_id".into(), self.session.snapshot_id.clone()); values.insert("snapshot_sha256".into(), self.session.snapshot_sha256.clone()); values.insert("sources".into(), self.session.active_source_ids.len().to_string()); values.insert("bundles".into(), self.session.bundles.len().to_string()); values.insert("target".into(), format!("{target:?}")); Ok(EngineResponse::Inspection(InspectionResponse { meta: self.meta(EngineStatus::Ok, request_id.into(), run_id.into(), Vec::new()), values })) }
}

impl ConversationEngine {
    fn ensure_source_for_question(
        &mut self,
        run_id: &str,
        request_id: &str,
        question: &crate::core::interlingua::QuestionSemantics,
        language: &str,
    ) -> Result<Option<BundleId>, EngineError> {
        let candidates = source_candidates(question);
        self.record_trace(run_id, request_id, "source.discovery", Some(json!({
            "candidates": candidates,
            "policy": self.auto_source_policy,
        })));
        for candidate in candidates {
            let existing = self.session.sources.iter().find_map(|(source_id, source)| {
                if !source.title.eq_ignore_ascii_case(&candidate) || source.language.0 != language { return None; }
                let prefix = format!("bundle:{source_id}:");
                self.session.bundles.keys().find(|id| id.starts_with(&prefix)).cloned().map(|bundle_id| (bundle_id, source_id.clone()))
            });
            if let Some((bundle_id, source_id)) = existing {
                self.record_trace(run_id, request_id, "source.selected", Some(json!({
                    "title": candidate,
                    "source_id": source_id,
                    "origin": "session",
                })));
                return Ok(Some(bundle_id));
            }
            let source = SourceRequest {
                source_kind: SourceKind::Wikipedia,
                title: candidate.clone(),
                language: crate::core::interlingua::LanguageId::new(language),
                policy: self.auto_source_policy,
            };
            self.record_trace(run_id, request_id, "source.selected", Some(json!({
                "title": source.title,
                "language": source.language,
                "policy": source.policy,
                "origin": "automatic",
            })));
            match self.ingest(run_id, request_id, source) {
                Ok(EngineResponse::Ingest(value)) => return Ok(Some(value.bundle_id)),
                Ok(_) => {}
                Err(error @ EngineError::SourceUnavailable(_)) | Err(error @ EngineError::Source(_)) => {
                    self.record_trace(run_id, request_id, "source.auto.failed", Some(json!({ "candidate": candidate, "error": error })));
                }
                Err(error) => return Err(error),
            }
        }
        if let Some(bundle_id) = self.session.bundles.keys().next().cloned() {
            self.record_trace(run_id, request_id, "source.selected", Some(json!({
                "bundle_id": bundle_id,
                "origin": "existing-session-fallback",
            })));
            return Ok(Some(bundle_id));
        }
        Ok(None)
    }
}

fn source_candidates(question: &crate::core::interlingua::QuestionSemantics) -> Vec<String> {
    let mut values = Vec::new();
    for term in question.proposition.subject.iter().chain(question.proposition.object.iter()) {
        if let crate::core::interlingua::QueryTerm::Entity(entity) = term {
            if !matches!(entity.reference, crate::core::interlingua::Reference::Direct) {
                continue;
            }
            let Some(name) = entity.name.as_deref().map(str::trim).filter(|name| !name.is_empty()) else { continue; };
            if name.len() < 2 || matches!(name.to_ascii_lowercase().as_str(), "my" | "your" | "our" | "their" | "this" | "that") {
                continue;
            }
            if !values.iter().any(|value: &String| value.eq_ignore_ascii_case(name)) {
                values.push(name.to_string());
            }
        }
    }
    values
}

fn request_id(snapshot: &str, request: &EngineRequest) -> String { let mut h = Sha256::new(); h.update(snapshot.as_bytes()); h.update(serde_json::to_vec(request).expect("request serialization")); format!("request:{:x}", h.finalize()) }
fn detect_language_with_api(api: &LexFlexAPI, text: &str) -> String {
    let mut best = (i32::MIN, "en".to_string());
    for language in ["pl", "en"] {
        let Ok(il) = api.parse(text, language) else { continue; };
        let Some(utterance) = il.as_natural() else { continue; };
        let question_count = utterance.sentences.iter().filter(|sentence| sentence.question.is_some()).count() as i32;
        let frame_count = utterance.sentences.iter().map(|sentence| sentence.frames.len() as i32).sum::<i32>();
        let score = question_count * 1000 + frame_count * 10 + utterance.sentences.len() as i32;
        if score > best.0 || (score == best.0 && language == "en") {
            best = (score, language.to_string());
        }
    }
    best.1
}
fn question_to_query(q: &crate::core::interlingua::QuestionSemantics, lang: &str, text: &str, bundle: &crate::runtime::DocumentArtifactBundle) -> Result<crate::query::QueryInterlingua, EngineError> {
    crate::query::QueryInterlingua::from_question(
        q,
        lang.to_string(),
        Some(text.to_string()),
        |entity| entity.name.as_deref().and_then(|name| cluster_for_name(name, bundle)),
    ).map_err(|error| EngineError::InvalidRequest(error.to_string()))
}
fn cluster_for_name(name: &str, bundle: &crate::runtime::DocumentArtifactBundle) -> Option<crate::document::resolution::EntityClusterId> { let needle = name.to_lowercase(); bundle.entity_resolution.clusters.values().find(|cluster| cluster.canonical_name.as_deref().map(str::to_lowercase).as_deref() == Some(needle.as_str()) || cluster.aliases.iter().any(|alias| alias.to_lowercase() == needle)).map(|cluster| cluster.id.clone()) }
fn stable_hash(value: &str) -> String { let mut h = Sha256::new(); h.update(value.as_bytes()); format!("{:x}", h.finalize()) }
fn stable_bundle_hash(bundle_id: &str, source_sha256: &str) -> String { stable_hash(&format!("bundle:{bundle_id}:{source_sha256}")) }

fn merge_answer_status(left: AnswerStatus, right: AnswerStatus) -> AnswerStatus {
    use AnswerStatus::*;
    match (left, right) {
        (Conflicting, _) | (_, Conflicting) => Conflicting,
        (Exact, Exact) => Exact,
        (No, No) => No,
        (Unknown, other) | (other, Unknown) => other,
        (Unsupported, _) | (_, Unsupported) => Unsupported,
        (InvalidQuery, _) | (_, InvalidQuery) => InvalidQuery,
        (left, _) => left,
    }
}

fn diagnostics_for_answer(answer: &DocumentAnswer) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if answer.evidence.is_empty() {
        diagnostics.push("no_evidence".into());
    }
    if matches!(answer.status, AnswerStatus::Unknown) {
        diagnostics.push("answer_unknown".into());
    }
    if matches!(answer.status, AnswerStatus::Conflicting) {
        diagnostics.push("answer_conflicting".into());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn present_entity_names(
    answer: &mut DocumentAnswer,
    resolution: &crate::document::resolution::DocumentEntityResolution,
) {
    let labels = resolution
        .clusters
        .values()
        .filter_map(|cluster| {
            let label = cluster
                .mention_refs
                .iter()
                .filter_map(|mention| resolution.mention_profiles.get(mention))
                .find_map(|profile| profile.exact_surface.clone())
                .or_else(|| cluster.canonical_name.clone())?;
            Some((format!("entity_cluster:{}", cluster.id), label))
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    for row in &mut answer.rows {
        for value in row.columns.values_mut() {
            replace_entity_labels(value, &labels);
        }
    }
    if let Some(text) = answer.text.as_mut() {
        replace_entity_labels(text, &labels);
    }
}

fn replace_entity_labels(value: &mut String, labels: &std::collections::BTreeMap<String, String>) {
    for (technical_id, label) in labels {
        if value.contains(technical_id) {
            *value = value.replace(technical_id, label);
        }
    }
}

fn answer_hash(answer: &DocumentAnswer) -> Result<String, EngineError> {
    let mut value = serde_json::to_value(answer).map_err(|error| EngineError::Query(error.to_string()))?;
    if let serde_json::Value::Object(map) = &mut value { map.remove("answer_sha256"); }
    let bytes = serde_json::to_vec(&value).map_err(|error| EngineError::Query(error.to_string()))?;
    Ok(stable_hash(std::str::from_utf8(&bytes).map_err(|error| EngineError::Query(error.to_string()))?))
}

fn source_sentence_answer(
    question: &crate::core::interlingua::QuestionSemantics,
    bundle: &crate::runtime::DocumentArtifactBundle,
    answer: &DocumentAnswer,
) -> Result<Option<DocumentAnswer>, EngineError> {
    let should_consider = matches!(answer.status, AnswerStatus::Unknown)
        || (matches!(question.kind, QuestionKind::Where) && answer.rows.len() > 5)
        || (question.proposition.predicate.0 == "CAPITAL_OF" && answer.rows.len() > 2);
    if !should_consider {
        return Ok(None);
    }

    let document = &bundle.compilation.document;
    let mut required_terms = question
        .proposition
        .subject
        .iter()
        .chain(question.proposition.object.iter())
        .filter_map(|term| match term {
            crate::core::interlingua::QueryTerm::Entity(entity) => entity.name.clone(),
            _ => None,
        })
        .map(|name| name.to_lowercase())
        .collect::<Vec<_>>();
    let source_title = bundle.source_metadata.title.to_lowercase();
    if !source_title.is_empty() && !required_terms.iter().any(|term| term == &source_title) {
        required_terms.push(source_title.clone());
    }

    let predicate = question.proposition.predicate.0.as_str();
    let candidate = document
        .ordered_sentences()
        .into_iter()
        .filter_map(|sentence| {
            let text = document.sentence_text(&sentence.id)?.trim().to_string();
            let lower = text.to_lowercase();
            if !required_terms.iter().all(|term| lower.contains(term)) {
                return None;
            }
            let predicate_match = match predicate {
                "CAPITAL_OF" => lower.contains("capital") || lower.contains("stolica"),
                "LOCATED_IN" => {
                    lower.contains(" in ")
                        || lower.contains(" na ")
                        || lower.contains(" w ")
                        || lower.contains("located")
                        || lower.contains("through")
                }
                "IS_A" => true,
                _ => true,
            };
            predicate_match.then_some((sentence.id.to_string(), text, sentence.raw_span.clone()))
        })
        .next();

    let Some((sentence_id, text, span)) = candidate else {
        return Ok(None);
    };
    let mut fallback = answer.clone();
    let mut evidence = vec![
        format!("sentence:{sentence_id}"),
        format!("source_sentence:{sentence_id}"),
    ];
    if let Some(inner) = span.span {
        evidence.push(format!("source_span:{}:{}", inner.start, inner.end));
    }
    fallback.kind = AnswerKind::Text;
    fallback.status = AnswerStatus::Supported;
    fallback.text = Some(text.clone());
    fallback.rows = vec![crate::query::QueryExecutionResultRow {
        columns: std::collections::BTreeMap::from([
            ("text".into(), text),
            ("sentence".into(), sentence_id),
        ]),
        evidence: evidence.clone(),
    }];
    fallback.evidence = evidence;
    fallback.conflicts.clear();
    fallback.answer_sha256 = answer_hash(&fallback)?;
    Ok(Some(fallback))
}

fn summarize_rows(rows: &[crate::query::QueryExecutionResultRow]) -> Vec<serde_json::Value> {
    rows.iter()
        .take(10)
        .map(|row| json!({
            "columns": row.columns,
            "evidence_count": row.evidence.len(),
        }))
        .collect()
}

fn rank_answer_rows(answer: &mut DocumentAnswer, source_title: &str) {
    let source_title = source_title.to_lowercase();
    answer.rows.sort_by_key(|row| {
        let subject = row.columns.get("subject").cloned().unwrap_or_default().to_lowercase();
        let object = row.columns.get("object").cloned().unwrap_or_default().to_lowercase();
        let unresolved_penalty = usize::from(subject.contains("unresolved") || object.contains("unresolved"));
        let title_bonus = usize::from(!(subject == source_title || object == source_title));
        let text_literal_penalty = usize::from(object.starts_with("text:"));
        (
            title_bonus,
            unresolved_penalty,
            text_literal_penalty,
            std::cmp::Reverse(row.evidence.len()),
            row.columns.get("claim").cloned().unwrap_or_default(),
        )
    });
}
