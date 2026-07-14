use crate::model::{
    CanonicalDocumentSemantics, ExpectationResult, MetricStatus, MetricValue, DocumentMetrics,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct MetricInputs<'a> {
    pub source: &'a str,
    pub output: Option<&'a str>,
    pub source_semantics: Option<&'a CanonicalDocumentSemantics>,
    pub target_semantics: Option<&'a CanonicalDocumentSemantics>,
    pub deterministic: bool,
    pub translation_error: bool,
    pub references: &'a [String],
    pub glossary_results: &'a [ExpectationResult],
    pub expectation_results: &'a [ExpectationResult],
    pub runtime_ms: u128,
}

pub fn empty_metrics() -> DocumentMetrics {
    DocumentMetrics {
        technical: BTreeMap::new(),
        structure: BTreeMap::new(),
        semantics: BTreeMap::new(),
        glossary: BTreeMap::new(),
        expectations: BTreeMap::new(),
        reference: BTreeMap::new(),
    }
}

pub fn metric(value: Option<f64>, numerator: Option<f64>, denominator: Option<f64>, status: MetricStatus) -> MetricValue {
    MetricValue {
        value,
        numerator,
        denominator,
        status,
    }
}

pub fn build_metrics(input: MetricInputs<'_>) -> DocumentMetrics {
    let mut metrics = empty_metrics();
    let output = input.output.unwrap_or("");
    let source_paragraphs = count_paragraphs(input.source);
    let target_paragraphs = count_paragraphs(output);
    let source_sentence_count = sentence_count(input.source, input.source_semantics);
    let target_sentence_count = sentence_count(output, input.target_semantics);

    insert_bool(
        &mut metrics.technical,
        "parse_source_success",
        input.source_semantics.is_some(),
    );
    insert_bool(
        &mut metrics.technical,
        "translation_success",
        input.output.map(|text| !text.trim().is_empty()).unwrap_or(false) && !input.translation_error,
    );
    insert_bool(
        &mut metrics.technical,
        "parse_target_success",
        input.target_semantics.is_some(),
    );
    insert_bool(
        &mut metrics.technical,
        "output_non_empty",
        input.output.map(|text| !text.trim().is_empty()).unwrap_or(false),
    );
    insert_bool(&mut metrics.technical, "deterministic", input.deterministic);
    metrics.technical.insert(
        "runtime_ms".into(),
        metric(Some(input.runtime_ms as f64), None, None, MetricStatus::Exact),
    );
    metrics.technical.insert(
        "output_bytes".into(),
        metric(Some(output.as_bytes().len() as f64), None, None, MetricStatus::Exact),
    );

    metrics.structure.insert(
        "source_paragraph_count".into(),
        metric(Some(source_paragraphs as f64), Some(source_paragraphs as f64), Some(source_paragraphs as f64), MetricStatus::Exact),
    );
    metrics.structure.insert(
        "target_paragraph_count".into(),
        metric(Some(target_paragraphs as f64), Some(target_paragraphs as f64), Some(target_paragraphs as f64), MetricStatus::Exact),
    );
    metrics.structure.insert(
        "paragraph_preservation_ratio".into(),
        ratio_metric(source_paragraphs, target_paragraphs),
    );
    metrics.structure.insert(
        "paragraph_preservation".into(),
        ratio_metric(source_paragraphs, target_paragraphs),
    );
    metrics.structure.insert(
        "source_sentence_count".into(),
        metric(Some(source_sentence_count as f64), Some(source_sentence_count as f64), Some(source_sentence_count as f64), MetricStatus::Exact),
    );
    metrics.structure.insert(
        "target_sentence_count".into(),
        metric(Some(target_sentence_count as f64), Some(target_sentence_count as f64), Some(target_sentence_count as f64), MetricStatus::Exact),
    );
    metrics.structure.insert(
        "sentence_count_ratio".into(),
        ratio_metric(source_sentence_count, target_sentence_count),
    );
    metrics.structure.insert(
        "sentence_retention".into(),
        ratio_metric(source_sentence_count, target_sentence_count),
    );

    let source_frames = frame_count(input.source_semantics);
    let target_frames = frame_count(input.target_semantics);
    metrics.semantics.insert(
        "source_frame_count".into(),
        metric(Some(source_frames as f64), Some(source_frames as f64), Some(source_frames as f64), MetricStatus::Exact),
    );
    metrics.semantics.insert(
        "target_frame_count".into(),
        metric(Some(target_frames as f64), Some(target_frames as f64), Some(target_frames as f64), MetricStatus::Exact),
    );
    metrics.semantics.insert(
        "frame_type_recall".into(),
        multiset_recall_metric(
            input.source_semantics.map(frame_types).unwrap_or_default(),
            input.target_semantics.map(frame_types).unwrap_or_default(),
        ),
    );
    metrics.semantics.insert(
        "frame_recall".into(),
        multiset_recall_metric(
            input.source_semantics.map(frame_types).unwrap_or_default(),
            input.target_semantics.map(frame_types).unwrap_or_default(),
        ),
    );
    metrics.semantics.insert(
        "verb_concept_recall".into(),
        multiset_recall_metric(
            input.source_semantics.map(verb_concepts).unwrap_or_default(),
            input.target_semantics.map(verb_concepts).unwrap_or_default(),
        ),
    );
    metrics.semantics.insert(
        "role_signature_recall".into(),
        multiset_recall_metric(
            input.source_semantics.map(role_signatures).unwrap_or_default(),
            input.target_semantics.map(role_signatures).unwrap_or_default(),
        ),
    );
    metrics.semantics.insert(
        "role_recall".into(),
        multiset_recall_metric(
            input.source_semantics.map(role_signatures).unwrap_or_default(),
            input.target_semantics.map(role_signatures).unwrap_or_default(),
        ),
    );
    metrics.semantics.insert(
        "proper_name_recall".into(),
        multiset_recall_metric(
            input.source_semantics.map(proper_names).unwrap_or_default(),
            input.target_semantics.map(proper_names).unwrap_or_default(),
        ),
    );
    metrics.semantics.insert(
        "polarity_preservation".into(),
        aligned_string_metric(input.source_semantics, input.target_semantics, |s| s.polarity.clone()),
    );
    metrics.semantics.insert(
        "tense_preservation".into(),
        aligned_optional_metric(input.source_semantics, input.target_semantics, |s| s.tense.clone()),
    );
    metrics.semantics.insert(
        "quantification_preservation".into(),
        aligned_optional_metric(input.source_semantics, input.target_semantics, |s| s.quantification.clone().map(|q| format!("{q:?}"))),
    );
    metrics.semantics.insert(
        "temporal_preservation".into(),
        aligned_optional_metric(input.source_semantics, input.target_semantics, |s| s.temporal.clone()),
    );
    metrics.semantics.insert(
        "unresolved_source_count".into(),
        metric(
            input.source_semantics.map(|sem| sem.unresolved_count as f64),
            None,
            None,
            MetricStatus::Exact,
        ),
    );
    metrics.semantics.insert(
        "unresolved_target_count".into(),
        metric(
            input.target_semantics.map(|sem| sem.unresolved_count as f64),
            None,
            None,
            MetricStatus::Exact,
        ),
    );

    let glossary_total = input.glossary_results.len();
    let glossary_passed = input.glossary_results.iter().filter(|result| result.passed).count();
    metrics.glossary.insert(
        "glossary_required".into(),
        metric(Some(glossary_total as f64), Some(glossary_total as f64), Some(glossary_total as f64), MetricStatus::Exact),
    );
    metrics.glossary.insert(
        "glossary_satisfied".into(),
        metric(Some(glossary_passed as f64), Some(glossary_passed as f64), Some(glossary_total as f64), MetricStatus::Exact),
    );
    metrics.glossary.insert(
        "glossary_compliance".into(),
        ratio_metric(glossary_passed, glossary_total),
    );

    let expectation_total = input.expectation_results.len();
    let expectation_passed = input.expectation_results.iter().filter(|result| result.passed).count();
    let expectation_known_failed = input
        .expectation_results
        .iter()
        .filter(|result| result.known_failure.is_some())
        .count();
    metrics.expectations.insert(
        "expectation_total".into(),
        metric(Some(expectation_total as f64), Some(expectation_total as f64), Some(expectation_total as f64), MetricStatus::Exact),
    );
    metrics.expectations.insert(
        "expectation_passed".into(),
        metric(Some(expectation_passed as f64), Some(expectation_passed as f64), Some(expectation_total as f64), MetricStatus::Exact),
    );
    metrics.expectations.insert(
        "expectation_failed".into(),
        metric(
            Some((expectation_total - expectation_passed) as f64),
            Some((expectation_total - expectation_passed) as f64),
            Some(expectation_total as f64),
            MetricStatus::Exact,
        ),
    );
    metrics.expectations.insert(
        "expectation_known_failed".into(),
        metric(
            Some(expectation_known_failed as f64),
            Some(expectation_known_failed as f64),
            Some(expectation_total as f64),
            MetricStatus::Exact,
        ),
    );

    let normalized_references = input
        .references
        .iter()
        .map(|reference| normalize_text(reference))
        .collect::<Vec<_>>();
    let exact_match = normalized_references
        .iter()
        .any(|reference| *reference == normalize_text(output));
    let token_overlap = token_overlap_ratio(input.source, output);
    metrics.reference.insert(
        "normalized_exact_match_any_reference".into(),
        metric(
            Some(exact_match as u8 as f64),
            Some(exact_match as u8 as f64),
            Some(1.0),
            MetricStatus::Exact,
        ),
    );
    metrics.reference.insert(
        "simple_token_overlap".into(),
        token_overlap,
    );

    metrics
}

fn insert_bool(map: &mut BTreeMap<String, MetricValue>, key: &str, value: bool) {
    map.insert(
        key.into(),
        metric(Some(value as u8 as f64), Some(value as u8 as f64), Some(1.0), MetricStatus::Exact),
    );
}

fn ratio_metric(numerator: usize, denominator: usize) -> MetricValue {
    let value = if numerator == 0 && denominator == 0 {
        1.0
    } else if numerator == 0 || denominator == 0 {
        0.0
    } else {
        numerator.min(denominator) as f64 / numerator.max(denominator) as f64
    };
    metric(
        Some(value),
        Some(numerator as f64),
        Some(denominator as f64),
        if denominator == 0 { MetricStatus::Unavailable } else { MetricStatus::Exact },
    )
}

fn ratio_value(numerator: usize, denominator: usize) -> f64 {
    if numerator == 0 && denominator == 0 {
        1.0
    } else if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn multiset_recall_metric(source: Vec<String>, target: Vec<String>) -> MetricValue {
    if source.is_empty() {
        return metric(Some(1.0), Some(0.0), Some(0.0), MetricStatus::Unavailable);
    }
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut target_counts = BTreeMap::<String, usize>::new();
    for item in source {
        *source_counts.entry(item).or_insert(0) += 1;
    }
    for item in target {
        *target_counts.entry(item).or_insert(0) += 1;
    }
    let denominator = source_counts.values().sum::<usize>();
    let numerator = source_counts
        .into_iter()
        .map(|(item, source_count)| source_count.min(*target_counts.get(&item).unwrap_or(&0)))
        .sum::<usize>();
    metric(
        Some(ratio_value(numerator, denominator)),
        Some(numerator as f64),
        Some(denominator as f64),
        if denominator == 0 { MetricStatus::Unavailable } else { MetricStatus::Exact },
    )
}

fn aligned_string_metric<F>(
    source: Option<&CanonicalDocumentSemantics>,
    target: Option<&CanonicalDocumentSemantics>,
    getter: F,
) -> MetricValue
where
    F: Fn(&crate::model::CanonicalSentence) -> String,
{
    aligned_metric(source, target, getter, |a, b| a == b)
}

fn aligned_optional_metric<F, T>(
    source: Option<&CanonicalDocumentSemantics>,
    target: Option<&CanonicalDocumentSemantics>,
    getter: F,
) -> MetricValue
where
    F: Fn(&crate::model::CanonicalSentence) -> Option<T>,
    T: PartialEq,
{
    aligned_metric(source, target, getter, |a, b| a == b)
}

fn aligned_metric<F, T, C>(
    source: Option<&CanonicalDocumentSemantics>,
    target: Option<&CanonicalDocumentSemantics>,
    getter: F,
    cmp: C,
) -> MetricValue
where
    F: Fn(&crate::model::CanonicalSentence) -> T,
    C: Fn(&T, &T) -> bool,
{
    let Some(source) = source else {
        return metric(None, None, None, MetricStatus::Unavailable);
    };
    let Some(target) = target else {
        return metric(None, None, None, MetricStatus::Unavailable);
    };
    let limit = source.sentences.len().min(target.sentences.len());
    if limit == 0 {
        return metric(Some(1.0), Some(0.0), Some(0.0), MetricStatus::Unavailable);
    }
    let mut matches = 0usize;
    for index in 0..limit {
        let a = getter(&source.sentences[index]);
        let b = getter(&target.sentences[index]);
        if cmp(&a, &b) {
            matches += 1;
        }
    }
    let status = if source.sentences.len() == target.sentences.len() {
        MetricStatus::Exact
    } else {
        MetricStatus::Approximate
    };
    metric(
        Some(ratio_value(matches, limit)),
        Some(matches as f64),
        Some(limit as f64),
        status,
    )
}

fn frame_count(semantics: Option<&CanonicalDocumentSemantics>) -> usize {
    semantics
        .map(|semantics| semantics.sentences.iter().map(|sentence| sentence.frames.len()).sum())
        .unwrap_or_default()
}

fn frame_types(semantics: &CanonicalDocumentSemantics) -> Vec<String> {
    semantics
        .sentences
        .iter()
        .flat_map(|sentence| sentence.frames.iter())
        .map(|frame| frame.frame_type.clone())
        .collect()
}

fn verb_concepts(semantics: &CanonicalDocumentSemantics) -> Vec<String> {
    semantics
        .sentences
        .iter()
        .flat_map(|sentence| sentence.frames.iter())
        .map(|frame| frame.verb_concept.clone())
        .collect()
}

fn role_signatures(semantics: &CanonicalDocumentSemantics) -> Vec<String> {
    semantics
        .sentences
        .iter()
        .flat_map(|sentence| sentence.frames.iter())
        .flat_map(|frame| {
            frame.roles.iter().map(|role| {
                format!(
                    "{}|{}|{}|{}",
                    frame.frame_type,
                    role.role,
                    role.concept,
                    role.name.clone().unwrap_or_else(|| role.reference.clone())
                )
            })
        })
        .collect()
}

fn proper_names(semantics: &CanonicalDocumentSemantics) -> Vec<String> {
    semantics
        .entities
        .iter()
        .filter_map(|entity| entity.name.as_ref())
        .filter(|name| name.chars().next().map(|ch| ch.is_uppercase()).unwrap_or(false))
        .map(|name| normalize_text(name))
        .collect()
}

fn count_paragraphs(text: &str) -> usize {
    text.split("\n\n").filter(|part| !part.trim().is_empty()).count()
}

fn sentence_count(text: &str, semantics: Option<&CanonicalDocumentSemantics>) -> usize {
    semantics
        .map(|semantics| semantics.sentences.len())
        .unwrap_or_else(|| {
            text.split_terminator(['.', '!', '?'])
                .filter(|part| !part.trim().is_empty())
                .count()
        })
}

fn token_overlap_ratio(source: &str, output: &str) -> MetricValue {
    let source_tokens = tokens(source);
    let output_tokens = tokens(output);
    if source_tokens.is_empty() && output_tokens.is_empty() {
        return metric(Some(1.0), Some(0.0), Some(0.0), MetricStatus::Unavailable);
    }
    let source_set = source_tokens.into_iter().collect::<BTreeSet<_>>();
    let output_set = output_tokens.into_iter().collect::<BTreeSet<_>>();
    let overlap = source_set.intersection(&output_set).count();
    let denominator = source_set.len();
    metric(
        Some(ratio_value(overlap, denominator)),
        Some(overlap as f64),
        Some(denominator as f64),
        if denominator == 0 { MetricStatus::Unavailable } else { MetricStatus::Approximate },
    )
}

fn tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|ch: char| !ch.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn normalize_text(text: &str) -> String {
    tokens(text).join(" ")
}
