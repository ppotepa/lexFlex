use super::*;
use crate::runtime::formal_result::{FormalExpressionError, FormalExpressionResult};
use lexflex_lingua::CompileContext;
use std::collections::BTreeMap;

impl LexFlexRuntime {
    pub(crate) fn evaluate_formal_expression(
        &self,
        source_id: &str,
        expression: lexflex_lingua::LinguaExpression,
        query_variables: BTreeMap<lexflex_model::VariableId, lexflex_model::SemanticType>,
    ) -> Result<FormalExpressionResult, FormalExpressionError> {
        let source_digest = canonical_hash(&source_id)?;
        let program = LinguaProgram {
            id: lexflex_lingua::ProgramId::new_unchecked(format!(
                "text:{}",
                source_digest.as_str()
            )),
            declarations: self.lingua.base_declarations().to_vec(),
            entry: expression,
        };
        self.lingua
            .evaluate_with_context(
                &program,
                &CompileContext { query_variables },
                ExecutionPolicy {
                    expansion: ExpansionMode::PreserveApplications,
                },
            )
            .map(|result| FormalExpressionResult {
                value: result.execution.value,
                entry_type: result.entry_type,
                steps: result.execution.steps,
            })
            .map_err(FormalExpressionError::Lingua)
    }

    pub(crate) fn assertion_analysis(
        &self,
        draft: &lexflex_parser::AssertionDraft,
        canonical_expression: SemanticExpression,
        formal_steps: u64,
        include_derivation: bool,
    ) -> Result<TextAnalysis, FormalExpressionError> {
        TextAnalysis::new(TextAnalysisInput {
            source_id: draft.source_id.clone(),
            language: draft.language.clone(),
            span: draft.span.clone(),
            kind: TextAnalysisKind::Assertion,
            canonical_expression,
            variables: Default::default(),
            projection: Vec::new(),
            formal_steps,
            parser_metrics: draft.metrics.clone(),
            derivation: include_derivation.then(|| draft.derivation.clone()),
        })
        .map_err(FormalExpressionError::CanonicalHash)
    }

    pub(crate) fn goal_analysis(
        &self,
        draft: &lexflex_parser::GoalDraft,
        canonical_expression: SemanticExpression,
        formal_steps: u64,
        include_derivation: bool,
    ) -> Result<TextAnalysis, FormalExpressionError> {
        TextAnalysis::new(TextAnalysisInput {
            source_id: draft.source_id.clone(),
            language: draft.language.clone(),
            span: draft.span.clone(),
            kind: TextAnalysisKind::Goal,
            canonical_expression,
            variables: draft.variables.clone(),
            projection: draft.projection.clone(),
            formal_steps,
            parser_metrics: draft.metrics.clone(),
            derivation: include_derivation.then(|| draft.derivation.clone()),
        })
        .map_err(FormalExpressionError::CanonicalHash)
    }
}
