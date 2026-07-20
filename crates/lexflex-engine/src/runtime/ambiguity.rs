use super::*;
use crate::api::response::EngineDiagnostic;
use crate::api::text::TextAnalysisAlternativeInput;
use crate::runtime::alternative_failure::{classify_alternative_failure, AlternativeFailure};
use crate::runtime::diagnostics::with_diagnostics;
use crate::runtime::formal_identity::{formal_alternative_key, FormalAlternativeKey};
use crate::runtime::formal_result::FormalExpressionError;
use lexflex_parser::ParseScore;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpectedTextKind {
    Assertion,
    Goal,
}

#[derive(Debug, Clone)]
struct LoweredAlternative {
    analysis: TextAnalysisAlternative,
    internal_derivations: lexflex_parser::DerivationSet,
    score: ParseScore,
}

impl LexFlexRuntime {
    pub(crate) fn lower_ambiguous_alternatives(
        &self,
        source_id: &str,
        alternatives: Vec<lexflex_parser::ParseAlternative>,
        include_derivation: bool,
        expected: ExpectedTextKind,
    ) -> Result<(Vec<TextAnalysisAlternative>, Vec<EngineDiagnostic>), EngineResponse> {
        let observed_alternatives = alternatives.len();
        let max_alternatives = self.text_budget.max_formal_alternatives();
        if observed_alternatives > max_alternatives {
            return Err(EngineResponse::Error {
                code: EngineErrorCode::RuntimeBudget,
                message: format!(
                    "formal alternative limit exceeded: observed={observed_alternatives}, max={max_alternatives}"
                ),
                diagnostics: Vec::new(),
            });
        }

        let mut lowered = BTreeMap::<FormalAlternativeKey, LoweredAlternative>::new();
        let mut failures = Vec::new();

        for (index, alternative) in alternatives.into_iter().enumerate() {
            let result = match expected {
                ExpectedTextKind::Assertion => {
                    if !alternative.query_variables.is_empty() {
                        failures.push(EngineDiagnostic::InvalidAlternative {
                            index,
                            message: "assertion alternative contains query variables".into(),
                        });
                        continue;
                    } else {
                        self.evaluate_formal_expression(
                            source_id,
                            alternative.expression.clone(),
                            BTreeMap::new(),
                        )
                    }
                }
                ExpectedTextKind::Goal => {
                    if alternative.query_variables.is_empty() {
                        failures.push(EngineDiagnostic::InvalidAlternative {
                            index,
                            message: "goal alternative does not declare query variables".into(),
                        });
                        continue;
                    } else if alternative.projection.is_empty() {
                        failures.push(EngineDiagnostic::InvalidAlternative {
                            index,
                            message: "goal alternative does not declare projection".into(),
                        });
                        continue;
                    } else {
                        self.evaluate_formal_expression(
                            source_id,
                            alternative.expression.clone(),
                            alternative.query_variables.clone(),
                        )
                    }
                }
            };

            let semantic_expression = match result {
                Ok(value) => value,
                Err(error) => match classify_alternative_failure(error) {
                    AlternativeFailure::Rejected { message } => {
                        failures.push(EngineDiagnostic::InvalidAlternative { index, message });
                        continue;
                    }
                    AlternativeFailure::Fatal { response } => {
                        return Err(with_diagnostics(response, failures));
                    }
                },
            };

            let formal_steps = semantic_expression.steps;
            let semantic_value = semantic_expression.value;
            let entry_type = semantic_expression.entry_type;

            match expected {
                ExpectedTextKind::Assertion
                    if entry_type != lexflex_model::SemanticType::Boolean =>
                {
                    let error = FormalExpressionError::NonBooleanAssertion(entry_type);
                    failures.push(EngineDiagnostic::InvalidAlternative {
                        index,
                        message: error.to_string(),
                    });
                    continue;
                }
                ExpectedTextKind::Goal if entry_type != lexflex_model::SemanticType::Boolean => {
                    let error = FormalExpressionError::NonBooleanGoal(entry_type);
                    failures.push(EngineDiagnostic::InvalidAlternative {
                        index,
                        message: error.to_string(),
                    });
                    continue;
                }
                _ => {}
            }

            let analysis = match TextAnalysisAlternative::new(
                match expected {
                    ExpectedTextKind::Assertion => TextAnalysisKind::Assertion,
                    ExpectedTextKind::Goal => TextAnalysisKind::Goal,
                },
                TextAnalysisAlternativeInput {
                    canonical_expression: semantic_value.clone(),
                    variables: alternative.query_variables,
                    projection: alternative.projection,
                    formal_steps,
                    parser_metrics: alternative.metrics,
                    derivations: None,
                    score: alternative.score,
                },
            ) {
                Ok(value) => value,
                Err(error) => {
                    if !matches!(error, crate::api::text::TextAnalysisError::CanonicalHash(_)) {
                        failures.push(EngineDiagnostic::InvalidAlternative {
                            index,
                            message: error.to_string(),
                        });
                        continue;
                    }
                    return Err(with_diagnostics(
                        EngineResponse::Error {
                            code: EngineErrorCode::Canonicalization,
                            message: error.to_string(),
                            diagnostics: Vec::new(),
                        },
                        failures,
                    ));
                }
            };

            let key = match formal_alternative_key(
                analysis.canonical_hash().clone(),
                analysis.variables(),
                analysis.projection(),
            ) {
                Ok(key) => key,
                Err(error) => {
                    return Err(with_diagnostics(
                        EngineResponse::Error {
                            code: EngineErrorCode::Canonicalization,
                            message: error.to_string(),
                            diagnostics: Vec::new(),
                        },
                        failures,
                    ));
                }
            };

            let lowered_alternative = LoweredAlternative {
                analysis,
                internal_derivations: alternative.derivations,
                score: alternative.score,
            };

            match lowered.get_mut(&key) {
                None => {
                    lowered.insert(key, lowered_alternative);
                }
                Some(existing) if lowered_alternative.score < existing.score => {
                    lowered.insert(key, lowered_alternative);
                }
                Some(existing) if lowered_alternative.score == existing.score => {
                    existing
                        .internal_derivations
                        .try_merge(
                            &lowered_alternative.internal_derivations,
                            self.text_budget.max_formal_derivations(),
                        )
                        .map_err(|error| EngineResponse::Error {
                            code: EngineErrorCode::RuntimeBudget,
                            message: error.to_string(),
                            diagnostics: failures.clone(),
                        })?;
                }
                Some(_) => {}
            }
        }

        if lowered.is_empty() {
            return Err(EngineResponse::Error {
                code: match expected {
                    ExpectedTextKind::Assertion => EngineErrorCode::InvalidAssertion,
                    ExpectedTextKind::Goal => EngineErrorCode::InvalidGoal,
                },
                message: "all ambiguous alternatives failed formal lowering".into(),
                diagnostics: failures,
            });
        }

        let best_score = lowered
            .values()
            .map(|value| value.score)
            .min()
            .ok_or_else(|| EngineResponse::Error {
                code: EngineErrorCode::InternalInvariant,
                message: "lowered alternatives are empty after non-empty check".into(),
                diagnostics: failures.clone(),
            })?;

        let alternatives = lowered
            .into_values()
            .filter(|value| value.score == best_score)
            .map(|mut value| {
                if include_derivation {
                    value
                        .analysis
                        .set_derivations(Some(value.internal_derivations));
                }
                value.analysis
            })
            .collect::<Vec<_>>();

        Ok((alternatives, failures))
    }
}
