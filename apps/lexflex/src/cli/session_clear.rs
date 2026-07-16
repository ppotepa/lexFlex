use crate::cli::output::print_response;
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};

pub fn run(runtime: &mut LexFlexRuntime) -> Result<(), String> {
    let response = runtime.handle(EngineRequest::ClearSession);
    print_response(&response)?;
    Ok(())
}
