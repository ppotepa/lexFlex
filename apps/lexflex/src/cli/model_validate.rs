use crate::cli::output::print_json;
use crate::cli::validation_error::ValidationCommandError;
use lexflex_engine::catalog::ModelPackageLoader;
use lexflex_engine::LanguageRegistry;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct ModelValidationSummary {
    package_id: String,
    concepts: usize,
    entities: usize,
    parents: usize,
    programs: usize,
    declarations: usize,
    declaration_hash: String,
    validation_issues: usize,
    languages: usize,
    registry_hash: String,
    model_hash: String,
}

pub fn run(model_root: PathBuf, language_root: PathBuf) -> Result<(), ValidationCommandError> {
    let loader = ModelPackageLoader;
    let package = loader
        .load(&model_root)
        .map_err(ValidationCommandError::Model)?;
    let registry = LanguageRegistry::load(&language_root, package.catalog.clone())
        .map_err(ValidationCommandError::Language)?;
    print_json(&ModelValidationSummary {
        package_id: package.manifest.package_id.to_string(),
        concepts: package.catalog.concepts.len(),
        entities: package.catalog.entities.len(),
        parents: package.catalog.parents.len(),
        programs: package.programs.program_count(),
        declarations: package.programs.declaration_count(),
        declaration_hash: package.programs.declaration_hash().to_string(),
        validation_issues: package.validation.issues.len(),
        languages: registry.models.len(),
        registry_hash: registry.registry_hash.to_string(),
        model_hash: package.model_hash.to_string(),
    })
    .map_err(ValidationCommandError::Internal)?;
    Ok(())
}
