use lexflex_model::{canonical_hash, Evidence, EvidenceError, SourceSpan};

#[test]
fn create_produces_verifiable_evidence() {
    let evidence = Evidence::create(
        "source:test",
        Some(SourceSpan::new(0, 4).expect("span")),
        Some(canonical_hash("text").expect("hash")),
    ).expect("evidence");
    assert_eq!(evidence.verify(), Ok(()));
}

#[test]
fn changed_source_id_causes_id_mismatch() {
    let mut evidence = Evidence::create("source:test", None, None).expect("evidence");
    evidence.source_id = "source:other".into();
    assert!(matches!(evidence.verify(), Err(EvidenceError::IdMismatch { .. })));
}
