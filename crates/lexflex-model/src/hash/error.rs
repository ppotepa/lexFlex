use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CanonicalHashError {
    #[error("canonical serialization failed: {message}")]
    Serialization { message: String },
}
