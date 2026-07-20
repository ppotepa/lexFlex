use crate::catalog::{ModelProgramRegistry, ProgramRegistryError};
use lexflex_lingua::{compiler::CompileError, LinguaCompiler, LinguaProgram};
use lexflex_model::{
    canonical_hash, validate_catalog, CanonicalDigest, CanonicalHashError, CatalogValidationReport,
    ConceptCatalog, EntityDefinition, EntityId, ModelPackageId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use super::concept_program_validation::validate_program_manifest_structure;

const SUPPORTED_MANIFEST_SCHEMA: u32 = 1;
const MAX_PACKAGE_FILE_BYTES: usize = 32 * 1024 * 1024;

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

#[derive(Debug, Clone)]
pub struct LoadedModelPackage {
    pub manifest: ModelManifest,
    pub catalog: Arc<ConceptCatalog>,
    pub programs: ModelProgramRegistry,
    pub model_hash: CanonicalDigest,
    pub validation: CatalogValidationReport,
    pub compiled_model: Arc<lexflex_lingua::VerifiedCompiledModel>,
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
    #[error("program compilation error: {0}")]
    ProgramCompile(CompileError),
    #[error("canonical hash error: {0}")]
    CanonicalHash(#[from] CanonicalHashError),
    #[error("model package file exceeds resource limit")]
    ResourceLimit,
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

        validate_program_manifest_structure(&catalog, &concept_programs)?;

        let programs = ModelProgramRegistry::build(concept_programs, &catalog)?;
        let catalog = Arc::new(catalog);
        let compiler =
            LinguaCompiler::try_new(catalog.clone()).map_err(ModelLoadError::ProgramCompile)?;
        let compiled_model = compiler
            .compile_model_declarations(programs.declarations())
            .map_err(ModelLoadError::ProgramCompile)?;
        let model_hash = canonical_hash(&ModelIdentity {
            package_id: &manifest.package_id,
            catalog: catalog.as_ref(),
            declaration_hash: programs.declaration_hash(),
        })?;

        Ok(LoadedModelPackage {
            manifest,
            catalog,
            programs,
            model_hash,
            validation,
            compiled_model: Arc::new(compiled_model),
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
    let file = fs::File::open(path).map_err(|error| ModelLoadError::Io {
        path: path.to_path_buf(),
        kind: error.kind(),
    })?;
    let mut bytes = Vec::new();
    file.take((MAX_PACKAGE_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| ModelLoadError::Io {
            path: path.to_path_buf(),
            kind: error.kind(),
        })?;
    if bytes.len() > MAX_PACKAGE_FILE_BYTES {
        return Err(ModelLoadError::ResourceLimit);
    }
    let source = String::from_utf8(bytes).map_err(|error| ModelLoadError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    ron::from_str(&source).map_err(|error| ModelLoadError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}
