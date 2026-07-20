mod builder;
mod node;

use crate::GenerationError;
use lexflex_language::LanguageModel;
use lexflex_model::SemanticExpression;

pub(crate) use builder::build_plan;
pub use node::{GenerationNode, GenerationPlan};

pub fn build_generation_plan(
    expression: &SemanticExpression,
    language: &LanguageModel,
) -> Result<GenerationPlan, GenerationError> {
    build_plan(expression, language).map_err(|_| GenerationError::Canonicalization)
}
