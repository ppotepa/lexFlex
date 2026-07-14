pub mod app;
pub mod commands;
pub mod navigation;
pub mod service;
pub mod session;
pub mod settings;
pub mod shell;
pub mod transcript;
pub mod trace;
pub mod tui;
pub mod widgets;

use serde::{Deserialize, Serialize};
use std::error::Error;

pub use commands::{SlashCommand, SlashSuggestion};
pub use service::{ChatJob, ChatJobResult, ChatService, ChatWorker};
pub use session::ChatSession;
pub use settings::Verbosity;

#[derive(Debug, Clone)]
pub struct ChatOptions {
    pub from: String,
    pub to: String,
    pub data_dir: String,
    pub offline: bool,
    pub trace_mode: TraceMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceMode {
    Off,
    Brief,
    Full,
}

impl TraceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            TraceMode::Off => "off",
            TraceMode::Brief => "brief",
            TraceMode::Full => "full",
        }
    }
}

pub fn run(options: ChatOptions) -> Result<(), Box<dyn Error>> {
    run_tui(options)
}

pub fn run_tui(options: ChatOptions) -> Result<(), Box<dyn Error>> {
    app::run(options)
}
