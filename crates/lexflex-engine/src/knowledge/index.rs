use crate::knowledge::snapshot::KnowledgeSnapshot;
use lexflex_lingua::LinguaGoal;
use lexflex_model::{
    AssertionId, ConceptId, EntityId, SemanticAssertion, SemanticExpression, WorldId,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct KnowledgeIndex {
    pub(crate) by_root_concept: BTreeMap<ConceptId, BTreeSet<AssertionId>>,
    pub(crate) by_entity: BTreeMap<EntityId, BTreeSet<AssertionId>>,
    pub(crate) by_world: BTreeMap<WorldId, BTreeSet<AssertionId>>,
}

impl KnowledgeIndex {
    pub(crate) fn rebuild(snapshot: &KnowledgeSnapshot) -> Self {
        let mut index = Self::default();
        for (_, assertion) in snapshot.assertions() {
            index.insert(assertion);
        }
        index
    }

    pub(crate) fn insert(&mut self, assertion: &SemanticAssertion) {
        if let Some(world) = self.by_world.get_mut(assertion.world()) {
            world.insert(assertion.id().clone());
        } else {
            self.by_world.insert(
                assertion.world().clone(),
                BTreeSet::from([assertion.id().clone()]),
            );
        }

        for concept in root_concepts(assertion.expression()) {
            self.by_root_concept
                .entry(concept)
                .or_default()
                .insert(assertion.id().clone());
        }

        for entity in referenced_entities(assertion.expression()) {
            self.by_entity
                .entry(entity)
                .or_default()
                .insert(assertion.id().clone());
        }
    }

    pub(crate) fn candidate_ids(
        &self,
        goal: &LinguaGoal,
        snapshot: &KnowledgeSnapshot,
    ) -> Vec<AssertionId> {
        let mut candidates: Option<BTreeSet<AssertionId>> = None;

        for concept in root_concepts(&goal.expression) {
            let Some(ids) = self.by_root_concept.get(&concept) else {
                return Vec::new();
            };
            candidates = Some(match candidates {
                Some(current) => current.intersection(ids).cloned().collect(),
                None => ids.clone(),
            });
        }

        for entity in referenced_entities(&goal.expression) {
            let Some(ids) = self.by_entity.get(&entity) else {
                return Vec::new();
            };
            candidates = Some(match candidates {
                Some(current) => current.intersection(ids).cloned().collect(),
                None => ids.clone(),
            });
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

        let ids =
            candidates.unwrap_or_else(|| snapshot.assertions().map(|(id, _)| id.clone()).collect());

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
    use crate::knowledge::KnowledgeSnapshot;
    use lexflex_lingua::EvidencePolicy;
    use lexflex_model::{
        ConceptId, ConceptParameterSchema, ConceptSchema, EntityDefinition, Evidence, EvidenceSet,
        ParameterId, SemanticType, VariableId,
    };

    fn catalog() -> lexflex_model::ConceptCatalog {
        lexflex_model::ConceptCatalog {
            concepts: BTreeMap::from([(
                ConceptId::new_unchecked("ROOT_CONCEPT"),
                ConceptSchema {
                    id: ConceptId::new_unchecked("ROOT_CONCEPT"),
                    kind: lexflex_model::ConceptKind::Predicate,
                    parameters: BTreeMap::from([(
                        ParameterId::new_unchecked("scope"),
                        ConceptParameterSchema {
                            id: ParameterId::new_unchecked("scope"),
                            value_type: SemanticType::Entity,
                            required: true,
                        },
                    )]),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::Entity)),
                },
            )]),
            entities: BTreeMap::from([
                (
                    EntityId::new_unchecked("ENTITY_1"),
                    EntityDefinition {
                        id: EntityId::new_unchecked("ENTITY_1"),
                        primary_type: ConceptId::new_unchecked("ROOT_CONCEPT"),
                        additional_types: BTreeSet::new(),
                    },
                ),
                (
                    EntityId::new_unchecked("ENTITY_2"),
                    EntityDefinition {
                        id: EntityId::new_unchecked("ENTITY_2"),
                        primary_type: ConceptId::new_unchecked("ROOT_CONCEPT"),
                        additional_types: BTreeSet::new(),
                    },
                ),
            ]),
            parents: BTreeMap::new(),
        }
    }

    fn assertion() -> SemanticAssertion {
        SemanticAssertion::create(
            SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked(
                    "ENTITY_1",
                ))),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("ROOT_CONCEPT"),
                    bindings: BTreeMap::from([(
                        ParameterId::new_unchecked("scope"),
                        SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_2")),
                    )]),
                }),
            },
            EvidenceSet::singleton(Evidence::create("source:1", None, None).expect("evidence"))
                .expect("valid evidence set"),
            WorldId::new_unchecked("actual"),
            &catalog(),
        )
        .expect("assertion")
    }

    fn snapshot(assertion: SemanticAssertion) -> KnowledgeSnapshot {
        let assertions = BTreeMap::from([(assertion.id().clone(), assertion)]);
        KnowledgeSnapshot::from_assertions_for_test(assertions).expect("snapshot")
    }

    #[test]
    fn rebuild_indexes_world_root_concept_and_entity() {
        let assertion = assertion();
        let snapshot = snapshot(assertion.clone());
        let index = KnowledgeIndex::rebuild(&snapshot);

        assert!(index
            .by_root_concept
            .get(&ConceptId::new_unchecked("ROOT_CONCEPT"))
            .is_some_and(|ids| ids.contains(assertion.id())));
        assert!(index
            .by_entity
            .get(&EntityId::new_unchecked("ENTITY_1"))
            .is_some_and(|ids| ids.contains(assertion.id())));
        assert!(index
            .by_world
            .get(&WorldId::new_unchecked("actual"))
            .is_some_and(|ids| ids.contains(assertion.id())));
    }

    #[test]
    fn candidate_ids_intersect_indexes() {
        let assertion = assertion();
        let snapshot = snapshot(assertion.clone());
        let index = KnowledgeIndex::rebuild(&snapshot);
        let goal = LinguaGoal {
            expression: SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                    "answer",
                ))),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("ROOT_CONCEPT"),
                    bindings: BTreeMap::from([(
                        ParameterId::new_unchecked("scope"),
                        SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_2")),
                    )]),
                }),
            },
            variables: BTreeMap::from([(
                VariableId::new_unchecked("answer"),
                SemanticType::Entity,
            )]),
            projection: vec![VariableId::new_unchecked("answer")],
            evidence_policy: EvidencePolicy::Ignore,
            world: Some(WorldId::new_unchecked("actual")),
            limit: None,
        };

        assert_eq!(
            index.candidate_ids(&goal, &snapshot),
            vec![assertion.id().clone()]
        );
    }
}
