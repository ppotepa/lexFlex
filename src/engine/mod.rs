mod source;
mod types;
mod workspace;
mod persistence;

pub use source::{source_snapshot_from_text, LocalSnapshotSourceProvider, SourceProvider};
pub use types::*;
pub use workspace::{SessionClaimRef, SessionIndexes, SessionWorkspace};
pub use persistence::SessionStore;

use crate::api::LexFlexAPI;
use crate::runtime::{LexFlexDocumentEngine, LexFlexRuntimeConfig};
use crate::query::{AnswerStatus, DocumentAnswer, QueryService};
use sha2::{Digest, Sha256};
use serde_json::json;
use std::sync::Mutex;
use std::sync::Arc;

pub trait TraceSink: Send + Sync {
    fn record(
        &self,
        _request_id: &str,
        _stage: &str,
        _payload: Option<serde_json::Value>,
    ) {
    }
    fn snapshot_jsonl(&self) -> Option<String> { None }
}
pub struct NoopTraceSink;
impl TraceSink for NoopTraceSink {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceEvent {
    pub request_id: String,
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
}
impl TraceSink for TraceCollector {
    fn record(&self, request_id: &str, stage: &str, payload: Option<serde_json::Value>) {
        if let Ok(mut events) = self.events.lock() {
            events.push(TraceEvent {
                request_id: request_id.into(),
                stage: stage.into(),
                payload,
            });
        }
    }
    fn snapshot_jsonl(&self) -> Option<String> { Some(self.to_jsonl()) }
}
impl<T: TraceSink + ?Sized> TraceSink for Arc<T> {
    fn record(&self, request_id: &str, stage: &str, payload: Option<serde_json::Value>) {
        (**self).record(request_id, stage, payload);
    }
    fn snapshot_jsonl(&self) -> Option<String> { (**self).snapshot_jsonl() }
}

pub struct ConversationEngine {
    pub session: SessionWorkspace,
    document_engine: LexFlexDocumentEngine,
    source_provider: Box<dyn SourceProvider>,
    trace: Box<dyn TraceSink>,
    pub store: Option<SessionStore>,
}

impl ConversationEngine {
    pub fn new(data_dir: &str, session_id: impl Into<String>, offline: bool) -> Result<Self, EngineError> {
        let api = LexFlexAPI::builder().data_dir(data_dir).build().map_err(|e| EngineError::Pipeline(e.to_string()))?;
        Ok(Self { session: SessionWorkspace::new(session_id), document_engine: LexFlexDocumentEngine::with_config(api, LexFlexRuntimeConfig { data_dir: data_dir.into(), offline, ..Default::default() }), source_provider: Box::new(LocalSnapshotSourceProvider::new(data_dir, offline)), trace: Box::new(Arc::new(TraceCollector::default())), store: Some(SessionStore::new(data_dir)) })
    }
    pub fn with_provider(mut self, provider: Box<dyn SourceProvider>) -> Self { self.source_provider = provider; self }
    pub fn with_trace_sink(mut self, trace: Box<dyn TraceSink>) -> Self { self.trace = trace; self }
    pub fn save_session(&self) -> Result<std::path::PathBuf, EngineError> { self.store.as_ref().ok_or_else(|| EngineError::Persistence("session store disabled".into()))?.save(&self.session) }
    pub fn load_session(&mut self) -> Result<(), EngineError> { let loaded = self.store.as_ref().ok_or_else(|| EngineError::Persistence("session store disabled".into()))?.load(&self.session.session_id)?; self.session = loaded; Ok(()) }
    pub fn handle(&mut self, request: EngineRequest) -> EngineResponse {
        let request_id = request_id(&self.session.snapshot_id, &request);
        self.trace.record(&request_id, "request", Some(json!({ "request": request })));
        let result = self.handle_inner(&request_id, request);
        if let (Some(store), Some(trace)) = (&self.store, self.trace.snapshot_jsonl()) {
            if let Err(error) = store.save_trace(&self.session.session_id, &request_id, &trace) {
                self.trace.record(&request_id, "trace.persist_error", Some(json!({ "error": format!("{error:?}") })));
            }
        }
        result.unwrap_or_else(|error| {
            let diagnostics = vec![format!("engine_error: {error:?}")];
            EngineResponse::Error {
                meta: self.meta(EngineStatus::Error, request_id, diagnostics),
                error,
            }
        })
    }
    fn meta(&self, status: EngineStatus, request_id: RequestId, diagnostics: Vec<String>) -> ResponseMeta {
        let mut artifact_hashes = std::collections::BTreeMap::from([("session_snapshot".into(), self.session.snapshot_sha256.clone())]);
        for (bundle_id, bundle) in &self.session.bundles { artifact_hashes.insert(format!("bundle:{bundle_id}"), stable_bundle_hash(bundle_id, &bundle.source_sha256)); }
        ResponseMeta { status, request_id: request_id.clone(), session_snapshot_id: self.session.snapshot_id.clone(), diagnostics, trace_ref: Some(format!("trace:{request_id}")), artifact_hashes }
    }
    fn meta_with_artifacts(&self, status: EngineStatus, request_id: RequestId, diagnostics: Vec<String>, artifacts: &[(&str, String)]) -> ResponseMeta {
        let mut meta = self.meta(status, request_id, diagnostics);
        meta.artifact_hashes.extend(artifacts.iter().map(|(key, value)| ((*key).into(), value.clone())));
        meta
    }
    fn handle_inner(&mut self, request_id: &str, request: EngineRequest) -> Result<EngineResponse, EngineError> {
        match request {
            EngineRequest::ClearSession => { self.session.clear(); Ok(EngineResponse::Conversation(ConversationResponse { meta: self.meta(EngineStatus::Ok, request_id.into(), Vec::new()), text: Some("session cleared".into()), answer: None })) }
            EngineRequest::Translate { text, from, to } => { let out = self.document_engine.translate(&text, &from.0, &to.0).map_err(|e| EngineError::Pipeline(e.to_string()))?; Ok(EngineResponse::Translation(TranslationResponse { meta: self.meta(EngineStatus::Ok, request_id.into(), Vec::new()), text: out })) }
            EngineRequest::IngestSource { source } => self.ingest(request_id, source),
            EngineRequest::Query { query } => {
                let answer = self.answer_query(query)?;
                let status = if answer.evidence.is_empty() { EngineStatus::Unknown } else { EngineStatus::Ok };
                let diagnostics = diagnostics_for_answer(&answer);
                let meta = self.meta_with_artifacts(status, request_id.into(), diagnostics, &[("answer", answer.answer_sha256.clone())]);
                Ok(EngineResponse::Answer { meta, answer })
            }
            EngineRequest::UserTurn { text, language } => self.user_turn(request_id, text, language),
            EngineRequest::Inspect { target } => self.inspect(request_id, target),
        }
    }
    fn ingest(&mut self, request_id: &str, request: SourceRequest) -> Result<EngineResponse, EngineError> {
        let snapshot = self.source_provider.resolve(&request)?;
        self.trace.record(request_id, "source.resolved", Some(json!({
            "source_id": snapshot.source_id,
            "language": snapshot.language,
            "sha256": snapshot.content_sha256,
            "revision": snapshot.revision,
        })));
        let metadata = crate::runtime::bundle::SourceMetadata { title: snapshot.title.clone(), language: snapshot.language.0.clone(), uri: snapshot.uri.clone(), revision: snapshot.revision.clone() };
        let bundle = self.document_engine.ingest_document_bundle_with_metadata(&snapshot.text, &snapshot.language.0, metadata).map_err(|e| EngineError::Pipeline(e.to_string()))?;
        self.trace.record(request_id, "pipeline.bundle_ready", Some(json!({
            "bundle_sha256": bundle.bundle_sha256,
            "source_sha256": snapshot.content_sha256,
        })));
        let id = self.session.add_bundle(&snapshot, bundle);
        let bundle_sha = stable_bundle_hash(&id, &snapshot.content_sha256);
        self.trace.record(request_id, "session.snapshot", Some(json!({
            "snapshot_id": self.session.snapshot_id,
            "bundle_id": id,
        })));
        let meta = self.meta_with_artifacts(EngineStatus::Ok, request_id.into(), Vec::new(), &[("source", snapshot.content_sha256.clone()), ("bundle", bundle_sha.clone())]);
        Ok(EngineResponse::Ingest(IngestResponse { meta, source_id: snapshot.source_id, bundle_id: id, source_sha256: snapshot.content_sha256, bundle_sha256: bundle_sha }))
    }
    fn user_turn(&mut self, request_id: &str, text: String, language: LanguageMode) -> Result<EngineResponse, EngineError> {
        let Some(bundle) = self.session.sole_bundle() else {
            let diagnostics = vec!["no_active_session_bundle".into()];
            return Ok(EngineResponse::Conversation(ConversationResponse {
                meta: self.meta(EngineStatus::Unknown, request_id.into(), diagnostics),
                text: None,
                answer: None,
            }));
        };
        let lang = match language { LanguageMode::Explicit(value) => value, LanguageMode::Auto => {
            let api = LexFlexAPI::builder().data_dir(self.document_engine.data_dir()).build().map_err(|e| EngineError::Pipeline(e.to_string()))?;
            detect_language_with_api(&api, &text).into()
        }};
        self.trace.record(request_id, "language.detected", Some(json!({ "language": lang })));
        let il = self.document_engine.parse(&text, &lang).map_err(|e| EngineError::Pipeline(e.to_string()))?;
        self.trace.record(request_id, "interlingua.parsed", Some(json!({
            "sentences": il.as_natural().map(|u| u.sentences.len()).unwrap_or(0),
        })));
        let question = il.as_natural().and_then(|u| u.sentences.iter().find_map(|s| s.question.as_ref())).cloned();
        if let Some(q) = question {
            self.trace.record(request_id, "query.interlingua", Some(json!({
                "predicate": q.proposition.predicate.0,
            })));
            let query = question_to_query(&q, &lang, &text, bundle)?;
            let answer = self.answer_query(query)?;
            self.trace.record(request_id, "answer.selected", Some(json!({
                "status": format!("{:?}", answer.status),
                "evidence_count": answer.evidence.len(),
                "row_count": answer.rows.len(),
            })));
            let status = if answer.evidence.is_empty() { EngineStatus::Unknown } else { EngineStatus::Ok };
            let diagnostics = diagnostics_for_answer(&answer);
            return Ok(EngineResponse::Conversation(ConversationResponse { meta: self.meta(status, request_id.into(), diagnostics), text: answer.text.clone(), answer: Some(answer) }));
        }
        self.trace.record(request_id, "answer.unknown", Some(json!({ "reason": "no_question_semantics" })));
        Ok(EngineResponse::Conversation(ConversationResponse {
            meta: self.meta(EngineStatus::Unknown, request_id.into(), vec!["no_question_semantics".into()]),
            text: None,
            answer: None,
        }))
    }
    fn answer_query(&self, query: crate::query::QueryInterlingua) -> Result<DocumentAnswer, EngineError> {
        if self.session.bundles.is_empty() { return Err(EngineError::Query("no ingested evidence".into())); }
        let mut answers = self.session.bundles.values().map(|bundle| {
            let mut answer = QueryService::answer_document_query(query.clone(), &bundle.knowledge)
                .map_err(|error| EngineError::Query(error.to_string()))?;
            present_entity_names(&mut answer, &bundle.entity_resolution);
            Ok::<_, EngineError>(answer)
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
    fn inspect(&self, request_id: &str, target: InspectTarget) -> Result<EngineResponse, EngineError> { let mut values = std::collections::BTreeMap::new(); values.insert("session_id".into(), self.session.session_id.clone()); values.insert("snapshot_id".into(), self.session.snapshot_id.clone()); values.insert("snapshot_sha256".into(), self.session.snapshot_sha256.clone()); values.insert("sources".into(), self.session.active_source_ids.len().to_string()); values.insert("bundles".into(), self.session.bundles.len().to_string()); values.insert("target".into(), format!("{target:?}")); Ok(EngineResponse::Inspection(InspectionResponse { meta: self.meta(EngineStatus::Ok, request_id.into(), Vec::new()), values })) }
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
