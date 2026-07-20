use crate::cli::output::{print_response_and_check, CliExit};
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};

pub fn run(runtime: &mut LexFlexRuntime) -> Result<(), CliExit> {
    let response = runtime.handle(EngineRequest::InspectSession);
    print_response_and_check(&response)
}
