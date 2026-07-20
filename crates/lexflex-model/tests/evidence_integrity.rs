use lexflex_model::{canonical_hash, Evidence, SourceSpan};

#[test]
fn create_produces_verifiable_evidence() {
    let evidence = Evidence::create(
        "source:test",
        Some(SourceSpan::new(0, 4).expect("span")),
        Some(canonical_hash("text").expect("hash")),
    )
    .expect("evidence");
    assert_eq!(evidence.verify(), Ok(()));
}

#[test]
fn changed_source_id_causes_id_mismatch() {
    let evidence = Evidence::create("source:test", None, None).expect("evidence");
    let mut value = serde_json::to_value(evidence).expect("json");
    value["source_id"] = serde_json::Value::String("source:other".into());
    assert!(serde_json::from_value::<Evidence>(value).is_err());
}
