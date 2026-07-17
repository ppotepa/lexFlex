use lexflex_language::CategoryType;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MeaningInstance {
    pub(crate) expression: lexflex_lingua::LinguaExpression,
    pub(crate) query_variables: BTreeMap<lexflex_model::VariableId, CategoryType>,
    pub(crate) semantic_nodes: usize,
}
