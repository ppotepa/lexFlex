use super::*;
use std::collections::BTreeMap;

impl LexFlexRuntime {
    pub(crate) fn handle_analyze_text(
        &self,
        input: TextInput,
        include_derivation: bool,
    ) -> EngineResponse {
        match self.parse_text(&input) {
            Ok(ParseOutput::Assertion(draft)) => {
                let semantic_expression = match self.evaluate_formal_expression(
                    &draft.source_id,
                    draft.expression.clone(),
                    BTreeMap::new(),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        return EngineResponse::Error {
                            code: EngineErrorCode::InvalidProgram,
                            message: error,
                            diagnostics: Vec::new(),
                        };
                    }
                };
                let analysis =
                    self.assertion_analysis(&draft, semantic_expression, include_derivation);
                EngineResponse::TextAnalyzed { analysis }
            }
            Ok(ParseOutput::Goal(draft)) => {
                let semantic_expression = match self.evaluate_formal_expression(
                    &draft.source_id,
                    draft.expression.clone(),
                    draft.variables.clone(),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        return EngineResponse::Error {
                            code: EngineErrorCode::InvalidProgram,
                            message: error,
                            diagnostics: Vec::new(),
                        };
                    }
                };
                let analysis = self.goal_analysis(&draft, semantic_expression, include_derivation);
                EngineResponse::TextAnalyzed { analysis }
            }
            Ok(ParseOutput::Ambiguous { alternatives }) => {
                let mut lowered = Vec::new();
                for alternative in alternatives {
                    let semantic_expression = match self.evaluate_formal_expression(
                        "analysis:ambiguous",
                        alternative.expression.clone(),
                        BTreeMap::new(),
                    ) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    lowered.push(TextAnalysisAlternative::new(
                        semantic_expression,
                        include_derivation.then_some(alternative.derivation),
                        alternative.score,
                    ));
                }
                EngineResponse::TextAmbiguous {
                    alternatives: lowered,
                }
            }
            Err(error) => EngineResponse::TextNotParsed {
                diagnostics: vec![error],
            },
        }
    }
}
