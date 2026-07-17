use crate::cli::output::{print_response_and_check, CliExit};
use crate::cli::support::read_ron;
use lexflex_engine::{
    api::request::EngineRequest, runtime::LexFlexRuntime,
};
use lexflex_lingua::LinguaGoal;
use std::path::Path;

pub fn run(runtime: &mut LexFlexRuntime, goal_path: &Path) -> Result<(), CliExit> {
    let goal: LinguaGoal = read_ron(goal_path).map_err(CliExit::Command)?;
    let response = runtime.handle(EngineRequest::QueryLingua { goal });
    print_response_and_check(&response)
}
