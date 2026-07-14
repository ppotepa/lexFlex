use super::assembler::DocumentOutputAssembler;
use super::error::DocumentTranslationError;
use super::generator::{DocumentSentenceGenerator, SentenceGenerationInput};
use super::hash::{translation_hash, translation_output_hash};
use super::provenance::{
    generation_failure_fallback_provenance, generation_skipped_fallback_provenance,
    generation_success_provenance, TranslationFallbackKind,
};
use super::model::{
    DocumentTranslation, DocumentTranslationOptions, DocumentTranslationSummary,
    SentenceTranslationResult, SentenceTranslationStatus, TranslationFallbackPolicy,
};
use super::validation::DocumentTranslationValidator;
use crate::core::interlingua::LanguageId;
use crate::document::compilation::{
    DocumentCompilation, DocumentCompilationValidator,
};
use crate::document::diagnostic::{
    DocumentDiagnosticCollector, DocumentDiagnosticSeverity, DocumentDiagnosticSpec,
    DocumentStage, DOC_TRANSLATION_EMPTY_OUTPUT, DOC_TRANSLATION_PLACEHOLDER,
    DOC_TRANSLATION_SOURCE_FALLBACK,
};
use std::collections::BTreeMap;

pub struct BestEffortDocumentTranslator<G> {
    generator: G,
    options: DocumentTranslationOptions,
}

impl<G> BestEffortDocumentTranslator<G>
where
    G: DocumentSentenceGenerator,
{
    pub fn new(generator: G) -> Self {
        Self {
            generator,
            options: DocumentTranslationOptions::default(),
        }
    }

    pub fn with_options(generator: G, options: DocumentTranslationOptions) -> Self {
        Self { generator, options }
    }

    pub fn translate(
        &self,
        compilation: &DocumentCompilation,
        target_language: LanguageId,
    ) -> Result<DocumentTranslation, DocumentTranslationError> {
        self.options.validate()?;
        DocumentCompilationValidator::validate(compilation).map_err(|errors| {
            DocumentTranslationError::InvalidCompilation { errors }
        })?;
        let document = &compilation.document;
        let mut generated = BTreeMap::new();
        let mut sentence_results = BTreeMap::new();
        let mut collector = DocumentDiagnosticCollector::new(document.id.clone());
        for sentence in document.ordered_sentences() {
            let Some(compilation_result) = compilation.sentence_results.get(&sentence.id) else {
                return Err(DocumentTranslationError::InvalidCompilation {
                    errors: vec![crate::document::DocumentCompilationValidationError::MissingResult {
                        sentence_id: sentence.id.clone(),
                    }],
                });
            };
            let result = self.translate_sentence(
                document,
                sentence,
                compilation_result,
                &target_language,
                &mut collector,
            )?;
            generated.insert(sentence.id.clone(), result.generated_content.clone());
            sentence_results.insert(sentence.id.clone(), result);
        }
        let assembled = DocumentOutputAssembler::assemble(document, &generated)?;
        let diagnostics = collector.into_sorted();
        let summary = summarize_translation(&sentence_results);
        let mut translation = DocumentTranslation {
            source_document_id: document.id.clone(),
            source_language: document.source_language().clone(),
            target_language,
            output_sha256: translation_output_hash(&assembled.text),
            copied_block_count: assembled.copied_block_count,
            copied_block_sha256: assembled.copied_block_sha256,
            output: assembled.text,
            sentence_results,
            diagnostics,
            summary,
            translation_sha256: String::new(),
        };
        translation.translation_sha256 = translation_hash(&translation)?;
        DocumentTranslationValidator::validate(compilation, &translation).map_err(|errors| {
            DocumentTranslationError::InvalidTranslation { errors }
        })?;
        Ok(translation)
    }

    fn translate_sentence(
        &self,
        document: &crate::document::Document,
        sentence: &crate::document::DocumentSentence,
        compilation: &crate::document::compilation::SentenceCompilation,
        target_language: &LanguageId,
        collector: &mut DocumentDiagnosticCollector,
    ) -> Result<SentenceTranslationResult, DocumentTranslationError> {
        let source_text = document
            .sentence_text(&sentence.id)
            .ok_or(DocumentTranslationError::MissingSourceSentenceContent {
                sentence_id: sentence.id.clone(),
            })?;
        match (compilation.status, compilation.semantics.as_ref()) {
            (
                crate::document::compilation::SentenceProcessingStatus::Resolved
                | crate::document::compilation::SentenceProcessingStatus::Partial,
                Some(semantics),
            ) => match self.generator.generate(SentenceGenerationInput {
                sentence_id: &sentence.id,
                semantics,
                target_language,
            }) {
                Ok(text) => {
                    let generated = if self.options.trim_generated_output {
                        text.trim().to_string()
                    } else {
                        text
                    };
                    if self.options.reject_empty_generated_output && generated.is_empty() {
                        return Ok(self.fallback(
                            sentence,
                            compilation,
                            source_text,
                            collector,
                            DOC_TRANSLATION_EMPTY_OUTPUT,
                            "target generator returned empty output",
                            true,
                        ));
                    }
                    Ok(SentenceTranslationResult {
                        sentence_id: sentence.id.clone(),
                        status: SentenceTranslationStatus::Translated,
                        generated_content: generated,
                        diagnostics: diagnostics_for_sentence(collector, &sentence.id),
                        provenance: generation_success_provenance(
                            compilation,
                            sentence,
                            self.generator.generator_id(),
                        ),
                    })
                }
                Err(error) => Ok(self.fallback(
                    sentence,
                    compilation,
                    source_text,
                    collector,
                    &error.code,
                    &error.message,
                    true,
                )),
            },
            _ => Ok(self.fallback(
                sentence,
                compilation,
                source_text,
                collector,
                DOC_TRANSLATION_SOURCE_FALLBACK,
                "sentence semantics are not usable",
                false,
            )),
        }
    }

    fn fallback(
        &self,
        sentence: &crate::document::DocumentSentence,
        compilation: &crate::document::compilation::SentenceCompilation,
        source_text: &str,
        collector: &mut DocumentDiagnosticCollector,
        cause_code: &str,
        cause_message: &str,
        generation_attempted: bool,
    ) -> SentenceTranslationResult {
        match &self.options.fallback_policy {
            TranslationFallbackPolicy::CopySource => {
                collector.push_spec(DocumentDiagnosticSpec {
                    stage: DocumentStage::Generation,
                    severity: DocumentDiagnosticSeverity::Warning,
                    code: DOC_TRANSLATION_SOURCE_FALLBACK.to_string(),
                    message: format!("source sentence copied: {cause_message}"),
                    span: sentence.content_span.clone(),
                    sentence_id: Some(sentence.id.clone()),
                    cause: Some(cause_code.to_string()),
                    recoverable: true,
                });
                SentenceTranslationResult {
                    sentence_id: sentence.id.clone(),
                    status: SentenceTranslationStatus::SourceFallback,
                    generated_content: source_text.to_string(),
                    diagnostics: diagnostics_for_sentence(collector, &sentence.id),
                    provenance: if generation_attempted {
                        generation_failure_fallback_provenance(
                            compilation,
                            sentence,
                            self.generator.generator_id(),
                            cause_code,
                            cause_message,
                            TranslationFallbackKind::CopySource,
                        )
                    } else {
                        generation_skipped_fallback_provenance(
                            compilation,
                            sentence,
                            self.generator.generator_id(),
                            cause_code,
                            cause_message,
                            TranslationFallbackKind::CopySource,
                        )
                    },
                }
            }
            TranslationFallbackPolicy::Placeholder { template } => {
                if template.is_empty() {
                    collector.push_spec(DocumentDiagnosticSpec {
                        stage: DocumentStage::Generation,
                        severity: DocumentDiagnosticSeverity::Error,
                        code: DOC_TRANSLATION_PLACEHOLDER.to_string(),
                        message: "empty placeholder template".to_string(),
                        span: sentence.content_span.clone(),
                        sentence_id: Some(sentence.id.clone()),
                        cause: Some(cause_code.to_string()),
                        recoverable: false,
                    });
                }
                let replacement = template
                    .replace("{sentence_id}", sentence.id.as_str())
                    .replace("{ordinal}", &sentence.document_ordinal.to_string());
                collector.push_spec(DocumentDiagnosticSpec {
                    stage: DocumentStage::Generation,
                    severity: DocumentDiagnosticSeverity::Warning,
                    code: DOC_TRANSLATION_PLACEHOLDER.to_string(),
                    message: "placeholder inserted".to_string(),
                    span: sentence.content_span.clone(),
                    sentence_id: Some(sentence.id.clone()),
                    cause: Some(cause_code.to_string()),
                    recoverable: true,
                });
                SentenceTranslationResult {
                    sentence_id: sentence.id.clone(),
                    status: SentenceTranslationStatus::Placeholder,
                    generated_content: replacement,
                    diagnostics: diagnostics_for_sentence(collector, &sentence.id),
                    provenance: if generation_attempted {
                        generation_failure_fallback_provenance(
                            compilation,
                            sentence,
                            self.generator.generator_id(),
                            cause_code,
                            cause_message,
                            TranslationFallbackKind::Placeholder,
                        )
                    } else {
                        generation_skipped_fallback_provenance(
                            compilation,
                            sentence,
                            self.generator.generator_id(),
                            cause_code,
                            cause_message,
                            TranslationFallbackKind::Placeholder,
                        )
                    },
                }
            }
        }
    }
}

fn summarize_translation(
    sentence_results: &BTreeMap<crate::document::SentenceId, SentenceTranslationResult>,
) -> DocumentTranslationSummary {
    let mut summary = DocumentTranslationSummary {
        total_sentences: sentence_results.len(),
        ..Default::default()
    };
    for result in sentence_results.values() {
        match result.status {
            SentenceTranslationStatus::Translated => summary.translated += 1,
            SentenceTranslationStatus::SourceFallback => summary.source_fallback += 1,
            SentenceTranslationStatus::Placeholder => summary.placeholder += 1,
        }
        if result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DOC_TRANSLATION_EMPTY_OUTPUT)
        {
            summary.empty_generated_rejected += 1;
        }
    }
    summary
}

fn diagnostics_for_sentence(
    collector: &DocumentDiagnosticCollector,
    sentence_id: &crate::document::SentenceId,
) -> Vec<crate::document::DocumentDiagnostic> {
    collector
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.sentence_id.as_ref() == Some(sentence_id))
        .cloned()
        .collect()
}
