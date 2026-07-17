use crate::{ConceptCatalog, SemanticType, VariableId};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ExpressionTypeEnvironment<'a> {
    pub catalog: &'a ConceptCatalog,
    pub free_variables: BTreeMap<VariableId, SemanticType>,
    bound_scopes: Vec<BTreeMap<VariableId, SemanticType>>,
}

impl<'a> ExpressionTypeEnvironment<'a> {
    pub fn new(catalog: &'a ConceptCatalog) -> Self {
        Self {
            catalog,
            free_variables: BTreeMap::new(),
            bound_scopes: Vec::new(),
        }
    }

    pub fn with_free_variables(
        catalog: &'a ConceptCatalog,
        free_variables: BTreeMap<VariableId, SemanticType>,
    ) -> Self {
        Self {
            catalog,
            free_variables,
            bound_scopes: Vec::new(),
        }
    }

    pub fn push_bound(&mut self, variable: VariableId, value_type: SemanticType) {
        self.bound_scopes
            .push(BTreeMap::from([(variable, value_type)]));
    }

    pub fn pop_bound(&mut self) -> bool {
        self.bound_scopes.pop().is_some()
    }

    pub fn variable_type(&self, variable: &VariableId) -> Option<&SemanticType> {
        for scope in self.bound_scopes.iter().rev() {
            if let Some(value_type) = scope.get(variable) {
                return Some(value_type);
            }
        }

        self.free_variables.get(variable)
    }
}
