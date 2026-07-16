use crate::{QualifierId, SemanticExpression};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Qualifier {
    pub id: QualifierId,
    pub value: SemanticExpression,
}
