#![forbid(unsafe_code)]

mod assertion;
mod assertion_catalog;
mod catalog;
mod concept;
mod entity;
mod evidence;
mod evidence_set;
mod hash;
mod id;
mod integrity;
mod normalize;
mod qualifier;
mod resource_budget;
mod semantic;
mod semantic_type_reference;
mod type_check;
mod type_relation;
mod type_system;
mod validation;
mod value;

pub use assertion::{AssertionError, SemanticAssertion};
pub use assertion_catalog::AssertionCatalogError;
pub use catalog::ConceptCatalog;
pub use concept::{ConceptParameterSchema, ConceptSchema};
pub use entity::EntityDefinition;
pub use evidence::{Evidence, EvidenceError, SourceSpan};
pub use evidence_set::EvidenceSet;
pub use hash::{
    canonical_bytes, canonical_hash, CanonicalDigest, CanonicalDigestError, CanonicalHashError,
};
pub use id::{
    validate_identifier, AssertionId, ConceptId, EntityId, EvidenceId, IdError, LanguageId,
    ModelPackageId, ParameterId, QualifierId, VariableId, WorldId,
};
pub use integrity::{AssertionIntegrityError, EvidenceIntegrityError};
pub use normalize::{
    normalize_expression, NormalizationBudget, NormalizationError, NormalizationReport,
    SemanticNormalizer,
};
pub use qualifier::Qualifier;
pub use resource_budget::ResourceBudget;
pub use semantic::SemanticExpression;
pub use semantic_type_reference::{validate_semantic_type_references, SemanticTypeReferenceError};
pub use type_check::{
    ExpressionTypeBudget, ExpressionTypeChecker, ExpressionTypeEnvironment, ExpressionTypeError,
};
pub use type_relation::{TypeRelation, TypeRelationError};
pub use type_system::{ConceptKind, FunctionType, SemanticType, ValueType};
pub use validation::{validate_catalog, CatalogValidationIssue, CatalogValidationReport};
pub use value::{DateValue, DecimalValue, QuantityValue, SemanticValue};
