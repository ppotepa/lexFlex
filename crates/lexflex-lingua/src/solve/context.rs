use lexflex_model::{ConceptCatalog, SemanticType, VariableId};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct UnificationContext {
    pub catalog: Arc<ConceptCatalog>,
    pub variable_types: BTreeMap<VariableId, SemanticType>,
    pub mode: UnificationMode,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnificationMode {
    Exact,
    Pattern,
}
