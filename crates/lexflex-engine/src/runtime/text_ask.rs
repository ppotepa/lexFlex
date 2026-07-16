use super::*;
use lexflex_parser::ParseError;
use std::collections::BTreeMap;

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
            Ok(ParseOutput::Ambiguous { alternatives }) => {
                let mut lowered = Vec::new();
                for alternative in alternatives {
                    let semantic_expression = match self.evaluate_formal_expression(
                        "ask:ambiguous",
                        alternative.expression.clone(),
                        BTreeMap::new(),
                    ) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    lowered.push(TextAnalysisAlternative::new(
                        semantic_expression,
                        Some(alternative.derivation),
                        alternative.score,
                    ));
                }
                return EngineResponse::TextAmbiguous {
                    alternatives: lowered,
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
                    message: error,
                    diagnostics: Vec::new(),
                };
            }
        };
        let analysis = self.goal_analysis(&parse, semantic_expression.clone(), false);
        let goal = LinguaGoal {
            expression: semantic_expression,
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
            },
            Err(error) => EngineResponse::Error {
                code: EngineErrorCode::InvalidGoal,
                message: error.to_string(),
                diagnostics: Vec::new(),
            },
        }
    }
}
