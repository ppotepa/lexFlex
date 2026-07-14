#[path = "expectations_helpers.rs"]
mod expectations_helpers;
use expectations_helpers as helpers;

use crate::corpus::LoadedDocumentCase;
use crate::model::{
    CanonicalDocumentSemantics, DocumentInvariant, ExpectationResult,
};
use expectations_helpers::{
    evidence_for, invariant_kind, known_failure, preserve_name, preserve_negation,
    preserve_number, require_frame, require_paragraph_count, require_reference,
    require_role_binding, require_temporal, result,
};

pub fn evaluate_expectations(
    case: &LoadedDocumentCase,
    output: Option<&str>,
    source_semantics: Option<&CanonicalDocumentSemantics>,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> Vec<ExpectationResult> {
    let mut results = Vec::new();
    let output_text = output.unwrap_or("");

    for (idx, invariant) in case.expectations.invariants.iter().enumerate() {
        let known_failure = known_failure(case, idx, invariant_kind(invariant));
        let evidence = evidence_for(invariant, output_text, source_semantics, target_semantics);
        let passed = match invariant {
            DocumentInvariant::PreserveName {
                source,
                accepted_targets,
                minimum_occurrences,
            } => preserve_name(
                source,
                accepted_targets,
                *minimum_occurrences,
                output_text,
                target_semantics,
            ),
            DocumentInvariant::PreserveNumber {
                source_surface,
                normalized_value,
                accepted_targets,
            } => preserve_number(
                source_surface,
                normalized_value,
                accepted_targets,
                output_text,
                target_semantics,
            ),
            DocumentInvariant::PreserveNegation {
                sentence_hint,
                predicate_concept,
            } => preserve_negation(
                *sentence_hint,
                predicate_concept.as_deref(),
                source_semantics,
                target_semantics,
                output_text,
            ),
            DocumentInvariant::RequireFrame {
                frame_type,
                verb_concept,
                minimum_occurrences,
            } => require_frame(
                frame_type,
                verb_concept.as_deref(),
                *minimum_occurrences,
                target_semantics,
            ),
            DocumentInvariant::RequireRoleBinding {
                frame_type,
                role,
                concept,
                name,
                minimum_occurrences,
            } => require_role_binding(
                frame_type,
                role,
                concept,
                name.as_deref(),
                *minimum_occurrences,
                target_semantics,
            ),
            DocumentInvariant::RequireReference {
                mention,
                antecedent_name,
                target_forms,
            } => require_reference(
                mention,
                antecedent_name,
                target_forms,
                output_text,
                target_semantics,
            ),
            DocumentInvariant::RequireTemporal { kind, target_forms } => {
                require_temporal(kind, target_forms, output_text, target_semantics)
            }
            DocumentInvariant::RequireParagraphCount { count } => {
                require_paragraph_count(output_text, *count)
            }
            DocumentInvariant::RequireOutputContains {
                any_of,
                case_sensitive,
            } => helpers::contains_any(output_text, any_of, *case_sensitive),
            DocumentInvariant::ForbidOutputContains {
                any_of,
                case_sensitive,
            } => !helpers::contains_any(output_text, any_of, *case_sensitive),
        };
        results.push(result(
            idx,
            invariant_kind(invariant),
            passed,
            known_failure,
            evidence,
        ));
    }

    if let Some(expected) = case.expectations.expected_paragraph_count {
        let idx = results.len();
        let actual = helpers::count_paragraphs(output_text);
        results.push(result(
            idx,
            "ExpectedParagraphCount",
            actual == expected,
            None,
            vec![format!("expected={expected}"), format!("actual={actual}")],
        ));
    }

    if let Some(expected) = case.expectations.expected_min_sentence_count {
        let idx = results.len();
        let actual = helpers::count_sentences(output_text, target_semantics);
        results.push(result(
            idx,
            "ExpectedMinSentenceCount",
            actual >= expected,
            None,
            vec![format!("expected_min={expected}"), format!("actual={actual}")],
        ));
    }

    results
}
