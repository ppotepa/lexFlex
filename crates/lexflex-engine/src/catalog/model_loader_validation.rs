use super::{CatalogValidationIssue, CatalogValidationReport, ModelLoadError};
use lexflex_lingua::{LinguaCompiler, LinguaDeclaration, LinguaProgram};
use lexflex_model::{ConceptCatalog, ConceptId, ValueType};
use std::collections::BTreeMap;
use std::sync::Arc;

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

pub fn validate_concept_programs(
    catalog: &ConceptCatalog,
    programs: &[LinguaProgram],
) -> Result<(), ModelLoadError> {
    let mut ids = std::collections::BTreeSet::new();
    for program in programs {
        if !ids.insert(program.id.clone()) {
            return Err(ModelLoadError::Validation(CatalogValidationReport {
                issues: vec![CatalogValidationIssue::ConceptProgramDuplicateId {
                    program_id: program.id.to_string(),
                }],
            }));
        }

        for declaration in &program.declarations {
            if let LinguaDeclaration::Concept(concept) = declaration {
                if !catalog.concepts.contains_key(&concept.concept_id) {
                    return Err(ModelLoadError::Validation(CatalogValidationReport {
                        issues: vec![CatalogValidationIssue::ConceptProgramUnknownTarget {
                            program_id: program.id.to_string(),
                            concept_id: concept.concept_id.to_string(),
                        }],
                    }));
                }
            }
        }

        LinguaCompiler::new(Arc::new(catalog.clone()))
            .compile(program)
            .map_err(|error| {
                ModelLoadError::Validation(CatalogValidationReport {
                    issues: vec![CatalogValidationIssue::ConceptProgramValidation {
                        program_id: program.id.to_string(),
                        message: error.to_string(),
                    }],
                })
            })?;
    }
    Ok(())
}

fn validate_concept_schema(
    schema: &lexflex_model::ConceptSchema,
    catalog: &ConceptCatalog,
) -> Result<(), CatalogValidationIssue> {
    validate_semantic_type(&schema.result_type, catalog)?;

    match schema.kind {
        lexflex_model::ConceptKind::EntityType => match &schema.result_type {
            lexflex_model::SemanticType::Predicate(inner) => match &**inner {
                lexflex_model::SemanticType::Entity => Ok(()),
                lexflex_model::SemanticType::EntityOf(concept) if concept == &schema.id => Ok(()),
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
        lexflex_model::ConceptKind::RoleType => match &schema.result_type {
            lexflex_model::SemanticType::ConceptOf(lexflex_model::ConceptKind::RoleType) => Ok(()),
            other => Err(CatalogValidationIssue::UnsupportedRoleType {
                concept_id: schema.id.to_string(),
                found: format!("{other:?}"),
            }),
        },
        lexflex_model::ConceptKind::RelationType => match &schema.result_type {
            lexflex_model::SemanticType::Boolean => Ok(()),
            other => Err(CatalogValidationIssue::UnsupportedRelationType {
                concept_id: schema.id.to_string(),
                found: format!("{other:?}"),
            }),
        },
        _ => Ok(()),
    }
}

fn validate_semantic_type(
    semantic_type: &lexflex_model::SemanticType,
    catalog: &ConceptCatalog,
) -> Result<(), CatalogValidationIssue> {
    match semantic_type {
        lexflex_model::SemanticType::Boolean
        | lexflex_model::SemanticType::Concept
        | lexflex_model::SemanticType::Entity => Ok(()),
        lexflex_model::SemanticType::ConceptOf(_) => Ok(()),
        lexflex_model::SemanticType::EntityOf(concept) => {
            if catalog.concepts.contains_key(concept) {
                Ok(())
            } else {
                Err(CatalogValidationIssue::UnknownConceptInEntityType {
                    concept_id: concept.to_string(),
                })
            }
        }
        lexflex_model::SemanticType::Value(value_type) => match value_type {
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
        lexflex_model::SemanticType::Predicate(inner)
        | lexflex_model::SemanticType::Set(inner)
        | lexflex_model::SemanticType::Optional(inner) => validate_semantic_type(inner, catalog),
        lexflex_model::SemanticType::Record(entries) => {
            for value in entries.values() {
                validate_semantic_type(value, catalog)?;
            }
            Ok(())
        }
        lexflex_model::SemanticType::Function(function) => {
            for value in function.parameters.values() {
                validate_semantic_type(value, catalog)?;
            }
            validate_semantic_type(&function.result, catalog)
        }
    }
}

fn find_hierarchy_cycle(catalog: &ConceptCatalog) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Visiting,
        Done,
    }

    fn dfs(
        node: &str,
        catalog: &ConceptCatalog,
        marks: &mut BTreeMap<String, Mark>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        match marks.get(node) {
            Some(Mark::Visiting) => {
                let start = stack.iter().position(|current| current == node)?;
                let mut cycle = stack[start..].to_vec();
                cycle.push(node.to_string());
                return Some(cycle);
            }
            Some(Mark::Done) => return None,
            None => {}
        }

        marks.insert(node.to_string(), Mark::Visiting);
        stack.push(node.to_string());

        if let Some(parents) = catalog.parents.get(&ConceptId::new_unchecked(node)) {
            for parent in parents {
                if let Some(cycle) = dfs(parent.as_str(), catalog, marks, stack) {
                    return Some(cycle);
                }
            }
        }

        stack.pop();
        marks.insert(node.to_string(), Mark::Done);
        None
    }

    let mut marks = BTreeMap::new();
    let mut stack = Vec::new();
    for node in catalog.parents.keys().map(|id| id.as_str().to_string()) {
        if let Some(cycle) = dfs(&node, catalog, &mut marks, &mut stack) {
            return Some(cycle);
        }
    }
    None
}
