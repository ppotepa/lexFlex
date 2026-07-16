use lexflex_engine::{
    api::request::EngineRequest, api::response::EngineResponse, runtime::LexFlexRuntime,
};
use lexflex_lingua::LinguaProgram;
use std::path::Path;

pub fn run(runtime: &mut LexFlexRuntime, path: &Path, trace: bool) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let program: LinguaProgram = ron::from_str(&text).map_err(|error| error.to_string())?;
    let response = runtime.handle(EngineRequest::EvaluateLingua { program, trace });
    match response {
        EngineResponse::LinguaEvaluated { result } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result.value).map_err(|error| error.to_string())?
            );
        }
        EngineResponse::Unsupported {
            capability,
            message,
        } => {
            return Err(format!("{capability}: {message}"));
        }
        other => {
            return Err(format!("unexpected engine response: {other:?}"));
        }
    }
    Ok(())
}
