use crate::chat::service::ChatService;
use crate::chat::session::ChatSession;
use crate::chat::shell;
use crate::chat::tui::terminal::TerminalGuard;
use crate::chat::ChatOptions;
use std::error::Error;

pub fn run(options: ChatOptions) -> Result<(), Box<dyn Error>> {
    configure_env(&options);
    let service = ChatService::new(&options.data_dir, options.offline, options.trace_mode, options.source_policy)?;
    let mut session = ChatSession::new(options);
    session.push_system_message(format!(
        "lexFlex chat ready. data root: {}. source policy: {}. offline: {}. Activity stays in one fixed panel; Tab switches focus, arrows move through visible entries, Enter/+/- control the selected tree, Alt+M toggles mouse mode, Ctrl+Shift+C copies selection, and Ctrl+Shift+B copies the transcript.",
        session.runtime.data_dir,
        if session.runtime.offline { "snapshot-only" } else { session.runtime.source_policy.as_str() },
        session.runtime.offline,
    ));

    let mut terminal = TerminalGuard::new()?;
    shell::run(terminal.terminal_mut(), &mut session, service)
}

fn configure_env(options: &ChatOptions) {
    let _ = options;
}
