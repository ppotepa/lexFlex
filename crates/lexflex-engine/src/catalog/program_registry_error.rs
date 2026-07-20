use lexflex_lingua::{DeclarationId, FunctionId, ProgramId};
use lexflex_model::{CanonicalHashError, ConceptId};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProgramRegistryError {
    #[error("duplicate model program id: {0}")]
    DuplicateProgram(ProgramId),

    #[error("duplicate declaration id: {0}")]
    DuplicateDeclaration(DeclarationId),

    #[error("duplicate concept declaration: {0}")]
    DuplicateConcept(ConceptId),

    #[error("duplicate function declaration: {0}")]
    DuplicateFunction(FunctionId),

    #[error("concept declaration targets unknown catalog concept: {0}")]
    UnknownConcept(ConceptId),

    #[error("program registry canonical hash failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),
}
