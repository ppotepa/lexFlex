use lexflex_model::{SemanticExpression, VariableId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Substitution {
    bindings: BTreeMap<VariableId, SemanticExpression>,
}

impl Substitution {
    pub fn bind(
        &mut self,
        variable: VariableId,
        value: SemanticExpression,
    ) -> Result<(), crate::solve::UnifyError> {
        match self.bindings.get(&variable) {
            Some(existing) if existing != &value => {
                Err(crate::solve::UnifyError::ConflictingBinding {
                    variable,
                    existing: existing.clone(),
                    incoming: value,
                })
            }
            Some(_) => Ok(()),
            None => {
                self.bindings.insert(variable, value);
                Ok(())
            }
        }
    }

    pub fn get(&self, variable: &VariableId) -> Option<&SemanticExpression> {
        self.bindings.get(variable)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&VariableId, &SemanticExpression)> {
        self.bindings.iter()
    }

    pub fn apply(&self, expression: &SemanticExpression) -> SemanticExpression {
        match expression {
            SemanticExpression::Variable(variable) => self
                .get(variable)
                .cloned()
                .unwrap_or_else(|| expression.clone()),
            SemanticExpression::Apply { concept, bindings } => SemanticExpression::Apply {
                concept: concept.clone(),
                bindings: bindings
                    .iter()
                    .map(|(parameter, value)| (parameter.clone(), self.apply(value)))
                    .collect(),
            },
            SemanticExpression::Satisfies { subject, predicate } => SemanticExpression::Satisfies {
                subject: Box::new(self.apply(subject)),
                predicate: Box::new(self.apply(predicate)),
            },
            SemanticExpression::Equals { left, right } => SemanticExpression::Equals {
                left: Box::new(self.apply(left)),
                right: Box::new(self.apply(right)),
            },
            SemanticExpression::And(items) => {
                SemanticExpression::And(items.iter().map(|item| self.apply(item)).collect())
            }
            SemanticExpression::Or(items) => {
                SemanticExpression::Or(items.iter().map(|item| self.apply(item)).collect())
            }
            SemanticExpression::Not(inner) => SemanticExpression::Not(Box::new(self.apply(inner))),
            SemanticExpression::Exists { variable, body } => {
                if self.bindings.contains_key(variable) {
                    let mut child = self.clone();
                    child.bindings.remove(variable);
                    SemanticExpression::Exists {
                        variable: variable.clone(),
                        body: Box::new(child.apply(body)),
                    }
                } else {
                    SemanticExpression::Exists {
                        variable: variable.clone(),
                        body: Box::new(self.apply(body)),
                    }
                }
            }
            SemanticExpression::ForAll { variable, body } => {
                if self.bindings.contains_key(variable) {
                    let mut child = self.clone();
                    child.bindings.remove(variable);
                    SemanticExpression::ForAll {
                        variable: variable.clone(),
                        body: Box::new(child.apply(body)),
                    }
                } else {
                    SemanticExpression::ForAll {
                        variable: variable.clone(),
                        body: Box::new(self.apply(body)),
                    }
                }
            }
            SemanticExpression::Qualified {
                expression,
                qualifiers,
            } => SemanticExpression::Qualified {
                expression: Box::new(self.apply(expression)),
                qualifiers: qualifiers
                    .iter()
                    .map(|(id, value)| (id.clone(), self.apply(value)))
                    .collect(),
            },
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_model::{ConceptId, EntityId, SemanticExpression};

    #[test]
    fn substitution_does_not_cross_shadowed_exists() {
        let variable = VariableId::new_unchecked("x");
        let other = VariableId::new_unchecked("y");

        let mut substitution = Substitution::default();
        substitution
            .bind(
                variable.clone(),
                SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
            )
            .expect("bind x");
        substitution
            .bind(
                other.clone(),
                SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
            )
            .expect("bind y");

        let expression = SemanticExpression::Exists {
            variable: variable.clone(),
            body: Box::new(SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(variable.clone())),
                predicate: Box::new(SemanticExpression::Variable(other.clone())),
            }),
        };

        let applied = substitution.apply(&expression);

        assert_eq!(
            applied,
            SemanticExpression::Exists {
                variable,
                body: Box::new(SemanticExpression::Satisfies {
                    subject: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("x"))),
                    predicate: Box::new(SemanticExpression::Entity(EntityId::new_unchecked(
                        "FRANCE"
                    ))),
                }),
            }
        );
    }

    #[test]
    fn substitution_does_not_cross_shadowed_forall() {
        let variable = VariableId::new_unchecked("x");
        let expression = SemanticExpression::ForAll {
            variable: variable.clone(),
            body: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("PREDICATE"),
                bindings: BTreeMap::from([(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Variable(variable.clone()),
                )]),
            }),
        };

        let mut substitution = Substitution::default();
        substitution
            .bind(
                variable.clone(),
                SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
            )
            .expect("bind x");

        let applied = substitution.apply(&expression);
        assert_eq!(applied, expression);
    }
}
