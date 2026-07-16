use lexflex_lingua::LinguaProgram;
use lexflex_model::{canonical_hash, ConceptCatalog, EntityDefinition, EntityId, ModelPackageId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[path = "model_loader_validation.rs"]
mod validation;

pub use validation::{validate_catalog, validate_concept_programs};

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogValidationReport {
    pub issues: Vec<CatalogValidationIssue>,
}

impl CatalogValidationReport {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }
}

impl std::fmt::Display for CatalogValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self
            .issues
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        write!(f, "{text}")
    }
}

impl CatalogValidationIssue {
    fn with_context(self, concept_id: &str) -> Self {
        match self {
            Self::UnsupportedEntityType { found, .. } => Self::UnsupportedEntityType {
                concept_id: concept_id.to_owned(),
                found,
            },
            Self::UnsupportedRoleType { found, .. } => Self::UnsupportedRoleType {
                concept_id: concept_id.to_owned(),
                found,
            },
            Self::UnsupportedRelationType { found, .. } => Self::UnsupportedRelationType {
                concept_id: concept_id.to_owned(),
                found,
            },
            Self::UnknownConceptInEntityType { .. } => Self::UnknownConceptInEntityType {
                concept_id: concept_id.to_owned(),
            },
            Self::UnknownQuantityDimension { dimension } => {
                Self::UnknownQuantityDimension { dimension }
            }
            other => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CatalogValidationIssue {
    ConceptKeyMismatch {
        concept_id: String,
        schema_id: String,
    },
    ParameterKeyMismatch {
        concept_id: String,
        parameter_id: String,
        schema_id: String,
    },
    EntityKeyMismatch {
        entity_id: String,
        schema_id: String,
    },
    UnknownPrimaryType {
        entity_id: String,
        primary_type: String,
    },
    UnknownAdditionalType {
        entity_id: String,
        additional_type: String,
    },
    MissingHierarchyChild {
        concept_id: String,
    },
    MissingHierarchyParent {
        concept_id: String,
        parent_id: String,
    },
    HierarchyCycle {
        cycle: Vec<String>,
    },
    UnsupportedEntityType {
        concept_id: String,
        found: String,
    },
    UnsupportedRoleType {
        concept_id: String,
        found: String,
    },
    UnsupportedRelationType {
        concept_id: String,
        found: String,
    },
    UnknownConceptInEntityType {
        concept_id: String,
    },
    UnknownQuantityDimension {
        dimension: String,
    },
    ConceptProgramDuplicateId {
        program_id: String,
    },
    ConceptProgramUnknownTarget {
        program_id: String,
        concept_id: String,
    },
    ConceptProgramValidation {
        program_id: String,
        message: String,
    },
}

impl std::fmt::Display for CatalogValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConceptKeyMismatch {
                concept_id,
                schema_id,
            } => write!(f, "concept key mismatch: key={concept_id}, schema={schema_id}"),
            Self::ParameterKeyMismatch {
                concept_id,
                parameter_id,
                schema_id,
            } => write!(
                f,
                "parameter key mismatch in concept {concept_id}: key={parameter_id}, schema={schema_id}"
            ),
            Self::EntityKeyMismatch { entity_id, schema_id } => {
                write!(f, "entity key mismatch: key={entity_id}, entity={schema_id}")
            }
            Self::UnknownPrimaryType {
                entity_id,
                primary_type,
            } => write!(
                f,
                "entity {entity_id} references unknown primary type {primary_type}"
            ),
            Self::UnknownAdditionalType {
                entity_id,
                additional_type,
            } => write!(
                f,
                "entity {entity_id} references unknown additional type {additional_type}"
            ),
            Self::MissingHierarchyChild { concept_id } => {
                write!(f, "hierarchy child missing from concepts: {concept_id}")
            }
            Self::MissingHierarchyParent {
                concept_id,
                parent_id,
            } => write!(
                f,
                "hierarchy parent missing from concepts: {parent_id} (child {concept_id})"
            ),
            Self::HierarchyCycle { cycle } => write!(f, "hierarchy cycle: {}", cycle.join(" -> ")),
            Self::UnsupportedEntityType { concept_id, found } => write!(
                f,
                "entity type {concept_id} must return Predicate(Entity) or Predicate(EntityOf(self)), found {found}"
            ),
            Self::UnsupportedRoleType { concept_id, found } => {
                write!(f, "role type {concept_id} must return ConceptOf(RoleType), found {found}")
            }
            Self::UnsupportedRelationType { concept_id, found } => write!(
                f,
                "relation type {concept_id} must return Boolean, found {found}"
            ),
            Self::UnknownConceptInEntityType { concept_id } => {
                write!(f, "unknown concept in entity type: {concept_id}")
            }
            Self::UnknownQuantityDimension { dimension } => {
                write!(f, "unknown quantity dimension: {dimension}")
            }
            Self::ConceptProgramDuplicateId { program_id } => {
                write!(f, "duplicate concept program id: {program_id}")
            }
            Self::ConceptProgramUnknownTarget {
                program_id,
                concept_id,
            } => write!(
                f,
                "concept program {program_id} targets unknown concept {concept_id}"
            ),
            Self::ConceptProgramValidation {
                program_id,
                message,
            } => write!(f, "concept program {program_id} failed validation: {message}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModelPackage {
    pub manifest: ModelManifest,
    pub catalog: ConceptCatalog,
    pub concept_programs: Vec<LinguaProgram>,
    pub model_hash: String,
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

        let model_hash = canonical_hash(&(&manifest.package_id, &catalog, &concept_programs));

        Ok(LoadedModelPackage {
            manifest,
            catalog,
            concept_programs,
            model_hash,
            validation,
        })
    }
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
