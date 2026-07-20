use lexflex_language::CategoryType;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BooleanOperator {
    Not,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MeaningInstance {
    pub(crate) expression: lexflex_lingua::LinguaExpression,
    pub(crate) query_variables: BTreeMap<lexflex_model::VariableId, CategoryType>,
    pub(crate) semantic_nodes: usize,
    pub(crate) boolean_operator: Option<BooleanOperator>,
    pub(crate) boolean_scope_violations: usize,
}
