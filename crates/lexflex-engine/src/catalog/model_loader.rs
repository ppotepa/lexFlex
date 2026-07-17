use crate::catalog::{ModelProgramRegistry, ProgramRegistryError};
use lexflex_lingua::LinguaProgram;
use lexflex_model::{
    canonical_hash, validate_catalog, CanonicalDigest, CanonicalHashError, CatalogValidationReport,
    ConceptCatalog, EntityDefinition, EntityId, ModelPackageId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

pub use super::concept_program_validation::validate_concept_programs;

const SUPPORTED_MANIFEST_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelManifest {
    pub schema: u32,
    pub package_id: ModelPackageId,
    pub concepts: String,
    pub entities: String,
    pub programs: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityPackage {
    pub entities: BTreeMap<EntityId, EntityDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModelPackage {
    pub manifest: ModelManifest,
    pub catalog: ConceptCatalog,
    pub programs: ModelProgramRegistry,
    pub model_hash: CanonicalDigest,
    pub validation: CatalogValidationReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelLoadError {
    #[error("io error reading {path}: {kind:?}")]
    Io {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    #[error("parse error reading {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("unsafe package path: {relative}")]
    UnsafePath { relative: PathBuf },
    #[error("unsupported model manifest schema: {schema}")]
    UnsupportedSchema { schema: u32 },
    #[error("validation error: {0}")]
    Validation(CatalogValidationReport),
    #[error("program registry error: {0}")]
    ProgramRegistry(#[from] ProgramRegistryError),
    #[error("canonical hash error: {0}")]
    CanonicalHash(#[from] CanonicalHashError),
}

pub struct ModelPackageLoader;

impl ModelPackageLoader {
    pub fn load(&self, root: &Path) -> Result<LoadedModelPackage, ModelLoadError> {
        let manifest: ModelManifest = read_ron(&root.join("manifest.ron"))?;
        if manifest.schema != SUPPORTED_MANIFEST_SCHEMA {
            return Err(ModelLoadError::UnsupportedSchema {
                schema: manifest.schema,
            });
        }

        let concepts: ConceptCatalog = read_ron(&safe_package_path(root, &manifest.concepts)?)?;
        let entities: EntityPackage = read_ron(&safe_package_path(root, &manifest.entities)?)?;
        let concept_programs: Vec<LinguaProgram> =
            read_ron(&safe_package_path(root, &manifest.programs)?)?;

        let mut catalog = concepts;
        catalog.entities = entities.entities;

        let validation = validate_catalog(&catalog);
        if !validation.is_clean() {
            return Err(ModelLoadError::Validation(validation));
        }

        validate_concept_programs(&catalog, &concept_programs)?;

        let programs = ModelProgramRegistry::build(concept_programs, &catalog)?;
        let model_hash = canonical_hash(&ModelIdentity {
            package_id: &manifest.package_id,
            catalog: &catalog,
            declaration_hash: programs.declaration_hash(),
        })?;

        Ok(LoadedModelPackage {
            manifest,
            catalog,
            programs,
            model_hash,
            validation,
        })
    }
}

#[derive(Serialize)]
struct ModelIdentity<'a> {
    package_id: &'a ModelPackageId,
    catalog: &'a ConceptCatalog,
    declaration_hash: &'a CanonicalDigest,
}

fn safe_package_path(root: &Path, relative: &str) -> Result<PathBuf, ModelLoadError> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ModelLoadError::UnsafePath {
            relative: relative.to_path_buf(),
        });
    }
    Ok(root.join(relative))
}

fn read_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ModelLoadError> {
    let source = fs::read_to_string(path).map_err(|error| ModelLoadError::Io {
        path: path.to_path_buf(),
        kind: error.kind(),
    })?;
    ron::from_str(&source).map_err(|error| ModelLoadError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}
