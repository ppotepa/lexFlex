use crate::document::compilation::DocumentCompilationValidationError;
use crate::document::id::{DocumentBlockId, ParagraphId, SentenceId};
use crate::document::span::SourceSpanError;
use super::validation::DocumentTranslationValidationError;

#[derive(Debug, thiserror::Error)]
pub enum DocumentTranslationOptionError {
    #[error("preserve_source_whitespace=false is not supported by the lossless document translator")]
    DestructiveWhitespaceModeUnsupported,
    #[error("placeholder template must not be empty")]
    EmptyPlaceholderTemplate,
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentAssemblyError {
    #[error("missing block {0}")]
    MissingBlock(DocumentBlockId),
    #[error("missing paragraph {0}")]
    MissingParagraph(ParagraphId),
    #[error("missing sentence {0}")]
    MissingSentence(SentenceId),
    #[error("missing generated content for {0}")]
    MissingGeneratedContent(SentenceId),
    #[error("missing exact span for {field}")]
    MissingExactSpan { field: String },
    #[error("content span is outside raw span for {sentence_id}")]
    ContentOutsideRaw { sentence_id: SentenceId },
    #[error("invalid source span: {0}")]
    Span(#[from] SourceSpanError),
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentTranslationError {
    #[error("validated semantic rewrite is not implemented for this document profile")]
    UnsupportedResolvedRewrite,
    #[error("compilation artifact is invalid")]
    InvalidCompilation {
        errors: Vec<DocumentCompilationValidationError>,
    },
    #[error("translation options are invalid: {0}")]
    InvalidOptions(#[from] DocumentTranslationOptionError),
    #[error("source sentence content is missing for {sentence_id}")]
    MissingSourceSentenceContent { sentence_id: SentenceId },
    #[error("output assembly failed: {0}")]
    Assembly(#[from] DocumentAssemblyError),
    #[error("translation artifact is invalid")]
    InvalidTranslation {
        errors: Vec<DocumentTranslationValidationError>,
    },
    #[error("failed to serialize translation artifact: {0}")]
    Serialization(#[from] serde_json::Error),
}
