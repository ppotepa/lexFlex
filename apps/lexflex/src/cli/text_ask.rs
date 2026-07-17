use crate::cli::output::{print_response_and_check, CliExit};
use crate::cli::{input::TextInputArgs};
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};
use lexflex_lingua::solve::EvidencePolicy;

pub fn run(
    runtime: &mut LexFlexRuntime,
    input: TextInputArgs,
    limit: usize,
    evidence_policy: EvidencePolicy,
) -> Result<(), CliExit> {
    let input = input.into_text_input().map_err(CliExit::Command)?;
    let response = runtime.handle(EngineRequest::AskText {
        input,
        evidence_policy,
        limit: Some(limit),
    });
    print_response_and_check(&response)
}
