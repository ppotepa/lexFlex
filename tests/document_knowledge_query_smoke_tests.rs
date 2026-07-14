use lexflex::api::LexFlexAPI;
use lexflex::document::knowledge::{DocumentKnowledgeService, DocumentKnowledgeValidator};
use lexflex::query::{
    AnswerStatus, QueryAggregation, QueryConstraint, QueryEvidencePolicy, QueryId, QueryIntent,
    QueryInterlingua, QueryProjection, QuerySchema, QueryService, QueryWorldPolicy,
};
use serde::Serialize;
use std::str::FromStr;

fn api() -> LexFlexAPI {
    LexFlexAPI::builder().data_dir("data").build().unwrap()
}

fn case_source() -> &'static str {
    include_str!("../benchmarks/document_v1/cases/dev/doc-pl-en-001/source.pl.txt")
}

fn hash<T: Serialize>(value: &T) -> String {
    let mut value = serde_json::to_value(value).unwrap();
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("query_sha256");
    }
    let bytes = serde_json::to_vec(&value).unwrap();
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn chapter7_knowledge_extraction_is_valid() {
    let api = api();
    let compilation = api.compile_document(case_source(), "pl").unwrap();
    let graph = api.build_document_graph(&compilation).unwrap();
    let resolution = api.resolve_document_graph(&graph).ok();
    let temporal = api
        .resolve_document_temporal_discourse(&compilation, &graph, resolution.as_ref())
        .ok();
    let knowledge = DocumentKnowledgeService::default()
        .extract(&compilation, &graph, resolution.as_ref(), temporal.as_ref())
        .unwrap();

    assert!(knowledge.summary.claims_total > 0);
    assert!(knowledge.summary.proposition_occurrences_total > 0);
    DocumentKnowledgeValidator::validate(&knowledge).unwrap();
}

#[test]
fn chapter8_query_answer_smoke_test() {
    let api = api();
    let compilation = api.compile_document(case_source(), "pl").unwrap();
    let graph = api.build_document_graph(&compilation).unwrap();
    let resolution = api.resolve_document_graph(&graph).ok();
    let temporal = api
        .resolve_document_temporal_discourse(&compilation, &graph, resolution.as_ref())
        .ok();
    let knowledge = DocumentKnowledgeService::default()
        .extract(&compilation, &graph, resolution.as_ref(), temporal.as_ref())
        .unwrap();

    let query = QueryInterlingua {
        schema: QuerySchema::V1,
        id: QueryId::from_str("query-smoke").unwrap(),
        source_language: "pl".to_string(),
        source_text: Some("Pokaż fakty".to_string()),
        intent: QueryIntent::Explain,
        constraints: QueryConstraint::EvidenceRequired(true),
        projection: vec![QueryProjection::Text],
        aggregation: None,
        ordering: vec![],
        limit: Some(10),
        world_policy: QueryWorldPolicy::OpenWorld,
        evidence_policy: QueryEvidencePolicy::Required,
        query_sha256: String::new(),
    };
    let query = QueryInterlingua {
        query_sha256: hash(&QueryInterlingua {
            query_sha256: String::new(),
            ..query.clone()
        }),
        ..query
    };

    let answer = QueryService::answer_document_query(query, &knowledge).unwrap();
    assert_ne!(answer.status, AnswerStatus::InvalidQuery);
    assert!(!answer.answer_sha256.is_empty());
    assert!(answer.text.as_deref().unwrap_or("").starts_with("matched "));
    assert!(answer
        .evidence
        .iter()
        .any(|value| value.starts_with("claim:")));
}

#[test]
fn chapter8_query_can_filter_and_count_claims() {
    let api = api();
    let compilation = api.compile_document(case_source(), "pl").unwrap();
    let graph = api.build_document_graph(&compilation).unwrap();
    let resolution = api.resolve_document_graph(&graph).ok();
    let temporal = api
        .resolve_document_temporal_discourse(&compilation, &graph, resolution.as_ref())
        .ok();
    let knowledge = DocumentKnowledgeService::default()
        .extract(&compilation, &graph, resolution.as_ref(), temporal.as_ref())
        .unwrap();

    let first_claim_id = knowledge.claim_order.first().cloned().expect("knowledge claim");
    let first_claim = knowledge.claims.get(&first_claim_id).expect("claim exists");
    let query = QueryInterlingua {
        schema: QuerySchema::V1,
        id: QueryId::from_str("query-count").unwrap(),
        source_language: "pl".to_string(),
        source_text: Some("Ile trafień".to_string()),
        intent: QueryIntent::Count,
        constraints: QueryConstraint::Claim(first_claim_id.clone()),
        projection: vec![QueryProjection::Claim],
        aggregation: Some(QueryAggregation::Count),
        ordering: vec![],
        limit: None,
        world_policy: QueryWorldPolicy::OpenWorld,
        evidence_policy: QueryEvidencePolicy::Required,
        query_sha256: String::new(),
    };
    let query = QueryInterlingua {
        query_sha256: hash(&QueryInterlingua {
            query_sha256: String::new(),
            ..query.clone()
        }),
        ..query
    };

    let answer = QueryService::answer_document_query(query, &knowledge).unwrap();
    assert_eq!(answer.status, AnswerStatus::Exact);
    assert_eq!(answer.rows.len(), 1);
    assert_eq!(answer.rows[0].columns.get("count"), Some(&"1".to_string()));
    assert_eq!(answer.rows[0].columns.get("mode"), Some(&"count".to_string()));
    assert!(answer
        .rows
        .iter()
        .all(|row| row.evidence.iter().any(|value| value == &format!("claim:{}", first_claim.id))));
}
