use super::*;
use crate::runtime::ambiguity::ExpectedTextKind;
use crate::runtime::diagnostics::prepend_diagnostics;
use crate::runtime::error_mapping::request_program_error_response;
use crate::runtime::evidence_error_mapping::{evidence_error_response, EvidenceOrigin};
use crate::runtime::formal_error_mapping::formal_error_response;
use crate::runtime::formal_result::FormalExpressionError;
use crate::runtime::text_evidence::build_full_text_evidence;
use lexflex_parser::ParseError;
use std::collections::BTreeMap;

impl LexFlexRuntime {
    pub(crate) fn handle_ingest_text(&mut self, input: TextInput) -> EngineResponse {
        if let Err(error) = input.validate() {
            return EngineResponse::Error {
                code: match error {
                    crate::api::input_error::TextInputError::EmptySourceId => {
                        EngineErrorCode::InvalidEvidence
                    }
                    crate::api::input_error::TextInputError::EmptyText => {
                        EngineErrorCode::InvalidProgram
                    }
                },
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }

        let parse = match self.parse_text(&input) {
            Ok(ParseOutput::Assertion(draft)) => draft,
            Ok(ParseOutput::Goal(_)) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![ParseError::NoParse],
                };
            }
            Ok(ParseOutput::Ambiguous {
                mode, alternatives, ..
            }) => {
                if mode != lexflex_parser::ClauseMode::Declarative {
                    return EngineResponse::TextNotParsed {
                        diagnostics: vec![ParseError::UnexpectedQueryVariable],
                    };
                }
                return match self.lower_ambiguous_alternatives(
                    &input.source_id,
                    alternatives,
                    false,
                    ExpectedTextKind::Assertion,
                ) {
                    Ok((mut alternatives, diagnostics)) if alternatives.len() == 1 => {
                        let alternative = alternatives.remove(0);
                        self.ingest_lowered_assertion(input, alternative, diagnostics)
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
            BTreeMap::new(),
        ) {
            Ok(value) => value,
            Err(error) => {
                return formal_error_response(error);
            }
        };
        if semantic_expression.entry_type != lexflex_model::SemanticType::Boolean {
            let error =
                FormalExpressionError::NonBooleanAssertion(semantic_expression.entry_type.clone());
            return formal_error_response(error);
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
                return formal_error_response(error);
            }
        };
        let built = match build_full_text_evidence(&input) {
            Ok(value) => value,
            Err(response) => return response,
        };
        let assertion = match SemanticAssertion::create(
            semantic_expression.value,
            built.evidence,
            WorldId::new_unchecked("actual"),
            self.lingua.catalog().as_ref(),
        ) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidAssertion,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };
        self.persist_text_assertion(analysis, assertion, Vec::new())
    }

    fn ingest_lowered_assertion(
        &mut self,
        input: TextInput,
        alternative: TextAnalysisAlternative,
        diagnostics: Vec<crate::api::response::EngineDiagnostic>,
    ) -> EngineResponse {
        let built = match build_full_text_evidence(&input) {
            Ok(value) => value,
            Err(response) => return prepend_diagnostics(response, diagnostics),
        };
        let span = built.span;
        let evidence = built.evidence;
        let analysis = match TextAnalysis::new(TextAnalysisInput {
            source_id: input.source_id.clone(),
            language: input.language.clone(),
            span,
            kind: TextAnalysisKind::Assertion,
            canonical_expression: alternative.canonical_expression().clone(),
            variables: BTreeMap::new(),
            projection: Vec::new(),
            formal_steps: alternative.formal_steps(),
            parser_metrics: alternative.parser_metrics().clone(),
            derivations: alternative.derivations().cloned(),
        }) {
            Ok(value) => value,
            Err(error) => {
                return crate::runtime::formal_error_mapping::analysis_error_response(
                    error,
                    diagnostics,
                );
            }
        };
        let assertion = match SemanticAssertion::create(
            alternative.canonical_expression().clone(),
            evidence,
            WorldId::new_unchecked("actual"),
            self.lingua.catalog().as_ref(),
        ) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidAssertion,
                    message: error.to_string(),
                    diagnostics,
                };
            }
        };
        self.persist_text_assertion(analysis, assertion, diagnostics)
    }

    fn persist_text_assertion(
        &mut self,
        analysis: TextAnalysis,
        assertion: SemanticAssertion,
        diagnostics: Vec<crate::api::response::EngineDiagnostic>,
    ) -> EngineResponse {
        let (assertion, outcome) = match self.persist_assertion(assertion) {
            Ok(value) => value,
            Err(response) => return prepend_diagnostics(response, diagnostics),
        };
        EngineResponse::TextIngested {
            analysis,
            assertion,
            outcome,
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
            diagnostics,
        }
    }

    pub(crate) fn handle_ingest(
        &mut self,
        program: LinguaProgram,
        evidence: lexflex_model::EvidenceSet,
    ) -> EngineResponse {
        let result = match self.lingua.evaluate_with_policy(
            &program,
            ExecutionPolicy {
                expansion: ExpansionMode::PreserveApplications,
            },
        ) {
            Ok(result) => result,
            Err(error) => {
                return request_program_error_response(error);
            }
        };
        if result.entry_type != lexflex_model::SemanticType::Boolean {
            return EngineResponse::Error {
                code: EngineErrorCode::InvalidAssertion,
                message: FormalExpressionError::NonBooleanAssertion(result.entry_type).to_string(),
                diagnostics: Vec::new(),
            };
        }
        if let Err(error) = evidence.verify() {
            return evidence_error_response(error, EvidenceOrigin::Incoming);
        }
        let assertion = match SemanticAssertion::create(
            result.execution.value,
            evidence,
            WorldId::new_unchecked("actual"),
            self.lingua.catalog().as_ref(),
        ) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidAssertion,
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
