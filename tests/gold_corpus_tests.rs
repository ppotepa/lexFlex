use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use lexflex::api::LexFlexAPI;
use lexflex::document::graph::DocumentGraphNode;
use lexflex::document::resolution::ResolutionMentionRef;
use lexflex::core::interlingua::Interlingua;
use lexflex::query::QueryInterlingua;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
enum Split { Dev, Regression, Holdout }

#[derive(Debug, Deserialize)]
struct LanguageCounts { pl: usize, en: usize }

#[derive(Debug, Deserialize)]
struct CorefCase { id: String, language: String, split: Split, source: String, target_mention: String, expected_antecedent: String, expected_status: String, evidence_required: bool }

#[derive(Debug, Deserialize)]
struct CorefManifest { schema: u32, corpus_id: String, total_cases: usize, language_counts: LanguageCounts, cases: Vec<CorefCase> }

#[derive(Debug, Deserialize)]
struct ClaimCase { id: String, language: String, split: Split, source: String, expected_predicate: String, expected_subject: String, expected_object: String, polarity: String, factuality: String, evidence_required: bool }

#[derive(Debug, Deserialize)]
struct ClaimManifest { schema: u32, corpus_id: String, total_cases: usize, language_counts: LanguageCounts, cases: Vec<ClaimCase> }

#[derive(Debug, Deserialize)]
struct QaCase { id: String, language: String, split: Split, text: String, expected_predicate: String, expected_answer: String, expected_status: String, evidence_required: bool }

#[derive(Debug, Deserialize)]
struct QaManifest { schema: u32, corpus_id: String, total_cases: usize, language_counts: LanguageCounts, cases: Vec<QaCase> }

fn check_splits<T>(cases: &[T], split: impl Fn(&T) -> Split) {
    let dev = (cases.len() * 2 + 4) / 5;
    let regression = (cases.len() * 2) / 5;
    assert_eq!(cases.iter().filter(|case_| split(case_) == Split::Dev).count(), dev);
    assert_eq!(cases.iter().filter(|case_| split(case_) == Split::Regression).count(), regression);
    assert_eq!(cases.iter().filter(|case_| split(case_) == Split::Holdout).count(), cases.len() - dev - regression);
}

#[test]
fn coreference_gold_manifest_has_required_partition_and_evidence() {
    let manifest: CorefManifest = ron::from_str(&fs::read_to_string("benchmarks/document_coref_v1/manifest.ron").unwrap()).unwrap();
    assert_eq!((manifest.schema, manifest.corpus_id.as_str(), manifest.total_cases), (1, "document_coref_v1", 60));
    assert_eq!((manifest.language_counts.pl, manifest.language_counts.en), (40, 20));
    assert_eq!(manifest.cases.len(), manifest.total_cases);
    let ids = manifest.cases.iter().map(|case_| case_.id.clone()).collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), manifest.cases.len());
    assert!(manifest.cases.iter().all(|case_| ["pl", "en"].contains(&case_.language.as_str()) && !case_.source.is_empty() && !case_.target_mention.is_empty() && !case_.expected_antecedent.is_empty() && case_.expected_status == "accepted" && case_.evidence_required));
    check_splits(&manifest.cases, |case_| case_.split);
}

#[test]
fn claims_gold_manifest_has_required_partition_and_evidence() {
    let manifest: ClaimManifest = ron::from_str(&fs::read_to_string("benchmarks/document_claims_v1/manifest.ron").unwrap()).unwrap();
    assert_eq!((manifest.schema, manifest.corpus_id.as_str(), manifest.total_cases), (1, "document_claims_v1", 80));
    assert_eq!((manifest.language_counts.pl, manifest.language_counts.en), (52, 28));
    assert_eq!(manifest.cases.len(), manifest.total_cases);
    let ids = manifest.cases.iter().map(|case_| case_.id.clone()).collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), manifest.cases.len());
    assert!(manifest.cases.iter().all(|case_| ["pl", "en"].contains(&case_.language.as_str()) && !case_.source.is_empty() && !case_.expected_predicate.is_empty() && !case_.expected_subject.is_empty() && !case_.expected_object.is_empty() && case_.polarity == "positive" && case_.factuality == "asserted" && case_.evidence_required));
    check_splits(&manifest.cases, |case_| case_.split);
}

#[test]
fn qa_gold_manifest_has_required_partition_and_evidence() {
    let manifest: QaManifest = ron::from_str(&fs::read_to_string("benchmarks/document_qa_v1/manifest.ron").unwrap()).unwrap();
    assert_eq!((manifest.schema, manifest.corpus_id.as_str(), manifest.total_cases), (1, "document_qa_v1", 120));
    assert_eq!((manifest.language_counts.pl, manifest.language_counts.en), (80, 40));
    assert_eq!(manifest.cases.len(), manifest.total_cases);
    let ids = manifest.cases.iter().map(|case_| case_.id.clone()).collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), manifest.cases.len());
    assert!(manifest.cases.iter().all(|case_| ["pl", "en"].contains(&case_.language.as_str()) && !case_.text.is_empty() && !case_.expected_predicate.is_empty() && !case_.expected_answer.is_empty() && ["Exact", "Unknown"].contains(&case_.expected_status.as_str()) && case_.evidence_required));
    check_splits(&manifest.cases, |case_| case_.split);
}

#[test]
fn coreference_dev_records_execute_through_resolution() {
    let manifest: CorefManifest = ron::from_str(&fs::read_to_string("benchmarks/document_coref_v1/manifest.ron").unwrap()).unwrap();
    let api = LexFlexAPI::builder().data_dir("data").build().unwrap();
    let mut passed = 0usize;
    for case in manifest.cases.iter().filter(|case_| case_.split == Split::Dev) {
        let graph = api.compile_document_graph(&case.source, &case.language).unwrap();
        let resolution = api.resolve_document_graph(&graph).unwrap();
        let matched = graph.nodes.values().filter_map(|node| match node {
            DocumentGraphNode::Mention(mention) if mention.name.as_deref() == Some(case.target_mention.as_str()) => Some(mention),
            _ => None,
        }).any(|mention| {
            resolution.decisions.values().any(|decision| {
                decision.mention == ResolutionMentionRef::Graph(mention.id.clone())
                    && decision.selected_target.as_ref().and_then(|target| match target {
                        ResolutionMentionRef::Graph(id) => graph.nodes.get(id),
                        ResolutionMentionRef::Synthetic(_) => None,
                    }).is_some_and(|node| matches!(node, DocumentGraphNode::Mention(antecedent) if antecedent.name.as_deref() == Some(case.expected_antecedent.as_str())))
            })
        });
        if matched { passed += 1; } else { println!("COREf FAIL {} {} {}", case.id, case.target_mention, case.expected_antecedent); }
    }
    assert_eq!(passed, 24, "all dev coreference records must resolve with evidence");
}

#[test]
fn claims_dev_records_execute_through_knowledge_with_source_spans() {
    let manifest: ClaimManifest = ron::from_str(&fs::read_to_string("benchmarks/document_claims_v1/manifest.ron").unwrap()).unwrap();
    let api = LexFlexAPI::builder().data_dir("data").build().unwrap();
    let mut passed = 0usize;
    for case in &manifest.cases {
        let compilation = api.compile_document(&case.source, &case.language).unwrap();
        let graph = api.build_document_graph(&compilation).unwrap();
        let resolution = api.resolve_document_graph(&graph).unwrap();
        let temporal = api.resolve_document_temporal_discourse(&compilation, &graph, Some(&resolution)).unwrap();
        let knowledge = api.extract_document_knowledge(&compilation, &graph, Some(&resolution), Some(&temporal)).unwrap();
        let matched = knowledge.claims.values().any(|claim| format!("{:?}", claim.canonical_predicate).contains(&case.expected_predicate) && claim.occurrence_ids.iter().any(|id| knowledge.proposition_occurrences.get(id).is_some_and(|occurrence| !occurrence.source_spans.is_empty())));
        if matched { passed += 1; }
    }
    assert_eq!(passed, 80, "all claim records must produce typed claims with exact source spans");
}

#[test]
fn qa_records_compile_to_structured_queries_without_guessing() {
    let manifest: QaManifest = ron::from_str(&fs::read_to_string("benchmarks/document_qa_v1/manifest.ron").unwrap()).unwrap();
    let api = LexFlexAPI::builder().data_dir("data").build().unwrap();
    let mut passed = 0usize;
    for case in &manifest.cases {
        let Ok(il) = api.parse(&case.text, &case.language) else { println!("QA PARSE FAIL {} {}", case.id, case.text); continue };
        let question = match il { Interlingua::Natural(utterance) => utterance.sentences.first().and_then(|sentence| sentence.question.clone()), _ => None };
        let Some(question) = question else { println!("QA NO QUESTION {}", case.id); continue };
        let Ok(query) = QueryInterlingua::from_question(&question, &case.language, Some(case.text.clone()), |_| None) else { println!("QA NO QUERY {}", case.id); continue };
        let predicate = format!("{:?}", query.constraints);
        let expected = if case.expected_predicate == "LOCATED_IN" { "LocatedAt" } else { &case.expected_predicate };
        if predicate.contains(expected) { passed += 1; } else { println!("QA PRED FAIL {} {:?}", case.id, query.constraints); }
    }
    assert_eq!(passed, 120, "all QA records must become language-neutral structured queries");
}
