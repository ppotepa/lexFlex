mod context;
mod context_error;
mod diagnostic;
mod implementation;
mod model_context;
mod resolved;
mod resolver;
mod type_reference_error;
mod type_reference_validation;

pub use context::CompileContext;
pub use context_error::CompileContextError;
pub use diagnostic::{CompileDiagnostic, CompileError};
pub use implementation::LinguaCompiler;
pub use model_context::{CompiledModelContext, ModelContextIdentity, VerifiedCompiledModel};
pub use resolved::{
    CompiledConcept, CompiledConceptSemantics, CompiledFunction, ResolvedExpression,
    ResolvedParameter, VerifiedCompiledEntry, VerifiedStandaloneProgram,
};
pub use resolver::SymbolResolver;
pub use type_reference_error::{CompileTypeReferenceError, CompileTypeReferenceLocation};
