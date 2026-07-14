use crate::chat::service::ChatService;
use crate::chat::session::ChatSession;
use crate::chat::shell;
use crate::chat::tui::terminal::TerminalGuard;
use crate::chat::ChatOptions;
use std::error::Error;

pub fn run(options: ChatOptions) -> Result<(), Box<dyn Error>> {
    configure_env(&options);
    let service = ChatService::new(&options.data_dir, options.offline, options.trace_mode)?;
    let mut session = ChatSession::new(options);
    session.push_system_message(
        "lexFlex chat ready. Chat language is detected automatically; use /lang en or /lang pl to override it, and /translate en pl for translation.",
    );

    let mut terminal = TerminalGuard::new()?;
    shell::run(terminal.terminal_mut(), &mut session, service)
}

fn configure_env(options: &ChatOptions) {
    if options.offline {
        std::env::remove_var("LEXFLEX_LLM_BASE_URL");
        std::env::set_var("LEXFLEX_NO_AUTO_WRITE", "1");
    }
    if let Some(url) = &options.base_url {
        std::env::set_var("LEXFLEX_LLM_BASE_URL", url);
    }
    if let Some(model) = &options.model {
        std::env::set_var("LEXFLEX_LLM_MODEL", model);
    }
    if let Some(prompt) = &options.system_prompt {
        std::env::set_var("LEXFLEX_LLM_SYSTEM_PROMPT", prompt);
    }
}
