use super::ModelLoadError;
use lexflex_lingua::{LinguaDeclaration, LinguaProgram};
use lexflex_model::{CatalogValidationIssue, CatalogValidationReport, ConceptCatalog};
pub fn validate_program_manifest_structure(
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
    }
    Ok(())
}
