use crate::{ConceptId, ConceptSchema, EntityDefinition, EntityId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptCatalog {
    pub concepts: BTreeMap<ConceptId, ConceptSchema>,
    pub entities: BTreeMap<EntityId, EntityDefinition>,
    pub parents: BTreeMap<ConceptId, BTreeSet<ConceptId>>,
}

impl ConceptCatalog {
    pub fn concept(&self, id: &ConceptId) -> Option<&ConceptSchema> {
        self.concepts.get(id)
    }

    pub fn entity(&self, id: &EntityId) -> Option<&EntityDefinition> {
        self.entities.get(id)
    }

    pub fn is_subtype(&self, child: &ConceptId, parent: &ConceptId) -> bool {
        if child == parent {
            return true;
        }
        let mut stack = vec![child.clone()];
        let mut visited = BTreeSet::new();
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            let Some(parents) = self.parents.get(&current) else {
                continue;
            };
            if parents.contains(parent) {
                return true;
            }
            stack.extend(parents.iter().cloned());
        }
        false
    }

    pub fn entity_is(&self, entity: &EntityId, expected: &ConceptId) -> bool {
        self.entity(entity).is_some_and(|definition| {
            definition
                .all_types()
                .any(|actual| self.is_subtype(actual, expected))
        })
    }
}
