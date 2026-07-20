use lexflex_engine::KnowledgeSnapshot;

#[test]
fn empty_snapshot_verifies() {
    let snapshot = KnowledgeSnapshot::new().expect("snapshot");
    assert_eq!(
        snapshot.verify(&lexflex_model::ConceptCatalog::default()),
        Ok(())
    );
}
