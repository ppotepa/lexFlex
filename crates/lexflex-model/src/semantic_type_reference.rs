use crate::{ConceptCatalog, ConceptId, FunctionType, SemanticType, ValueType};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SemanticTypeReferenceError {
    #[error("unknown concept referenced by semantic type: {0}")]
    UnknownConcept(ConceptId),

    #[error("unknown quantity dimension referenced by semantic type: {0}")]
    UnknownQuantityDimension(ConceptId),
}

pub fn validate_semantic_type_references(
    value_type: &SemanticType,
    catalog: &ConceptCatalog,
) -> Result<(), SemanticTypeReferenceError> {
    match value_type {
        SemanticType::Boolean
        | SemanticType::Concept
        | SemanticType::ConceptOf(_)
        | SemanticType::Entity => Ok(()),
        SemanticType::EntityOf(concept) => {
            if catalog.concept(concept).is_some() {
                Ok(())
            } else {
                Err(SemanticTypeReferenceError::UnknownConcept(concept.clone()))
            }
        }
        SemanticType::Value(ValueType::Quantity { dimension }) => {
            if catalog.concept(dimension).is_some() {
                Ok(())
            } else {
                Err(SemanticTypeReferenceError::UnknownQuantityDimension(
                    dimension.clone(),
                ))
            }
        }
        SemanticType::Value(_) => Ok(()),
        SemanticType::Predicate(inner)
        | SemanticType::Set(inner)
        | SemanticType::Optional(inner) => validate_semantic_type_references(inner, catalog),
        SemanticType::Record(fields) => {
            for field_type in fields.values() {
                validate_semantic_type_references(field_type, catalog)?;
            }
            Ok(())
        }
        SemanticType::Function(FunctionType { parameters, result }) => {
            for parameter_type in parameters.values() {
                validate_semantic_type_references(parameter_type, catalog)?;
            }
            validate_semantic_type_references(result, catalog)
        }
    }
}
