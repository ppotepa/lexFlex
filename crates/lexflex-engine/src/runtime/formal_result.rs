use crate::api::text::TextAnalysisError;
use crate::runtime::lingua::EngineError;
use lexflex_model::{CanonicalHashError, SemanticExpression, SemanticType};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FormalExpressionResult {
    pub value: SemanticExpression,
    pub entry_type: SemanticType,
    pub steps: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FormalExpressionError {
    #[error("canonicalization failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),
    #[error("text analysis invalid: {0}")]
    Analysis(#[from] TextAnalysisError),
    #[error("Lingua execution failed: {0}")]
    Lingua(#[from] EngineError),
    #[error("natural-language assertion must evaluate to Boolean, found {0:?}")]
    NonBooleanAssertion(SemanticType),
    #[error("natural-language goal must evaluate to Boolean, found {0:?}")]
    NonBooleanGoal(SemanticType),
}
