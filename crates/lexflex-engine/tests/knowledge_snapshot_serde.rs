use lexflex_engine::KnowledgeSnapshot;
use lexflex_model::{
    ConceptCatalog, Evidence, EvidenceSet, SemanticAssertion, SemanticExpression, SemanticValue,
    WorldId,
};

fn assertion() -> SemanticAssertion {
    SemanticAssertion::create(
        SemanticExpression::Equals {
            left: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
            right: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
        },
        EvidenceSet::singleton(
            Evidence::create("source:snapshot:serde", None, None).expect("evidence"),
        )
        .expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &ConceptCatalog::default(),
    )
    .expect("valid assertion")
}

#[test]
fn valid_snapshot_round_trips() {
    let mut snapshot = KnowledgeSnapshot::new().expect("snapshot");
    snapshot
        .upsert(assertion(), &ConceptCatalog::default())
        .expect("insert");

    let encoded = serde_json::to_string(&snapshot).expect("serialize");
    let decoded: KnowledgeSnapshot = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded.snapshot_hash(), snapshot.snapshot_hash());
    assert_eq!(decoded.len(), 1);
}

#[test]
fn wrong_snapshot_hash_deserialize_fails() {
    let snapshot = KnowledgeSnapshot::new().expect("snapshot");
    let mut value = serde_json::to_value(snapshot).expect("json");
    value["snapshot_hash"] = serde_json::Value::String(
        "0000000000000000000000000000000000000000000000000000000000000000".into(),
    );

    assert!(serde_json::from_value::<KnowledgeSnapshot>(value).is_err());
}

#[test]
fn wrong_assertion_key_deserialize_fails() {
    let mut snapshot = KnowledgeSnapshot::new().expect("snapshot");
    snapshot
        .upsert(assertion(), &ConceptCatalog::default())
        .expect("insert");
    let mut value = serde_json::to_value(snapshot).expect("json");
    let assertions = value["assertions"].as_object_mut().expect("assertion map");
    let first_key = assertions.keys().next().cloned().expect("assertion key");
    let first_value = assertions.remove(&first_key).expect("assertion value");
    assertions.insert("assertion:wrong".into(), first_value);

    assert!(serde_json::from_value::<KnowledgeSnapshot>(value).is_err());
}
