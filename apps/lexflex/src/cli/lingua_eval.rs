use crate::cli::input_error::CliLocalError;
use crate::cli::internal_error::CliInternalError;
use crate::cli::output::CliExit;
use lexflex_engine::{
    api::request::EngineRequest, api::response::EngineResponse, runtime::LexFlexRuntime,
};
use lexflex_lingua::{ExecutionPolicy, ExpansionMode, LinguaProgram};
use std::path::Path;

pub fn run(
    runtime: &mut LexFlexRuntime,
    path: &Path,
    expansion: ExpansionMode,
    include_trace: bool,
) -> Result<(), CliExit> {
    let text = crate::cli::input::read_file_bounded(path)?;
    let program: LinguaProgram = ron::from_str(&text).map_err(|error| {
        CliExit::Local(CliLocalError::Ron {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
    })?;
    let response = runtime.handle(EngineRequest::EvaluateLingua {
        program,
        policy: ExecutionPolicy { expansion },
        include_trace,
    });
    match response {
        EngineResponse::LinguaEvaluated { result } => {
            let payload = if include_trace {
                serde_json::to_string_pretty(&result)
            } else {
                serde_json::to_string_pretty(&result.value)
            }
            .map_err(|error| {
                CliExit::Internal(CliInternalError::JsonSerialization {
                    message: error.to_string(),
                })
            })?;
            println!("{payload}");
        }
        EngineResponse::Error { code, message, .. } => {
            return Err(CliExit::Engine { code, message });
        }
        other => {
            return Err(CliExit::Internal(
                CliInternalError::UnexpectedEngineResponse {
                    message: format!("unexpected engine response: {other:?}"),
                },
            ));
        }
    }
    Ok(())
}
