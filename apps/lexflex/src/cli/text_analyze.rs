use crate::cli::output::print_response_and_check;
use crate::cli::{input::TextInputArgs, output::CliExit};
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};

pub fn run(
    runtime: &mut LexFlexRuntime,
    input: TextInputArgs,
    derivation: bool,
) -> Result<(), CliExit> {
    let input = input.into_text_input().map_err(CliExit::Command)?;
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input,
        include_derivation: derivation,
    });
    print_response_and_check(&response)
}
