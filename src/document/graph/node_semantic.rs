use crate::core::interlingua::{Aspect, ConceptId, Illocution, Polarity, SemanticRole, Tense};
use crate::document::{GraphNodeId, SentenceId};
use crate::document::span::LocatedSpan;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstructionOccurrenceSource {
    ExplicitConstruction,
    AttachedConcept,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSentenceOccurrenceNode {
    pub id: GraphNodeId,
    pub source_sentence_id: SentenceId,
    pub semantic_ordinal: usize,
    pub tense: Option<Tense>,
    pub aspect: Option<Aspect>,
    pub polarity: Polarity,
    pub modality: Option<crate::core::interlingua::Modality>,
    pub illocution: Illocution,
    pub voice: Option<crate::core::interlingua::Voice>,
    pub reflexive: bool,
    pub temporal: Option<crate::core::interlingua::TemporalReference>,
    pub quantification: Option<crate::core::interlingua::Quantifier>,
    pub construction_concepts: Vec<ConceptId>,
    pub linguistic_graph_present: bool,
    pub frame_count: usize,
    pub construction_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionOccurrenceNode {
    pub id: GraphNodeId,
    pub source_sentence_id: SentenceId,
    pub semantic_sentence_id: GraphNodeId,
    pub construction_ordinal: usize,
    pub construction_concept: ConceptId,
    pub source: ConstructionOccurrenceSource,
    pub inner_frame_type: Option<String>,
    pub inner_verb_concept: Option<String>,
    pub linked_frame_id: Option<GraphNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameOccurrenceNode {
    pub id: GraphNodeId,
    pub source_sentence_id: SentenceId,
    pub semantic_sentence_id: GraphNodeId,
    pub semantic_sentence_ordinal: usize,
    pub frame_ordinal: usize,
    pub global_frame_ordinal: usize,
    pub frame_type: String,
    pub verb_concept: String,
    pub required_roles: Vec<SemanticRole>,
    pub role_count: usize,
    pub source_provenance_ids: Vec<crate::document::ProvenanceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedFragmentNode {
    pub id: GraphNodeId,
    pub sentence_id: SentenceId,
    pub status: crate::document::compilation::SentenceProcessingStatus,
    pub content_span: LocatedSpan,
    pub diagnostic_codes: Vec<String>,
    pub reason: String,
}
