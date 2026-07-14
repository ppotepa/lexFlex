mod diagnostic;
mod error;
mod hash;
mod id;
mod model;
mod options;
mod query;
mod resolver;
mod schema;
mod serialization;
mod service;
mod profile;
mod resolver_support;
mod validation;
mod summary;

pub use diagnostic::{
    EntityResolutionDiagnostic, EntityResolutionDiagnosticSeverity,
};
pub use error::{
    DocumentEntityResolutionError, DocumentEntityResolutionSerializationError,
    DocumentEntityResolutionServiceError, DocumentEntityResolutionValidationError,
};
pub use hash::document_entity_resolution_hash;
pub use id::{
    DocumentEntityResolutionIdFactory, EntityClusterId, ResolutionDecisionId, ResolutionId,
    ResolutionIdError, ResolutionMentionRef, ResolutionDiagnosticId, SyntheticMentionId,
};
pub use model::{
    DocumentEntityResolution, EntityResolutionDecision, EntityResolutionDecisionKind,
    EntityResolutionStage, EntityCluster, MentionResolutionProfile, MentionSourceForm,
    ResolutionAlternative, ResolutionAlternativeKind, ResolvedEntityCluster,
    SyntheticResolutionMention,
};
pub use options::{DocumentEntityResolutionOptions, EntityResolutionProfileKind};
pub use profile::{
    classify_mention, EnglishReferenceProfile, LanguageReferenceProfile, NeutralReferenceProfile,
    PolishReferenceProfile,
};
pub use query::DocumentEntityResolutionQuery;
pub use resolver::DocumentEntityResolver;
pub use schema::{
    DocumentEntityResolutionSchema, ENTITY_RESOLUTION_ALGORITHM_VERSION,
    ENTITY_RESOLUTION_SCHEMA_VERSION,
};
pub use serialization::{
    entity_resolution_from_json, entity_resolution_to_canonical_json,
    entity_resolution_to_pretty_json,
};
pub use service::DocumentEntityResolutionService;
pub use validation::{DocumentEntityResolutionValidator};
pub use summary::{summarize_entity_resolution, EntityResolutionSummary};
