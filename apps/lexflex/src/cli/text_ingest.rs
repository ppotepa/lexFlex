use crate::cli::{input::TextInputArgs, output::print_response};
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};

pub fn run(runtime: &mut LexFlexRuntime, input: TextInputArgs) -> Result<(), String> {
    let response = runtime.handle(EngineRequest::IngestText {
        input: input.into_text_input()?,
    });
    print_response(&response)?;
    Ok(())
}
