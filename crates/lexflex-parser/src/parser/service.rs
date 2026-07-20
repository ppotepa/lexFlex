use crate::budget::ParseBudget;
use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::input::ParseInput;
use crate::output::ParseOutput;
use crate::parser::ambiguity::parse_with_ambiguity;
use crate::parser::context::ParseContext;
use crate::token::{tokenize, Token, TokenKind};
use lexflex_language::LanguageModel;
use lexflex_model::ConceptCatalog;
use std::sync::Arc;

pub struct LexicalCompositionParser {
    catalog: Arc<ConceptCatalog>,
    language: Arc<LanguageModel>,
    budget: ParseBudget,
}

impl LexicalCompositionParser {
    pub fn new(catalog: Arc<ConceptCatalog>, language: Arc<LanguageModel>) -> Self {
        Self {
            catalog,
            language,
            budget: ParseBudget::default(),
        }
    }

    pub fn with_budget(
        catalog: Arc<ConceptCatalog>,
        language: Arc<LanguageModel>,
        budget: ParseBudget,
    ) -> Self {
        Self {
            catalog,
            language,
            budget,
        }
    }

    pub fn parse(&self, input: ParseInput) -> Result<ParseOutput, ParseError> {
        if input.text.is_empty() {
            return Err(ParseError::EmptyInput);
        }
        if self.language.manifest.language.as_str() != input.language.as_str() {
            return Err(ParseError::UnsupportedLanguage(input.language));
        }
        let tokenization = tokenize(&input)?;
        if tokenization.tokens.len() > self.budget.max_tokens {
            return Err(ParseError::BudgetExceeded(ParseBudgetLimit::TokenLimit));
        }
        let words = collect_words(&tokenization.tokens)?;
        let ctx = ParseContext::new(
            input,
            tokenization,
            words,
            &self.catalog,
            &self.language,
            &self.budget,
        );
        parse_with_ambiguity(ctx)
    }
}

fn collect_words(tokens: &[Token]) -> Result<Vec<Token>, ParseError> {
    let punctuation = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Punctuation)
        .cloned()
        .collect::<Vec<_>>();
    let words = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Word)
        .cloned()
        .collect::<Vec<_>>();
    if words.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    match punctuation.as_slice() {
        [] => Ok(words),
        [token] if token.surface == "." || token.surface == "?" || token.surface == "!" => {
            if Some(token.id.clone()) == tokens.last().map(|value| value.id.clone()) {
                Ok(words)
            } else {
                Err(ParseError::UnsupportedPunctuationLayout {
                    token: token.clone(),
                })
            }
        }
        [token, ..] => Err(ParseError::UnsupportedPunctuationLayout {
            token: token.clone(),
        }),
    }
}
