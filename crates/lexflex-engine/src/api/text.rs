use lexflex_language::LanguageId;
use lexflex_lingua::LinguaGoal;
use lexflex_model::{canonical_hash, SemanticExpression, SemanticType, VariableId};
use lexflex_parser::DerivationNode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAnalysisKind {
    Assertion,
    Goal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextAnalysis {
    pub source_id: String,
    pub language: LanguageId,
    pub kind: TextAnalysisKind,
    pub canonical_expression: SemanticExpression,
    pub canonical_hash: String,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub derivation: Option<DerivationNode>,
}

impl TextAnalysis {
    pub fn new(
        source_id: impl Into<String>,
        language: LanguageId,
        kind: TextAnalysisKind,
        canonical_expression: SemanticExpression,
        variables: BTreeMap<VariableId, SemanticType>,
        projection: Vec<VariableId>,
        derivation: Option<DerivationNode>,
    ) -> Self {
        let source_id = source_id.into();
        let canonical_hash = canonical_hash(&canonical_expression);
        Self {
            source_id,
            language,
            kind,
            canonical_expression,
            canonical_hash,
            variables,
            projection,
            derivation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextAnalysisAlternative {
    pub canonical_expression: SemanticExpression,
    pub canonical_hash: String,
    pub derivation: Option<DerivationNode>,
    pub score: i64,
}

impl TextAnalysisAlternative {
    pub fn new(
        canonical_expression: SemanticExpression,
        derivation: Option<DerivationNode>,
        score: i64,
    ) -> Self {
        let canonical_hash = canonical_hash(&canonical_expression);
        Self {
            canonical_expression,
            canonical_hash,
            derivation,
            score,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextGoal {
    pub goal: LinguaGoal,
    pub analysis: TextAnalysis,
}
