use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::error::Error;
use std::io::{self, Stdout};

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Keep mouse capture disabled initially so the terminal emulator can select
    // and copy text natively. The chat can opt into application mouse events via
    // Alt+M after the UI has started.
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub struct TerminalGuard {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
}

impl TerminalGuard {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            terminal: Some(setup_terminal()?),
        })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        self.terminal
            .as_mut()
            .expect("terminal guard must contain an active terminal")
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if let Some(terminal) = self.terminal.as_mut() {
            let _ = restore_terminal(terminal);
        }
    }
}

pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        Show,
        DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}
