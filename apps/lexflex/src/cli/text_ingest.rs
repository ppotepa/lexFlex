use crate::cli::output::{print_response_and_check, CliExit};
use crate::cli::input::TextInputArgs;
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};

pub fn run(runtime: &mut LexFlexRuntime, input: TextInputArgs) -> Result<(), CliExit> {
    let input = input.into_text_input().map_err(CliExit::Command)?;
    let response = runtime.handle(EngineRequest::IngestText { input });
    print_response_and_check(&response)
}
