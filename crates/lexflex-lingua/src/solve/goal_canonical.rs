use crate::normalize::normalize_expression;
use crate::solve::LinguaGoal;
use crate::types::SemanticType;
use lexflex_model::{SemanticExpression, VariableId, WorldId};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CanonicalSemanticGoal {
    pub(crate) expression: SemanticExpression,
    pub(crate) variables: BTreeMap<VariableId, SemanticType>,
    pub(crate) projection: Vec<VariableId>,
    pub(crate) world: Option<WorldId>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CanonicalRequestGoal {
    pub(crate) semantic: CanonicalSemanticGoal,
    pub(crate) evidence_policy: crate::solve::EvidencePolicy,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Default)]
struct GoalCanonicalizer {
    next_free: usize,
    next_bound: usize,
    free_mapping: BTreeMap<VariableId, VariableId>,
    bound_stack: Vec<(VariableId, VariableId)>,
    seen_free_order: Vec<VariableId>,
    seen_free_set: BTreeSet<VariableId>,
}

impl GoalCanonicalizer {
    fn canonicalize_expression(&mut self, expression: &SemanticExpression) -> SemanticExpression {
        match expression {
            SemanticExpression::Variable(variable) => {
                SemanticExpression::Variable(self.rename_free(variable))
            }
            SemanticExpression::Apply { concept, bindings } => SemanticExpression::Apply {
                concept: concept.clone(),
                bindings: bindings
                    .iter()
                    .map(|(parameter, value)| {
                        (parameter.clone(), self.canonicalize_expression(value))
                    })
                    .collect(),
            },
            SemanticExpression::Satisfies { subject, predicate } => SemanticExpression::Satisfies {
                subject: Box::new(self.canonicalize_expression(subject)),
                predicate: Box::new(self.canonicalize_expression(predicate)),
            },
            SemanticExpression::Equals { left, right } => SemanticExpression::Equals {
                left: Box::new(self.canonicalize_expression(left)),
                right: Box::new(self.canonicalize_expression(right)),
            },
            SemanticExpression::And(items) => SemanticExpression::And(
                items
                    .iter()
                    .map(|item| self.canonicalize_expression(item))
                    .collect(),
            ),
            SemanticExpression::Or(items) => SemanticExpression::Or(
                items
                    .iter()
                    .map(|item| self.canonicalize_expression(item))
                    .collect(),
            ),
            SemanticExpression::Not(inner) => {
                SemanticExpression::Not(Box::new(self.canonicalize_expression(inner)))
            }
            SemanticExpression::Exists {
                variable,
                value_type,
                body,
            } => {
                let fresh = self.rename_bound(variable);
                let body = self.canonicalize_expression(body);
                self.bound_stack.pop();
                SemanticExpression::Exists {
                    variable: fresh,
                    value_type: value_type.clone(),
                    body: Box::new(body),
                }
            }
            SemanticExpression::ForAll {
                variable,
                value_type,
                body,
            } => {
                let fresh = self.rename_bound(variable);
                let body = self.canonicalize_expression(body);
                self.bound_stack.pop();
                SemanticExpression::ForAll {
                    variable: fresh,
                    value_type: value_type.clone(),
                    body: Box::new(body),
                }
            }
            SemanticExpression::Qualified {
                expression,
                qualifiers,
            } => SemanticExpression::Qualified {
                expression: Box::new(self.canonicalize_expression(expression)),
                qualifiers: qualifiers
                    .iter()
                    .map(|(qualifier, value)| {
                        (qualifier.clone(), self.canonicalize_expression(value))
                    })
                    .collect(),
            },
            SemanticExpression::Concept(id) => SemanticExpression::Concept(id.clone()),
            SemanticExpression::Entity(id) => SemanticExpression::Entity(id.clone()),
            SemanticExpression::Value(value) => SemanticExpression::Value(value.clone()),
        }
    }

    fn canonicalize_variables(
        &self,
        variables: &BTreeMap<VariableId, SemanticType>,
    ) -> BTreeMap<VariableId, SemanticType> {
        self.seen_free_order
            .iter()
            .filter_map(|variable| {
                variables.get(variable).map(|semantic_type| {
                    (
                        self.free_mapping
                            .get(variable)
                            .cloned()
                            .unwrap_or_else(|| variable.clone()),
                        semantic_type.clone(),
                    )
                })
            })
            .collect()
    }

    fn canonicalize_projection(&self, projection: &[VariableId]) -> Vec<VariableId> {
        projection
            .iter()
            .map(|variable| {
                self.free_mapping
                    .get(variable)
                    .cloned()
                    .unwrap_or_else(|| variable.clone())
            })
            .collect()
    }

    fn rename_free(&mut self, variable: &VariableId) -> VariableId {
        if let Some((_, mapped)) = self
            .bound_stack
            .iter()
            .rev()
            .find(|(bound, _)| bound == variable)
        {
            return mapped.clone();
        }

        if self.seen_free_set.insert(variable.clone()) {
            self.seen_free_order.push(variable.clone());
        }

        self.free_mapping
            .entry(variable.clone())
            .or_insert_with(|| {
                let renamed = VariableId::new_unchecked(format!("q{}", self.next_free));
                self.next_free += 1;
                renamed
            })
            .clone()
    }

    fn rename_bound(&mut self, variable: &VariableId) -> VariableId {
        let renamed = VariableId::new_unchecked(format!("b{}", self.next_bound));
        self.next_bound += 1;
        self.bound_stack.push((variable.clone(), renamed.clone()));
        renamed
    }
}

pub(crate) fn canonical_semantic_goal(
    goal: &LinguaGoal,
) -> Result<CanonicalSemanticGoal, crate::normalize::NormalizationError> {
    let expression = normalize_expression(goal.expression.clone())?.expression;
    let mut canonicalizer = GoalCanonicalizer::default();
    let expression = canonicalizer.canonicalize_expression(&expression);
    let variables = canonicalizer.canonicalize_variables(&goal.variables);
    let projection = canonicalizer.canonicalize_projection(&goal.projection);

    Ok(CanonicalSemanticGoal {
        expression,
        variables,
        projection,
        world: goal.world.clone(),
    })
}
