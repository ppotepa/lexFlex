use crate::budget::ParseBudget;
use crate::input::ParseInput;
use crate::metrics::ParseMetrics;
use crate::token::{Token, TokenizationResult};
use lexflex_language::LanguageModel;
use lexflex_model::ConceptCatalog;
use std::sync::Arc;

pub struct ParseContext<'a> {
    pub input: ParseInput,
    pub tokenization: TokenizationResult,
    pub words: Vec<Token>,
    pub metrics: ParseMetrics,
    pub catalog: &'a Arc<ConceptCatalog>,
    pub language: &'a Arc<LanguageModel>,
    pub budget: &'a ParseBudget,
}

impl<'a> ParseContext<'a> {
    pub fn new(
        input: ParseInput,
        tokenization: TokenizationResult,
        words: Vec<Token>,
        catalog: &'a Arc<ConceptCatalog>,
        language: &'a Arc<LanguageModel>,
        budget: &'a ParseBudget,
    ) -> Self {
        let metrics = ParseMetrics {
            token_count: words.len(),
            ..ParseMetrics::default()
        };
        Self {
            input,
            tokenization,
            words,
            metrics,
            catalog,
            language,
            budget,
        }
    }
}
