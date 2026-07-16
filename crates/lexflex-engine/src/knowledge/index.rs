use crate::knowledge::snapshot::KnowledgeSnapshot;
use lexflex_lingua::LinguaGoal;
use lexflex_model::{
    AssertionId, ConceptId, EntityId, SemanticAssertion, SemanticExpression, WorldId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeIndex {
    pub by_root_concept: BTreeMap<ConceptId, BTreeSet<AssertionId>>,
    pub by_entity: BTreeMap<EntityId, BTreeSet<AssertionId>>,
    pub by_world: BTreeMap<WorldId, BTreeSet<AssertionId>>,
}

impl KnowledgeIndex {
    pub fn rebuild(snapshot: &KnowledgeSnapshot) -> Self {
        let mut index = Self::default();
        for assertion in snapshot.assertions.values() {
            index.insert(assertion);
        }
        index
    }

    pub fn insert(&mut self, assertion: &SemanticAssertion) {
        if let Some(world) = self.by_world.get_mut(&assertion.world) {
            world.insert(assertion.id.clone());
        } else {
            self.by_world.insert(
                assertion.world.clone(),
                BTreeSet::from([assertion.id.clone()]),
            );
        }

        for concept in root_concepts(&assertion.expression) {
            self.by_root_concept
                .entry(concept)
                .or_default()
                .insert(assertion.id.clone());
        }

        for entity in referenced_entities(&assertion.expression) {
            self.by_entity
                .entry(entity)
                .or_default()
                .insert(assertion.id.clone());
        }
    }

    pub fn candidate_ids(
        &self,
        goal: &LinguaGoal,
        snapshot: &KnowledgeSnapshot,
    ) -> Vec<AssertionId> {
        let mut candidates: Option<BTreeSet<AssertionId>> = None;

        for concept in root_concepts(&goal.expression) {
            if let Some(ids) = self.by_root_concept.get(&concept) {
                candidates = Some(match candidates {
                    Some(current) => current.intersection(ids).cloned().collect(),
                    None => ids.clone(),
                });
            }
        }

        for entity in referenced_entities(&goal.expression) {
            if let Some(ids) = self.by_entity.get(&entity) {
                candidates = Some(match candidates {
                    Some(current) => current.intersection(ids).cloned().collect(),
                    None => ids.clone(),
                });
            }
        }

        if let Some(world) = &goal.world {
            if let Some(ids) = self.by_world.get(world) {
                candidates = Some(match candidates {
                    Some(current) => current.intersection(ids).cloned().collect(),
                    None => ids.clone(),
                });
            } else {
                return Vec::new();
            }
        }

        let mut ids = candidates.unwrap_or_else(|| snapshot.assertions.keys().cloned().collect());

        if ids.is_empty() {
            ids = snapshot.assertions.keys().cloned().collect();
        }

        ids.into_iter().collect()
    }
}

fn root_concepts(expression: &SemanticExpression) -> BTreeSet<ConceptId> {
    let mut output = BTreeSet::new();
    match expression {
        SemanticExpression::Apply { concept, .. } => {
            output.insert(concept.clone());
        }
        SemanticExpression::Satisfies { subject, predicate } => {
            output.extend(root_concepts(subject));
            output.extend(root_concepts(predicate));
        }
        SemanticExpression::Equals { left, right } => {
            output.extend(root_concepts(left));
            output.extend(root_concepts(right));
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            for item in items {
                output.extend(root_concepts(item));
            }
        }
        SemanticExpression::Not(inner)
        | SemanticExpression::Exists { body: inner, .. }
        | SemanticExpression::ForAll { body: inner, .. }
        | SemanticExpression::Qualified {
            expression: inner, ..
        } => {
            output.extend(root_concepts(inner));
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_)
        | SemanticExpression::Variable(_) => {}
    }
    output
}

fn referenced_entities(expression: &SemanticExpression) -> BTreeSet<EntityId> {
    let mut output = BTreeSet::new();
    match expression {
        SemanticExpression::Entity(entity) => {
            output.insert(entity.clone());
        }
        SemanticExpression::Apply { bindings, .. } => {
            for value in bindings.values() {
                output.extend(referenced_entities(value));
            }
        }
        SemanticExpression::Satisfies { subject, predicate }
        | SemanticExpression::Equals {
            left: subject,
            right: predicate,
        } => {
            output.extend(referenced_entities(subject));
            output.extend(referenced_entities(predicate));
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            for item in items {
                output.extend(referenced_entities(item));
            }
        }
        SemanticExpression::Not(inner)
        | SemanticExpression::Exists { body: inner, .. }
        | SemanticExpression::ForAll { body: inner, .. }
        | SemanticExpression::Qualified {
            expression: inner, ..
        } => {
            output.extend(referenced_entities(inner));
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Value(_)
        | SemanticExpression::Variable(_) => {}
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_lingua::{solve::EvidencePolicy, LinguaGoal};
    use lexflex_model::{Evidence, SemanticAssertion, VariableId, WorldId};

    #[test]
    fn rebuild_indexes_world_root_concept_and_entity() {
        let assertion = SemanticAssertion::create(
            SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked(
                    "ENTITY_1",
                ))),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("ROOT_CONCEPT"),
                    bindings: BTreeMap::new(),
                }),
            },
            vec![Evidence::create("source:1", None, None)],
            WorldId::new_unchecked("actual"),
        );
        let snapshot = KnowledgeSnapshot {
            assertions: BTreeMap::from([(assertion.id.clone(), assertion.clone())]),
            snapshot_hash: String::new(),
        };
        let index = KnowledgeIndex::rebuild(&snapshot);
        assert!(index
            .by_root_concept
            .get(&ConceptId::new_unchecked("ROOT_CONCEPT"))
            .is_some_and(|ids| ids.contains(&assertion.id)));
        assert!(index
            .by_entity
            .get(&EntityId::new_unchecked("ENTITY_1"))
            .is_some_and(|ids| ids.contains(&assertion.id)));
        assert!(index
            .by_world
            .get(&WorldId::new_unchecked("actual"))
            .is_some_and(|ids| ids.contains(&assertion.id)));

        let goal = LinguaGoal {
            expression: SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                    "answer",
                ))),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("ROOT_CONCEPT"),
                    bindings: BTreeMap::new(),
                }),
            },
            variables: BTreeMap::new(),
            projection: Vec::new(),
            evidence_policy: EvidencePolicy::Ignore,
            world: Some(WorldId::new_unchecked("actual")),
            limit: None,
        };
        let candidates = index.candidate_ids(&goal, &snapshot);
        assert_eq!(candidates, vec![assertion.id.clone()]);
    }

    #[test]
    fn candidate_ids_use_entity_index() {
        let assertion = SemanticAssertion::create(
            SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_1")),
            vec![Evidence::create("source:1", None, None)],
            WorldId::new_unchecked("actual"),
        );
        let snapshot = KnowledgeSnapshot {
            assertions: BTreeMap::from([(assertion.id.clone(), assertion.clone())]),
            snapshot_hash: String::new(),
        };
        let index = KnowledgeIndex::rebuild(&snapshot);
        let goal = LinguaGoal {
            expression: SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_1")),
            variables: BTreeMap::new(),
            projection: Vec::new(),
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: None,
        };
        let candidates = index.candidate_ids(&goal, &snapshot);
        assert_eq!(candidates, vec![assertion.id.clone()]);
    }
}
