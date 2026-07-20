use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TextInputError {
    #[error("text source id is empty")]
    EmptySourceId,
    #[error("text is empty")]
    EmptyText,
}
