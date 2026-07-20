use crate::{SemanticExpression, VariableId};
use std::collections::{BTreeMap, BTreeSet};

use super::{NormalizationBudget, NormalizationError};

#[derive(Debug, Default)]
pub(crate) struct AlphaState {
    next: u64,
    free_variables: BTreeSet<VariableId>,
    allocated_bound: BTreeSet<VariableId>,
    scopes: Vec<BTreeMap<VariableId, VariableId>>,
}

impl AlphaState {
    pub(crate) fn from_expression(expression: &SemanticExpression) -> Self {
        let mut state = Self::default();
        collect_free_variables(expression, &mut Vec::new(), &mut state.free_variables);
        state
    }

    pub(crate) fn allocate_bound(
        &mut self,
        budget: &NormalizationBudget,
    ) -> Result<VariableId, NormalizationError> {
        for _ in 0..budget.max_variable_attempts {
            let candidate = VariableId::new_unchecked(format!("bound:{}", self.next));
            self.next += 1;

            if !self.free_variables.contains(&candidate)
                && self.allocated_bound.insert(candidate.clone())
            {
                return Ok(candidate);
            }
        }

        Err(NormalizationError::VariableAllocationExhausted)
    }

    pub(crate) fn push(&mut self, from: VariableId, to: VariableId) {
        self.scopes.push(BTreeMap::from([(from, to)]));
    }

    pub(crate) fn pop(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn resolve(&self, variable: &VariableId) -> VariableId {
        for scope in self.scopes.iter().rev() {
            if let Some(replacement) = scope.get(variable) {
                return replacement.clone();
            }
        }

        variable.clone()
    }
}

fn collect_free_variables(
    expression: &SemanticExpression,
    bound: &mut Vec<VariableId>,
    output: &mut BTreeSet<VariableId>,
) {
    match expression {
        SemanticExpression::Variable(variable) => {
            if !bound.iter().rev().any(|current| current == variable) {
                output.insert(variable.clone());
            }
        }
        SemanticExpression::Apply { bindings, .. } => {
            for value in bindings.values() {
                collect_free_variables(value, bound, output);
            }
        }
        SemanticExpression::Satisfies { subject, predicate } => {
            collect_free_variables(subject, bound, output);
            collect_free_variables(predicate, bound, output);
        }
        SemanticExpression::Equals { left, right } => {
            collect_free_variables(left, bound, output);
            collect_free_variables(right, bound, output);
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            for item in items {
                collect_free_variables(item, bound, output);
            }
        }
        SemanticExpression::Not(inner) => collect_free_variables(inner, bound, output),
        SemanticExpression::Exists { variable, body, .. }
        | SemanticExpression::ForAll { variable, body, .. } => {
            bound.push(variable.clone());
            collect_free_variables(body, bound, output);
            bound.pop();
        }
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            collect_free_variables(expression, bound, output);
            for value in qualifiers.values() {
                collect_free_variables(value, bound, output);
            }
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_) => {}
    }
}
