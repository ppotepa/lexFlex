use crate::cli::output::print_response;
use crate::cli::support::read_ron;
use lexflex_engine::{
    api::request::EngineRequest, api::response::EngineResponse, runtime::LexFlexRuntime,
};
use lexflex_lingua::LinguaGoal;
use std::path::Path;

pub fn run(runtime: &mut LexFlexRuntime, goal_path: &Path) -> Result<(), String> {
    let goal: LinguaGoal = read_ron(goal_path)?;
    let response = runtime.handle(EngineRequest::QueryLingua { goal });
    print_response(&response)?;
    match response {
        EngineResponse::Error { message, .. } => Err(message),
        _ => Ok(()),
    }
}
