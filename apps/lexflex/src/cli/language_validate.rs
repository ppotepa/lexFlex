use crate::cli::output::print_json;
use lexflex_engine::catalog::{LanguageRegistry, ModelPackageLoader};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize)]
struct LanguageValidationSummary {
    languages: usize,
    registry_hash: String,
    model_hash: String,
}

pub fn run(model_root: PathBuf, language_root: PathBuf) -> Result<(), String> {
    let loader = ModelPackageLoader;
    let package = loader
        .load(&model_root)
        .map_err(|error| error.to_string())?;
    let registry = LanguageRegistry::load(&language_root, Arc::new(package.catalog.clone()))
        .map_err(|error| error.to_string())?;

    print_json(&LanguageValidationSummary {
        languages: registry.models.len(),
        registry_hash: registry.registry_hash,
        model_hash: package.model_hash,
    })?;
    Ok(())
}
