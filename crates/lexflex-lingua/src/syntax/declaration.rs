use crate::id::{DeclarationId, FunctionId, SymbolName};
use crate::syntax::{LambdaParameter, LinguaExpression};
use crate::types::SemanticType;
use lexflex_model::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpansionPolicy {
    Opaque,
    OnDemand,
    Transparent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConceptSemantics {
    Primitive,
    Defined { body: LinguaExpression },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptDeclaration {
    pub declaration_id: DeclarationId,
    pub concept_id: ConceptId,
    pub self_parameter: Option<LambdaParameter>,
    pub parameters: Vec<LambdaParameter>,
    pub semantics: ConceptSemantics,
    pub expansion: ExpansionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub declaration_id: DeclarationId,
    pub function_id: FunctionId,
    pub name: SymbolName,
    pub parameters: Vec<LambdaParameter>,
    pub result_type: SemanticType,
    pub body: LinguaExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinguaDeclaration {
    Concept(ConceptDeclaration),
    Function(FunctionDeclaration),
}
