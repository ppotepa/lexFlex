use crate::cli::output::CliExit;
use lexflex_engine::runtime::RuntimeInitError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Runtime(RuntimeInitError),
    Command(String),
    CliExit(CliExit),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Runtime(_) => 3,
            Self::Command(_) => 4,
            Self::CliExit(exit) => exit.exit_code(),
        }
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) | Self::Command(message) => formatter.write_str(message),
            Self::Runtime(error) => Display::fmt(error, formatter),
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
