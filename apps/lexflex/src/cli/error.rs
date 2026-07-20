use crate::cli::output::CliExit;
use crate::cli::validation_error::ValidationCommandError;
use lexflex_engine::runtime::RuntimeInitError;
use lexflex_store::SessionStoreError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Runtime(RuntimeInitError),
    Validation(ValidationCommandError),
    CliExit(CliExit),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Runtime(error) => match error {
                RuntimeInitError::Model(_)
                | RuntimeInitError::Languages(_)
                | RuntimeInitError::Integrity(_) => 6,
                RuntimeInitError::Store(error) => store_error_code(error),
            },
            Self::Validation(error) => match error {
                ValidationCommandError::Model(_) | ValidationCommandError::Language(_) => 6,
                ValidationCommandError::Internal(_) => 8,
            },
            Self::CliExit(exit) => exit.exit_code(),
        }
    }
}

fn store_error_code(error: &SessionStoreError) -> i32 {
    match error {
        SessionStoreError::Io { .. } => 7,
        SessionStoreError::InvalidSessionId(_) => 4,
        SessionStoreError::Serde { .. }
        | SessionStoreError::SchemaMismatch { .. }
        | SessionStoreError::SessionIdMismatch { .. }
        | SessionStoreError::ResourceLimit => 6,
        SessionStoreError::NotFound { .. } => 7,
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => formatter.write_str(message),
            Self::Runtime(error) => Display::fmt(error, formatter),
            Self::Validation(error) => Display::fmt(error, formatter),
            Self::CliExit(exit) => Display::fmt(exit, formatter),
        }
    }
}

impl From<RuntimeInitError> for CliError {
    fn from(value: RuntimeInitError) -> Self {
        Self::Runtime(value)
    }
}

impl From<CliExit> for CliError {
    fn from(value: CliExit) -> Self {
        Self::CliExit(value)
    }
}
