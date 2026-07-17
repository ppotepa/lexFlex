use lexflex_language::CategoryTypeVariableId;
use lexflex_model::SemanticType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CategoryBinding {
    Alias(CategoryTypeVariableId),
    Concrete(SemanticType),
}
