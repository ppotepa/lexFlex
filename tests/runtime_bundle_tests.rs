use lexflex::api::LexFlexAPI;
use lexflex::runtime::LexFlexDocumentEngine;

#[test]
fn runtime_ingests_one_immutable_document_bundle() {
    let api = LexFlexAPI::builder()
        .data_dir("data")
        .build()
        .expect("API should build");
    let engine = LexFlexDocumentEngine::new(api);
    let bundle = engine
        .ingest_document_bundle("Paris is a city.", "en")
        .expect("bundle should build");

    assert_eq!(bundle.source_sha256, bundle.compilation.document.source_sha256);
    assert!(!bundle.bundle_sha256.is_empty());
    assert_eq!(bundle.bundle_sha256, bundle.recompute_hash().unwrap());
    assert!(!bundle.graph.nodes.is_empty());
    assert!(bundle.knowledge.summary.claims_total > 0);
    assert!(bundle.validate_lineage().is_ok());
}
