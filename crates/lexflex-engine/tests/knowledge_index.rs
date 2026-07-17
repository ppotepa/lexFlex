use lexflex_engine::{KnowledgeIndex, KnowledgeSnapshot};
use lexflex_lingua::{solve::EvidencePolicy, LinguaGoal};
use lexflex_model::{
    canonical_hash, ConceptId, EntityId, Evidence, ParameterId, SemanticAssertion,
    SemanticExpression, VariableId, WorldId,
};
use std::collections::BTreeMap;

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
        vec![Evidence::create("source:1", None, None).expect("evidence")],
        WorldId::new_unchecked("actual"),
    )
    .expect("assertion");
    let snapshot = KnowledgeSnapshot {
        assertions: BTreeMap::from([(assertion.id.clone(), assertion.clone())]),
        snapshot_hash: canonical_hash(&BTreeMap::from([(assertion.id.clone(), assertion.clone())]))
            .expect("snapshot hash"),
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
        variables: BTreeMap::from([(
            VariableId::new_unchecked("answer"),
            lexflex_model::SemanticType::Entity,
        )]),
        projection: vec![VariableId::new_unchecked("answer")],
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
        vec![Evidence::create("source:1", None, None).expect("evidence")],
        WorldId::new_unchecked("actual"),
    )
    .expect("assertion");
    let snapshot = KnowledgeSnapshot {
        assertions: BTreeMap::from([(assertion.id.clone(), assertion.clone())]),
        snapshot_hash: canonical_hash(&BTreeMap::from([(assertion.id.clone(), assertion.clone())]))
            .expect("snapshot hash"),
    };
    let index = KnowledgeIndex::rebuild(&snapshot);
    let goal = LinguaGoal {
        expression: SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_1")),
        variables: BTreeMap::new(),
        projection: vec![VariableId::new_unchecked("answer")],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    };
    let candidates = index.candidate_ids(&goal, &snapshot);
    assert_eq!(candidates, vec![assertion.id.clone()]);
}

#[test]
fn empty_intersection_stays_empty() {
    let assertion = SemanticAssertion::create(
        SemanticExpression::Apply {
            concept: ConceptId::new_unchecked("FACT_A"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_A")),
            )]),
        },
        vec![Evidence::create("source:1", None, None).expect("evidence")],
        WorldId::new_unchecked("actual"),
    )
    .expect("assertion");
    let assertions = BTreeMap::from([(assertion.id.clone(), assertion.clone())]);
    let snapshot = KnowledgeSnapshot {
        snapshot_hash: canonical_hash(&assertions).expect("snapshot hash"),
        assertions,
    };
    let index = KnowledgeIndex::rebuild(&snapshot);
    let goal = LinguaGoal {
        expression: SemanticExpression::Apply {
            concept: ConceptId::new_unchecked("FACT_B"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_B")),
            )]),
        },
        variables: BTreeMap::new(),
        projection: vec![VariableId::new_unchecked("answer")],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    };
    let candidates = index.candidate_ids(&goal, &snapshot);
    assert!(candidates.is_empty());
}
