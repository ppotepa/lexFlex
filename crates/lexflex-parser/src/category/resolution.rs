use crate::category::outcome::CategoryInvariantError;
use lexflex_language::CategoryTypeVariableId;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum CategoryResolutionError {
    #[error("unresolved category type: {variable}")]
    UnresolvedType { variable: CategoryTypeVariableId },
    #[error(transparent)]
    Invariant(#[from] CategoryInvariantError),
}
