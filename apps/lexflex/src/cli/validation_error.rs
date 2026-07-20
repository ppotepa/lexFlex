use crate::cli::internal_error::CliInternalError;
use lexflex_engine::catalog::{LanguageRegistryError, ModelLoadError};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ValidationCommandError {
    Model(ModelLoadError),
    Language(LanguageRegistryError),
    Internal(CliInternalError),
}

impl Display for ValidationCommandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Model(error) => Display::fmt(error, formatter),
            Self::Language(error) => Display::fmt(error, formatter),
            Self::Internal(error) => Display::fmt(error, formatter),
        }
    }
}
