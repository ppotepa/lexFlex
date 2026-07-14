use crate::core::interlingua::LanguageId;
use crate::document::{DocumentDiagnostic, DocumentId, SentenceId};
use crate::document::compilation::SentenceProvenance;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SentenceTranslationStatus {
    Translated,
    SourceFallback,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationFallbackPolicy {
    CopySource,
    Placeholder { template: String },
}

impl Default for TranslationFallbackPolicy {
    fn default() -> Self {
        Self::CopySource
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentTranslationOptions {
    pub fallback_policy: TranslationFallbackPolicy,
    pub trim_generated_output: bool,
    pub reject_empty_generated_output: bool,
    pub preserve_source_whitespace: bool,
}

impl Default for DocumentTranslationOptions {
    fn default() -> Self {
        Self {
            fallback_policy: TranslationFallbackPolicy::CopySource,
            trim_generated_output: true,
            reject_empty_generated_output: true,
            preserve_source_whitespace: true,
        }
    }
}

impl DocumentTranslationOptions {
    pub fn validate(
        &self,
    ) -> Result<(), crate::document::translation::error::DocumentTranslationOptionError> {
        if !self.preserve_source_whitespace {
            return Err(
                crate::document::translation::error::DocumentTranslationOptionError::DestructiveWhitespaceModeUnsupported,
            );
        }
        match &self.fallback_policy {
            TranslationFallbackPolicy::Placeholder { template }
                if template.trim().is_empty() =>
            {
                return Err(
                    crate::document::translation::error::DocumentTranslationOptionError::EmptyPlaceholderTemplate,
                );
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SentenceTranslationResult {
    pub sentence_id: SentenceId,
    pub status: SentenceTranslationStatus,
    pub generated_content: String,
    pub diagnostics: Vec<DocumentDiagnostic>,
    pub provenance: SentenceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DocumentTranslationSummary {
    pub total_sentences: usize,
    pub translated: usize,
    pub source_fallback: usize,
    pub placeholder: usize,
    pub empty_generated_rejected: usize,
}

impl DocumentTranslationSummary {
    pub fn coverage_count(&self) -> usize {
        self.translated + self.source_fallback + self.placeholder
    }

    pub fn coverage_ratio(&self) -> f64 {
        if self.total_sentences == 0 {
            1.0
        } else {
            self.coverage_count() as f64 / self.total_sentences as f64
        }
    }

    pub fn fallback_ratio(&self) -> f64 {
        if self.total_sentences == 0 {
            0.0
        } else {
            (self.source_fallback + self.placeholder) as f64 / self.total_sentences as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTranslation {
    pub source_document_id: DocumentId,
    pub source_language: LanguageId,
    pub target_language: LanguageId,
    pub output: String,
    pub output_sha256: String,
    pub copied_block_count: usize,
    pub copied_block_sha256: String,
    pub sentence_results: BTreeMap<SentenceId, SentenceTranslationResult>,
    pub diagnostics: Vec<DocumentDiagnostic>,
    pub summary: DocumentTranslationSummary,
    pub translation_sha256: String,
}
