use lexflex_language::LanguageId;
use lexflex_lingua::LinguaGoal;
use lexflex_model::{
    canonical_hash, CanonicalDigest, CanonicalHashError, SemanticExpression, SemanticType,
    SourceSpan, VariableId,
};
use lexflex_parser::{DerivationNode, ParseMetrics, ParseScore};
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
    pub span: SourceSpan,
    pub kind: TextAnalysisKind,
    pub canonical_expression: SemanticExpression,
    pub canonical_hash: CanonicalDigest,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub formal_steps: u64,
    pub parser_metrics: ParseMetrics,
    pub derivation: Option<DerivationNode>,
}

#[derive(Debug, Clone)]
pub struct TextAnalysisInput {
    pub source_id: String,
    pub language: LanguageId,
    pub span: SourceSpan,
    pub kind: TextAnalysisKind,
    pub canonical_expression: SemanticExpression,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub formal_steps: u64,
    pub parser_metrics: ParseMetrics,
    pub derivation: Option<DerivationNode>,
}

impl TextAnalysis {
    pub fn new(input: TextAnalysisInput) -> Result<Self, CanonicalHashError> {
        let canonical_hash = canonical_hash(&input.canonical_expression)?;
        Ok(Self {
            source_id: input.source_id,
            language: input.language,
            span: input.span,
            kind: input.kind,
            canonical_expression: input.canonical_expression,
            canonical_hash,
            variables: input.variables,
            projection: input.projection,
            formal_steps: input.formal_steps,
            parser_metrics: input.parser_metrics,
            derivation: input.derivation,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextAnalysisAlternative {
    pub canonical_expression: SemanticExpression,
    pub canonical_hash: CanonicalDigest,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub formal_steps: u64,
    pub parser_metrics: ParseMetrics,
    pub derivation: Option<DerivationNode>,
    pub score: ParseScore,
}

impl TextAnalysisAlternative {
    pub fn new(
        canonical_expression: SemanticExpression,
        variables: BTreeMap<VariableId, SemanticType>,
        projection: Vec<VariableId>,
        formal_steps: u64,
        parser_metrics: ParseMetrics,
        derivation: Option<DerivationNode>,
        score: ParseScore,
    ) -> Result<Self, CanonicalHashError> {
        let canonical_hash = canonical_hash(&canonical_expression)?;
        Ok(Self {
            canonical_expression,
            canonical_hash,
            variables,
            projection,
            formal_steps,
            parser_metrics,
            derivation,
            score,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextGoal {
    pub goal: LinguaGoal,
    pub analysis: TextAnalysis,
}
