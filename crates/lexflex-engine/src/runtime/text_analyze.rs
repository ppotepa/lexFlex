use super::*;
use crate::runtime::formal_result::FormalExpressionError;
use crate::runtime::ambiguity::ExpectedTextKind;
use lexflex_parser::ClauseMode;
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
                            message: error.to_string(),
                            diagnostics: Vec::new(),
                        };
                    }
                };
                if semantic_expression.entry_type != lexflex_model::SemanticType::Boolean {
                    let error = FormalExpressionError::NonBooleanAssertion(
                        semantic_expression.entry_type.clone(),
                    );
                    return EngineResponse::Error {
                        code: EngineErrorCode::InvalidProgram,
                        message: error.to_string(),
                        diagnostics: Vec::new(),
                    };
                }
                let analysis = self.assertion_analysis(
                    &draft,
                    semantic_expression.value,
                    semantic_expression.steps,
                    include_derivation,
                );
                let analysis = match analysis {
                    Ok(analysis) => analysis,
                    Err(error) => {
                        return EngineResponse::Error {
                            code: EngineErrorCode::Canonicalization,
                            message: error.to_string(),
                            diagnostics: Vec::new(),
                        };
                    }
                };
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
                            message: error.to_string(),
                            diagnostics: Vec::new(),
                        };
                    }
                };
                if semantic_expression.entry_type != lexflex_model::SemanticType::Boolean {
                    let error = FormalExpressionError::NonBooleanGoal(
                        semantic_expression.entry_type.clone(),
                    );
                    return EngineResponse::Error {
                        code: EngineErrorCode::InvalidProgram,
                        message: error.to_string(),
                        diagnostics: Vec::new(),
                    };
                }
                let analysis = self.goal_analysis(
                    &draft,
                    semantic_expression.value,
                    semantic_expression.steps,
                    include_derivation,
                );
                let analysis = match analysis {
                    Ok(analysis) => analysis,
                    Err(error) => {
                        return EngineResponse::Error {
                            code: EngineErrorCode::Canonicalization,
                            message: error.to_string(),
                            diagnostics: Vec::new(),
                        };
                    }
                };
                EngineResponse::TextAnalyzed { analysis }
            }
            Ok(ParseOutput::Ambiguous { alternatives, mode, .. }) => {
                let expected = match mode {
                    ClauseMode::Declarative => ExpectedTextKind::Assertion,
                    ClauseMode::Interrogative => ExpectedTextKind::Goal,
                };
                match self.lower_ambiguous_alternatives(
                    &input.source_id,
                    alternatives,
                    include_derivation,
                    expected,
                ) {
                    Ok(mut alternatives) => {
                        if alternatives.len() == 1 {
                            let Some(alternative) = alternatives.pop() else {
                                return EngineResponse::Error {
                                    code: EngineErrorCode::InternalInvariant,
                                    message: "expected exactly one lowered alternative".into(),
                                    diagnostics: Vec::new(),
                                };
                            };
                            let span = match SourceSpan::new(0, input.text.len() as u64) {
                                Ok(span) => span,
                                Err(error) => {
                                    return EngineResponse::Error {
                                        code: EngineErrorCode::Canonicalization,
                                        message: error.to_string(),
                                        diagnostics: Vec::new(),
                                    };
                                }
                            };
                            let analysis = match TextAnalysis::new(TextAnalysisInput {
                                source_id: input.source_id,
                                language: input.language,
                                span,
                                kind: if alternative.variables.is_empty() {
                                    TextAnalysisKind::Assertion
                                } else {
                                    TextAnalysisKind::Goal
                                },
                                canonical_expression: alternative.canonical_expression,
                                variables: alternative.variables,
                                projection: alternative.projection,
                                formal_steps: alternative.formal_steps,
                                parser_metrics: alternative.parser_metrics,
                                derivation: alternative.derivation,
                            }) {
                                Ok(analysis) => analysis,
                                Err(error) => {
                                    return EngineResponse::Error {
                                        code: EngineErrorCode::Canonicalization,
                                        message: error.to_string(),
                                        diagnostics: Vec::new(),
                                    };
                                }
                            };
                            EngineResponse::TextAnalyzed { analysis }
                        } else {
                            EngineResponse::TextAmbiguous {
                                alternatives,
                                diagnostics: Vec::new(),
                            }
                        }
                    }
                    Err(response) => response,
                }
            }
            Err(error) => EngineResponse::TextNotParsed {
                diagnostics: vec![error],
            },
        }
    }
}
