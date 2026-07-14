use lexflex::api::LexFlexAPI;
use lexflex::document::resolution::{
    DocumentEntityResolution, DocumentEntityResolutionQuery, DocumentEntityResolutionValidator,
    EntityResolutionDecisionKind, ResolutionMentionRef,
};

fn api() -> LexFlexAPI {
    LexFlexAPI::builder().data_dir("data").build().unwrap()
}

fn case_source() -> &'static str {
    include_str!("../benchmarks/document_v1/cases/dev/doc-pl-en-001/source.pl.txt")
}

#[test]
fn resolved_document_is_deterministic() {
    let api = api();
    let first = api
        .compile_resolved_document(case_source(), "pl")
        .unwrap();
    assert!(!first.resolution_sha256.is_empty());
    DocumentEntityResolutionValidator::validate(&first).unwrap();
}

#[test]
fn resolved_document_json_roundtrip_is_stable() {
    let api = api();
    let resolution = api
        .compile_resolved_document(case_source(), "pl")
        .unwrap();
    let json = resolution.to_canonical_json().unwrap();
    let roundtrip: DocumentEntityResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(resolution, roundtrip);
    assert_eq!(json, roundtrip.to_canonical_json().unwrap());
}

#[test]
fn seeded_mentions_expose_reusable_clusters() {
    let api = api();
    let resolution = api.compile_resolved_document(case_source(), "pl").unwrap();
    let query = DocumentEntityResolutionQuery::new(&resolution);

    let seeded_decision = resolution
        .decisions
        .values()
        .find(|decision| decision.kind == EntityResolutionDecisionKind::Seeded)
        .expect("expected at least one seeded decision");
    let mention = ResolutionMentionRef::Graph(
        resolution
            .mention_profiles
            .get(&seeded_decision.mention)
            .expect("seeded profile exists")
            .graph_mention_id
            .clone()
            .expect("seeded graph mention id"),
    );

    let cluster = query
        .cluster_for_mention(&mention)
        .expect("seeded mention should expose a reusable cluster");
    assert!(cluster.mention_refs.iter().any(|value| value == &mention));
}
