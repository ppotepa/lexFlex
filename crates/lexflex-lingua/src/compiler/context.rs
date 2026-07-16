use lexflex_model::{SemanticType, VariableId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompileContext {
    pub query_variables: BTreeMap<VariableId, SemanticType>,
}
