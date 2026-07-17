use crate::cli::output::{print_response_and_check, CliExit};
use crate::cli::support::read_ron;
use lexflex_engine::{
    api::request::EngineRequest, runtime::LexFlexRuntime,
};
use lexflex_lingua::LinguaProgram;
use lexflex_model::Evidence;
use std::path::Path;

pub fn run(
    runtime: &mut LexFlexRuntime,
    program_path: &Path,
    evidence_path: &Path,
) -> Result<(), CliExit> {
    let program: LinguaProgram = read_ron(program_path).map_err(CliExit::Command)?;
    let evidence: Vec<Evidence> = read_ron(evidence_path).map_err(CliExit::Command)?;
    let response = runtime.handle(EngineRequest::IngestLingua { program, evidence });
    print_response_and_check(&response)
}
