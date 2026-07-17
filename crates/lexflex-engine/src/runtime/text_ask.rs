use super::*;
use crate::runtime::ambiguity::ExpectedTextKind;
use crate::runtime::formal_result::FormalExpressionError;
use lexflex_parser::ParseError;

impl LexFlexRuntime {
    pub(crate) fn handle_ask_text(
        &self,
        input: TextInput,
        evidence_policy: EvidencePolicy,
        limit: Option<usize>,
    ) -> EngineResponse {
        let parse = match self.parse_text(&input) {
            Ok(ParseOutput::Goal(draft)) => draft,
            Ok(ParseOutput::Assertion(_)) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![ParseError::QuestionWithoutProjection],
                };
            }
            Ok(ParseOutput::Ambiguous { mode, alternatives, .. }) => {
                if mode != lexflex_parser::ClauseMode::Interrogative {
                    return EngineResponse::TextNotParsed {
                        diagnostics: vec![ParseError::QuestionWithoutProjection],
                    };
                }
                return match self.lower_ambiguous_alternatives(
                    &input.source_id,
                    alternatives,
                    true,
                    ExpectedTextKind::Goal,
                ) {
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
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidProgram,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };
        if semantic_expression.entry_type != lexflex_model::SemanticType::Boolean {
            let error =
                FormalExpressionError::NonBooleanGoal(semantic_expression.entry_type.clone());
            return EngineResponse::Error {
                code: EngineErrorCode::InvalidProgram,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
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
                return EngineResponse::Error {
                    code: EngineErrorCode::Canonicalization,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
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
        let candidates = match self.candidate_assertions(&goal) {
            Ok(candidates) => candidates,
            Err(assertion_id) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: format!("missing indexed assertion: {assertion_id}"),
                    diagnostics: vec![crate::api::response::EngineDiagnostic::MissingAssertion {
                        assertion_id,
                    }],
                };
            }
        };
        match self.lingua.solve(&goal, candidates) {
            Ok(solutions) => EngineResponse::TextAnswer {
                analysis,
                goal,
                solutions,
                snapshot_hash: self.session.state.snapshot_hash().to_string(),
                diagnostics: Vec::new(),
            },
            Err(error) => EngineResponse::Error {
                code: EngineErrorCode::InvalidGoal,
                message: error.to_string(),
                diagnostics: Vec::new(),
            },
        }
    }
}
