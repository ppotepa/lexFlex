use crate::core::interlingua::{ConceptId, FeatureBundle, SemanticRole};
use crate::document::graph::{
    DocumentGraphDiagnostic, DocumentMentionKind, GraphNodeId,
};
use crate::document::id::{DocumentId, ParagraphId, SentenceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::diagnostic::EntityResolutionDiagnostic;
use super::id::{
    EntityClusterId, ResolutionDecisionId, ResolutionId, ResolutionMentionRef, SyntheticMentionId,
};
use super::options::DocumentEntityResolutionOptions;
use super::schema::DocumentEntityResolutionSchema;
use super::summary::EntityResolutionSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionSourceForm {
    ProperName,
    Pronoun,
    ReflexivePronoun,
    PossessivePronoun,
    DefiniteDescription,
    IndefiniteDescription,
    DemonstrativeDescription,
    BareCommonNoun,
    Group,
    Generic,
    Modifier,
    ZeroSubject,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityResolutionStage {
    Seed,
    Reflexive,
    ExplicitReference,
    ProperName,
    Pronoun,
    Possessive,
    DefiniteDescription,
    Demonstrative,
    Cataphora,
    ZeroAnaphora,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityResolutionDecisionKind {
    Seeded,
    Accepted,
    HardAccepted,
    Ambiguous,
    Deferred,
    Unresolved,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionAlternativeKind {
    Compatible,
    HardRejected,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionAlternative {
    pub target: ResolutionMentionRef,
    pub score: i32,
    pub confidence_milli: u16,
    pub kind: ResolutionAlternativeKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityResolutionDecision {
    pub id: ResolutionDecisionId,
    pub mention: ResolutionMentionRef,
    pub stage: EntityResolutionStage,
    pub kind: EntityResolutionDecisionKind,
    pub selected_cluster: Option<EntityClusterId>,
    pub selected_target: Option<ResolutionMentionRef>,
    pub score: i32,
    pub threshold: i32,
    pub margin: i32,
    pub alternatives: Vec<ResolutionAlternative>,
    pub evidence: Vec<String>,
    pub rejections: Vec<String>,
    pub sentence_id: SentenceId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MentionResolutionProfile {
    pub mention: ResolutionMentionRef,
    pub graph_mention_id: Option<GraphNodeId>,
    pub source_sentence_id: SentenceId,
    pub paragraph_id: ParagraphId,
    pub source_sentence_ordinal: usize,
    pub paragraph_ordinal: usize,
    pub semantic_sentence_id: GraphNodeId,
    pub frame_occurrence_id: GraphNodeId,
    pub mention_kind: DocumentMentionKind,
    pub source_form: MentionSourceForm,
    pub exact_surface: Option<String>,
    pub normalized_surface: Option<String>,
    pub concept: ConceptId,
    pub features: FeatureBundle,
    pub reference: crate::core::interlingua::Reference,
    pub anchor: crate::document::graph::MentionAnchor,
    pub semantic_entity_id: Option<crate::core::interlingua::EntityId>,
    pub role_context: SemanticRole,
    pub role_ordinal: usize,
    pub is_subject_like: bool,
    pub is_group: bool,
    pub is_modifier: bool,
    pub is_pronoun: bool,
    pub is_proper_name: bool,
    pub is_reflexive: bool,
    pub is_zero_subject: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntheticResolutionMention {
    pub id: SyntheticMentionId,
    pub source_sentence_id: SentenceId,
    pub semantic_sentence_id: GraphNodeId,
    pub frame_occurrence_id: GraphNodeId,
    pub subject_role: SemanticRole,
    pub concept: ConceptId,
    pub features: FeatureBundle,
    pub anchor: crate::document::graph::MentionAnchor,
    pub origin: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityCluster {
    pub id: EntityClusterId,
    pub mention_refs: Vec<ResolutionMentionRef>,
    pub canonical_name: Option<String>,
    pub canonical_concept: ConceptId,
    pub features: FeatureBundle,
    pub confidence_milli: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedEntityCluster {
    pub id: EntityClusterId,
    pub representative: ResolutionMentionRef,
    pub mention_refs: Vec<ResolutionMentionRef>,
    pub canonical_name: Option<String>,
    pub aliases: Vec<String>,
    pub canonical_concept: ConceptId,
    pub compatible_concepts: Vec<ConceptId>,
    pub features: FeatureBundle,
    pub decision_ids: Vec<ResolutionDecisionId>,
    pub confidence_milli: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentEntityResolution {
    pub schema: DocumentEntityResolutionSchema,
    pub id: ResolutionId,
    pub source_document_id: DocumentId,
    pub source_graph_id: crate::document::graph::GraphId,
    pub source_graph_sha256: String,
    pub source_sha256: String,
    pub options: DocumentEntityResolutionOptions,
    pub options_sha256: String,
    pub graph_candidate_atoms: Vec<GraphNodeId>,
    pub synthetic_mentions: BTreeMap<SyntheticMentionId, SyntheticResolutionMention>,
    pub synthetic_mention_order: Vec<SyntheticMentionId>,
    pub mention_profiles: BTreeMap<ResolutionMentionRef, MentionResolutionProfile>,
    pub mention_order: Vec<ResolutionMentionRef>,
    pub decisions: BTreeMap<ResolutionDecisionId, EntityResolutionDecision>,
    pub decision_order: Vec<ResolutionDecisionId>,
    pub clusters: BTreeMap<EntityClusterId, ResolvedEntityCluster>,
    pub cluster_order: Vec<EntityClusterId>,
    pub diagnostics: Vec<EntityResolutionDiagnostic>,
    pub source_graph_diagnostics: Vec<DocumentGraphDiagnostic>,
    pub summary: EntityResolutionSummary,
    pub resolution_sha256: String,
}

impl DocumentEntityResolution {
    pub fn mention_profile(&self, mention: &ResolutionMentionRef) -> Option<&MentionResolutionProfile> {
        self.mention_profiles.get(mention)
    }

    pub fn decision_for_mention(&self, mention: &ResolutionMentionRef) -> Option<&EntityResolutionDecision> {
        self.decision_order
            .iter()
            .filter_map(|id| self.decisions.get(id))
            .find(|decision| &decision.mention == mention)
    }

    pub fn cluster(&self, id: &EntityClusterId) -> Option<&ResolvedEntityCluster> {
        self.clusters.get(id)
    }

    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        crate::document::resolution::entity_resolution_to_canonical_json(self)
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        crate::document::resolution::entity_resolution_to_pretty_json(self)
    }
}
