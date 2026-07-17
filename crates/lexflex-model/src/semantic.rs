use crate::{
    canonical_hash, normalize_expression, CanonicalDigest, CanonicalHashError, ConceptId, EntityId,
    NormalizationError, ParameterId, QualifierId, SemanticValue, SemanticType, VariableId,
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
        value_type: SemanticType,
        body: Box<SemanticExpression>,
    },
    ForAll {
        variable: VariableId,
        value_type: SemanticType,
        body: Box<SemanticExpression>,
    },
    Qualified {
        expression: Box<SemanticExpression>,
        qualifiers: BTreeMap<QualifierId, SemanticExpression>,
    },
}

impl SemanticExpression {
    pub fn normalized(&self) -> Result<SemanticExpression, NormalizationError> {
        normalize_expression(self.clone()).map(|result| result.expression)
    }

    pub fn canonical_hash(&self) -> Result<CanonicalDigest, CanonicalHashError> {
        canonical_hash(&self.normalized().map_err(|error| CanonicalHashError::Serialization {
            message: error.to_string(),
        })?)
    }
}
