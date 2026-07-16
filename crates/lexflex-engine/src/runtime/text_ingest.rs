use super::*;
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
            Ok(ParseOutput::Ambiguous { alternatives }) => {
                let mut lowered = Vec::new();
                for alternative in alternatives {
                    let semantic_expression = match self.evaluate_formal_expression(
                        "ingest:ambiguous",
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
        let analysis = self.assertion_analysis(&parse, semantic_expression.clone(), false);
        let source_hash = canonical_hash(&input.text);
        let evidence = Evidence {
            id: EvidenceId::new_unchecked(format!("evidence:{source_hash}")),
            source_id: input.source_id.clone(),
            span: Some(SourceSpan {
                start: 0,
                end: input.text.len() as u64,
            }),
            source_hash: Some(source_hash),
        };
        let assertion = SemanticAssertion::create(
            semantic_expression,
            vec![evidence],
            WorldId::new_unchecked("actual"),
        );
        let (assertion, outcome) = match self.persist_assertion(assertion) {
            Ok(value) => value,
            Err(response) => return response,
        };
        EngineResponse::TextIngested {
            analysis,
            assertion,
            outcome,
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
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
        let assertion =
            SemanticAssertion::create(result.value, evidence, WorldId::new_unchecked("actual"));
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
