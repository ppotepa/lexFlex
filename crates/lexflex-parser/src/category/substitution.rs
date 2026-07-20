use crate::category::binding::CategoryBinding;
use crate::category::outcome::{CategoryInvariantError, CategoryMismatch};
use crate::category::{CategoryOutcome, CategoryResolutionError, CategoryResult};
use lexflex_language::{CategoryType, CategoryTypeVariableId, SyntacticCategory};
use lexflex_model::{ConceptCatalog, SemanticType, TypeRelation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CategorySubstitution {
    bindings: BTreeMap<CategoryTypeVariableId, CategoryBinding>,
}

impl CategorySubstitution {
    pub(crate) fn resolve_variable(
        &self,
        variable: &CategoryTypeVariableId,
    ) -> Result<CategoryType, CategoryInvariantError> {
        let mut current = variable.clone();
        let mut visited = BTreeSet::new();

        loop {
            if !visited.insert(current.clone()) {
                let mut variables = visited.into_iter().collect::<Vec<_>>();
                variables.push(current);
                return Err(CategoryInvariantError::AliasCycle { variables });
            }

            match self.bindings.get(&current) {
                None => return Ok(CategoryType::Variable(current)),
                Some(CategoryBinding::Alias(next)) => current = next.clone(),
                Some(CategoryBinding::Concrete(semantic_type)) => {
                    return Ok(CategoryType::Concrete(semantic_type.clone()));
                }
            }
        }
    }

    pub(crate) fn require_concrete(
        &self,
        category_type: &CategoryType,
    ) -> Result<SemanticType, CategoryResolutionError> {
        match self.resolve(category_type)? {
            CategoryType::Concrete(value) => Ok(value),
            CategoryType::Variable(variable) => {
                Err(CategoryResolutionError::UnresolvedType { variable })
            }
        }
    }

    pub(crate) fn resolve(
        &self,
        category_type: &CategoryType,
    ) -> Result<CategoryType, CategoryInvariantError> {
        match category_type {
            CategoryType::Concrete(value) => Ok(CategoryType::Concrete(value.clone())),
            CategoryType::Variable(variable) => self.resolve_variable(variable),
        }
    }

    pub(crate) fn apply_category(
        &self,
        category: &SyntacticCategory,
    ) -> Result<SyntacticCategory, CategoryInvariantError> {
        category.try_map_types(&mut |value| self.resolve(value))
    }

    pub(crate) fn apply_query_category_types(
        &self,
        query_variables: &BTreeMap<lexflex_model::VariableId, lexflex_language::CategoryType>,
    ) -> Result<
        BTreeMap<lexflex_model::VariableId, lexflex_language::CategoryType>,
        CategoryInvariantError,
    > {
        query_variables
            .iter()
            .map(|(variable, value)| Ok((variable.clone(), self.resolve(value)?)))
            .collect()
    }

    pub(crate) fn unresolved_query_type_count(
        &self,
        query_variables: &BTreeMap<lexflex_model::VariableId, lexflex_language::CategoryType>,
    ) -> Result<usize, CategoryInvariantError> {
        query_variables
            .values()
            .map(|value| self.resolve(value))
            .collect::<Result<Vec<_>, _>>()
            .map(|resolved| {
                resolved
                    .into_iter()
                    .filter(|resolved| {
                        matches!(resolved, lexflex_language::CategoryType::Variable(_))
                    })
                    .count()
            })
    }

    pub(crate) fn merged_for_composition(
        &self,
        incoming: &CategorySubstitution,
        catalog: &ConceptCatalog,
    ) -> CategoryResult<Self> {
        let mut candidate = self.clone();
        for (variable, binding) in &incoming.bindings {
            match binding {
                CategoryBinding::Alias(next) => {
                    match candidate.alias(variable.clone(), next.clone(), catalog)? {
                        CategoryOutcome::Applied(()) => {}
                        CategoryOutcome::NotApplicable(mismatch) => {
                            return Ok(CategoryOutcome::NotApplicable(mismatch));
                        }
                    }
                }
                CategoryBinding::Concrete(semantic_type) => {
                    match candidate.bind_concrete(
                        variable.clone(),
                        semantic_type.clone(),
                        catalog,
                    )? {
                        CategoryOutcome::Applied(()) => {}
                        CategoryOutcome::NotApplicable(mismatch) => {
                            return Ok(CategoryOutcome::NotApplicable(mismatch));
                        }
                    }
                }
            }
        }
        Ok(CategoryOutcome::Applied(candidate))
    }

    pub(crate) fn alias(
        &mut self,
        left: CategoryTypeVariableId,
        right: CategoryTypeVariableId,
        catalog: &ConceptCatalog,
    ) -> CategoryResult<()> {
        let mut candidate = self.clone();
        let outcome = candidate.alias_inner(left, right, catalog)?;
        if matches!(outcome, CategoryOutcome::Applied(())) {
            *self = candidate;
        }
        Ok(outcome)
    }

    fn alias_inner(
        &mut self,
        left: CategoryTypeVariableId,
        right: CategoryTypeVariableId,
        catalog: &ConceptCatalog,
    ) -> CategoryResult<()> {
        let left_root = self.root_name(&left)?;
        let right_root = self.root_name(&right)?;
        if left_root == right_root {
            return Ok(CategoryOutcome::Applied(()));
        }
        let (root, alias) = if left_root <= right_root {
            (left_root, right_root)
        } else {
            (right_root, left_root)
        };

        let left_value = self.take_concrete(&root);
        let right_value = self.take_concrete(&alias);
        self.bindings
            .insert(alias.clone(), CategoryBinding::Alias(root.clone()));

        match (left_value, right_value) {
            (Some(existing), Some(incoming)) => {
                self.bindings
                    .insert(root.clone(), CategoryBinding::Concrete(existing));
                self.bind_concrete(root, incoming, catalog)
            }
            (Some(existing), None) | (None, Some(existing)) => {
                self.bindings
                    .insert(root, CategoryBinding::Concrete(existing));
                Ok(CategoryOutcome::Applied(()))
            }
            (None, None) => Ok(CategoryOutcome::Applied(())),
        }
    }

    pub(crate) fn bind_concrete(
        &mut self,
        variable: CategoryTypeVariableId,
        incoming: SemanticType,
        catalog: &ConceptCatalog,
    ) -> CategoryResult<()> {
        let mut candidate = self.clone();
        let outcome = candidate.bind_concrete_inner(variable, incoming, catalog)?;
        if matches!(outcome, CategoryOutcome::Applied(())) {
            *self = candidate;
        }
        Ok(outcome)
    }

    fn bind_concrete_inner(
        &mut self,
        variable: CategoryTypeVariableId,
        incoming: SemanticType,
        catalog: &ConceptCatalog,
    ) -> CategoryResult<()> {
        let root = self.root_name(&variable)?;
        match self.concrete(&root) {
            None => {
                self.bindings
                    .insert(root, CategoryBinding::Concrete(incoming));
                Ok(CategoryOutcome::Applied(()))
            }
            Some(existing) => {
                let relation = TypeRelation::new(catalog);
                let existing_accepts_incoming = relation.accepts(&existing, &incoming);
                let incoming_accepts_existing = relation.accepts(&incoming, &existing);
                match (existing_accepts_incoming, incoming_accepts_existing) {
                    (true, false) => {
                        self.bindings
                            .insert(root, CategoryBinding::Concrete(incoming));
                        Ok(CategoryOutcome::Applied(()))
                    }
                    (false, true) | (true, true) => Ok(CategoryOutcome::Applied(())),
                    (false, false) => Ok(CategoryOutcome::NotApplicable(
                        CategoryMismatch::SubstitutionConflict {
                            existing: CategoryType::Concrete(existing),
                            incoming: CategoryType::Concrete(incoming),
                        },
                    )),
                }
            }
        }
    }

    fn root_name(
        &self,
        variable: &CategoryTypeVariableId,
    ) -> Result<CategoryTypeVariableId, CategoryInvariantError> {
        let mut current = variable.clone();
        let mut visited = BTreeSet::new();
        while let Some(CategoryBinding::Alias(next)) = self.bindings.get(&current) {
            if !visited.insert(current.clone()) {
                let mut variables = visited.into_iter().collect::<Vec<_>>();
                variables.push(current);
                return Err(CategoryInvariantError::AliasCycle { variables });
            }
            current = next.clone();
        }
        Ok(current)
    }

    fn concrete(&self, variable: &CategoryTypeVariableId) -> Option<SemanticType> {
        match self.bindings.get(variable) {
            Some(CategoryBinding::Concrete(value)) => Some(value.clone()),
            _ => None,
        }
    }

    fn take_concrete(&mut self, variable: &CategoryTypeVariableId) -> Option<SemanticType> {
        match self.bindings.remove(variable) {
            Some(CategoryBinding::Concrete(value)) => Some(value),
            Some(other) => {
                self.bindings.insert(variable.clone(), other);
                None
            }
            None => None,
        }
    }
}
