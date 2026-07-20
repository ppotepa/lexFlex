mod base_environment;
mod checker;
mod environment;
mod error;

pub use base_environment::BaseTypeEnvironment;
pub use checker::TypeChecker;
pub use environment::TypeEnvironment;
pub use error::TypeError;
pub use lexflex_model::{ConceptKind, FunctionType, SemanticType, ValueType};
