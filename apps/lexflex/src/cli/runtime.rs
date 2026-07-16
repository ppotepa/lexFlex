use crate::cli::error::CliError;
use lexflex_engine::runtime::LexFlexRuntime;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RuntimeArgs {
    pub session: String,
    pub state_dir: PathBuf,
    pub model_root: PathBuf,
    pub language_root: PathBuf,
}

impl RuntimeArgs {
    pub fn build(&self) -> Result<LexFlexRuntime, CliError> {
        LexFlexRuntime::with_session_and_roots(
            self.session.clone(),
            &self.state_dir,
            &self.model_root,
            &self.language_root,
        )
        .map_err(CliError::Runtime)
    }
}
