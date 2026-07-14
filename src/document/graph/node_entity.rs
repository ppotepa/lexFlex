use crate::core::interlingua::{ConceptId, EntityId, FeatureBundle, SemanticRole};
use crate::document::{GraphNodeId, SentenceId};
use serde::{Deserialize, Serialize};

use super::anchor::{MentionAnchor, MentionAnchorSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentMentionKind {
    RoleRoot,
    CoordinationGroup,
    CoordinationMember,
    Modifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentMentionNode {
    pub id: GraphNodeId,
    pub source_sentence_id: SentenceId,
    pub semantic_sentence_id: GraphNodeId,
    pub frame_occurrence_id: GraphNodeId,
    pub mention_kind: DocumentMentionKind,
    pub role_context: SemanticRole,
    pub role_ordinal: usize,
    pub concept: ConceptId,
    pub name: Option<String>,
    pub normalized_name: Option<String>,
    pub features: FeatureBundle,
    pub reference: crate::core::interlingua::Reference,
    pub semantic_entity_id: Option<EntityId>,
    pub anchor: MentionAnchor,
    pub anchor_source: MentionAnchorSource,
    pub entity_path: String,
    pub parent_mention_id: Option<GraphNodeId>,
    pub nested_ordinal: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityCandidateKind {
    Referent,
    Group,
    ModifierConcept,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityCandidateState {
    Introduced,
    ProvisionalCluster,
    ResolvedByEntityId,
    ResolvedByExplicitReference,
    Ambiguous,
    Conflict,
    Generic,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateLinkEvidenceKind {
    NewCandidate,
    SharedSemanticEntityId,
    UniqueExplicitAnaphoricLabel,
    AmbiguousExplicitAnaphoricLabel,
    CataphoricDeferred,
    DeicticUnresolved,
    GenericReference,
    UnresolvedReference,
    CoordinationGroup,
    CoordinationMember,
    ModifierConcept,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateLinkEvidence {
    pub kind: CandidateLinkEvidenceKind,
    pub confidence_milli: u16,
    pub details: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityCandidateNode {
    pub id: GraphNodeId,
    pub candidate_kind: EntityCandidateKind,
    pub state: EntityCandidateState,
    pub canonical_name: Option<String>,
    pub normalized_name: Option<String>,
    pub primary_concept: ConceptId,
    pub features: FeatureBundle,
    pub mention_ids: Vec<GraphNodeId>,
    pub semantic_entity_ids: Vec<EntityId>,
    pub reference_kinds: Vec<crate::core::interlingua::Reference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentEventNode {
    pub id: GraphNodeId,
    pub frame_occurrence_id: GraphNodeId,
    pub source_sentence_id: SentenceId,
    pub semantic_sentence_id: GraphNodeId,
    pub global_event_ordinal: usize,
    pub frame_type: String,
    pub verb_concept: String,
    pub participant_count: usize,
}
