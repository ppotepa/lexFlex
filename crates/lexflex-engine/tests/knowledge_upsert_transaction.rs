use lexflex_engine::KnowledgeSnapshot;
use lexflex_model::{
    Evidence, EvidenceSet, SemanticAssertion, SemanticExpression, SemanticValue, WorldId,
};

fn assertion(source: &str) -> SemanticAssertion {
    let catalog = lexflex_model::ConceptCatalog::default();
    SemanticAssertion::create(
        SemanticExpression::Equals {
            left: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
            right: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
        },
        EvidenceSet::singleton(Evidence::create(source, None, None).expect("evidence"))
            .expect("evidence set"),
        WorldId::new_unchecked("actual"),
        &catalog,
    )
    .expect("assertion")
}

#[test]
fn knowledge_mutations_commit_and_failures_preserve_snapshot() {
    let catalog = lexflex_model::ConceptCatalog::default();
    let mut snapshot = KnowledgeSnapshot::new().expect("snapshot");

    let first = assertion("source:one");
    let inserted = snapshot.upsert(first.clone(), &catalog).expect("insert");
    assert!(matches!(
        inserted,
        lexflex_engine::UpsertOutcome::Inserted { .. }
    ));
    let after_insert = serde_json::to_vec(&snapshot).expect("serialize");

    let unchanged = snapshot.upsert(first, &catalog).expect("duplicate");
    assert!(matches!(
        unchanged,
        lexflex_engine::UpsertOutcome::Unchanged { .. }
    ));
    assert_eq!(
        serde_json::to_vec(&snapshot).expect("serialize"),
        after_insert
    );

    let merged = snapshot
        .merge_evidence(
            assertion("source:one").id(),
            EvidenceSet::singleton(Evidence::create("source:two", None, None).expect("evidence"))
                .expect("evidence set"),
            &catalog,
        )
        .expect("merge");
    assert_eq!(merged, 1);
    let before_failure = serde_json::to_vec(&snapshot).expect("serialize");

    let error = snapshot
        .merge_evidence(
            &lexflex_model::AssertionId::new_unchecked("assertion:missing"),
            EvidenceSet::singleton(
                Evidence::create("source:missing", None, None).expect("evidence"),
            )
            .expect("evidence set"),
            &catalog,
        )
        .expect_err("missing assertion");
    assert!(matches!(
        error,
        lexflex_engine::knowledge::snapshot::KnowledgeSnapshotError::MissingAssertion { .. }
    ));
    assert_eq!(
        serde_json::to_vec(&snapshot).expect("serialize"),
        before_failure
    );

    snapshot.clear(&catalog).expect("clear");
    assert_eq!(snapshot.assertions().count(), 0);
    snapshot.verify(&catalog).expect("verified clear");
}
