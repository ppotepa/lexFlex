use serde::{Deserialize, Serialize};

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
    MissingHierarchyChild { concept_id: String },
    MissingHierarchyParent {
        concept_id: String,
        parent_id: String,
    },
    HierarchyCycle { cycle: Vec<String> },
    UnsupportedEntityType { concept_id: String, found: String },
    UnsupportedRoleType { concept_id: String, found: String },
    UnsupportedRelationType { concept_id: String, found: String },
    EventTypeMustReturnBoolean { concept_id: String, found: String },
    EventTypeRequiresParameter { concept_id: String },
    UnknownConceptInEntityType { concept_id: String },
    UnknownQuantityDimension { dimension: String },
    InvalidOptionalNesting { context: String },
    ConceptProgramDuplicateId { program_id: String },
    ConceptProgramUnknownTarget {
        program_id: String,
        concept_id: String,
    },
    ConceptProgramValidation { program_id: String, message: String },
}

impl CatalogValidationIssue {
    pub(crate) fn with_context(self, concept_id: &str) -> Self {
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
            Self::EventTypeMustReturnBoolean { found, .. } => Self::EventTypeMustReturnBoolean {
                concept_id: concept_id.to_owned(),
                found,
            },
            Self::EventTypeRequiresParameter { .. } => Self::EventTypeRequiresParameter {
                concept_id: concept_id.to_owned(),
            },
            Self::UnknownConceptInEntityType { .. } => Self::UnknownConceptInEntityType {
                concept_id: concept_id.to_owned(),
            },
            Self::UnknownQuantityDimension { dimension } => {
                Self::UnknownQuantityDimension { dimension }
            }
            Self::InvalidOptionalNesting { context } => Self::InvalidOptionalNesting { context },
            Self::ConceptProgramDuplicateId { program_id } => {
                Self::ConceptProgramDuplicateId { program_id }
            }
            Self::ConceptProgramUnknownTarget {
                program_id,
                concept_id,
            } => Self::ConceptProgramUnknownTarget {
                program_id,
                concept_id,
            },
            Self::ConceptProgramValidation {
                program_id,
                message,
            } => Self::ConceptProgramValidation {
                program_id,
                message,
            },
            other => other,
        }
    }
}

impl std::fmt::Display for CatalogValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConceptKeyMismatch {
                concept_id,
                schema_id,
            } => {
                write!(
                    f,
                    "concept key mismatch: key={concept_id}, schema={schema_id}"
                )
            }
            Self::ParameterKeyMismatch {
                concept_id,
                parameter_id,
                schema_id,
            } => {
                write!(
                    f,
                    "parameter key mismatch in concept {concept_id}: key={parameter_id}, schema={schema_id}"
                )
            }
            Self::EntityKeyMismatch {
                entity_id,
                schema_id,
            } => {
                write!(
                    f,
                    "entity key mismatch: key={entity_id}, entity={schema_id}"
                )
            }
            Self::UnknownPrimaryType {
                entity_id,
                primary_type,
            } => {
                write!(
                    f,
                    "entity {entity_id} references unknown primary type {primary_type}"
                )
            }
            Self::UnknownAdditionalType {
                entity_id,
                additional_type,
            } => {
                write!(
                    f,
                    "entity {entity_id} references unknown additional type {additional_type}"
                )
            }
            Self::MissingHierarchyChild { concept_id } => {
                write!(f, "hierarchy child missing from concepts: {concept_id}")
            }
            Self::MissingHierarchyParent {
                concept_id,
                parent_id,
            } => {
                write!(
                    f,
                    "hierarchy parent missing from concepts: {parent_id} (child {concept_id})"
                )
            }
            Self::HierarchyCycle { cycle } => write!(f, "hierarchy cycle: {}", cycle.join(" -> ")),
            Self::UnsupportedEntityType { concept_id, found } => {
                write!(
                    f,
                    "entity type {concept_id} must return Predicate(Entity) or Predicate(EntityOf(self)), found {found}"
                )
            }
            Self::UnsupportedRoleType { concept_id, found } => {
                write!(
                    f,
                    "role type {concept_id} must return ConceptOf(RoleType), found {found}"
                )
            }
            Self::UnsupportedRelationType { concept_id, found } => {
                write!(
                    f,
                    "relation type {concept_id} must return Boolean, found {found}"
                )
            }
            Self::EventTypeMustReturnBoolean { concept_id, found } => {
                write!(
                    f,
                    "event type {concept_id} must return Boolean, found {found}"
                )
            }
            Self::EventTypeRequiresParameter { concept_id } => {
                write!(f, "event type {concept_id} requires at least one parameter")
            }
            Self::UnknownConceptInEntityType { concept_id } => {
                write!(f, "unknown concept in entity type: {concept_id}")
            }
            Self::UnknownQuantityDimension { dimension } => {
                write!(f, "unknown quantity dimension: {dimension}")
            }
            Self::InvalidOptionalNesting { context } => {
                write!(f, "invalid nested optional semantic type: {context}")
            }
            Self::ConceptProgramDuplicateId { program_id } => {
                write!(f, "duplicate concept program id: {program_id}")
            }
            Self::ConceptProgramUnknownTarget {
                program_id,
                concept_id,
            } => {
                write!(
                    f,
                    "concept program {program_id} targets unknown concept {concept_id}"
                )
            }
            Self::ConceptProgramValidation {
                program_id,
                message,
            } => {
                write!(
                    f,
                    "concept program {program_id} failed validation: {message}"
                )
            }
        }
    }
}
