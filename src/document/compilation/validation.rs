use super::model::{DocumentCompilation, SentenceProcessingStatus};
use super::provenance::ProvenanceOperation;
use crate::document::hash::sha256_span;
use crate::document::compilation::inspect::SemanticInspector;
use crate::document::id::SentenceId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentCompilationValidationError {
    MissingResult { sentence_id: SentenceId },
    UnexpectedResult { sentence_id: SentenceId },
    ResultSentenceIdMismatch { key: SentenceId, value: SentenceId },
    PendingResult { sentence_id: SentenceId },
    UsableStatusWithoutSemantics {
        sentence_id: SentenceId,
        status: SentenceProcessingStatus,
    },
    FailedStatusWithSemantics { sentence_id: SentenceId },
    SummaryTotalMismatch { expected: usize, actual: usize },
    SummaryTerminalMismatch { expected: usize, actual: usize },
    SummaryUsableMismatch { expected: usize, actual: usize },
    CompilationHashMismatch { expected: String, actual: String },
    DiagnosticReferencesUnknownSentence {
        diagnostic_id: String,
        sentence_id: SentenceId,
    },
    ResultDiagnosticSetMismatch {
        sentence_id: SentenceId,
        expected_ids: Vec<String>,
        actual_ids: Vec<String>,
    },
    ProvenanceSentenceMismatch {
        sentence_id: SentenceId,
        provenance_sentence_id: SentenceId,
    },
    ProvenanceSourceSpanMismatch { sentence_id: SentenceId },
    ProvenanceSourceHashMismatch {
        sentence_id: SentenceId,
        expected: String,
        actual: String,
    },
    DuplicateProvenanceStepId {
        sentence_id: SentenceId,
        step_id: String,
    },
    ProvenanceStepOrderMismatch {
        sentence_id: SentenceId,
        expected_ordinal: usize,
        actual_id: String,
    },
    MissingAnalysisStarted { sentence_id: SentenceId },
    MissingTerminalAnalysisOperation { sentence_id: SentenceId },
    EmptyAnalyzerId { sentence_id: SentenceId },
    MissingStatusClassified { sentence_id: SentenceId },
    InvalidAnalysisOperationSequence {
        sentence_id: SentenceId,
        expected: String,
        actual: String,
    },
    UnexpectedGenerationOperation {
        sentence_id: SentenceId,
        operation: String,
    },
    StatusClassificationDetailMismatch {
        sentence_id: SentenceId,
        expected: String,
        actual: Option<String>,
    },
    InspectionMismatch {
        sentence_id: SentenceId,
        expected: crate::document::compilation::SentenceSemanticInspection,
        actual: Option<crate::document::compilation::SentenceSemanticInspection>,
    },
    SourceSentenceIdMismatch { sentence_id: SentenceId },
}

pub struct DocumentCompilationValidator;

impl DocumentCompilationValidator {
    pub fn validate(
        compilation: &DocumentCompilation,
    ) -> Result<(), Vec<DocumentCompilationValidationError>> {
        let mut errors = Vec::new();
        validate_coverage(compilation, &mut errors);
        validate_results(compilation, &mut errors);
        validate_summary(compilation, &mut errors);
        validate_diagnostics(compilation, &mut errors);
        validate_provenance(compilation, &mut errors);
        validate_hash(compilation, &mut errors);
        sort_errors(&mut errors);
        errors.dedup();
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn validate_coverage(
    compilation: &DocumentCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    let document_ids: BTreeSet<_> = compilation.document.sentences().keys().cloned().collect();
    let result_ids: BTreeSet<_> = compilation.sentence_results.keys().cloned().collect();
    for sentence_id in document_ids.difference(&result_ids) {
        errors.push(DocumentCompilationValidationError::MissingResult {
            sentence_id: sentence_id.clone(),
        });
    }
    for sentence_id in result_ids.difference(&document_ids) {
        errors.push(DocumentCompilationValidationError::UnexpectedResult {
            sentence_id: sentence_id.clone(),
        });
    }
}

fn validate_results(
    compilation: &DocumentCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    for (key, result) in &compilation.sentence_results {
        if key != &result.sentence_id {
            errors.push(DocumentCompilationValidationError::ResultSentenceIdMismatch {
                key: key.clone(),
                value: result.sentence_id.clone(),
            });
        }
        if result.status == SentenceProcessingStatus::Pending {
            errors.push(DocumentCompilationValidationError::PendingResult {
                sentence_id: result.sentence_id.clone(),
            });
        }
        if result.status.is_usable() && result.semantics.is_none() {
            errors.push(DocumentCompilationValidationError::UsableStatusWithoutSemantics {
                sentence_id: result.sentence_id.clone(),
                status: result.status,
            });
        }
        if result.status == SentenceProcessingStatus::Failed && result.semantics.is_some() {
            errors.push(DocumentCompilationValidationError::FailedStatusWithSemantics {
                sentence_id: result.sentence_id.clone(),
            });
        }
    }
}

fn validate_summary(
    compilation: &DocumentCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    let summary = &compilation.summary;
    let expected_total = compilation.document.sentences().len();
    if summary.total_sentences != expected_total {
        errors.push(DocumentCompilationValidationError::SummaryTotalMismatch {
            expected: expected_total,
            actual: summary.total_sentences,
        });
    }
    if summary.terminal_count() != compilation.sentence_results.len() {
        errors.push(DocumentCompilationValidationError::SummaryTerminalMismatch {
            expected: compilation.sentence_results.len(),
            actual: summary.terminal_count(),
        });
    }
    let expected_usable = compilation
        .sentence_results
        .values()
        .filter(|result| result.status.is_usable())
        .count();
    if summary.usable != expected_usable {
        errors.push(DocumentCompilationValidationError::SummaryUsableMismatch {
            expected: expected_usable,
            actual: summary.usable,
        });
    }
}

fn validate_diagnostics(
    compilation: &DocumentCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    let sentence_ids: BTreeSet<_> = compilation.document.sentences().keys().cloned().collect();
    for diagnostic in &compilation.diagnostics {
        if let Some(sentence_id) = &diagnostic.sentence_id {
            if !sentence_ids.contains(sentence_id) {
                errors.push(DocumentCompilationValidationError::DiagnosticReferencesUnknownSentence {
                    diagnostic_id: diagnostic.id.as_str().to_string(),
                    sentence_id: sentence_id.clone(),
                });
            }
        }
    }
    for (sentence_id, result) in &compilation.sentence_results {
        let expected = canonical_diagnostics(
            &compilation.diagnostics,
            Some(sentence_id),
        );
        let actual = canonical_diagnostics(&result.diagnostics, Some(sentence_id));
        if expected != actual {
            errors.push(DocumentCompilationValidationError::ResultDiagnosticSetMismatch {
                sentence_id: sentence_id.clone(),
                expected_ids: expected.keys().cloned().collect(),
                actual_ids: actual.keys().cloned().collect(),
            });
        }
    }
}

fn validate_provenance(
    compilation: &DocumentCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    for (sentence_id, result) in &compilation.sentence_results {
        if result.provenance.analyzer_id.as_deref().unwrap_or("").trim().is_empty() {
            errors.push(DocumentCompilationValidationError::EmptyAnalyzerId {
                sentence_id: sentence_id.clone(),
            });
        }
        if result.provenance.sentence_id != *sentence_id {
            errors.push(DocumentCompilationValidationError::ProvenanceSentenceMismatch {
                sentence_id: sentence_id.clone(),
                provenance_sentence_id: result.provenance.sentence_id.clone(),
            });
        }
        if let Some(sentence) = compilation.document.sentence(sentence_id) {
            let Some(span) = sentence.content_span.span else {
                errors.push(DocumentCompilationValidationError::ProvenanceSourceSpanMismatch {
                    sentence_id: sentence_id.clone(),
                });
                continue;
            };
            if result.provenance.source_span != span {
                errors.push(DocumentCompilationValidationError::ProvenanceSourceSpanMismatch {
                    sentence_id: sentence_id.clone(),
                });
            }
            if let Ok(expected) = sha256_span(&compilation.document, span) {
                if expected != result.provenance.source_sha256 {
                    errors.push(DocumentCompilationValidationError::ProvenanceSourceHashMismatch {
                        sentence_id: sentence_id.clone(),
                        expected,
                        actual: result.provenance.source_sha256.clone(),
                    });
                }
            }
        }
        validate_steps(sentence_id, result, errors);
        validate_inspection(sentence_id, result, errors);
    }
}

fn validate_steps(
    sentence_id: &SentenceId,
    result: &crate::document::compilation::SentenceCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    let mut seen = BTreeSet::new();
    for (ordinal, step) in result.provenance.steps.iter().enumerate() {
        if !seen.insert(step.id.clone()) {
            errors.push(DocumentCompilationValidationError::DuplicateProvenanceStepId {
                sentence_id: sentence_id.clone(),
                step_id: step.id.as_str().to_string(),
            });
        }
        let expected = crate::document::DocumentIdFactory::sentence_provenance(sentence_id, ordinal);
        if step.id != expected {
            errors.push(DocumentCompilationValidationError::ProvenanceStepOrderMismatch {
                sentence_id: sentence_id.clone(),
                expected_ordinal: ordinal,
                actual_id: step.id.as_str().to_string(),
            });
        }
        if matches!(step.operation, ProvenanceOperation::GenerationStarted | ProvenanceOperation::GenerationSucceeded | ProvenanceOperation::GenerationFailed | ProvenanceOperation::GenerationSkipped | ProvenanceOperation::SourceFallbackApplied | ProvenanceOperation::PlaceholderApplied) {
            errors.push(DocumentCompilationValidationError::UnexpectedGenerationOperation {
                sentence_id: sentence_id.clone(),
                operation: format!("{:?}", step.operation),
            });
        }
    }
    let expected = expected_analysis_sequence(result);
    let actual: Vec<String> = result
        .provenance
        .steps
        .iter()
        .map(|step| format!("{:?}/{:?}", step.operation, step.outcome))
        .collect();
    if actual.len() < expected.len() {
        errors.push(DocumentCompilationValidationError::MissingAnalysisStarted {
            sentence_id: sentence_id.clone(),
        });
    }
    if actual != expected {
        errors.push(DocumentCompilationValidationError::InvalidAnalysisOperationSequence {
            sentence_id: sentence_id.clone(),
            expected: expected.join(" -> "),
            actual: actual.join(" -> "),
        });
    }
    if let Some(classified) = result
        .provenance
        .steps
        .iter()
        .find(|step| step.operation == ProvenanceOperation::StatusClassified)
    {
        let actual = classified.details.get("status").cloned();
        let expected = Some(sentence_status_name(result.status).to_string());
        if actual != expected {
            errors.push(DocumentCompilationValidationError::StatusClassificationDetailMismatch {
                sentence_id: sentence_id.clone(),
                expected: sentence_status_name(result.status).to_string(),
                actual,
            });
        }
    } else {
        errors.push(DocumentCompilationValidationError::MissingStatusClassified {
            sentence_id: sentence_id.clone(),
        });
    }
}

fn validate_inspection(
    sentence_id: &SentenceId,
    result: &crate::document::compilation::SentenceCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    match &result.semantics {
        Some(semantics) => {
            let expected = SemanticInspector::inspect(semantics);
            if result.inspection != expected {
                errors.push(DocumentCompilationValidationError::InspectionMismatch {
                    sentence_id: sentence_id.clone(),
                    expected,
                    actual: Some(result.inspection.clone()),
                });
            }
        }
        None => {
            if result.inspection != crate::document::compilation::SentenceSemanticInspection::default() {
                errors.push(DocumentCompilationValidationError::InspectionMismatch {
                    sentence_id: sentence_id.clone(),
                    expected: crate::document::compilation::SentenceSemanticInspection::default(),
                    actual: Some(result.inspection.clone()),
                });
            }
        }
    }
}

fn sentence_status_name(status: SentenceProcessingStatus) -> &'static str {
    match status {
        SentenceProcessingStatus::Pending => "Pending",
        SentenceProcessingStatus::Resolved => "Resolved",
        SentenceProcessingStatus::Partial => "Partial",
        SentenceProcessingStatus::Unresolved => "Unresolved",
        SentenceProcessingStatus::Failed => "Failed",
    }
}

fn expected_analysis_sequence(
    result: &crate::document::compilation::SentenceCompilation,
) -> Vec<String> {
    let terminal = if result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == crate::document::DOC_ANALYSIS_PANIC)
    {
        ProvenanceOperation::AnalysisPanicked
    } else if result.status == SentenceProcessingStatus::Failed {
        ProvenanceOperation::AnalysisFailed
    } else {
        ProvenanceOperation::AnalysisSucceeded
    };
    let terminal_outcome = if matches!(terminal, ProvenanceOperation::AnalysisSucceeded) {
        crate::document::compilation::ProvenanceOutcome::Succeeded
    } else {
        crate::document::compilation::ProvenanceOutcome::Failed
    };
    let mut expected = vec![
        format!("{:?}/{:?}", ProvenanceOperation::SentenceSelected, crate::document::compilation::ProvenanceOutcome::Started),
        format!("{:?}/{:?}", ProvenanceOperation::AnalysisStarted, crate::document::compilation::ProvenanceOutcome::Started),
    ];
    expected.push(format!(
        "{:?}/{:?}",
        terminal,
        terminal_outcome
    ));
    if matches!(terminal, ProvenanceOperation::AnalysisSucceeded) {
        expected.push(format!(
            "{:?}/{:?}",
            ProvenanceOperation::SemanticInspected,
            crate::document::compilation::ProvenanceOutcome::Succeeded
        ));
    }
    expected.push(format!(
        "{:?}/{:?}",
        ProvenanceOperation::StatusClassified,
        crate::document::compilation::ProvenanceOutcome::Succeeded
    ));
    expected
}

fn validate_hash(
    compilation: &DocumentCompilation,
    errors: &mut Vec<DocumentCompilationValidationError>,
) {
    let expected = super::hash::compilation_hash(compilation).unwrap_or_default();
    if expected != compilation.compilation_sha256 {
        errors.push(DocumentCompilationValidationError::CompilationHashMismatch {
            expected,
            actual: compilation.compilation_sha256.clone(),
        });
    }
}

fn sort_errors(errors: &mut [DocumentCompilationValidationError]) {
    errors.sort_by_key(|error| format!("{error:?}"));
}

fn canonical_diagnostics(
    diagnostics: &[crate::document::DocumentDiagnostic],
    sentence_id: Option<&SentenceId>,
) -> BTreeMap<String, crate::document::DocumentDiagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.sentence_id.as_ref() == sentence_id)
        .cloned()
        .map(|diagnostic| (diagnostic.id.as_str().to_string(), diagnostic))
        .collect()
}
