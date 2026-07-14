use crate::core::interlingua::LanguageId;
use crate::query::{DocumentAnswer, QueryInterlingua};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type RequestId = String;
pub type SessionId = String;
pub type SnapshotId = String;
pub type SourceId = String;
pub type BundleId = String;
pub type ClaimId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageMode { Auto, Explicit(String) }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineStatus { Ok, Unknown, Unsupported, Error }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind { Wikipedia, File, Inline }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceFetchPolicy { SnapshotOnly, CacheFirst, Live }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRequest {
    pub source_kind: SourceKind,
    pub title: String,
    pub language: LanguageId,
    pub policy: SourceFetchPolicy,
}

impl SourceRequest {
    pub fn wikipedia_snapshot(title: impl Into<String>, language: impl Into<LanguageId>) -> Self {
        Self { source_kind: SourceKind::Wikipedia, title: title.into(), language: language.into(), policy: SourceFetchPolicy::SnapshotOnly }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSnapshot {
    pub source_id: SourceId,
    pub title: String,
    pub language: LanguageId,
    pub uri: Option<String>,
    pub revision: Option<String>,
    pub fetched_at: Option<String>,
    pub content_sha256: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectTarget { Session, Sources, Bundles, Snapshot(SnapshotId) }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineRequest {
    UserTurn { text: String, language: LanguageMode },
    Translate { text: String, from: LanguageId, to: LanguageId },
    IngestSource { source: SourceRequest },
    Query { query: QueryInterlingua },
    Inspect { target: InspectTarget },
    ClearSession,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub status: EngineStatus,
    pub request_id: RequestId,
    pub session_snapshot_id: SnapshotId,
    pub diagnostics: Vec<String>,
    pub trace_ref: Option<String>,
    pub artifact_hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub meta: ResponseMeta,
    pub text: Option<String>,
    pub answer: Option<DocumentAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationResponse { pub meta: ResponseMeta, pub text: String }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestResponse {
    pub meta: ResponseMeta,
    pub source_id: SourceId,
    pub bundle_id: BundleId,
    pub source_sha256: String,
    pub bundle_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionResponse { pub meta: ResponseMeta, pub values: BTreeMap<String, String> }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineResponse {
    Conversation(ConversationResponse),
    Translation(TranslationResponse),
    Ingest(IngestResponse),
    Answer { meta: ResponseMeta, answer: DocumentAnswer },
    Inspection(InspectionResponse),
    Error { meta: ResponseMeta, error: EngineError },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineError {
    InvalidRequest(String),
    SourceUnavailable(String),
    Source(String),
    Pipeline(String),
    Query(String),
    Persistence(String),
}
