use crate::ConceptId;
use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityDefinition {
    pub id: EntityId,
    pub primary_type: ConceptId,
    #[serde(default)]
    pub additional_types: BTreeSet<ConceptId>,
}

impl EntityDefinition {
    pub fn all_types(&self) -> impl Iterator<Item = &ConceptId> {
        std::iter::once(&self.primary_type).chain(self.additional_types.iter())
    }
}
