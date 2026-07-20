use lexflex_model::{Evidence, SourceSpan};

#[test]
fn valid_evidence_round_trips() {
    let evidence = Evidence::create(
        "source:evidence:serde",
        Some(SourceSpan::new(0, 4).expect("valid span")),
        None,
    )
    .expect("valid evidence");
    let encoded = serde_json::to_string(&evidence).expect("serialize");
    let decoded: Evidence = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded.id(), evidence.id());
    assert_eq!(decoded.source_id(), "source:evidence:serde");
}

#[test]
fn empty_source_id_deserialize_fails() {
    let mut value =
        serde_json::to_value(Evidence::create("source:ok", None, None).expect("evidence"))
            .expect("json");
    value["source_id"] = serde_json::Value::String(String::new());

    assert!(serde_json::from_value::<Evidence>(value).is_err());
}

#[test]
fn wrong_evidence_id_deserialize_fails() {
    let mut value =
        serde_json::to_value(Evidence::create("source:ok", None, None).expect("evidence"))
            .expect("json");
    value["id"] = serde_json::Value::String("evidence:wrong".into());

    assert!(serde_json::from_value::<Evidence>(value).is_err());
}

#[test]
fn invalid_nested_span_deserialize_fails() {
    let mut value =
        serde_json::to_value(Evidence::create("source:ok", None, None).expect("evidence"))
            .expect("json");
    value["span"] = serde_json::json!({ "start": 4, "end": 1 });

    assert!(serde_json::from_value::<Evidence>(value).is_err());
}
