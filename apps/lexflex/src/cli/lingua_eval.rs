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
    let text = std::fs::read_to_string(path).map_err(|error| CliExit::Command(error.to_string()))?;
    let program: LinguaProgram =
        ron::from_str(&text).map_err(|error| CliExit::Command(error.to_string()))?;
    let response = runtime.handle(EngineRequest::EvaluateLingua {
        program,
        policy: ExecutionPolicy { expansion },
        include_trace,
    });
    match response {
        EngineResponse::LinguaEvaluated { result } => {
            let value = serde_json::to_string_pretty(&result.value)
                .map_err(|error| CliExit::Command(error.to_string()))?;
            println!("{value}");
        }
        EngineResponse::Unsupported {
            capability,
            message,
        } => {
            return Err(CliExit::Command(format!("{capability}: {message}")));
        }
        EngineResponse::Error { code, message, .. } => {
            return Err(CliExit::Engine {
                code,
                message,
            });
        }
        other => {
            return Err(CliExit::Command(format!("unexpected engine response: {other:?}")));
        }
    }
    Ok(())
}
