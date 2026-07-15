use crate::core::interlingua::LanguageId;
use crate::query::{DocumentAnswer, QueryInterlingua};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type RequestId = String;
pub type RunId = String;
pub type SessionId = String;
pub type SnapshotId = String;
pub type SourceId = String;
pub type BundleId = String;
pub type ClaimId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageMode { Auto, Explicit(String) }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerLanguage { Auto, Source, Explicit(LanguageId) }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineStatus { Ok, Unknown, Unsupported, Error }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineArea { Conversation, Translation, Debug, Session }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind { Wikipedia, File, Inline }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceFetchPolicy { SnapshotOnly, CacheFirst, Live }

impl SourceFetchPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceFetchPolicy::SnapshotOnly => "snapshot-only",
            SourceFetchPolicy::CacheFirst => "cache-first",
            SourceFetchPolicy::Live => "live",
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugFactRole { Any, Subject, Object }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsDebugQuery {
    pub selector: String,
    pub role: DebugFactRole,
    /// `None` means return every matching fact.
    pub limit: Option<usize>,
    #[serde(default = "default_facts_page")]
    pub page: usize,
}

fn default_facts_page() -> usize { 1 }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceDebugQuery {
    pub claim_id: ClaimId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugInspectTarget {
    Interlingua,
    Entities,
    Query,
    Pipeline,
    Sources,
    Snapshot,
    Trace {
        run_id: Option<RunId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugCommand {
    Facts(FactsDebugQuery),
    Evidence(EvidenceDebugQuery),
    Inspect(DebugInspectTarget),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationRequest {
    Turn { text: String, language: LanguageMode },
    IngestSource { source: SourceRequest },
    Query { query: QueryInterlingua },
    Inspect { target: InspectTarget },
    SetAnswerLanguage { language: AnswerLanguage },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationRequest {
    Configure { from: LanguageMode, to: LanguageId },
    Turn { text: String, from: Option<LanguageMode>, to: Option<LanguageId> },
    Inspect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearTarget { Conversation, Translation, All }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionRequest { Clear { target: ClearTarget }, Inspect }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugEntityMatch {
    pub bundle_id: BundleId,
    pub entity_cluster_id: String,
    pub canonical_name: Option<String>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugSelectorOutcome {
    Matched { entities: Vec<DebugEntityMatch> },
    Ambiguous { candidates: Vec<DebugEntityMatch> },
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugSourceSpan {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugEvidence {
    pub source_id: SourceId,
    pub source_title: String,
    pub source_language: String,
    pub source_uri: Option<String>,
    pub source_revision: Option<String>,
    pub source_sha256: String,
    pub occurrence_id: String,
    pub sentence_id: String,
    pub sentence_text: String,
    pub source_spans: Vec<DebugSourceSpan>,
    pub derivation: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugFact {
    pub claim_id: ClaimId,
    pub bundle_id: BundleId,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub selector_role: String,
    pub quality: String,
    pub claim_status: String,
    pub factuality: String,
    pub world: String,
    pub confidence_milli: u16,
    pub evidence: Vec<DebugEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnginePresentationTone {
    Success,
    Info,
    Warning,
    Error,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnginePresentationBlock {
    Text(String),
    Fields(Vec<(String, String)>),
    BulletList(Vec<String>),
    Notice { tone: EnginePresentationTone, message: String },
    Timeline(Vec<String>),
    TechnicalRefs(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnginePresentationSection {
    pub title: String,
    pub tone: EnginePresentationTone,
    pub blocks: Vec<EnginePresentationBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnginePresentation {
    pub title: String,
    pub summary: String,
    pub status_line: String,
    pub tone: EnginePresentationTone,
    pub sections: Vec<EnginePresentationSection>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugPresentationCategory {
    Facts,
    Evidence,
    Interlingua,
    Entities,
    Query,
    Pipeline,
    Sources,
    Snapshot,
    Trace,
    Diagnostics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugCard {
    pub kind: String,
    pub title: String,
    pub body: Vec<String>,
    pub fields: Vec<(String, String)>,
    pub evidence: Vec<String>,
    pub status: Option<String>,
    pub confidence: Option<String>,
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugPresentation {
    pub category: DebugPresentationCategory,
    pub title: String,
    pub summary: String,
    pub status_line: String,
    pub cards: Vec<DebugCard>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugResponse {
    pub meta: ResponseMeta,
    pub command: DebugCommand,
    pub selector_outcome: DebugSelectorOutcome,
    pub total_facts: usize,
    pub returned_facts: usize,
    pub truncated: bool,
    pub facts: Vec<DebugFact>,
    pub presentation: DebugPresentation,
    pub ui: EnginePresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineRequest {
    Conversation(ConversationRequest),
    Translation(TranslationRequest),
    Session(SessionRequest),
    Debug { command: DebugCommand },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub area: EngineArea,
    pub status: EngineStatus,
    pub request_id: RequestId,
    pub run_id: RunId,
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
    pub presentation: EnginePresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub meta: ResponseMeta,
    pub text: Option<String>,
    pub turn_id: Option<String>,
    pub source_language: LanguageId,
    pub target_language: LanguageId,
    pub knowledge_committed: bool,
    pub translation_snapshot_id: SnapshotId,
    pub presentation: EnginePresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestResponse {
    pub meta: ResponseMeta,
    pub source_id: SourceId,
    pub bundle_id: BundleId,
    pub source_sha256: String,
    pub bundle_sha256: String,
    pub presentation: EnginePresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionResponse {
    pub meta: ResponseMeta,
    pub values: BTreeMap<String, String>,
    pub presentation: EnginePresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineResponse {
    Conversation(ConversationResponse),
    Translation(TranslationResponse),
    Ingest(IngestResponse),
    Answer { meta: ResponseMeta, answer: DocumentAnswer, presentation: EnginePresentation },
    Inspection(InspectionResponse),
    Debug(DebugResponse),
    Error { meta: ResponseMeta, error: EngineError, presentation: EnginePresentation },
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
