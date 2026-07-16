mod context;
mod goal;
mod occurs;
mod solver;
mod substitution;
mod type_inference;
mod unify;

pub use context::{UnificationContext, UnificationMode};
pub use goal::{EvidencePolicy, LinguaGoal};
pub use solver::{LinguaSolver, QuerySolution};
pub use substitution::Substitution;
pub use type_inference::{semantic_types_compatible, SolveTypeError};
pub use unify::{unify, UnifyError};
