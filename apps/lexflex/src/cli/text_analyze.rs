use crate::cli::{input::TextInputArgs, output::print_response};
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};

pub fn run(
    runtime: &mut LexFlexRuntime,
    input: TextInputArgs,
    derivation: bool,
) -> Result<(), String> {
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input: input.into_text_input()?,
        include_derivation: derivation,
    });
    print_response(&response)?;
    Ok(())
}
