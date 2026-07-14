mod analyzer;
mod compiler;
mod error;
mod hash;
mod invariants;
mod inspect;
mod model;
mod provenance;
mod validation;

pub use analyzer::{
    DocumentSentenceAnalyzer, LexFlexSentenceAnalyzer, SentenceAnalysisError,
    SentenceAnalysisInput,
};
pub use compiler::DocumentCompiler;
pub use error::DocumentCompilationError;
pub use hash::compilation_hash;
pub use inspect::SemanticInspector;
pub use model::{
    classify_sentence_status, summarize_compilation, DocumentCompilation,
    DocumentCompilationOptions, DocumentCompilationSummary, SentenceCompilation,
    SentenceProcessingStatus, SentenceSemanticInspection,
};
pub use provenance::{
    ProvenanceOperation, ProvenanceOutcome, ProvenanceStep, SentenceProvenance,
    SentenceProvenanceBuilder,
};
pub use validation::{
    DocumentCompilationValidationError, DocumentCompilationValidator,
};
