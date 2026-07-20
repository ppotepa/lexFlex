use crate::SemanticExpression;
use std::collections::BTreeMap;

use super::alpha::AlphaState;
use super::logical::{collapse_and, collapse_or};
use super::{NormalizationError, NormalizationReport};

#[derive(Debug, Clone)]
pub struct NormalizationBudget {
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_variable_attempts: usize,
}

impl Default for NormalizationBudget {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_depth: 512,
            max_variable_attempts: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SemanticNormalizer;

struct NormalizationState {
    budget: NormalizationBudget,
    alpha: AlphaState,
    nodes: usize,
    report: NormalizationReport,
}

pub fn normalize_expression(
    expression: SemanticExpression,
) -> Result<NormalizationOutput, NormalizationError> {
    SemanticNormalizer.normalize(expression)
}

#[derive(Debug, Clone)]
pub struct NormalizationOutput {
    pub expression: SemanticExpression,
    pub report: NormalizationReport,
}

impl SemanticNormalizer {
    pub fn normalize(
        &self,
        expression: SemanticExpression,
    ) -> Result<NormalizationOutput, NormalizationError> {
        let original = expression.clone();
        let input_nodes = count_nodes(&expression);
        let mut state = NormalizationState {
            budget: NormalizationBudget::default(),
            alpha: AlphaState::from_expression(&expression),
            nodes: 0,
            report: NormalizationReport {
                changed: false,
                input_nodes,
                output_nodes: 0,
                max_depth: 0,
                renamed_bound_variables: 0,
                flattened_logical_nodes: 0,
                removed_duplicates: 0,
            },
        };
        let normalized = state.normalize(expression, 0)?;
        state.report.changed = normalized != original;
        state.report.output_nodes = count_nodes(&normalized);
        Ok(NormalizationOutput {
            expression: normalized,
            report: state.report,
        })
    }
}

impl NormalizationState {
    fn normalize(
        &mut self,
        expression: SemanticExpression,
        depth: usize,
    ) -> Result<SemanticExpression, NormalizationError> {
        self.nodes += 1;
        if self.nodes > self.budget.max_nodes {
            return Err(NormalizationError::NodeBudgetExceeded {
                max: self.budget.max_nodes,
            });
        }
        if depth > self.budget.max_depth {
            return Err(NormalizationError::DepthBudgetExceeded {
                max: self.budget.max_depth,
            });
        }
        self.report.max_depth = self.report.max_depth.max(depth);

        match expression {
            SemanticExpression::Apply { concept, bindings } => Ok(SemanticExpression::Apply {
                concept,
                bindings: bindings
                    .into_iter()
                    .map(|(parameter, value)| {
                        self.normalize(value, depth + 1).map(|v| (parameter, v))
                    })
                    .collect::<Result<BTreeMap<_, _>, _>>()?,
            }),
            SemanticExpression::Satisfies { subject, predicate } => {
                Ok(SemanticExpression::Satisfies {
                    subject: Box::new(self.normalize(*subject, depth + 1)?),
                    predicate: Box::new(self.normalize(*predicate, depth + 1)?),
                })
            }
            SemanticExpression::Equals { left, right } => Ok(SemanticExpression::Equals {
                left: Box::new(self.normalize(*left, depth + 1)?),
                right: Box::new(self.normalize(*right, depth + 1)?),
            }),
            SemanticExpression::And(items) => {
                let mut flat = Vec::new();
                for item in items {
                    match self.normalize(item, depth + 1)? {
                        SemanticExpression::And(nested) => {
                            self.report.flattened_logical_nodes += 1;
                            flat.extend(nested);
                        }
                        other => flat.push(other),
                    }
                }
                Ok(collapse_and(flat, &mut self.report))
            }
            SemanticExpression::Or(items) => {
                let mut flat = Vec::new();
                for item in items {
                    match self.normalize(item, depth + 1)? {
                        SemanticExpression::Or(nested) => {
                            self.report.flattened_logical_nodes += 1;
                            flat.extend(nested);
                        }
                        other => flat.push(other),
                    }
                }
                Ok(collapse_or(flat, &mut self.report))
            }
            SemanticExpression::Not(inner) => match self.normalize(*inner, depth + 1)? {
                SemanticExpression::Not(double) => Ok(*double),
                inner => Ok(SemanticExpression::Not(Box::new(inner))),
            },
            SemanticExpression::Exists {
                variable,
                value_type,
                body,
            } => {
                let fresh = self.alpha.allocate_bound(&self.budget)?;
                self.report.renamed_bound_variables += usize::from(fresh != variable);
                self.alpha.push(variable, fresh.clone());
                let body = self.normalize(*body, depth + 1)?;
                self.alpha.pop();
                Ok(SemanticExpression::Exists {
                    variable: fresh,
                    value_type,
                    body: Box::new(body),
                })
            }
            SemanticExpression::ForAll {
                variable,
                value_type,
                body,
            } => {
                let fresh = self.alpha.allocate_bound(&self.budget)?;
                self.report.renamed_bound_variables += usize::from(fresh != variable);
                self.alpha.push(variable, fresh.clone());
                let body = self.normalize(*body, depth + 1)?;
                self.alpha.pop();
                Ok(SemanticExpression::ForAll {
                    variable: fresh,
                    value_type,
                    body: Box::new(body),
                })
            }
            SemanticExpression::Qualified {
                expression,
                qualifiers,
            } => Ok(SemanticExpression::Qualified {
                expression: Box::new(self.normalize(*expression, depth + 1)?),
                qualifiers: qualifiers
                    .into_iter()
                    .map(|(qualifier, value)| {
                        self.normalize(value, depth + 1).map(|v| (qualifier, v))
                    })
                    .collect::<Result<BTreeMap<_, _>, _>>()?,
            }),
            SemanticExpression::Variable(variable) => {
                Ok(SemanticExpression::Variable(self.alpha.resolve(&variable)))
            }
            SemanticExpression::Concept(id) => Ok(SemanticExpression::Concept(id)),
            SemanticExpression::Entity(id) => Ok(SemanticExpression::Entity(id)),
            SemanticExpression::Value(value) => Ok(SemanticExpression::Value(value)),
        }
    }
}

fn count_nodes(expression: &SemanticExpression) -> usize {
    match expression {
        SemanticExpression::Apply { bindings, .. } => {
            1 + bindings.values().map(count_nodes).sum::<usize>()
        }
        SemanticExpression::Satisfies { subject, predicate }
        | SemanticExpression::Equals {
            left: subject,
            right: predicate,
        } => 1 + count_nodes(subject) + count_nodes(predicate),
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            1 + items.iter().map(count_nodes).sum::<usize>()
        }
        SemanticExpression::Not(inner)
        | SemanticExpression::Exists { body: inner, .. }
        | SemanticExpression::ForAll { body: inner, .. } => 1 + count_nodes(inner),
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => 1 + count_nodes(expression) + qualifiers.values().map(count_nodes).sum::<usize>(),
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_)
        | SemanticExpression::Variable(_) => 1,
    }
}
