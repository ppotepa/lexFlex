use crate::api::LexFlexAPI;
use crate::core::interlingua::{LanguageId, Utterance};
use crate::document::{Document, DocumentSentence, SourceSpan, DOC_PARSE_FAILED};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct SentenceAnalysisInput<'a> {
    pub document: &'a Document,
    pub sentence: &'a DocumentSentence,
    pub text: &'a str,
    pub content_span: SourceSpan,
    pub source_language: &'a LanguageId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceAnalysisError {
    pub sentence_id: crate::document::SentenceId,
    pub code: String,
    pub message: String,
    pub cause: Option<String>,
}

impl Display for SentenceAnalysisError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} for {}: {}", self.code, self.sentence_id, self.message)
    }
}

impl std::error::Error for SentenceAnalysisError {}

pub trait DocumentSentenceAnalyzer: Send + Sync {
    fn analyzer_id(&self) -> &'static str;
    fn analyze(&self, input: SentenceAnalysisInput<'_>) -> Result<Utterance, SentenceAnalysisError>;
}

pub struct LexFlexSentenceAnalyzer<'a> {
    api: &'a LexFlexAPI,
}

impl<'a> LexFlexSentenceAnalyzer<'a> {
    pub fn new(api: &'a LexFlexAPI) -> Self {
        Self { api }
    }
}

impl DocumentSentenceAnalyzer for LexFlexSentenceAnalyzer<'_> {
    fn analyzer_id(&self) -> &'static str {
        "lexflex-api-v1"
    }

    fn analyze(&self, input: SentenceAnalysisInput<'_>) -> Result<Utterance, SentenceAnalysisError> {
        let utterance = self
            .api
            .parse_multi_sentence(input.text, &input.source_language.0)
            .map_err(|error| SentenceAnalysisError {
                sentence_id: input.sentence.id.clone(),
                code: DOC_PARSE_FAILED.to_string(),
                message: "sentence parser returned an error".to_string(),
                cause: Some(error.to_string()),
            })?;
        Ok(utterance)
    }
}
