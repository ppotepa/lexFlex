//! Semantic digest — IL-derived actor/action/role/topic summary.

use lexflex::api::LexFlexAPI;
use lexflex::core::summary::{format_digest_human, summarize_utterance, SemanticDigest};

fn build_api() -> LexFlexAPI {
    LexFlexAPI::builder()
        .data_dir("data")
        .build()
        .expect("Failed to initialize lexFlex")
}

fn clause_agent<'a>(digest: &'a SemanticDigest, idx: usize) -> &'a lexflex::core::summary::RoleBinding {
    digest.clauses[idx]
        .roles
        .iter()
        .find(|r| r.role == "Agent")
        .unwrap_or_else(|| panic!("clause {idx} missing Agent role: {:?}", digest.clauses[idx]))
}

#[test]
fn test_digest_zero_anaphora_second_clause() {
    let api = build_api();
    let utt = api
        .parse_multi_sentence("Tomek poszedł. Kupił mleko.", "pl")
        .expect("parse");
    let digest = summarize_utterance(&utt);

    assert!(digest.clauses.len() >= 2);
    let agent = clause_agent(&digest, 1);
    assert_eq!(agent.name.as_deref(), Some("Tomek"));
    assert!(
        agent.reference.starts_with("Anaphoric"),
        "expected anaphoric agent, got {}",
        agent.reference
    );
    assert!(
        digest.clauses[1]
            .constructions
            .iter()
            .any(|c| c == "ZERO_ANAPHORA"),
        "expected ZERO_ANAPHORA, got {:?}",
        digest.clauses[1].constructions
    );
}

#[test]
fn test_digest_multi_clause_coordination_distinct_actors() {
    let api = build_api();
    let input = "Tomek poszedł po piwo, a moja mama kupiła pomidory.";
    let utt = api.parse_multi_sentence(input, "pl").expect("parse");
    let digest = summarize_utterance(&utt);

    assert!(digest.clauses.len() >= 2, "expected 2 clauses, got {}", digest.clauses.len());

    assert_eq!(digest.clauses[0].verb_concept, "GO");
    assert_eq!(digest.clauses[0].frame_type, "Motion");
    let c0_agent = clause_agent(&digest, 0);
    assert_eq!(c0_agent.name.as_deref(), Some("Tomek"));
    assert!(
        digest.clauses[0]
            .roles
            .iter()
            .any(|r| r.role == "Goal"),
        "clause 1 should have Goal role"
    );

    assert_eq!(digest.clauses[1].verb_concept, "BUY");
    assert_eq!(digest.clauses[1].frame_type, "Transfer");
    assert!(
        digest.clauses[1]
            .roles
            .iter()
            .any(|r| r.concept == "TOMATO" && r.name.as_deref() == Some("pomidory")),
        "clause 2 should expose TOMATO/pomidory role binding, got {:?}",
        digest.clauses[1].roles
    );
    assert!(
        digest.clauses[1]
            .roles
            .iter()
            .any(|r| r.name.as_deref() == Some("mama")),
        "clause 2 should expose mama entity, got {:?}",
        digest.clauses[1].roles
    );
    assert_ne!(c0_agent.name.as_deref(), Some("mama"));
}

#[test]
fn test_digest_json_roundtrip() {
    let api = build_api();
    let utt = api
        .parse_multi_sentence("Tomek poszedł do sklepu.", "pl")
        .expect("parse");
    let digest = summarize_utterance(&utt);
    let json = serde_json::to_string(&digest).expect("serialize");
    let back: SemanticDigest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.clauses.len(), digest.clauses.len());
    assert_eq!(back.clauses[0].verb_concept, "GO");
    assert!(
        back.clauses[0]
            .roles
            .iter()
            .any(|r| r.role == "Goal" && r.concept == "STORE")
    );
}

#[test]
fn test_explain_api_human_output_has_role_lines() {
    let api = build_api();
    let text = api
        .explain_human("Tomek poszedł do sklepu. Kupił mleko i wyszedł.", "pl")
        .expect("explain");
    assert!(text.contains("Actor:") || text.contains("Aktor:"));
    assert!(text.contains("GO") || text.contains("Motion"));
    assert!(!text.contains("ConstructionInstance"));
}

#[test]
fn test_explain_cli_deterministic_rich_discourse() {
    let api = build_api();
    let input = "Tomek poszedł do sklepu. Kupił mleko i wyszedł.";
    let d1 = api.explain_json(input, "pl").expect("run1");
    let d2 = api.explain_json(input, "pl").expect("run2");
    assert_eq!(d1, d2);

    let digest: SemanticDigest = serde_json::from_str(&d1).expect("json");
    assert!(digest.clauses.len() >= 2);
    let goal = digest.clauses[0]
        .roles
        .iter()
        .find(|r| r.role == "Goal")
        .expect("goal in clause 1");
    assert_eq!(goal.concept, "STORE");

    let leave = digest
        .clauses
        .iter()
        .find(|c| c.verb_concept == "LEAVE")
        .expect("leave verb clause");
    assert_eq!(leave.frame_type, "Motion");
}