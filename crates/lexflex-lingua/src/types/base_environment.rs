use crate::id::FunctionId;
use lexflex_model::{ConceptCatalog, FunctionType};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BaseTypeEnvironment {
    pub(crate) catalog: Arc<ConceptCatalog>,
    pub(crate) functions: Arc<BTreeMap<FunctionId, FunctionType>>,
}

impl BaseTypeEnvironment {
    pub(crate) fn new(
        catalog: Arc<ConceptCatalog>,
        functions: BTreeMap<FunctionId, FunctionType>,
    ) -> Self {
        Self {
            catalog,
            functions: Arc::new(functions),
        }
    }
}
