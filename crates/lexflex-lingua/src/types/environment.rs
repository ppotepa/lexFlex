use crate::id::FunctionId;
use lexflex_model::{ConceptCatalog, SemanticType, VariableId};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    pub catalog: Arc<ConceptCatalog>,
    pub locals: BTreeMap<crate::id::SymbolId, SemanticType>,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub functions: BTreeMap<FunctionId, lexflex_model::FunctionType>,
}

impl TypeEnvironment {
    pub fn new(
        catalog: Arc<ConceptCatalog>,
        functions: BTreeMap<FunctionId, lexflex_model::FunctionType>,
    ) -> Self {
        Self {
            catalog,
            locals: BTreeMap::new(),
            variables: BTreeMap::new(),
            functions,
        }
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new(Arc::new(ConceptCatalog::default()), BTreeMap::new())
    }
}
