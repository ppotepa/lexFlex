use crate::{CatalogValidationIssue, ConceptCatalog, SemanticType, ValueType};

pub(crate) fn validate_semantic_type(
    semantic_type: &SemanticType,
    catalog: &ConceptCatalog,
) -> Result<(), CatalogValidationIssue> {
    match semantic_type {
        SemanticType::Boolean | SemanticType::Concept | SemanticType::Entity => Ok(()),
        SemanticType::ConceptOf(_) => Ok(()),
        SemanticType::EntityOf(concept) => {
            if catalog.concepts.contains_key(concept) {
                Ok(())
            } else {
                Err(CatalogValidationIssue::UnknownConceptInEntityType {
                    concept_id: concept.to_string(),
                })
            }
        }
        SemanticType::Value(value_type) => match value_type {
            ValueType::Quantity { dimension } => {
                if catalog.concepts.contains_key(dimension) {
                    Ok(())
                } else {
                    Err(CatalogValidationIssue::UnknownQuantityDimension {
                        dimension: dimension.to_string(),
                    })
                }
            }
            _ => Ok(()),
        },
        SemanticType::Predicate(inner) | SemanticType::Set(inner) => {
            validate_semantic_type(inner, catalog)
        }
        SemanticType::Optional(inner) => match &**inner {
            SemanticType::Optional(_) => Err(CatalogValidationIssue::InvalidOptionalNesting {
                context: format!("{semantic_type:?}"),
            }),
            _ => validate_semantic_type(inner, catalog),
        },
        SemanticType::Record(entries) => {
            for value in entries.values() {
                validate_semantic_type(value, catalog)?;
            }
            Ok(())
        }
        SemanticType::Function(function) => {
            for value in function.parameters.values() {
                validate_semantic_type(value, catalog)?;
            }
            validate_semantic_type(&function.result, catalog)
        }
    }
}
