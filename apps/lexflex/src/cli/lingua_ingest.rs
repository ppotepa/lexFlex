use crate::cli::output::print_response;
use crate::cli::support::read_ron;
use lexflex_engine::{
    api::request::EngineRequest, api::response::EngineResponse, runtime::LexFlexRuntime,
};
use lexflex_lingua::LinguaProgram;
use lexflex_model::Evidence;
use std::path::Path;

pub fn run(
    runtime: &mut LexFlexRuntime,
    program_path: &Path,
    evidence_path: &Path,
) -> Result<(), String> {
    let program: LinguaProgram = read_ron(program_path)?;
    let evidence: Vec<Evidence> = read_ron(evidence_path)?;
    let response = runtime.handle(EngineRequest::IngestLingua { program, evidence });
    print_response(&response)?;
    match response {
        EngineResponse::Error { message, .. } => Err(message),
        _ => Ok(()),
    }
}
