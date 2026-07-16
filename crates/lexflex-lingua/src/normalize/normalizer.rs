use lexflex_model::{SemanticExpression, SemanticValue, VariableId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct ExpressionNormalizer;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationReport {
    pub changed: bool,
}

pub fn normalize_expression(expression: SemanticExpression) -> SemanticExpression {
    ExpressionNormalizer.normalize(expression).0
}

impl ExpressionNormalizer {
    pub fn normalize(
        &self,
        expression: SemanticExpression,
    ) -> (SemanticExpression, NormalizationReport) {
        let normalized = self.normalize_inner(expression.clone(), &mut AlphaState::default());
        (
            normalized.clone(),
            NormalizationReport {
                changed: normalized != expression,
            },
        )
    }

    fn normalize_inner(
        &self,
        expression: SemanticExpression,
        alpha: &mut AlphaState,
    ) -> SemanticExpression {
        match expression {
            SemanticExpression::Apply { concept, bindings } => SemanticExpression::Apply {
                concept,
                bindings: bindings
                    .into_iter()
                    .map(|(parameter, value)| (parameter, self.normalize_inner(value, alpha)))
                    .collect::<BTreeMap<_, _>>(),
            },
            SemanticExpression::Satisfies { subject, predicate } => SemanticExpression::Satisfies {
                subject: Box::new(self.normalize_inner(*subject, alpha)),
                predicate: Box::new(self.normalize_inner(*predicate, alpha)),
            },
            SemanticExpression::Equals { left, right } => SemanticExpression::Equals {
                left: Box::new(self.normalize_inner(*left, alpha)),
                right: Box::new(self.normalize_inner(*right, alpha)),
            },
            SemanticExpression::And(items) => self.normalize_and(items, alpha),
            SemanticExpression::Or(items) => self.normalize_or(items, alpha),
            SemanticExpression::Not(inner) => match self.normalize_inner(*inner, alpha) {
                SemanticExpression::Not(inner) => *inner,
                inner => SemanticExpression::Not(Box::new(inner)),
            },
            SemanticExpression::Exists { variable, body } => {
                let fresh = alpha.fresh();
                alpha
                    .scopes
                    .push(BTreeMap::from([(variable.clone(), fresh.clone())]));
                let body = self.normalize_inner(*body, alpha);
                alpha.scopes.pop();
                SemanticExpression::Exists {
                    variable: fresh,
                    body: Box::new(body),
                }
            }
            SemanticExpression::ForAll { variable, body } => {
                let fresh = alpha.fresh();
                alpha
                    .scopes
                    .push(BTreeMap::from([(variable.clone(), fresh.clone())]));
                let body = self.normalize_inner(*body, alpha);
                alpha.scopes.pop();
                SemanticExpression::ForAll {
                    variable: fresh,
                    body: Box::new(body),
                }
            }
            SemanticExpression::Qualified {
                expression,
                qualifiers,
            } => SemanticExpression::Qualified {
                expression: Box::new(self.normalize_inner(*expression, alpha)),
                qualifiers: qualifiers
                    .into_iter()
                    .map(|(qualifier, value)| (qualifier, self.normalize_inner(value, alpha)))
                    .collect(),
            },
            SemanticExpression::Concept(id) => SemanticExpression::Concept(id),
            SemanticExpression::Entity(id) => SemanticExpression::Entity(id),
            SemanticExpression::Value(value) => SemanticExpression::Value(value),
            SemanticExpression::Variable(variable) => alpha.resolve(variable),
        }
    }

    fn normalize_and(
        &self,
        items: Vec<SemanticExpression>,
        alpha: &mut AlphaState,
    ) -> SemanticExpression {
        let mut flat = Vec::new();
        for item in items {
            match self.normalize_inner(item, alpha) {
                SemanticExpression::And(nested) => flat.extend(nested),
                other => flat.push(other),
            }
        }
        flat.sort();
        flat.dedup();
        match flat.len() {
            0 => SemanticExpression::Value(SemanticValue::Boolean(true)),
            1 => flat.remove(0),
            _ => SemanticExpression::And(flat),
        }
    }

    fn normalize_or(
        &self,
        items: Vec<SemanticExpression>,
        alpha: &mut AlphaState,
    ) -> SemanticExpression {
        let mut flat = Vec::new();
        for item in items {
            match self.normalize_inner(item, alpha) {
                SemanticExpression::Or(nested) => flat.extend(nested),
                other => flat.push(other),
            }
        }
        flat.sort();
        flat.dedup();
        match flat.len() {
            0 => SemanticExpression::Value(SemanticValue::Boolean(false)),
            1 => flat.remove(0),
            _ => SemanticExpression::Or(flat),
        }
    }
}

#[derive(Default)]
struct AlphaState {
    next: u64,
    scopes: Vec<BTreeMap<VariableId, VariableId>>,
}

impl AlphaState {
    fn fresh(&mut self) -> VariableId {
        let id = VariableId::new_unchecked(format!("v{}", self.next));
        self.next += 1;
        id
    }

    fn resolve(&self, variable: VariableId) -> SemanticExpression {
        for scope in self.scopes.iter().rev() {
            if let Some(replacement) = scope.get(&variable) {
                return SemanticExpression::Variable(replacement.clone());
            }
        }
        SemanticExpression::Variable(variable)
    }
}
