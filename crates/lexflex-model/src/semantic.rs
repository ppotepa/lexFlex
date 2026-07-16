use crate::{
    canonical_hash, ConceptId, EntityId, ParameterId, QualifierId, SemanticValue, VariableId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SemanticExpression {
    Concept(ConceptId),
    Entity(EntityId),
    Value(SemanticValue),
    Variable(VariableId),
    Apply {
        concept: ConceptId,
        bindings: BTreeMap<ParameterId, SemanticExpression>,
    },
    Satisfies {
        subject: Box<SemanticExpression>,
        predicate: Box<SemanticExpression>,
    },
    Equals {
        left: Box<SemanticExpression>,
        right: Box<SemanticExpression>,
    },
    And(Vec<SemanticExpression>),
    Or(Vec<SemanticExpression>),
    Not(Box<SemanticExpression>),
    Exists {
        variable: VariableId,
        body: Box<SemanticExpression>,
    },
    ForAll {
        variable: VariableId,
        body: Box<SemanticExpression>,
    },
    Qualified {
        expression: Box<SemanticExpression>,
        qualifiers: BTreeMap<QualifierId, SemanticExpression>,
    },
}

impl SemanticExpression {
    pub fn canonical_hash(&self) -> String {
        canonical_hash(self)
    }
}
