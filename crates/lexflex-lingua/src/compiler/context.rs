use crate::compiler::CompileContextError;
use lexflex_model::{validate_semantic_type_references, ConceptCatalog, SemanticType, VariableId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompileContext {
    pub query_variables: BTreeMap<VariableId, SemanticType>,
}

impl CompileContext {
    pub fn validate(&self, catalog: &ConceptCatalog) -> Result<(), CompileContextError> {
        for (variable, value_type) in &self.query_variables {
            validate_semantic_type_references(value_type, catalog).map_err(|source| {
                CompileContextError::InvalidQueryVariableType {
                    variable: variable.clone(),
                    source,
                }
            })?;
        }
        Ok(())
    }
}
