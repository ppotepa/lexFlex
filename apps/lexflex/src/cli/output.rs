use crate::cli::input_error::{CliInputError, CliLocalError};
use crate::cli::internal_error::CliInternalError;
use lexflex_engine::api::response::EngineResponse;
use lexflex_engine::error::EngineErrorCode;
use serde::Serialize;
use std::fmt::{Display, Formatter};

pub fn print_response_and_check(response: &EngineResponse) -> Result<(), CliExit> {
    print_json(response).map_err(CliExit::Internal)?;
    match response {
        EngineResponse::Error { code, message, .. } => Err(CliExit::Engine {
            code: *code,
            message: message.clone(),
        }),
        EngineResponse::TextNotParsed { .. } => Err(CliExit::NotParsed),
        EngineResponse::TextAmbiguous { .. } => Err(CliExit::Ambiguous),
        _ => Ok(()),
    }
}

#[derive(Debug)]
pub enum CliExit {
    Input(CliInputError),
    Local(CliLocalError),
    Internal(CliInternalError),
    Engine {
        code: EngineErrorCode,
        message: String,
    },
    NotParsed,
    Ambiguous,
}

impl CliExit {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Input(_) => 4,
            Self::Local(_) => 9,
            Self::Internal(_) => 8,
            Self::Engine { code, .. } => match code {
                EngineErrorCode::InvalidProgram
                | EngineErrorCode::InvalidGoal
                | EngineErrorCode::InvalidAssertion
                | EngineErrorCode::InvalidEvidence
                | EngineErrorCode::Canonicalization => 4,
                EngineErrorCode::Integrity | EngineErrorCode::ModelError => 6,
                EngineErrorCode::StoreError => 7,
                EngineErrorCode::RuntimeBudget => 5,
                EngineErrorCode::InternalInvariant => 8,
            },
            Self::NotParsed => 4,
            Self::Ambiguous => 5,
        }
    }
}

impl Display for CliExit {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(error) => Display::fmt(error, formatter),
            Self::Local(error) => Display::fmt(error, formatter),
            Self::Internal(error) => Display::fmt(error, formatter),
            Self::Engine { message, .. } => formatter.write_str(message),
            Self::NotParsed => formatter.write_str("text not parsed"),
            Self::Ambiguous => formatter.write_str("text ambiguous"),
        }
    }
}

pub fn print_json<T: Serialize>(value: &T) -> Result<(), CliInternalError> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|error| {
            CliInternalError::JsonSerialization {
                message: error.to_string(),
            }
        })?
    );
    Ok(())
}
