use super::model::{DocumentTranslation, SentenceTranslationStatus};
use super::preserved::canonical_preserved_blocks;
use crate::document::compilation::{DocumentCompilation, SentenceCompilation};
use crate::document::id::{DiagnosticId, SentenceId};
use crate::document::ProvenanceOperation;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentTranslationValidationError {
    MissingSentenceResult { sentence_id: SentenceId },
    UnexpectedSentenceResult { sentence_id: SentenceId },
    ResultSentenceIdMismatch { key: SentenceId, value: SentenceId },
    SourceDocumentIdMismatch {
        expected: String,
        actual: String,
    },
    SourceLanguageMismatch {
        expected: String,
        actual: String,
    },
    EmptyTargetLanguage,
    SameSourceAndTargetLanguage,
    EmptyGeneratedContent { sentence_id: SentenceId },
    SourceFallbackMismatch { sentence_id: SentenceId },
    ResultDiagnosticSetMismatch {
        sentence_id: SentenceId,
        expected_ids: Vec<DiagnosticId>,
        actual_ids: Vec<DiagnosticId>,
    },
    MissingGeneratorId { sentence_id: SentenceId },
    DuplicateProvenanceStepId { sentence_id: SentenceId, step_id: String },
    ProvenanceStepOrderMismatch {
        sentence_id: SentenceId,
        expected_ordinal: usize,
        actual_id: String,
    },
    ProvenancePrefixTooShort {
        sentence_id: SentenceId,
        expected_prefix_len: usize,
        actual_len: usize,
    },
    ProvenancePrefixMismatch {
        sentence_id: SentenceId,
        mismatch_ordinal: usize,
    },
    AnalyzerIdMismatch {
        sentence_id: SentenceId,
        expected: Option<String>,
        actual: Option<String>,
    },
    ProvenanceSentenceIdMismatch {
        sentence_id: SentenceId,
        actual: SentenceId,
    },
    ProvenanceSourceSpanMismatch { sentence_id: SentenceId },
    ProvenanceSourceHashMismatch {
        sentence_id: SentenceId,
        expected: String,
        actual: String,
    },
    InvalidGenerationSequence {
        sentence_id: SentenceId,
        expected: Vec<String>,
        actual: Vec<String>,
    },
    InvalidGenerationOutcome {
        sentence_id: SentenceId,
        operation: String,
        expected: String,
        actual: String,
    },
    MultipleGenerationTerminalOperations {
        sentence_id: SentenceId,
        count: usize,
    },
    UnexpectedGenerationOperation {
        sentence_id: SentenceId,
        operation: String,
    },
    WrongFallbackOperation {
        sentence_id: SentenceId,
        status: SentenceTranslationStatus,
    },
    PreservedBlockDigestUnavailable,
    SummaryTotalMismatch { expected: usize, actual: usize },
    SummaryCoverageMismatch { expected: usize, actual: usize },
    EmptyDocumentOutput,
    OutputHashMismatch { expected: String, actual: String },
    TranslationHashMismatch { expected: String, actual: String },
    CopiedBlockCountMismatch { expected: usize, actual: usize },
    CopiedBlockHashMismatch { expected: String, actual: String },
}

pub struct DocumentTranslationValidator;

impl DocumentTranslationValidator {
    pub fn validate(
        compilation: &DocumentCompilation,
        translation: &DocumentTranslation,
    ) -> Result<(), Vec<DocumentTranslationValidationError>> {
        let mut errors = Vec::new();
        validate_metadata(compilation, translation, &mut errors);
        validate_sentence_results(compilation, translation, &mut errors);
        validate_summary(compilation, translation, &mut errors);
        validate_output(compilation, translation, &mut errors);
        validate_provenance(compilation, translation, &mut errors);
        validate_hashes(translation, &mut errors);
        sort_errors(&mut errors);
        errors.dedup();
    if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn validate_metadata(
    compilation: &DocumentCompilation,
    translation: &DocumentTranslation,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    if translation.source_document_id != compilation.document.id {
        errors.push(DocumentTranslationValidationError::SourceDocumentIdMismatch {
            expected: compilation.document.id.as_str().to_string(),
            actual: translation.source_document_id.as_str().to_string(),
        });
    }
    let expected_source = compilation.document.source_language().0.clone();
    let actual_source = translation.source_language.0.clone();
    if actual_source.trim().is_empty() || expected_source != actual_source {
        errors.push(DocumentTranslationValidationError::SourceLanguageMismatch {
            expected: expected_source,
            actual: actual_source,
        });
    }
    if translation.target_language.0.trim().is_empty() {
        errors.push(DocumentTranslationValidationError::EmptyTargetLanguage);
    }
    if translation.target_language == translation.source_language {
        errors.push(DocumentTranslationValidationError::SameSourceAndTargetLanguage);
    }
}

fn validate_sentence_results(
    compilation: &DocumentCompilation,
    translation: &DocumentTranslation,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    let source_ids: BTreeSet<_> = compilation.document.sentences().keys().cloned().collect();
    let result_ids: BTreeSet<_> = translation.sentence_results.keys().cloned().collect();
    for sentence_id in source_ids.difference(&result_ids) {
        errors.push(DocumentTranslationValidationError::MissingSentenceResult {
            sentence_id: sentence_id.clone(),
        });
    }
    for sentence_id in result_ids.difference(&source_ids) {
        errors.push(DocumentTranslationValidationError::UnexpectedSentenceResult {
            sentence_id: sentence_id.clone(),
        });
    }
    for (key, result) in &translation.sentence_results {
        if key != &result.sentence_id {
            errors.push(DocumentTranslationValidationError::ResultSentenceIdMismatch {
                key: key.clone(),
                value: result.sentence_id.clone(),
            });
        }
        if result.generated_content.is_empty()
            && !compilation
            .document
            .sentence(&result.sentence_id)
            .map(|sentence| sentence.content_span.span.map(|span| span.is_empty()).unwrap_or(true))
            .unwrap_or(true)
        {
            errors.push(DocumentTranslationValidationError::EmptyGeneratedContent {
                sentence_id: result.sentence_id.clone(),
            });
        }
        if matches!(result.status, SentenceTranslationStatus::SourceFallback) {
            if let Some(sentence) = compilation.document.sentence(&result.sentence_id) {
                let expected = sentence
                    .content_span
                    .span
                    .and_then(|span| span.slice(compilation.document.source()).ok())
                    .unwrap_or("");
                if result.generated_content != expected {
                    errors.push(DocumentTranslationValidationError::SourceFallbackMismatch {
                        sentence_id: result.sentence_id.clone(),
                    });
                }
            }
        }
        if result.provenance.generator_id.as_deref().unwrap_or("").trim().is_empty() {
            errors.push(DocumentTranslationValidationError::MissingGeneratorId {
                sentence_id: result.sentence_id.clone(),
            });
        }
        let expected = diagnostics_for_sentence(&translation.diagnostics, &result.sentence_id);
        let actual = diagnostics_for_sentence(&result.diagnostics, &result.sentence_id);
        if expected != actual {
            errors.push(DocumentTranslationValidationError::ResultDiagnosticSetMismatch {
                sentence_id: result.sentence_id.clone(),
                expected_ids: expected.keys().cloned().collect(),
                actual_ids: actual.keys().cloned().collect(),
            });
        }
    }
}

fn validate_summary(
    compilation: &DocumentCompilation,
    translation: &DocumentTranslation,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    let total = compilation.document.sentences().len();
    if translation.summary.total_sentences != total {
        errors.push(DocumentTranslationValidationError::SummaryTotalMismatch {
            expected: total,
            actual: translation.summary.total_sentences,
        });
    }
    if translation.summary.coverage_count() != translation.sentence_results.len() {
        errors.push(DocumentTranslationValidationError::SummaryCoverageMismatch {
            expected: translation.sentence_results.len(),
            actual: translation.summary.coverage_count(),
        });
    }
}

fn validate_output(
    compilation: &DocumentCompilation,
    translation: &DocumentTranslation,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    if !compilation.document.source().is_empty() && translation.output.is_empty() {
        errors.push(DocumentTranslationValidationError::EmptyDocumentOutput);
    }
    let Ok(digest) = canonical_preserved_blocks(&compilation.document) else {
        errors.push(DocumentTranslationValidationError::PreservedBlockDigestUnavailable);
        return;
    };
    if translation.copied_block_count != digest.count {
        errors.push(DocumentTranslationValidationError::CopiedBlockCountMismatch {
            expected: digest.count,
            actual: translation.copied_block_count,
        });
    }
    if translation.copied_block_sha256 != digest.sha256 {
        errors.push(DocumentTranslationValidationError::CopiedBlockHashMismatch {
            expected: digest.sha256,
            actual: translation.copied_block_sha256.clone(),
        });
    }
}

fn validate_provenance(
    compilation: &DocumentCompilation,
    translation: &DocumentTranslation,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    for (sentence_id, result) in &translation.sentence_results {
        if let Some(sentence) = compilation.document.sentence(sentence_id) {
            validate_sentence_identity(compilation, sentence, result, errors);
        }
        let prefix_len = match compilation.sentence_results.get(sentence_id) {
            Some(compilation_result) => validate_generation_sequence(
                sentence_id,
                compilation_result,
                result,
                errors,
            ),
            None => 0,
        };
        validate_generation_suffix(sentence_id, result, prefix_len, errors);
    }
}

fn validate_hashes(
    translation: &DocumentTranslation,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    let output_hash = super::hash::translation_output_hash(&translation.output);
    if output_hash != translation.output_sha256 {
        errors.push(DocumentTranslationValidationError::OutputHashMismatch {
            expected: output_hash,
            actual: translation.output_sha256.clone(),
        });
    }
    let expected = super::hash::translation_hash(translation).unwrap_or_default();
    if expected != translation.translation_sha256 {
        errors.push(DocumentTranslationValidationError::TranslationHashMismatch {
            expected,
            actual: translation.translation_sha256.clone(),
        });
    }
}

fn validate_sentence_identity(
    compilation: &DocumentCompilation,
    sentence: &crate::document::DocumentSentence,
    result: &crate::document::translation::SentenceTranslationResult,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    if result.provenance.sentence_id != result.sentence_id {
        errors.push(DocumentTranslationValidationError::ProvenanceSentenceIdMismatch {
            sentence_id: result.sentence_id.clone(),
            actual: result.provenance.sentence_id.clone(),
        });
    }
    if result.provenance.analyzer_id != compilation
        .sentence_results
        .get(&result.sentence_id)
        .and_then(|r| r.provenance.analyzer_id.clone())
    {
        errors.push(DocumentTranslationValidationError::AnalyzerIdMismatch {
            sentence_id: result.sentence_id.clone(),
            expected: compilation
                .sentence_results
                .get(&result.sentence_id)
                .and_then(|r| r.provenance.analyzer_id.clone()),
            actual: result.provenance.analyzer_id.clone(),
        });
    }
    match sentence.content_span.span {
        Some(span) if result.provenance.source_span != span => {
            errors.push(DocumentTranslationValidationError::ProvenanceSourceSpanMismatch {
                sentence_id: result.sentence_id.clone(),
            });
        }
        None => {
            errors.push(DocumentTranslationValidationError::ProvenanceSourceSpanMismatch {
                sentence_id: result.sentence_id.clone(),
            });
        }
        _ => {}
    }
    if let Some(span) = sentence.content_span.span {
        if let Ok(expected) = crate::document::hash::sha256_span(&compilation.document, span) {
            if expected != result.provenance.source_sha256 {
                errors.push(DocumentTranslationValidationError::ProvenanceSourceHashMismatch {
                    sentence_id: result.sentence_id.clone(),
                    expected,
                    actual: result.provenance.source_sha256.clone(),
                });
            }
        }
    }
}

fn validate_generation_sequence(
    sentence_id: &SentenceId,
    compilation_result: &SentenceCompilation,
    result: &crate::document::translation::SentenceTranslationResult,
    errors: &mut Vec<DocumentTranslationValidationError>,
) -> usize {
    let prefix_len = compilation_result.provenance.steps.len();
    let steps = &result.provenance.steps;
    if steps.len() < prefix_len {
        errors.push(DocumentTranslationValidationError::ProvenancePrefixTooShort {
            sentence_id: sentence_id.clone(),
            expected_prefix_len: prefix_len,
            actual_len: steps.len(),
        });
        return prefix_len;
    }
    if steps[..prefix_len] != compilation_result.provenance.steps[..] {
        for (ordinal, (left, right)) in steps[..prefix_len]
            .iter()
            .zip(&compilation_result.provenance.steps)
            .enumerate()
        {
            if left != right {
                errors.push(DocumentTranslationValidationError::ProvenancePrefixMismatch {
                    sentence_id: sentence_id.clone(),
                    mismatch_ordinal: ordinal,
                });
                break;
            }
        }
    }
    prefix_len
}

fn validate_generation_suffix(
    sentence_id: &SentenceId,
    result: &crate::document::translation::SentenceTranslationResult,
    prefix_len: usize,
    errors: &mut Vec<DocumentTranslationValidationError>,
) {
    let suffix = &result.provenance.steps[prefix_len..];
    let expected = match result.status {
        SentenceTranslationStatus::Translated => vec![
            (ProvenanceOperation::GenerationStarted, crate::document::compilation::ProvenanceOutcome::Started),
            (ProvenanceOperation::GenerationSucceeded, crate::document::compilation::ProvenanceOutcome::Succeeded),
        ],
        SentenceTranslationStatus::SourceFallback => match suffix.first().map(|step| step.operation) {
            Some(ProvenanceOperation::GenerationStarted) => vec![
                (ProvenanceOperation::GenerationStarted, crate::document::compilation::ProvenanceOutcome::Started),
                (ProvenanceOperation::GenerationFailed, crate::document::compilation::ProvenanceOutcome::Failed),
                (ProvenanceOperation::SourceFallbackApplied, crate::document::compilation::ProvenanceOutcome::Recovered),
            ],
            Some(ProvenanceOperation::GenerationSkipped) => vec![
                (ProvenanceOperation::GenerationSkipped, crate::document::compilation::ProvenanceOutcome::Skipped),
                (ProvenanceOperation::SourceFallbackApplied, crate::document::compilation::ProvenanceOutcome::Recovered),
            ],
            _ => {
                errors.push(DocumentTranslationValidationError::WrongFallbackOperation {
                    sentence_id: sentence_id.clone(),
                    status: result.status,
                });
                return;
            }
        },
        SentenceTranslationStatus::Placeholder => match suffix.first().map(|step| step.operation) {
            Some(ProvenanceOperation::GenerationStarted) => vec![
                (ProvenanceOperation::GenerationStarted, crate::document::compilation::ProvenanceOutcome::Started),
                (ProvenanceOperation::GenerationFailed, crate::document::compilation::ProvenanceOutcome::Failed),
                (ProvenanceOperation::PlaceholderApplied, crate::document::compilation::ProvenanceOutcome::Recovered),
            ],
            Some(ProvenanceOperation::GenerationSkipped) => vec![
                (ProvenanceOperation::GenerationSkipped, crate::document::compilation::ProvenanceOutcome::Skipped),
                (ProvenanceOperation::PlaceholderApplied, crate::document::compilation::ProvenanceOutcome::Recovered),
            ],
            _ => {
                errors.push(DocumentTranslationValidationError::WrongFallbackOperation {
                    sentence_id: sentence_id.clone(),
                    status: result.status,
                });
                return;
            }
        },
    };
    if suffix.len() != expected.len() {
        errors.push(DocumentTranslationValidationError::InvalidGenerationSequence {
            sentence_id: sentence_id.clone(),
            expected: expected
                .iter()
                .map(|(op, outcome)| format!("{op:?}/{outcome:?}"))
                .collect(),
            actual: suffix
                .iter()
                .map(|step| format!("{:?}/{:?}", step.operation, step.outcome))
                .collect(),
        });
        return;
    }
    for (step, (expected_op, expected_outcome)) in suffix.iter().zip(expected.iter()) {
        if step.operation != *expected_op {
            errors.push(DocumentTranslationValidationError::InvalidGenerationSequence {
                sentence_id: sentence_id.clone(),
                expected: expected
                    .iter()
                    .map(|(op, outcome)| format!("{op:?}/{outcome:?}"))
                    .collect(),
                actual: suffix
                    .iter()
                    .map(|step| format!("{:?}/{:?}", step.operation, step.outcome))
                    .collect(),
            });
            return;
        }
        if step.outcome != *expected_outcome {
            errors.push(DocumentTranslationValidationError::InvalidGenerationOutcome {
                sentence_id: sentence_id.clone(),
                operation: format!("{:?}", step.operation),
                expected: format!("{expected_outcome:?}"),
                actual: format!("{:?}", step.outcome),
            });
            return;
        }
    }
}

fn diagnostics_for_sentence(
    diagnostics: &[crate::document::DocumentDiagnostic],
    sentence_id: &SentenceId,
) -> BTreeMap<DiagnosticId, crate::document::DocumentDiagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.sentence_id.as_ref() == Some(sentence_id))
        .cloned()
        .map(|diagnostic| (diagnostic.id.clone(), diagnostic))
        .collect()
}

fn sort_errors(errors: &mut [DocumentTranslationValidationError]) {
    errors.sort_by_key(|error| format!("{error:?}"));
}
