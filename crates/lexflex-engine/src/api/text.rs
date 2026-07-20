use lexflex_language::LanguageId;
use lexflex_model::{
    canonical_hash, CanonicalDigest, CanonicalHashError, SemanticExpression, SemanticType,
    SourceSpan, VariableId,
};
use lexflex_parser::{DerivationSet, ParseMetrics, ParseScore};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAnalysisKind {
    Assertion,
    Goal,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TextAnalysisError {
    #[error("text analysis source id is empty")]
    EmptySourceId,
    #[error("assertion analysis contains query variables")]
    AssertionVariables,
    #[error("assertion analysis contains projection")]
    AssertionProjection,
    #[error("goal analysis projection is empty")]
    EmptyGoalProjection,
    #[error("projection variable is undeclared: {0}")]
    ProjectionVariableUndeclared(VariableId),
    #[error("duplicate projection variable: {0}")]
    DuplicateProjection(VariableId),
    #[error("canonical hash failed: {0}")]
    CanonicalHash(CanonicalHashError),
    #[error("text analysis hash mismatch: stored={stored}, expected={expected}")]
    HashMismatch {
        stored: CanonicalDigest,
        expected: CanonicalDigest,
    },
}

fn validate_shape(
    kind: TextAnalysisKind,
    variables: &BTreeMap<VariableId, SemanticType>,
    projection: &[VariableId],
) -> Result<(), TextAnalysisError> {
    match kind {
        TextAnalysisKind::Assertion if !variables.is_empty() => {
            return Err(TextAnalysisError::AssertionVariables)
        }
        TextAnalysisKind::Assertion if !projection.is_empty() => {
            return Err(TextAnalysisError::AssertionProjection)
        }
        TextAnalysisKind::Goal if projection.is_empty() => {
            return Err(TextAnalysisError::EmptyGoalProjection)
        }
        _ => {}
    }
    let mut seen = BTreeMap::new();
    for variable in projection {
        if !variables.contains_key(variable) {
            return Err(TextAnalysisError::ProjectionVariableUndeclared(
                variable.clone(),
            ));
        }
        if seen.insert(variable, ()).is_some() {
            return Err(TextAnalysisError::DuplicateProjection(variable.clone()));
        }
    }
    Ok(())
}

fn validate_parts(
    source_id: Option<&str>,
    kind: TextAnalysisKind,
    expression: &SemanticExpression,
    variables: &BTreeMap<VariableId, SemanticType>,
    projection: &[VariableId],
) -> Result<CanonicalDigest, TextAnalysisError> {
    if let Some(source_id) = source_id {
        if source_id.trim().is_empty() {
            return Err(TextAnalysisError::EmptySourceId);
        }
    }
    validate_shape(kind, variables, projection)?;
    canonical_hash(expression).map_err(TextAnalysisError::CanonicalHash)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextAnalysis {
    source_id: String,
    language: LanguageId,
    span: SourceSpan,
    kind: TextAnalysisKind,
    canonical_expression: SemanticExpression,
    canonical_hash: CanonicalDigest,
    variables: BTreeMap<VariableId, SemanticType>,
    projection: Vec<VariableId>,
    formal_steps: u64,
    parser_metrics: ParseMetrics,
    derivations: Option<DerivationSet>,
}

impl<'de> Deserialize<'de> for TextAnalysis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = TextAnalysisInputSerde::deserialize(deserializer)?;
        let expected = validate_parts(
            Some(&raw.source_id),
            raw.kind,
            &raw.canonical_expression,
            &raw.variables,
            &raw.projection,
        )
        .map_err(serde::de::Error::custom)?;
        if raw.canonical_hash != expected {
            return Err(serde::de::Error::custom(TextAnalysisError::HashMismatch {
                stored: raw.canonical_hash,
                expected,
            }));
        }
        Ok(Self {
            source_id: raw.source_id,
            language: raw.language,
            span: raw.span,
            kind: raw.kind,
            canonical_expression: raw.canonical_expression,
            canonical_hash: raw.canonical_hash,
            variables: raw.variables,
            projection: raw.projection,
            formal_steps: raw.formal_steps,
            parser_metrics: raw.parser_metrics,
            derivations: raw.derivations,
        })
    }
}

#[derive(Deserialize)]
struct TextAnalysisInputSerde {
    source_id: String,
    language: LanguageId,
    span: SourceSpan,
    kind: TextAnalysisKind,
    canonical_expression: SemanticExpression,
    canonical_hash: CanonicalDigest,
    variables: BTreeMap<VariableId, SemanticType>,
    projection: Vec<VariableId>,
    formal_steps: u64,
    parser_metrics: ParseMetrics,
    derivations: Option<DerivationSet>,
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
    pub derivations: Option<DerivationSet>,
}

impl TextAnalysis {
    pub fn new(input: TextAnalysisInput) -> Result<Self, TextAnalysisError> {
        let canonical_hash = validate_parts(
            Some(&input.source_id),
            input.kind,
            &input.canonical_expression,
            &input.variables,
            &input.projection,
        )?;
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
            derivations: input.derivations,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextAnalysisAlternative {
    kind: TextAnalysisKind,
    canonical_expression: SemanticExpression,
    canonical_hash: CanonicalDigest,
    variables: BTreeMap<VariableId, SemanticType>,
    projection: Vec<VariableId>,
    formal_steps: u64,
    parser_metrics: ParseMetrics,
    derivations: Option<DerivationSet>,
    score: ParseScore,
}

#[derive(Debug, Clone)]
pub struct TextAnalysisAlternativeInput {
    pub canonical_expression: SemanticExpression,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub formal_steps: u64,
    pub parser_metrics: ParseMetrics,
    pub derivations: Option<DerivationSet>,
    pub score: ParseScore,
}

macro_rules! text_accessors {
    ($type:ty) => {
        impl $type {
            pub fn canonical_expression(&self) -> &SemanticExpression {
                &self.canonical_expression
            }
            pub fn kind(&self) -> TextAnalysisKind {
                self.kind
            }
            pub fn canonical_hash(&self) -> &CanonicalDigest {
                &self.canonical_hash
            }
            pub fn variables(&self) -> &BTreeMap<VariableId, SemanticType> {
                &self.variables
            }
            pub fn projection(&self) -> &[VariableId] {
                &self.projection
            }
            pub fn formal_steps(&self) -> u64 {
                self.formal_steps
            }
            pub fn parser_metrics(&self) -> &ParseMetrics {
                &self.parser_metrics
            }
            pub fn derivations(&self) -> Option<&DerivationSet> {
                self.derivations.as_ref()
            }
        }
    };
}

text_accessors!(TextAnalysis);
text_accessors!(TextAnalysisAlternative);

impl TextAnalysisAlternative {
    pub(crate) fn set_derivations(&mut self, value: Option<DerivationSet>) {
        self.derivations = value;
    }
}

impl TextAnalysis {
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    pub fn language(&self) -> &LanguageId {
        &self.language
    }
    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

impl<'de> Deserialize<'de> for TextAnalysisAlternative {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = TextAnalysisAlternativeSerde::deserialize(deserializer)?;
        let expected = validate_parts(
            None,
            raw.kind,
            &raw.canonical_expression,
            &raw.variables,
            &raw.projection,
        )
        .map_err(serde::de::Error::custom)?;
        if raw.canonical_hash != expected {
            return Err(serde::de::Error::custom(TextAnalysisError::HashMismatch {
                stored: raw.canonical_hash,
                expected,
            }));
        }
        Ok(Self {
            kind: raw.kind,
            canonical_expression: raw.canonical_expression,
            canonical_hash: raw.canonical_hash,
            variables: raw.variables,
            projection: raw.projection,
            formal_steps: raw.formal_steps,
            parser_metrics: raw.parser_metrics,
            derivations: raw.derivations,
            score: raw.score,
        })
    }
}

#[derive(Deserialize)]
struct TextAnalysisAlternativeSerde {
    kind: TextAnalysisKind,
    canonical_expression: SemanticExpression,
    canonical_hash: CanonicalDigest,
    variables: BTreeMap<VariableId, SemanticType>,
    projection: Vec<VariableId>,
    formal_steps: u64,
    parser_metrics: ParseMetrics,
    derivations: Option<DerivationSet>,
    score: ParseScore,
}

impl TextAnalysisAlternative {
    pub fn new(
        kind: TextAnalysisKind,
        input: TextAnalysisAlternativeInput,
    ) -> Result<Self, TextAnalysisError> {
        let canonical_hash = validate_parts(
            None,
            kind,
            &input.canonical_expression,
            &input.variables,
            &input.projection,
        )?;
        Ok(Self {
            kind,
            canonical_expression: input.canonical_expression,
            canonical_hash,
            variables: input.variables,
            projection: input.projection,
            formal_steps: input.formal_steps,
            parser_metrics: input.parser_metrics,
            derivations: input.derivations,
            score: input.score,
        })
    }
}
