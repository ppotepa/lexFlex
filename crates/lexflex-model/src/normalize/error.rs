use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NormalizationError {
    #[error("normalization node budget exceeded: max={max}")]
    NodeBudgetExceeded { max: usize },

    #[error("normalization depth budget exceeded: max={max}")]
    DepthBudgetExceeded { max: usize },

    #[error("unable to allocate a capture-free bound variable id")]
    VariableAllocationExhausted,
}
