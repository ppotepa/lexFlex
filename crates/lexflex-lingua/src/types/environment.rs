use crate::types::BaseTypeEnvironment;
use lexflex_model::{ConceptCatalog, SemanticType, VariableId};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    pub base: Arc<BaseTypeEnvironment>,
    pub locals: BTreeMap<crate::id::SymbolId, SemanticType>,
    pub variables: BTreeMap<VariableId, SemanticType>,
}

impl TypeEnvironment {
    pub fn new(
        catalog: Arc<ConceptCatalog>,
        functions: BTreeMap<crate::id::FunctionId, lexflex_model::FunctionType>,
    ) -> Self {
        Self {
            base: Arc::new(BaseTypeEnvironment::new(catalog, functions)),
            locals: BTreeMap::new(),
            variables: BTreeMap::new(),
        }
    }

    pub fn catalog(&self) -> &Arc<ConceptCatalog> {
        &self.base.catalog
    }
    pub fn functions(&self) -> &Arc<BTreeMap<crate::id::FunctionId, lexflex_model::FunctionType>> {
        &self.base.functions
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new(Arc::new(ConceptCatalog::default()), BTreeMap::new())
    }
}
