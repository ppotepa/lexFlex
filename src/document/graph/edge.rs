use crate::core::interlingua::SemanticRole;
use serde::{Deserialize, Serialize};

use super::id::GraphNodeId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentGraphEdgeKind {
    ContainsBlock,
    ContainsParagraph,
    ContainsSourceSentence,
    NextBlock,
    NextParagraph,
    NextSourceSentence,
    HasSemanticSentence,
    NextSemanticSentence,
    HasConstruction,
    WrapsFrame,
    HasFrameOccurrence,
    HasMention,
    FillsRole { role: SemanticRole, role_ordinal: usize },
    CoordinationMember { ordinal: usize },
    ModifierMention { ordinal: usize },
    RefersToCandidate { primary: bool },
    CandidateSameSurface { confidence_milli: u16 },
    CandidateSurfaceConflict { reason: String },
    CandidatePossibleAntecedent { confidence_milli: u16, label: String },
    CandidateIdentityConflict { semantic_entity_id: usize, reason: String },
    GroupHasMemberCandidate { ordinal: usize },
    RepresentsEvent,
    EventParticipant { role: SemanticRole, role_ordinal: usize, mention_id: GraphNodeId },
    HasUnresolvedFragment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentGraphEdge {
    pub id: super::id::GraphEdgeId,
    pub from: GraphNodeId,
    pub to: GraphNodeId,
    pub kind: DocumentGraphEdgeKind,
}
