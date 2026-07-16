use crate::cli::{input::TextInputArgs, output::print_response};
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};
use lexflex_lingua::solve::EvidencePolicy;

pub fn run(
    runtime: &mut LexFlexRuntime,
    input: TextInputArgs,
    limit: usize,
    evidence_policy: EvidencePolicy,
) -> Result<(), String> {
    let response = runtime.handle(EngineRequest::AskText {
        input: input.into_text_input()?,
        evidence_policy,
        limit: Some(limit),
    });
    print_response(&response)?;
    Ok(())
}
