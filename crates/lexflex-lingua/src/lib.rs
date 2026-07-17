#![forbid(unsafe_code)]

pub mod compiler;
pub mod error;
pub mod id;
pub mod normalize;
pub mod runtime;
pub mod solve;
pub mod syntax;
pub mod types;
pub mod verifier;

pub use compiler::{
    CompileContext, CompiledConcept, CompiledFunction, CompiledProgram, LinguaCompiler,
};
pub use error::LinguaError;
pub use id::{DeclarationId, FunctionId, ModuleId, ProgramId, SymbolId, SymbolName};
pub use normalize::{
    normalize_expression, NormalizationBudget, NormalizationError, NormalizationReport,
    SemanticNormalizer,
};
pub use runtime::{
    ExecutionBudget, ExecutionPolicy, ExecutionResult, ExecutionTrace, ExpansionMode,
    LinguaInterpreter, TypedExecutionResult,
};
pub use solve::unify;
pub use solve::{
    canonical_goal_request_hash, canonical_goal_semantic_hash, validate_goal, EvidencePolicy,
    GoalValidationError, LinguaGoal, LinguaSolver, QuerySolution, SolveError, Substitution,
    UnificationContext, UnificationMode,
};
pub use syntax::{
    ConceptDeclaration, ConceptSemantics, ExpansionPolicy, FunctionDeclaration, LambdaParameter,
    LinguaDeclaration, LinguaExpression, LinguaProgram,
};
pub use types::{SemanticType, TypeChecker, TypeEnvironment, TypeError, ValueType};
pub use verifier::{LinguaVerifier, VerificationLimits, VerificationReport};
