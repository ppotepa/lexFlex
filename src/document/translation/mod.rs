mod preserved;
mod assembler;
mod error;
mod generator;
mod hash;
mod model;
mod provenance;
mod translator;
mod validation;

pub use assembler::{AssembledDocumentOutput, DocumentOutputAssembler};
pub use error::{DocumentAssemblyError, DocumentTranslationError};
pub use generator::{
    DocumentSentenceGenerator, LexFlexSentenceGenerator, SentenceGenerationError,
    SentenceGenerationInput,
};
pub use hash::{translation_hash, translation_output_hash};
pub use preserved::{canonical_preserved_blocks, PreservedBlockDigest};
pub use model::{
    DocumentTranslation, DocumentTranslationOptions, DocumentTranslationSummary,
    SentenceTranslationResult, SentenceTranslationStatus, TranslationFallbackPolicy,
};
pub use provenance::{
    generation_failure_fallback_provenance, generation_skipped_fallback_provenance,
    generation_success_provenance, TranslationFallbackKind,
};
pub use translator::BestEffortDocumentTranslator;
pub use validation::{
    DocumentTranslationValidationError, DocumentTranslationValidator,
};
