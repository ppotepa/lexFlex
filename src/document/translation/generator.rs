use crate::api::LexFlexAPI;
use crate::core::interlingua::{Interlingua, LanguageId, Utterance};
use crate::document::SentenceId;

#[derive(Debug, Clone, Copy)]
pub struct SentenceGenerationInput<'a> {
    pub sentence_id: &'a SentenceId,
    pub semantics: &'a Utterance,
    pub target_language: &'a LanguageId,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{code} for {sentence_id}: {message}")]
pub struct SentenceGenerationError {
    pub sentence_id: SentenceId,
    pub code: String,
    pub message: String,
    pub cause: Option<String>,
}

pub trait DocumentSentenceGenerator: Send + Sync {
    fn generator_id(&self) -> &'static str;
    fn generate(&self, input: SentenceGenerationInput<'_>) -> Result<String, SentenceGenerationError>;
}

pub struct LexFlexSentenceGenerator<'a> {
    api: &'a LexFlexAPI,
}

impl<'a> LexFlexSentenceGenerator<'a> {
    pub fn new(api: &'a LexFlexAPI) -> Self {
        Self { api }
    }
}

impl DocumentSentenceGenerator for LexFlexSentenceGenerator<'_> {
    fn generator_id(&self) -> &'static str {
        "lexflex-api-generator-v1"
    }

    fn generate(&self, input: SentenceGenerationInput<'_>) -> Result<String, SentenceGenerationError> {
        let il = Interlingua::Natural(input.semantics.clone());
        self.api
            .generate(&il, &input.target_language.0)
            .map_err(|error| SentenceGenerationError {
                sentence_id: input.sentence_id.clone(),
                code: crate::document::DOC_TRANSLATE_FAILED.to_string(),
                message: "target generator returned an error".to_string(),
                cause: Some(error.to_string()),
            })
    }
}
