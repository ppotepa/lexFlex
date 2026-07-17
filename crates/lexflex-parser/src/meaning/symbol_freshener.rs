#![allow(dead_code)]

use crate::diagnostic::ParseError;
use crate::meaning::instance::MeaningInstance;
use lexflex_language::{CompiledLexicalSense, SyntacticCategory};

pub(crate) fn fresh_local_symbols(
    seed: &str,
    category: &SyntacticCategory,
    sense: &CompiledLexicalSense,
) -> Result<(SyntacticCategory, MeaningInstance), ParseError> {
    super::allocator::instantiate_meaning(seed, category, sense)
}
