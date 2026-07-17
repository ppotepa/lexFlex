use crate::token::Token;
use lexflex_language::{CategoryType, FormId, LanguageId};
use lexflex_model::{CanonicalHashError, SemanticType, SourceSpan, VariableId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum ParseBudgetLimit {
    #[error("token limit")]
    TokenLimit,
    #[error("lexical candidate limit")]
    LexicalCandidateLimit,
    #[error("cell item limit")]
    CellItemLimit,
    #[error("total item limit")]
    TotalItemLimit,
    #[error("derivation depth limit")]
    DerivationDepthLimit,
    #[error("complete parse limit")]
    CompleteParseLimit,
    #[error("semantic node limit")]
    SemanticNodeLimit,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseError {
    #[error("empty input")]
    EmptyInput,
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(LanguageId),
    #[error("unknown surface: {token}")]
    UnknownSurface { token: String },
    #[error("budget exceeded: {0}")]
    BudgetExceeded(ParseBudgetLimit),
    #[error("unsupported character {character:?} at {index}")]
    UnsupportedCharacter { character: char, index: usize },
    #[error("unexpected query variable in declarative parse")]
    UnexpectedQueryVariable,
    #[error("question without projection")]
    QuestionWithoutProjection,
    #[error("unsupported punctuation layout near {token:?}")]
    UnsupportedPunctuationLayout { token: Token },
    #[error("unresolved query type for variable {0}")]
    UnresolvedQueryType(VariableId),
    #[error("unresolved query category type")]
    UnresolvedQueryCategoryType,
    #[error("conflicting query variable type for {variable}: existing {existing:?}, incoming {incoming:?}")]
    ConflictingQueryVariableType {
        variable: VariableId,
        existing: SemanticType,
        incoming: SemanticType,
    },
    #[error("conflicting query category type: existing {existing:?}, incoming {incoming:?}")]
    ConflictingQueryCategoryType {
        existing: CategoryType,
        incoming: CategoryType,
    },
    #[error("meaning freshening error: {0}")]
    MeaningFreshening(String),
    #[error("canonical hash error: {0}")]
    CanonicalHash(String),
    #[error("category error: {0}")]
    Category(String),
    #[error("no complete parse")]
    NoParse,
    #[error("ambiguous parse")]
    Ambiguous,
    #[error("invalid source span {start_byte}..{end_byte}")]
    InvalidSpan { start_byte: u64, end_byte: u64 },
}

impl From<CanonicalHashError> for ParseError {
    fn from(value: CanonicalHashError) -> Self {
        Self::CanonicalHash(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseSpan {
    pub span: SourceSpan,
    pub form_id: Option<FormId>,
}
