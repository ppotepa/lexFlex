use crate::category::binding::CategoryBinding;
use crate::diagnostic::ParseError;
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
    ) -> Result<CategoryType, ParseError> {
        let mut current = variable.clone();
        let mut visited = BTreeSet::new();

        loop {
            if !visited.insert(current.clone()) {
                return Err(ParseError::MeaningFreshening(format!(
                    "category alias cycle at {}",
                    current.as_str()
                )));
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
    ) -> Result<SemanticType, ParseError> {
        match self.resolve(category_type)? {
            CategoryType::Concrete(value) => Ok(value),
            CategoryType::Variable(_) => Err(ParseError::UnresolvedQueryCategoryType),
        }
    }

    pub(crate) fn resolve(&self, category_type: &CategoryType) -> Result<CategoryType, ParseError> {
        match category_type {
            CategoryType::Concrete(value) => Ok(CategoryType::Concrete(value.clone())),
            CategoryType::Variable(variable) => self.resolve_variable(variable),
        }
    }

    pub(crate) fn apply_category(
        &self,
        category: &SyntacticCategory,
    ) -> Result<SyntacticCategory, ParseError> {
        category.try_map_types(&mut |value| self.resolve(value))
    }

    pub(crate) fn apply_query_types(
        &self,
        query_variables: &BTreeMap<lexflex_model::VariableId, lexflex_language::CategoryType>,
    ) -> Result<BTreeMap<lexflex_model::VariableId, lexflex_model::SemanticType>, ParseError> {
        query_variables
            .iter()
            .map(|(variable, value)| Ok((variable.clone(), self.require_concrete(value)?)))
            .collect()
    }

    pub(crate) fn merge(
        &mut self,
        incoming: &CategorySubstitution,
        catalog: &ConceptCatalog,
    ) -> Result<(), ParseError> {
        for (variable, binding) in &incoming.bindings {
            match binding {
                CategoryBinding::Alias(next) => {
                    self.alias(variable.clone(), next.clone(), catalog)?;
                }
                CategoryBinding::Concrete(semantic_type) => {
                    self.bind_concrete(variable.clone(), semantic_type.clone(), catalog)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn alias(
        &mut self,
        left: CategoryTypeVariableId,
        right: CategoryTypeVariableId,
        catalog: &ConceptCatalog,
    ) -> Result<(), ParseError> {
        let left_root = self.root_name(&left)?;
        let right_root = self.root_name(&right)?;
        if left_root == right_root {
            return Ok(());
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
                Ok(())
            }
            (None, None) => Ok(()),
        }
    }

    pub(crate) fn bind_concrete(
        &mut self,
        variable: CategoryTypeVariableId,
        incoming: SemanticType,
        catalog: &ConceptCatalog,
    ) -> Result<(), ParseError> {
        let root = self.root_name(&variable)?;
        match self.concrete(&root) {
            None => {
                self.bindings
                    .insert(root, CategoryBinding::Concrete(incoming));
                Ok(())
            }
            Some(existing) => {
                let relation = TypeRelation::new(catalog);
                let existing_accepts_incoming = relation.accepts(&existing, &incoming);
                let incoming_accepts_existing = relation.accepts(&incoming, &existing);
                match (existing_accepts_incoming, incoming_accepts_existing) {
                    (true, false) => {
                        self.bindings
                            .insert(root, CategoryBinding::Concrete(incoming));
                        Ok(())
                    }
                    (false, true) | (true, true) => Ok(()),
                    (false, false) => Err(ParseError::ConflictingQueryCategoryType {
                        existing: CategoryType::Concrete(existing),
                        incoming: CategoryType::Concrete(incoming),
                    }),
                }
            }
        }
    }

    fn root_name(
        &self,
        variable: &CategoryTypeVariableId,
    ) -> Result<CategoryTypeVariableId, ParseError> {
        let mut current = variable.clone();
        let mut visited = BTreeSet::new();
        while let Some(CategoryBinding::Alias(next)) = self.bindings.get(&current) {
            if !visited.insert(current.clone()) {
                return Err(ParseError::MeaningFreshening(format!(
                    "category alias cycle at {}",
                    current.as_str()
                )));
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
