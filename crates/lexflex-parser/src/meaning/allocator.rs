#![allow(dead_code)]

use crate::diagnostic::ParseError;
use crate::meaning::instance::MeaningInstance;
use lexflex_language::{CompiledLexicalSense, SyntacticCategory};

pub(crate) fn instantiate_meaning(
    seed: &str,
    category: &SyntacticCategory,
    sense: &CompiledLexicalSense,
) -> Result<(SyntacticCategory, MeaningInstance), ParseError> {
    super::fresh::instantiate_meaning(seed, category, sense)
}
