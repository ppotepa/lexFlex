use lexflex_model::{
    ConceptCatalog, Evidence, EvidenceSet, SemanticAssertion, SemanticExpression, SemanticValue,
    WorldId,
};
use serde_json::Value;

fn assertion() -> SemanticAssertion {
    SemanticAssertion::create(
        SemanticExpression::Equals {
            left: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
            right: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
        },
        EvidenceSet::singleton(Evidence::create("source:serde", None, None).expect("evidence"))
            .expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &ConceptCatalog::default(),
    )
    .expect("valid assertion")
}

#[test]
fn valid_assertion_round_trips() {
    let assertion = assertion();
    let encoded = serde_json::to_string(&assertion).expect("serialize");
    let decoded: SemanticAssertion = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded, assertion);
}

#[test]
fn wrong_assertion_id_is_rejected() {
    let mut value = serde_json::to_value(assertion()).expect("json");
    value["id"] = Value::String("assertion:wrong".into());

    let result = serde_json::from_value::<SemanticAssertion>(value);
    assert!(result.is_err());
}

#[test]
fn wrong_assertion_hash_is_rejected() {
    let mut value = serde_json::to_value(assertion()).expect("json");
    value["canonical_hash"] =
        Value::String("0000000000000000000000000000000000000000000000000000000000000000".into());

    let result = serde_json::from_value::<SemanticAssertion>(value);
    assert!(result.is_err());
}

#[test]
fn nested_invalid_evidence_set_is_rejected() {
    let mut value = serde_json::to_value(assertion()).expect("json");
    let evidence = value["evidence"].as_object_mut().expect("evidence map");
    let first_key = evidence.keys().next().cloned().expect("evidence key");
    let first_value = evidence.remove(&first_key).expect("evidence value");
    evidence.insert("evidence:wrong".into(), first_value);

    let result = serde_json::from_value::<SemanticAssertion>(value);
    assert!(result.is_err());
}
