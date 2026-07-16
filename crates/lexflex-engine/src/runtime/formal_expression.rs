use super::*;
use lexflex_lingua::CompileContext;
use std::collections::BTreeMap;

impl LexFlexRuntime {
    pub(crate) fn evaluate_formal_expression(
        &self,
        source_id: &str,
        expression: lexflex_lingua::LinguaExpression,
        query_variables: BTreeMap<lexflex_model::VariableId, lexflex_model::SemanticType>,
    ) -> Result<SemanticExpression, String> {
        let program = LinguaProgram {
            id: lexflex_lingua::ProgramId::new_unchecked(format!(
                "text:{}",
                canonical_hash(&source_id)
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
            .map(|result| result.value)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn assertion_analysis(
        &self,
        draft: &lexflex_parser::AssertionDraft,
        canonical_expression: SemanticExpression,
        include_derivation: bool,
    ) -> TextAnalysis {
        TextAnalysis::new(
            draft.source_id.clone(),
            draft.language.clone(),
            TextAnalysisKind::Assertion,
            canonical_expression,
            Default::default(),
            Vec::new(),
            include_derivation.then(|| draft.derivation.clone()),
        )
    }

    pub(crate) fn goal_analysis(
        &self,
        draft: &lexflex_parser::GoalDraft,
        canonical_expression: SemanticExpression,
        include_derivation: bool,
    ) -> TextAnalysis {
        TextAnalysis::new(
            draft.source_id.clone(),
            draft.language.clone(),
            TextAnalysisKind::Goal,
            canonical_expression,
            draft.variables.clone(),
            draft.projection.clone(),
            include_derivation.then(|| draft.derivation.clone()),
        )
    }
}
