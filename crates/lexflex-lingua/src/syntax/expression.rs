use crate::id::SymbolName;
use crate::types::SemanticType;
use lexflex_model::{ConceptId, EntityId, ParameterId, SemanticValue, VariableId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LambdaParameter {
    pub name: SymbolName,
    pub parameter_id: ParameterId,
    pub value_type: SemanticType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinguaExpression {
    Concept(ConceptId),
    Entity(EntityId),
    Value(SemanticValue),
    Variable(SymbolName),
    QueryVariable(VariableId),
    Function(crate::id::FunctionId),
    Lambda {
        parameters: Vec<LambdaParameter>,
        body: Box<LinguaExpression>,
    },
    Call {
        callee: Box<LinguaExpression>,
        arguments: BTreeMap<ParameterId, LinguaExpression>,
    },
    ApplyConcept {
        concept: ConceptId,
        bindings: BTreeMap<ParameterId, LinguaExpression>,
    },
    Satisfies {
        subject: Box<LinguaExpression>,
        concept: Box<LinguaExpression>,
    },
    Equals {
        left: Box<LinguaExpression>,
        right: Box<LinguaExpression>,
    },
    And(Vec<LinguaExpression>),
    Or(Vec<LinguaExpression>),
    Not(Box<LinguaExpression>),
    Exists {
        variable: VariableId,
        value_type: SemanticType,
        body: Box<LinguaExpression>,
    },
    ForAll {
        variable: VariableId,
        value_type: SemanticType,
        body: Box<LinguaExpression>,
    },
    Let {
        name: SymbolName,
        value: Box<LinguaExpression>,
        body: Box<LinguaExpression>,
    },
}
