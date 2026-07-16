#![forbid(unsafe_code)]

mod assertion;
mod catalog;
mod concept;
mod entity;
mod evidence;
mod hash;
mod id;
mod qualifier;
mod semantic;
mod type_relation;
mod type_system;
mod value;

pub use assertion::SemanticAssertion;
pub use catalog::ConceptCatalog;
pub use concept::{ConceptParameterSchema, ConceptSchema};
pub use entity::EntityDefinition;
pub use evidence::{Evidence, SourceSpan};
pub use hash::canonical_hash;
pub use id::{
    validate_identifier, AssertionId, ConceptId, EntityId, EvidenceId, IdError, LanguageId,
    ModelPackageId, ParameterId, QualifierId, VariableId, WorldId,
};
pub use qualifier::Qualifier;
pub use semantic::SemanticExpression;
pub use type_relation::TypeRelation;
pub use type_system::{ConceptKind, FunctionType, SemanticType, ValueType};
pub use value::{DateValue, DecimalValue, QuantityValue, SemanticValue};
