use crate::normalize::normalize_expression;
use crate::types::SemanticType;
use lexflex_model::{canonical_hash, SemanticExpression, VariableId, WorldId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidencePolicy {
    Required,
    Optional,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinguaGoal {
    pub expression: SemanticExpression,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub evidence_policy: EvidencePolicy,
    pub world: Option<WorldId>,
    pub limit: Option<usize>,
}

pub fn canonical_goal_hash(goal: &LinguaGoal) -> String {
    canonical_hash(&canonical_goal(goal))
}

#[derive(Debug, Clone, Serialize)]
struct CanonicalGoal {
    expression: SemanticExpression,
    variables: BTreeMap<VariableId, SemanticType>,
    projection: Vec<VariableId>,
    evidence_policy: EvidencePolicy,
    world: Option<WorldId>,
    limit: Option<usize>,
}

fn canonical_goal(goal: &LinguaGoal) -> CanonicalGoal {
    let expression = normalize_expression(goal.expression.clone());
    let mut mapping = BTreeMap::new();
    let mut next_index = 0usize;

    for variable in collect_variables(&expression) {
        if goal.variables.contains_key(&variable) {
            mapping
                .entry(variable)
                .or_insert_with(|| fresh_goal_variable(&mut next_index));
        }
    }

    for variable in &goal.projection {
        if goal.variables.contains_key(variable) {
            mapping
                .entry(variable.clone())
                .or_insert_with(|| fresh_goal_variable(&mut next_index));
        }
    }

    for variable in goal.variables.keys() {
        mapping
            .entry(variable.clone())
            .or_insert_with(|| fresh_goal_variable(&mut next_index));
    }

    let expression = rename_expression(&expression, &mapping);
    let variables = goal
        .variables
        .iter()
        .map(|(variable, semantic_type)| {
            (
                mapping
                    .get(variable)
                    .cloned()
                    .unwrap_or_else(|| variable.clone()),
                semantic_type.clone(),
            )
        })
        .collect();
    let projection = goal
        .projection
        .iter()
        .map(|variable| {
            mapping
                .get(variable)
                .cloned()
                .unwrap_or_else(|| variable.clone())
        })
        .collect();

    CanonicalGoal {
        expression,
        variables,
        projection,
        evidence_policy: goal.evidence_policy,
        world: goal.world.clone(),
        limit: goal.limit,
    }
}

fn fresh_goal_variable(next_index: &mut usize) -> VariableId {
    let variable = VariableId::new_unchecked(format!("q{}", next_index));
    *next_index += 1;
    variable
}

fn collect_variables(expression: &SemanticExpression) -> Vec<VariableId> {
    let mut collected = Vec::new();
    collect_variables_inner(expression, &mut collected);
    collected
}

fn collect_variables_inner(expression: &SemanticExpression, collected: &mut Vec<VariableId>) {
    match expression {
        SemanticExpression::Variable(variable) => {
            if !collected.contains(variable) {
                collected.push(variable.clone());
            }
        }
        SemanticExpression::Apply { bindings, .. } => {
            for value in bindings.values() {
                collect_variables_inner(value, collected);
            }
        }
        SemanticExpression::Satisfies { subject, predicate } => {
            collect_variables_inner(subject, collected);
            collect_variables_inner(predicate, collected);
        }
        SemanticExpression::Equals { left, right } => {
            collect_variables_inner(left, collected);
            collect_variables_inner(right, collected);
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            for item in items {
                collect_variables_inner(item, collected);
            }
        }
        SemanticExpression::Not(inner)
        | SemanticExpression::Exists { body: inner, .. }
        | SemanticExpression::ForAll { body: inner, .. } => {
            collect_variables_inner(inner, collected);
        }
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            collect_variables_inner(expression, collected);
            for value in qualifiers.values() {
                collect_variables_inner(value, collected);
            }
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_) => {}
    }
}

fn rename_expression(
    expression: &SemanticExpression,
    mapping: &BTreeMap<VariableId, VariableId>,
) -> SemanticExpression {
    match expression {
        SemanticExpression::Variable(variable) => mapping
            .get(variable)
            .cloned()
            .map(SemanticExpression::Variable)
            .unwrap_or_else(|| SemanticExpression::Variable(variable.clone())),
        SemanticExpression::Apply { concept, bindings } => SemanticExpression::Apply {
            concept: concept.clone(),
            bindings: bindings
                .iter()
                .map(|(parameter, value)| (parameter.clone(), rename_expression(value, mapping)))
                .collect(),
        },
        SemanticExpression::Satisfies { subject, predicate } => SemanticExpression::Satisfies {
            subject: Box::new(rename_expression(subject, mapping)),
            predicate: Box::new(rename_expression(predicate, mapping)),
        },
        SemanticExpression::Equals { left, right } => SemanticExpression::Equals {
            left: Box::new(rename_expression(left, mapping)),
            right: Box::new(rename_expression(right, mapping)),
        },
        SemanticExpression::And(items) => SemanticExpression::And(
            items
                .iter()
                .map(|item| rename_expression(item, mapping))
                .collect(),
        ),
        SemanticExpression::Or(items) => SemanticExpression::Or(
            items
                .iter()
                .map(|item| rename_expression(item, mapping))
                .collect(),
        ),
        SemanticExpression::Not(inner) => {
            SemanticExpression::Not(Box::new(rename_expression(inner, mapping)))
        }
        SemanticExpression::Exists { variable, body } => SemanticExpression::Exists {
            variable: mapping
                .get(variable)
                .cloned()
                .unwrap_or_else(|| variable.clone()),
            body: Box::new(rename_expression(body, mapping)),
        },
        SemanticExpression::ForAll { variable, body } => SemanticExpression::ForAll {
            variable: mapping
                .get(variable)
                .cloned()
                .unwrap_or_else(|| variable.clone()),
            body: Box::new(rename_expression(body, mapping)),
        },
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => SemanticExpression::Qualified {
            expression: Box::new(rename_expression(expression, mapping)),
            qualifiers: qualifiers
                .iter()
                .map(|(qualifier, value)| (qualifier.clone(), rename_expression(value, mapping)))
                .collect(),
        },
        SemanticExpression::Concept(id) => SemanticExpression::Concept(id.clone()),
        SemanticExpression::Entity(id) => SemanticExpression::Entity(id.clone()),
        SemanticExpression::Value(value) => SemanticExpression::Value(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_model::{ConceptId, SemanticExpression, SemanticType};

    #[test]
    fn canonical_goal_hash_is_alpha_equivalent() {
        let left_variable = VariableId::new_unchecked("answer");
        let right_variable = VariableId::new_unchecked("x");

        let left = LinguaGoal {
            expression: SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(left_variable.clone())),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("PREDICATE"),
                    bindings: BTreeMap::new(),
                }),
            },
            variables: BTreeMap::from([(
                left_variable.clone(),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![left_variable],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: Some(10),
        };

        let right = LinguaGoal {
            expression: SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(right_variable.clone())),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("PREDICATE"),
                    bindings: BTreeMap::new(),
                }),
            },
            variables: BTreeMap::from([(
                right_variable.clone(),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![right_variable],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: Some(10),
        };

        assert_eq!(canonical_goal_hash(&left), canonical_goal_hash(&right));
    }
}
