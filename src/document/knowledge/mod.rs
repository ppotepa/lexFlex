mod hash;
mod id;
mod model;
mod options;
mod schema;
mod service;
mod validation;

pub use hash::document_knowledge_hash;
pub use id::{
    KnowledgeDiagnosticId, KnowledgeExtractionId, KnowledgeExtractionIdFactory, ClaimId,
    ContradictionSetId, PropositionOccurrenceId, QualifierId, ValueId,
};
pub use model::{
    ClaimAttribution, ClaimFactuality, ClaimStatus, ClaimTemporalScope, ClaimWorldRef,
    CivilDate, DecimalValue, DocumentClaim, DocumentKnowledgeExtraction, KnowledgeDiagnostic,
    KnowledgeDiagnosticSeverity, KnowledgeObjectRef, KnowledgePredicateRef, KnowledgeQualifier,
    KnowledgeRelationKind, KnowledgeSubjectRef, KnowledgeValue, PropositionOccurrence, QuantityValue,
    ContradictionSet, KnowledgeExtractionSummary,
};
pub use options::DocumentKnowledgeExtractionOptions;
pub use schema::{
    DocumentKnowledgeSchema, KNOWLEDGE_EXTRACTION_ALGORITHM_VERSION,
    KNOWLEDGE_EXTRACTION_SCHEMA_VERSION,
};
pub use service::{
    DocumentKnowledgeError, DocumentKnowledgeService,
};
pub use validation::DocumentKnowledgeValidator;
