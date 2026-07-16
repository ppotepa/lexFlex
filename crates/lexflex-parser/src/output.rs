use crate::input::ClauseMode;
use lexflex_language::LanguageId;
use lexflex_lingua::LinguaExpression;
use lexflex_model::{SemanticType, VariableId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum ParseOutput {
    Assertion(AssertionDraft),
    Goal(GoalDraft),
    Ambiguous { alternatives: Vec<ParseAlternative> },
}

#[derive(Debug, Clone)]
pub struct ParseAlternative {
    pub expression: LinguaExpression,
    pub derivation: crate::parser::DerivationNode,
    pub score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionDraft {
    pub source_id: String,
    pub language: LanguageId,
    pub expression: LinguaExpression,
    pub derivation: crate::parser::DerivationNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDraft {
    pub source_id: String,
    pub language: LanguageId,
    pub expression: LinguaExpression,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub mode: ClauseMode,
    pub derivation: crate::parser::DerivationNode,
}
