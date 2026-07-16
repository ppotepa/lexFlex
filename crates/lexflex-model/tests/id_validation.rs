use lexflex_model::ConceptId;

#[test]
fn invalid_id_is_rejected() {
    assert!(ConceptId::new("bad id").is_err());
}
