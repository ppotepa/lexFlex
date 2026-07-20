use super::*;
use crate::runtime::ambiguity::ExpectedTextKind;
use crate::runtime::formal_error_mapping::{analysis_error_response, formal_error_response};
use crate::runtime::formal_result::FormalExpressionError;
use lexflex_parser::ClauseMode;
use std::collections::BTreeMap;

impl LexFlexRuntime {
    pub(crate) fn handle_analyze_text(
        &self,
        input: TextInput,
        include_derivation: bool,
    ) -> EngineResponse {
        if let Err(error) = input.validate() {
            return EngineResponse::Error {
                code: EngineErrorCode::InvalidProgram,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }

        match self.parse_text(&input) {
            Ok(ParseOutput::Assertion(draft)) => {
                let semantic_expression = match self.evaluate_formal_expression(
                    &draft.source_id,
                    draft.expression.clone(),
                    BTreeMap::new(),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        return formal_error_response(error);
                    }
                };
                if semantic_expression.entry_type != lexflex_model::SemanticType::Boolean {
                    let error = FormalExpressionError::NonBooleanAssertion(
                        semantic_expression.entry_type.clone(),
                    );
                    return formal_error_response(error);
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
                        return formal_error_response(error);
                    }
                };
                EngineResponse::TextAnalyzed {
                    analysis,
                    diagnostics: Vec::new(),
                }
            }
            Ok(ParseOutput::Goal(draft)) => {
                let semantic_expression = match self.evaluate_formal_expression(
                    &draft.source_id,
                    draft.expression.clone(),
                    draft.variables.clone(),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        return formal_error_response(error);
                    }
                };
                if semantic_expression.entry_type != lexflex_model::SemanticType::Boolean {
                    let error = FormalExpressionError::NonBooleanGoal(
                        semantic_expression.entry_type.clone(),
                    );
                    return formal_error_response(error);
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
                        return formal_error_response(error);
                    }
                };
                EngineResponse::TextAnalyzed {
                    analysis,
                    diagnostics: Vec::new(),
                }
            }
            Ok(ParseOutput::Ambiguous {
                alternatives, mode, ..
            }) => {
                let expected_kind = match mode {
                    ClauseMode::Declarative => TextAnalysisKind::Assertion,
                    ClauseMode::Interrogative => TextAnalysisKind::Goal,
                };
                let expected = match expected_kind {
                    TextAnalysisKind::Assertion => ExpectedTextKind::Assertion,
                    TextAnalysisKind::Goal => ExpectedTextKind::Goal,
                };
                match self.lower_ambiguous_alternatives(
                    &input.source_id,
                    alternatives,
                    include_derivation,
                    expected,
                ) {
                    Ok((mut alternatives, diagnostics)) => {
                        if alternatives.len() == 1 {
                            let Some(alternative) = alternatives.pop() else {
                                return EngineResponse::Error {
                                    code: EngineErrorCode::InternalInvariant,
                                    message: "expected exactly one lowered alternative".into(),
                                    diagnostics,
                                };
                            };
                            let span = match SourceSpan::new(0, input.text.len() as u64) {
                                Ok(span) => span,
                                Err(error) => {
                                    return crate::runtime::evidence_error_mapping::evidence_error_response(
                                        error,
                                        crate::runtime::evidence_error_mapping::EvidenceOrigin::GeneratedText,
                                    );
                                }
                            };
                            let analysis = match TextAnalysis::new(TextAnalysisInput {
                                source_id: input.source_id,
                                language: input.language,
                                span,
                                kind: expected_kind,
                                canonical_expression: alternative.canonical_expression().clone(),
                                variables: alternative.variables().clone(),
                                projection: alternative.projection().to_vec(),
                                formal_steps: alternative.formal_steps(),
                                parser_metrics: alternative.parser_metrics().clone(),
                                derivations: alternative.derivations().cloned(),
                            }) {
                                Ok(analysis) => analysis,
                                Err(error) => {
                                    return analysis_error_response(error, diagnostics);
                                }
                            };
                            EngineResponse::TextAnalyzed {
                                analysis,
                                diagnostics,
                            }
                        } else {
                            EngineResponse::TextAmbiguous {
                                alternatives,
                                diagnostics,
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
