use crate::cli::input_error::CliLocalError;
use crate::cli::output::{print_json, CliExit};
use lexflex_learning::{LearningOverlay, Observation, Proposal};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, CliExit> {
    let text = crate::cli::input::read_file_bounded(path)?;
    serde_json::from_str(&text).map_err(|error| {
        CliExit::Local(CliLocalError::Io {
            path: Some(path.to_path_buf()),
            message: format!("invalid JSON: {error}"),
        })
    })
}

pub fn observe(observation_path: PathBuf) -> Result<(), CliExit> {
    let observation: Observation = read_json(&observation_path)?;
    print_json(&observation).map_err(CliExit::Internal)
}

fn write_input_error(error: impl ToString) -> CliExit {
    CliExit::Engine {
        code: lexflex_engine::error::EngineErrorCode::InvalidProgram,
        message: error.to_string(),
    }
}

pub fn propose(
    observation_path: PathBuf,
    proposal_path: PathBuf,
    expected_behavior: String,
) -> Result<(), CliExit> {
    let observation: Observation = read_json(&observation_path)?;
    let proposal: Proposal = read_json(&proposal_path)?;
    let mut overlay = LearningOverlay::default();
    overlay
        .propose(&observation, proposal, expected_behavior)
        .map_err(write_input_error)?;
    print_json(&overlay).map_err(CliExit::Internal)
}

pub fn approve(overlay_path: PathBuf, proposal_id: String) -> Result<(), CliExit> {
    mutate(overlay_path, proposal_id, |overlay, id| overlay.approve(id))
}

pub fn reject(overlay_path: PathBuf, proposal_id: String) -> Result<(), CliExit> {
    mutate(overlay_path, proposal_id, |overlay, id| overlay.reject(id))
}

pub fn promote(overlay_path: PathBuf, proposal_id: String) -> Result<(), CliExit> {
    let overlay: LearningOverlay = read_json(&overlay_path)?;
    let proposal = overlay.promote(&proposal_id).map_err(write_input_error)?;
    print_json(&proposal).map_err(CliExit::Internal)
}

fn mutate(
    overlay_path: PathBuf,
    proposal_id: String,
    operation: impl FnOnce(&mut LearningOverlay, &str) -> Result<(), lexflex_learning::LearningError>,
) -> Result<(), CliExit> {
    let mut overlay: LearningOverlay = read_json(&overlay_path)?;
    operation(&mut overlay, &proposal_id).map_err(write_input_error)?;
    overlay.verify().map_err(write_input_error)?;
    print_json(&overlay).map_err(CliExit::Internal)
}
