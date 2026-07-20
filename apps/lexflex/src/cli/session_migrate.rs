use crate::cli::error::CliError;
use crate::cli::output::print_json;
use lexflex_engine::SESSION_RECORD_SCHEMA;
use lexflex_store::SessionStore;
use serde_json::Value;
use std::path::Path;

pub fn run(
    session_id: &str,
    state_dir: impl AsRef<Path>,
    from_schema: u32,
) -> Result<(), CliError> {
    let store = SessionStore::<Value>::new(state_dir.as_ref().to_path_buf(), SESSION_RECORD_SCHEMA);
    let artifact = store
        .migrate(session_id, from_schema, Ok)
        .map_err(|error| {
            CliError::Runtime(lexflex_engine::runtime::RuntimeInitError::Store(error))
        })?;
    print_json(&artifact)
        .map_err(|error| CliError::CliExit(crate::cli::output::CliExit::Internal(error)))
}
