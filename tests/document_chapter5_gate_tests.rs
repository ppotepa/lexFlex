use lexflex::api::LexFlexAPI;
use lexflex::document::resolution::{DocumentEntityResolutionValidator, EntityResolutionDecisionKind};

fn api() -> LexFlexAPI {
    LexFlexAPI::builder().data_dir("data").build().unwrap()
}

fn case_source() -> &'static str {
    include_str!("../benchmarks/document_v1/cases/dev/doc-pl-en-001/source.pl.txt")
}

#[test]
fn chapter5_gate_validates_against_graph() {
    let api = api();
    let compilation = api.compile_document(case_source(), "pl").unwrap();
    let graph = api.build_document_graph(&compilation).unwrap();
    let resolution = api.resolve_document_graph(&graph).unwrap();
    DocumentEntityResolutionValidator::validate_against_graph(&resolution, &graph).unwrap();
}

#[test]
fn chapter5_decisions_respect_cluster_invariants() {
    let api = api();
    let graph = api.compile_document_graph(case_source(), "pl").unwrap();
    let resolution = api.resolve_document_graph(&graph).unwrap();

    for decision in resolution.decisions.values() {
        match decision.kind {
            EntityResolutionDecisionKind::Seeded
            | EntityResolutionDecisionKind::Ambiguous
            | EntityResolutionDecisionKind::Deferred
            | EntityResolutionDecisionKind::Unresolved
            | EntityResolutionDecisionKind::Excluded => {
                assert!(decision.selected_cluster.is_none(), "{:?}", decision);
            }
            EntityResolutionDecisionKind::Accepted | EntityResolutionDecisionKind::HardAccepted => {
                assert!(decision.selected_cluster.is_some(), "{:?}", decision);
            }
        }
    }
}
