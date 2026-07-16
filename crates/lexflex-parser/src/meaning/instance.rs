use lexflex_model::SemanticType;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct MeaningInstance {
    pub(crate) expression: lexflex_lingua::LinguaExpression,
    pub(crate) query_variables: BTreeMap<lexflex_model::VariableId, SemanticType>,
    pub(crate) semantic_nodes: usize,
}
