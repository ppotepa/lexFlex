use super::*;
use crate::runtime::ambiguity::ExpectedTextKind;
use crate::runtime::diagnostics::with_diagnostics;
use crate::runtime::error_mapping::{missing_assertion_response, request_program_error_response};
use crate::runtime::formal_error_mapping::{analysis_error_response, formal_error_response};
use crate::runtime::formal_result::FormalExpressionError;
use lexflex_parser::ParseError;

impl LexFlexRuntime {
    pub(crate) fn handle_ask_text(
        &self,
        input: TextInput,
        evidence_policy: EvidencePolicy,
        limit: Option<usize>,
    ) -> EngineResponse {
        if let Err(error) = input.validate() {
            return EngineResponse::Error {
                code: EngineErrorCode::InvalidProgram,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }

        let parse = match self.parse_text(&input) {
            Ok(ParseOutput::Goal(draft)) => draft,
            Ok(ParseOutput::Assertion(_)) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![ParseError::QuestionWithoutProjection],
                };
            }
            Ok(ParseOutput::Ambiguous {
                mode, alternatives, ..
            }) => {
                if mode != lexflex_parser::ClauseMode::Interrogative {
                    return EngineResponse::TextNotParsed {
                        diagnostics: vec![ParseError::QuestionWithoutProjection],
                    };
                }
                return match self.lower_ambiguous_alternatives(
                    &input.source_id,
                    alternatives,
                    false,
                    ExpectedTextKind::Goal,
                ) {
                    Ok((mut alternatives, diagnostics)) if alternatives.len() == 1 => {
                        let alternative = alternatives.remove(0);
                        self.answer_lowered_goal(
                            input,
                            alternative,
                            evidence_policy,
                            limit,
                            diagnostics,
                        )
                    }
                    Ok((alternatives, diagnostics)) => EngineResponse::TextAmbiguous {
                        alternatives,
                        diagnostics,
                    },
                    Err(response) => response,
                };
            }
            Err(error) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![error],
                };
            }
        };

        let semantic_expression = match self.evaluate_formal_expression(
            &parse.source_id,
            parse.expression.clone(),
            parse.variables.clone(),
        ) {
            Ok(value) => value,
            Err(error) => {
                return formal_error_response(error);
            }
        };
        if semantic_expression.entry_type != lexflex_model::SemanticType::Boolean {
            let error =
                FormalExpressionError::NonBooleanGoal(semantic_expression.entry_type.clone());
            return formal_error_response(error);
        }
        let analysis = self.goal_analysis(
            &parse,
            semantic_expression.value.clone(),
            semantic_expression.steps,
            false,
        );
        let analysis = match analysis {
            Ok(analysis) => analysis,
            Err(error) => {
                return formal_error_response(error);
            }
        };
        let goal = LinguaGoal {
            expression: semantic_expression.value,
            variables: parse.variables.clone(),
            projection: parse.projection.clone(),
            evidence_policy,
            world: None,
            limit,
        };
        self.solve_text_goal(analysis, goal, Vec::new())
    }

    fn answer_lowered_goal(
        &self,
        input: TextInput,
        alternative: TextAnalysisAlternative,
        evidence_policy: EvidencePolicy,
        limit: Option<usize>,
        diagnostics: Vec<crate::api::response::EngineDiagnostic>,
    ) -> EngineResponse {
        let span = match SourceSpan::new(0, input.text.len() as u64) {
            Ok(value) => value,
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
            kind: TextAnalysisKind::Goal,
            canonical_expression: alternative.canonical_expression().clone(),
            variables: alternative.variables().clone(),
            projection: alternative.projection().to_vec(),
            formal_steps: alternative.formal_steps(),
            parser_metrics: alternative.parser_metrics().clone(),
            derivations: alternative.derivations().cloned(),
        }) {
            Ok(value) => value,
            Err(error) => {
                return analysis_error_response(error, diagnostics);
            }
        };
        let goal = LinguaGoal {
            expression: alternative.canonical_expression().clone(),
            variables: alternative.variables().clone(),
            projection: alternative.projection().to_vec(),
            evidence_policy,
            world: None,
            limit,
        };
        self.solve_text_goal(analysis, goal, diagnostics)
    }

    fn solve_text_goal(
        &self,
        analysis: TextAnalysis,
        goal: LinguaGoal,
        diagnostics: Vec<crate::api::response::EngineDiagnostic>,
    ) -> EngineResponse {
        let candidates = match self.candidate_assertions(&goal) {
            Ok(candidates) => candidates,
            Err(assertion_id) => {
                return with_diagnostics(missing_assertion_response(assertion_id), diagnostics);
            }
        };
        match self.lingua.solve(&goal, candidates) {
            Ok(solutions) => EngineResponse::TextAnswer {
                analysis,
                goal,
                solutions,
                snapshot_hash: self.session.state.snapshot_hash().to_string(),
                diagnostics,
            },
            Err(error) => with_diagnostics(request_program_error_response(error), diagnostics),
        }
    }
}
