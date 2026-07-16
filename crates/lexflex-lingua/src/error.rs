use crate::compiler::CompileError;
use crate::runtime::RuntimeError;
use crate::solve::SolveTypeError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LinguaError {
    #[error(transparent)]
    Compile(#[from] CompileError),
    #[error(transparent)]
    Verify(CompileError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Solve(#[from] SolveTypeError),
}
