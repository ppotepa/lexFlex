use crate::id::MeaningTemplateId;
use crate::CategoryType;
use lexflex_lingua::LinguaExpression;
use lexflex_model::VariableId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeaningTemplate {
    pub id: MeaningTemplateId,
    pub expression: LinguaExpression,
    #[serde(default)]
    pub query_variables: BTreeMap<VariableId, CategoryType>,
}

impl MeaningTemplate {
    pub fn new(
        id: MeaningTemplateId,
        expression: LinguaExpression,
        query_variables: BTreeMap<VariableId, CategoryType>,
    ) -> Self {
        Self {
            id,
            expression,
            query_variables,
        }
    }
}
