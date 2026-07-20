use lexflex_model::{EvidenceError, IdError};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub enum CliInputError {
    EmptyText,
    InputTooLarge { max_bytes: usize },
    InvalidLanguage(IdError),
    InvalidEvidence(EvidenceError),
}

impl Display for CliInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyText => formatter.write_str("text input is empty"),
            Self::InputTooLarge { max_bytes } => {
                write!(formatter, "text input exceeds {max_bytes} byte limit")
            }
            Self::InvalidLanguage(error) => write!(formatter, "invalid language: {error}"),
            Self::InvalidEvidence(error) => write!(formatter, "invalid evidence: {error}"),
        }
    }
}

#[derive(Debug)]
pub enum CliLocalError {
    Io {
        path: Option<PathBuf>,
        message: String,
    },
    Ron {
        path: PathBuf,
        message: String,
    },
}

impl Display for CliLocalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io {
                path: Some(path),
                message,
            } => write!(formatter, "{}: {message}", path.display()),
            Self::Io {
                path: None,
                message,
            } => formatter.write_str(message),
            Self::Ron { path, message } => write!(formatter, "{}: {message}", path.display()),
        }
    }
}
