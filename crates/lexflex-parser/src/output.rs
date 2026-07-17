use crate::input::ClauseMode;
use crate::{ParseMetrics, ParseScore};
use lexflex_language::LanguageId;
use lexflex_lingua::LinguaExpression;
use lexflex_model::{SemanticType, SourceSpan, VariableId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum ParseOutput {
    Assertion(AssertionDraft),
    Goal(GoalDraft),
    Ambiguous {
        alternatives: Vec<ParseAlternative>,
        metrics: ParseMetrics,
    },
}

#[derive(Debug, Clone)]
pub struct ParseAlternative {
    pub expression: LinguaExpression,
    pub query_variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub derivation: crate::DerivationNode,
    pub score: ParseScore,
    pub metrics: ParseMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionDraft {
    pub source_id: String,
    pub language: LanguageId,
    pub span: SourceSpan,
    pub expression: LinguaExpression,
    pub derivation: crate::DerivationNode,
    pub metrics: ParseMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDraft {
    pub source_id: String,
    pub language: LanguageId,
    pub span: SourceSpan,
    pub expression: LinguaExpression,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub mode: ClauseMode,
    pub derivation: crate::DerivationNode,
    pub metrics: ParseMetrics,
}
