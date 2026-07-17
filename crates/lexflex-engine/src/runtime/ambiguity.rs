use super::*;
use crate::api::response::EngineDiagnostic;
use crate::runtime::formal_result::FormalExpressionError;
use lexflex_model::{canonical_hash, CanonicalDigest};
use lexflex_parser::ParseScore;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpectedTextKind {
    Assertion,
    Goal,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FormalAlternativeKey {
    expression_hash: CanonicalDigest,
    variable_hash: CanonicalDigest,
    projection_hash: CanonicalDigest,
}

#[derive(Debug, Clone)]
struct LoweredAlternative {
    analysis: TextAnalysisAlternative,
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
        let mut lowered = BTreeMap::<FormalAlternativeKey, LoweredAlternative>::new();
        let mut failures = Vec::new();

        for (index, alternative) in alternatives.into_iter().enumerate() {
            let result = match expected {
                ExpectedTextKind::Assertion => {
                    if !alternative.query_variables.is_empty() {
                        Err("assertion alternative contains query variables".to_owned())
                    } else {
                        self.evaluate_formal_expression(
                            source_id,
                            alternative.expression.clone(),
                            BTreeMap::new(),
                        )
                        .map_err(|error| error.to_string())
                    }
                }
                ExpectedTextKind::Goal => {
                    if alternative.query_variables.is_empty() {
                        Err("goal alternative does not declare query variables".to_owned())
                    } else if alternative.projection.is_empty() {
                        Err("goal alternative does not declare projection".to_owned())
                    } else {
                        self.evaluate_formal_expression(
                            source_id,
                            alternative.expression.clone(),
                            alternative.query_variables.clone(),
                        )
                        .map_err(|error| error.to_string())
                    }
                }
            };

            let semantic_expression = match result {
                Ok(value) => value,
                Err(error) => {
                    failures.push(EngineDiagnostic::InvalidAlternative {
                        index,
                        message: error.to_string(),
                    });
                    continue;
                }
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

            let expression_hash = match canonical_hash(&semantic_value) {
                Ok(value) => value,
                Err(error) => {
                    failures.push(EngineDiagnostic::InvalidAlternative {
                        index,
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            let variable_hash = match canonical_hash(&alternative.query_variables) {
                Ok(value) => value,
                Err(error) => {
                    failures.push(EngineDiagnostic::InvalidAlternative {
                        index,
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            let projection_hash = match canonical_hash(&alternative.projection) {
                Ok(value) => value,
                Err(error) => {
                    failures.push(EngineDiagnostic::InvalidAlternative {
                        index,
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            let analysis = match TextAnalysisAlternative::new(
                semantic_value,
                alternative.query_variables,
                alternative.projection,
                formal_steps,
                alternative.metrics,
                if include_derivation {
                    alternative.derivations.primary().cloned()
                } else {
                    None
                },
                alternative.score,
            ) {
                Ok(value) => value,
                Err(error) => {
                    failures.push(EngineDiagnostic::InvalidAlternative {
                        index,
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            let key = FormalAlternativeKey {
                expression_hash,
                variable_hash,
                projection_hash,
            };
            let lowered_alternative = LoweredAlternative {
                analysis,
                score: alternative.score,
            };

            match lowered.get(&key) {
                None => {
                    lowered.insert(key, lowered_alternative);
                }
                Some(existing) if lowered_alternative.score < existing.score => {
                    lowered.insert(key, lowered_alternative);
                }
                Some(_) => {}
            }
        }

        if lowered.is_empty() {
            return Err(EngineResponse::Error {
                code: EngineErrorCode::InvalidProgram,
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
            .map(|value| value.analysis)
            .collect::<Vec<_>>();

        Ok((alternatives, failures))
    }
}
