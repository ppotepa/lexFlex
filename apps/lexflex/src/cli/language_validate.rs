use crate::cli::output::print_json;
use crate::cli::validation_error::ValidationCommandError;
use lexflex_engine::catalog::{LanguageRegistry, ModelPackageLoader};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct LanguageValidationSummary {
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

    print_json(&LanguageValidationSummary {
        languages: registry.models.len(),
        registry_hash: registry.registry_hash.to_string(),
        model_hash: package.model_hash.to_string(),
    })
    .map_err(ValidationCommandError::Internal)?;
    Ok(())
}
