use lexflex_model::{ConceptId, EntityId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticAnchor {
    Concept(ConceptId),
    Entity(EntityId),
}
