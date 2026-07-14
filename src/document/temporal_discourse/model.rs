use crate::document::graph::GraphNodeId;
use crate::document::id::{DocumentId, SentenceId};
use crate::document::resolution::{EntityClusterId, ResolutionId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::id::{
    DiscourseRelationId, DocumentReferenceTimeId, EventCoreferenceClusterId,
    EventCoreferenceDecisionId, EventTemporalAssignmentId, TemporalExpressionId,
    TemporalRelationId,
};
use super::options::DocumentTemporalDiscourseOptions;
use super::schema::DocumentTemporalDiscourseSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockTime {
    pub hour: u8,
    pub minute: u8,
    pub second: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimezoneOffset {
    pub minutes_east_utc: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentReferenceTime {
    pub id: DocumentReferenceTimeId,
    pub label: String,
    pub civil_date: Option<crate::document::knowledge::CivilDate>,
    pub clock_time: Option<ClockTime>,
    pub timezone_offset: Option<TimezoneOffset>,
    pub explicit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalExpressionKind {
    Absolute,
    Relative,
    Duration,
    Frequency,
    PartOfDay,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalNormalizedValue {
    Absolute {
        civil_date: Option<crate::document::knowledge::CivilDate>,
        clock_time: Option<ClockTime>,
        timezone_offset: Option<TimezoneOffset>,
        iso_timestamp: Option<String>,
    },
    Relative { offset_days: i64, anchor: String },
    Duration { days: i64 },
    Frequency { times: i32, period: String },
    PartOfDay { label: String },
    Symbolic { label: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalExpression {
    pub id: TemporalExpressionId,
    pub sentence_id: SentenceId,
    pub source_text: String,
    pub kind: TemporalExpressionKind,
    pub normalized: TemporalNormalizedValue,
    pub explicit_reference_time: Option<DocumentReferenceTimeId>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventTemporalRef {
    Event(GraphNodeId),
    EventCluster(EventCoreferenceClusterId),
    DocumentReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalRelationKind {
    Before,
    After,
    Overlaps,
    Contains,
    ContainedBy,
    Simultaneous,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalRelation {
    pub id: TemporalRelationId,
    pub from_event_id: GraphNodeId,
    pub to_event_id: GraphNodeId,
    pub kind: TemporalRelationKind,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalClosureConflict {
    pub left_event_id: GraphNodeId,
    pub right_event_id: GraphNodeId,
    pub relation: TemporalRelationKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventProfile {
    pub id: GraphNodeId,
    pub sentence_id: SentenceId,
    pub event_node_id: GraphNodeId,
    pub frame_occurrence_id: GraphNodeId,
    pub frame_type: String,
    pub verb_concept: String,
    pub participant_count: usize,
    pub polarity: String,
    pub aspect: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCoreferenceDecisionKind {
    Seeded,
    Accepted,
    HardAccepted,
    Ambiguous,
    Deferred,
    Unresolved,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCoreferenceDecision {
    pub id: EventCoreferenceDecisionId,
    pub event_profile_id: GraphNodeId,
    pub selected_cluster: Option<EventCoreferenceClusterId>,
    pub kind: EventCoreferenceDecisionKind,
    pub score: i32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCoreferenceCluster {
    pub id: EventCoreferenceClusterId,
    pub representative_event_profile_id: GraphNodeId,
    pub event_profile_ids: Vec<GraphNodeId>,
    pub canonical_frame_type: String,
    pub canonical_verb_concept: String,
    pub confidence_milli: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscourseRelationKind {
    Cause,
    Result,
    Sequence,
    Elaboration,
    Background,
    Contrast,
    Condition,
    Purpose,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscourseRelation {
    pub id: DiscourseRelationId,
    pub from_sentence_id: SentenceId,
    pub to_sentence_id: SentenceId,
    pub kind: DiscourseRelationKind,
    pub canonical_direction: Option<String>,
    pub cue_evidence: Vec<String>,
    pub implicit: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedDocumentGenerationPlan {
    pub entity_cluster_refs: Vec<EntityClusterId>,
    pub temporal_directives: Vec<String>,
    pub discourse_directives: Vec<String>,
    pub ordered_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DocumentTemporalDiscourseSummary {
    pub sentences_total: usize,
    pub event_profiles_total: usize,
    pub temporal_expressions_total: usize,
    pub temporal_relations_total: usize,
    pub temporal_conflicts_total: usize,
    pub event_decisions_total: usize,
    pub event_clusters_total: usize,
    pub discourse_relations_total: usize,
    pub diagnostics_info: usize,
    pub diagnostics_warning: usize,
    pub diagnostics_error: usize,
    pub diagnostics_fatal: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentTemporalDiscourseDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentTemporalDiscourseDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: DocumentTemporalDiscourseDiagnosticSeverity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTemporalDiscourse {
    pub schema: DocumentTemporalDiscourseSchema,
    pub id: String,
    pub source_document_id: DocumentId,
    pub source_graph_id: crate::document::graph::GraphId,
    pub source_graph_sha256: String,
    pub source_resolution_id: Option<ResolutionId>,
    pub source_resolution_sha256: Option<String>,
    pub source_sha256: String,
    pub options: DocumentTemporalDiscourseOptions,
    pub options_sha256: String,
    pub reference_time: DocumentReferenceTime,
    pub event_profiles: BTreeMap<GraphNodeId, EventProfile>,
    pub event_profile_order: Vec<GraphNodeId>,
    pub temporal_expressions: BTreeMap<TemporalExpressionId, TemporalExpression>,
    pub temporal_expression_order: Vec<TemporalExpressionId>,
    pub event_assignments: BTreeMap<EventTemporalAssignmentId, EventTemporalAssignment>,
    pub event_assignment_order: Vec<EventTemporalAssignmentId>,
    pub temporal_relations: BTreeMap<TemporalRelationId, TemporalRelation>,
    pub temporal_relation_order: Vec<TemporalRelationId>,
    pub temporal_closure: Vec<TemporalRelation>,
    pub temporal_conflicts: Vec<TemporalClosureConflict>,
    pub event_coreference_decisions: BTreeMap<EventCoreferenceDecisionId, EventCoreferenceDecision>,
    pub event_coreference_decision_order: Vec<EventCoreferenceDecisionId>,
    pub event_coreference_clusters: BTreeMap<EventCoreferenceClusterId, EventCoreferenceCluster>,
    pub event_coreference_cluster_order: Vec<EventCoreferenceClusterId>,
    pub discourse_units: BTreeMap<SentenceId, String>,
    pub discourse_unit_order: Vec<SentenceId>,
    pub discourse_relations: BTreeMap<DiscourseRelationId, DiscourseRelation>,
    pub discourse_relation_order: Vec<DiscourseRelationId>,
    pub generation_plan: Option<ResolvedDocumentGenerationPlan>,
    pub diagnostics: Vec<DocumentTemporalDiscourseDiagnostic>,
    pub summary: DocumentTemporalDiscourseSummary,
    pub temporal_discourse_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventTemporalAssignment {
    pub id: EventTemporalAssignmentId,
    pub event_ref: EventTemporalRef,
    pub event_profile_id: GraphNodeId,
    pub temporal_expression_id: Option<TemporalExpressionId>,
    pub relation: TemporalRelationKind,
    pub evidence: Vec<String>,
}
