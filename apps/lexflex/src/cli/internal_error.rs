use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CliInternalError {
    JsonSerialization { message: String },
    CanonicalHash { message: String },
    UnexpectedEngineResponse { message: String },
}

impl Display for CliInternalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonSerialization { message } => {
                write!(formatter, "JSON serialization failed: {message}")
            }
            Self::CanonicalHash { message } => {
                write!(formatter, "canonical hash failed: {message}")
            }
            Self::UnexpectedEngineResponse { message } => formatter.write_str(message),
        }
    }
}
