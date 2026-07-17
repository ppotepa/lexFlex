use super::*;
use crate::runtime::formal_result::FormalExpressionError;
use crate::runtime::ambiguity::ExpectedTextKind;
use lexflex_parser::ParseError;
use std::collections::BTreeMap;

impl LexFlexRuntime {
    pub(crate) fn handle_ingest_text(&mut self, input: TextInput) -> EngineResponse {
        let parse = match self.parse_text(&input) {
            Ok(ParseOutput::Assertion(draft)) => draft,
            Ok(ParseOutput::Goal(_)) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![ParseError::NoParse],
                };
            }
            Ok(ParseOutput::Ambiguous { mode, alternatives, .. }) => {
                if mode != lexflex_parser::ClauseMode::Declarative {
                    return EngineResponse::TextNotParsed {
                        diagnostics: vec![ParseError::UnexpectedQueryVariable],
                    };
                }
                return match self.lower_ambiguous_alternatives(
                    &input.source_id,
                    alternatives,
                    true,
                    ExpectedTextKind::Assertion,
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
            let error =
                FormalExpressionError::NonBooleanAssertion(semantic_expression.entry_type.clone());
            return EngineResponse::Error {
                code: EngineErrorCode::InvalidProgram,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }
        let analysis = self.assertion_analysis(
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
        let source_hash = match canonical_hash(&input.text) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };
        let evidence = match Evidence::create(
            input.source_id.clone(),
            Some(match SourceSpan::new(0, input.text.len() as u64) {
                Ok(value) => value,
                Err(error) => {
                    return EngineResponse::Error {
                        code: EngineErrorCode::InvalidEvidence,
                        message: error.to_string(),
                        diagnostics: Vec::new(),
                    };
                }
            }),
            Some(source_hash),
        ) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };
        let assertion = match SemanticAssertion::create(
            semantic_expression.value,
            vec![evidence],
            WorldId::new_unchecked("actual"),
        ) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };
        let (assertion, outcome) = match self.persist_assertion(assertion) {
            Ok(value) => value,
            Err(response) => return response,
        };
        EngineResponse::TextIngested {
            analysis,
            assertion,
            outcome,
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn handle_ingest(
        &mut self,
        program: LinguaProgram,
        evidence: Vec<Evidence>,
    ) -> EngineResponse {
        let result = match self.lingua.evaluate_with_policy(
            &program,
            ExecutionPolicy {
                expansion: ExpansionMode::PreserveApplications,
            },
        ) {
            Ok(result) => result,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidProgram,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };
        if result.entry_type != lexflex_model::SemanticType::Boolean {
            return EngineResponse::Error {
                code: EngineErrorCode::InvalidAssertion,
                message: FormalExpressionError::NonBooleanAssertion(result.entry_type).to_string(),
                diagnostics: Vec::new(),
            };
        }
        let assertion = match SemanticAssertion::create(
            result.execution.value,
            evidence,
            WorldId::new_unchecked("actual"),
        ) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };
        let (assertion, outcome) = match self.persist_assertion(assertion) {
            Ok(value) => value,
            Err(response) => return response,
        };
        EngineResponse::LinguaIngested {
            assertion,
            outcome,
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
        }
    }
}
