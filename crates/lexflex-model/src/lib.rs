#![forbid(unsafe_code)]

mod assertion;
mod catalog;
mod concept;
mod entity;
mod evidence;
mod hash;
mod id;
mod integrity;
mod normalize;
mod qualifier;
mod semantic;
mod type_check;
mod type_relation;
mod type_system;
mod validation;
mod value;

pub use assertion::{AssertionError, SemanticAssertion};
pub use catalog::ConceptCatalog;
pub use concept::{ConceptParameterSchema, ConceptSchema};
pub use entity::EntityDefinition;
pub use evidence::{Evidence, EvidenceError, SourceSpan};
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
pub use semantic::SemanticExpression;
pub use type_check::{
    ExpressionTypeBudget, ExpressionTypeChecker, ExpressionTypeEnvironment, ExpressionTypeError,
};
pub use type_relation::{TypeRelation, TypeRelationError};
pub use type_system::{ConceptKind, FunctionType, SemanticType, ValueType};
pub use validation::{validate_catalog, CatalogValidationIssue, CatalogValidationReport};
pub use value::{DateValue, DecimalValue, QuantityValue, SemanticValue};
