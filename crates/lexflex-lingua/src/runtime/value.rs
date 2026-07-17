use crate::compiler::ResolvedExpression;
use crate::id::SymbolId;
use crate::runtime::RuntimeEnvironment;
use crate::types::SemanticType;
use lexflex_model::{ParameterId, SemanticExpression};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Semantic(SemanticExpression),
    Closure(ClosureValue),
}

#[derive(Debug, Clone)]
pub struct ClosureValue {
    pub parameters: BTreeMap<ParameterId, ClosureParameter>,
    pub body: Arc<ResolvedExpression>,
    pub captured: Arc<RuntimeEnvironment>,
}

#[derive(Debug, Clone)]
pub struct ClosureParameter {
    pub symbol: SymbolId,
    pub value_type: SemanticType,
}

impl RuntimeValue {
    pub fn into_semantic(self) -> Result<SemanticExpression, crate::runtime::RuntimeError> {
        match self {
            RuntimeValue::Semantic(value) => Ok(value),
            _ => Err(crate::runtime::RuntimeError::ExpectedSemantic),
        }
    }

    pub fn into_closure(self) -> Result<ClosureValue, crate::runtime::RuntimeError> {
        match self {
            RuntimeValue::Closure(value) => Ok(value),
            _ => Err(crate::runtime::RuntimeError::ExpectedClosure),
        }
    }
}
