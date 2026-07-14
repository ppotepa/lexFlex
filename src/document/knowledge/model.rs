use crate::document::graph::GraphNodeId;
use crate::document::id::{DocumentId, SentenceId};
use crate::document::resolution::{EntityClusterId, ResolutionId};
use crate::document::temporal_discourse::{
    DiscourseRelationId, DocumentTemporalDiscourseId,
    EventCoreferenceClusterId, EventTemporalAssignmentId,
    TemporalExpressionId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::id::{
    ClaimId, ContradictionSetId, KnowledgeDiagnosticId, KnowledgeExtractionId, PropositionOccurrenceId, ValueId,
};
use super::schema::DocumentKnowledgeSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeDiagnostic {
    pub id: KnowledgeDiagnosticId,
    pub code: String,
    pub message: String,
    pub severity: KnowledgeDiagnosticSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecimalValue {
    pub sign: i8,
    pub mantissa: i128,
    pub scale: u32,
}

impl DecimalValue {
    pub fn new(sign: i8, mantissa: i128, scale: u32) -> Self {
        let sign = if mantissa == 0 { 1 } else { sign.signum().clamp(-1, 1) };
        Self { sign, mantissa: mantissa.abs(), scale }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeValue {
    Integer(i128),
    Decimal(DecimalValue),
    Text(String),
    Boolean(bool),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeSubjectRef {
    EntityCluster(EntityClusterId),
    EventCluster(EventCoreferenceClusterId),
    Document(DocumentId),
    Generic,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgePredicateRef {
    Relation(KnowledgeRelationKind),
    Property(String),
    Concept(String),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeObjectRef {
    EntityCluster(EntityClusterId),
    EventCluster(EventCoreferenceClusterId),
    Value(ValueId),
    Concept(String),
    TextLiteral(String),
    Boolean(bool),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeRelationKind {
    IsA,
    HasProperty,
    HasValue,
    Possesses,
    LocatedAt,
    MovesFrom,
    MovesTo,
    Transfers,
    GivesTo,
    Receives,
    Creates,
    Destroys,
    Perceives,
    Thinks,
    Feels,
    Says,
    States,
    Causes,
    ResultsIn,
    Before,
    After,
    PartOf,
    MemberOf,
    CountOf,
    Measures,
    EquivalentTo,
    DifferentFrom,
    Sequence,
    Contrast,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimWorldRef {
    Actual,
    Reported,
    Conditional,
    Question,
    Purpose,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimFactuality {
    Asserted,
    Negated,
    Questioned,
    Conditional,
    Hypothetical,
    Intended,
    Desired,
    Reported,
    Attributed,
    Uncertain,
    Generic,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimStatus {
    Supported,
    Contradicted,
    Contested,
    AttributedOnly,
    NonFactual,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeQualifier {
    Time(TemporalExpressionId),
    Discourse(DiscourseRelationId),
    EventAssignment(EventTemporalAssignmentId),
    SourceDocument(DocumentId),
    SourceSentence(SentenceId),
    SourceGraphNode(GraphNodeId),
    EntityCluster(EntityClusterId),
    EventCluster(EventCoreferenceClusterId),
    Confidence(u16),
    Approximation(String),
    Location(String),
    Quantity(ValueId),
    World(ClaimWorldRef),
    Note(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimTemporalScope {
    pub temporal_expression_id: Option<TemporalExpressionId>,
    pub temporal_relation_id: Option<EventTemporalAssignmentId>,
    pub normalized_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimAttribution {
    pub source_sentence_id: Option<SentenceId>,
    pub source_document_id: DocumentId,
    pub attributed_to: Option<String>,
    pub nested_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropositionOccurrence {
    pub id: PropositionOccurrenceId,
    pub source_sentence_id: SentenceId,
    pub semantic_sentence_id: Option<GraphNodeId>,
    pub frame_occurrence_id: Option<GraphNodeId>,
    pub event_id: Option<GraphNodeId>,
    pub event_cluster_id: Option<EventCoreferenceClusterId>,
    pub subject: KnowledgeSubjectRef,
    pub predicate: KnowledgePredicateRef,
    pub object: KnowledgeObjectRef,
    pub qualifiers: Vec<KnowledgeQualifier>,
    pub factuality: ClaimFactuality,
    pub world: ClaimWorldRef,
    pub attribution: Option<ClaimAttribution>,
    pub temporal_scope: Option<ClaimTemporalScope>,
    pub confidence_milli: u16,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentClaim {
    pub id: ClaimId,
    pub signature_sha256: String,
    pub occurrence_ids: Vec<PropositionOccurrenceId>,
    pub status: ClaimStatus,
    pub canonical_subject: KnowledgeSubjectRef,
    pub canonical_predicate: KnowledgePredicateRef,
    pub canonical_object: KnowledgeObjectRef,
    pub canonical_qualifiers: Vec<KnowledgeQualifier>,
    pub factuality: ClaimFactuality,
    pub world: ClaimWorldRef,
    pub confidence_milli: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContradictionSet {
    pub id: ContradictionSetId,
    pub claim_ids: Vec<ClaimId>,
    pub kind: String,
    pub scope_label: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KnowledgeExtractionSummary {
    pub proposition_occurrences_total: usize,
    pub claims_total: usize,
    pub values_total: usize,
    pub qualifiers_total: usize,
    pub contradiction_sets_total: usize,
    pub diagnostics_info: usize,
    pub diagnostics_warning: usize,
    pub diagnostics_error: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentKnowledgeExtraction {
    pub schema: DocumentKnowledgeSchema,
    pub id: KnowledgeExtractionId,
    pub source_document_id: DocumentId,
    pub source_graph_id: crate::document::graph::GraphId,
    pub source_graph_sha256: String,
    pub source_resolution_id: Option<ResolutionId>,
    pub source_resolution_sha256: Option<String>,
    pub source_temporal_discourse_id: Option<DocumentTemporalDiscourseId>,
    pub source_temporal_discourse_sha256: Option<String>,
    pub source_sha256: String,
    pub options: super::options::DocumentKnowledgeExtractionOptions,
    pub options_sha256: String,
    pub proposition_occurrences: BTreeMap<PropositionOccurrenceId, PropositionOccurrence>,
    pub proposition_order: Vec<PropositionOccurrenceId>,
    pub values: BTreeMap<ValueId, KnowledgeValue>,
    pub value_order: Vec<ValueId>,
    pub claims: BTreeMap<ClaimId, DocumentClaim>,
    pub claim_order: Vec<ClaimId>,
    pub contradiction_sets: BTreeMap<ContradictionSetId, ContradictionSet>,
    pub contradiction_order: Vec<ContradictionSetId>,
    pub diagnostics: Vec<KnowledgeDiagnostic>,
    pub summary: KnowledgeExtractionSummary,
    pub knowledge_sha256: String,
}
