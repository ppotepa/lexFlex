use crate::{CatalogValidationIssue, CatalogValidationReport, ConceptCatalog, ConceptKind};

use super::{hierarchy::find_hierarchy_cycle, semantic_type::validate_semantic_type};

pub fn validate_catalog(catalog: &ConceptCatalog) -> CatalogValidationReport {
    let mut issues = Vec::new();

    for (id, schema) in &catalog.concepts {
        if &schema.id != id {
            issues.push(CatalogValidationIssue::ConceptKeyMismatch {
                concept_id: id.to_string(),
                schema_id: schema.id.to_string(),
            });
        }

        if let Err(message) = validate_concept_schema(schema, catalog) {
            issues.push(message.with_context(id.as_str()));
        }

        for (parameter_id, parameter) in &schema.parameters {
            if &parameter.id != parameter_id {
                issues.push(CatalogValidationIssue::ParameterKeyMismatch {
                    concept_id: id.to_string(),
                    parameter_id: parameter_id.to_string(),
                    schema_id: parameter.id.to_string(),
                });
            }
        }
    }

    for (id, entity) in &catalog.entities {
        if &entity.id != id {
            issues.push(CatalogValidationIssue::EntityKeyMismatch {
                entity_id: id.to_string(),
                schema_id: entity.id.to_string(),
            });
        }
        if !catalog.concepts.contains_key(&entity.primary_type) {
            issues.push(CatalogValidationIssue::UnknownPrimaryType {
                entity_id: id.to_string(),
                primary_type: entity.primary_type.to_string(),
            });
        }
        for additional in &entity.additional_types {
            if !catalog.concepts.contains_key(additional) {
                issues.push(CatalogValidationIssue::UnknownAdditionalType {
                    entity_id: id.to_string(),
                    additional_type: additional.to_string(),
                });
            }
        }
    }

    for (child, parents) in &catalog.parents {
        if !catalog.concepts.contains_key(child) {
            issues.push(CatalogValidationIssue::MissingHierarchyChild {
                concept_id: child.to_string(),
            });
        }
        for parent in parents {
            if !catalog.concepts.contains_key(parent) {
                issues.push(CatalogValidationIssue::MissingHierarchyParent {
                    concept_id: child.to_string(),
                    parent_id: parent.to_string(),
                });
            }
        }
    }

    if let Some(cycle) = find_hierarchy_cycle(catalog) {
        issues.push(CatalogValidationIssue::HierarchyCycle { cycle });
    }

    CatalogValidationReport { issues }
}

fn validate_concept_schema(
    schema: &crate::ConceptSchema,
    catalog: &ConceptCatalog,
) -> Result<(), CatalogValidationIssue> {
    validate_semantic_type(&schema.result_type, catalog)?;

    match schema.kind {
        ConceptKind::EntityType => match &schema.result_type {
            crate::SemanticType::Predicate(inner) => match &**inner {
                crate::SemanticType::Entity => Ok(()),
                crate::SemanticType::EntityOf(concept) if concept == &schema.id => Ok(()),
                other => Err(CatalogValidationIssue::UnsupportedEntityType {
                    concept_id: schema.id.to_string(),
                    found: format!("{other:?}"),
                }),
            },
            other => Err(CatalogValidationIssue::UnsupportedEntityType {
                concept_id: schema.id.to_string(),
                found: format!("{other:?}"),
            }),
        },
        ConceptKind::RoleType => match &schema.result_type {
            crate::SemanticType::ConceptOf(crate::ConceptKind::RoleType) => Ok(()),
            other => Err(CatalogValidationIssue::UnsupportedRoleType {
                concept_id: schema.id.to_string(),
                found: format!("{other:?}"),
            }),
        },
        ConceptKind::RelationType => match &schema.result_type {
            crate::SemanticType::Boolean => Ok(()),
            other => Err(CatalogValidationIssue::UnsupportedRelationType {
                concept_id: schema.id.to_string(),
                found: format!("{other:?}"),
            }),
        },
        ConceptKind::EventType => {
            if schema.parameters.is_empty() {
                return Err(CatalogValidationIssue::EventTypeRequiresParameter {
                    concept_id: schema.id.to_string(),
                });
            }

            if schema.result_type != crate::SemanticType::Boolean {
                return Err(CatalogValidationIssue::EventTypeMustReturnBoolean {
                    concept_id: schema.id.to_string(),
                    found: format!("{:?}", schema.result_type),
                });
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
