use crate::id::{FunctionId, SymbolId};
use crate::syntax::{ConceptDeclaration, ExpansionPolicy, FunctionDeclaration};
use crate::types::SemanticType;
use lexflex_model::{ConceptId, EntityId, ParameterId, SemanticValue, VariableId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedExpression {
    Concept(ConceptId),
    Entity(EntityId),
    Value(SemanticValue),
    Local(SymbolId),
    QueryVariable(VariableId),
    Function(FunctionId),
    Lambda {
        parameters: Vec<ResolvedParameter>,
        body: Box<ResolvedExpression>,
    },
    Call {
        callee: Box<ResolvedExpression>,
        arguments: BTreeMap<ParameterId, ResolvedExpression>,
    },
    ApplyConcept {
        concept: ConceptId,
        bindings: BTreeMap<ParameterId, ResolvedExpression>,
    },
    Satisfies {
        subject: Box<ResolvedExpression>,
        concept: Box<ResolvedExpression>,
    },
    Equals {
        left: Box<ResolvedExpression>,
        right: Box<ResolvedExpression>,
    },
    And(Vec<ResolvedExpression>),
    Or(Vec<ResolvedExpression>),
    Not(Box<ResolvedExpression>),
    Exists {
        variable: VariableId,
        value_type: SemanticType,
        body: Box<ResolvedExpression>,
    },
    ForAll {
        variable: VariableId,
        value_type: SemanticType,
        body: Box<ResolvedExpression>,
    },
    Let {
        symbol: SymbolId,
        value: Box<ResolvedExpression>,
        body: Box<ResolvedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedParameter {
    pub parameter_id: ParameterId,
    pub symbol: SymbolId,
    pub value_type: SemanticType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledConceptSemantics {
    Primitive,
    Defined { body: ResolvedExpression },
}

#[derive(Debug, Clone)]
pub struct CompiledConcept {
    pub declaration: ConceptDeclaration,
    pub self_parameter: Option<ResolvedParameter>,
    pub parameters: Vec<ResolvedParameter>,
    pub semantics: CompiledConceptSemantics,
    pub expansion: ExpansionPolicy,
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub declaration: FunctionDeclaration,
    pub parameters: Vec<ResolvedParameter>,
    pub body: ResolvedExpression,
}

#[derive(Debug, Clone)]
pub struct CompiledProgram {
    pub concepts: BTreeMap<ConceptId, CompiledConcept>,
    pub functions: BTreeMap<FunctionId, CompiledFunction>,
    pub entry: ResolvedExpression,
}
