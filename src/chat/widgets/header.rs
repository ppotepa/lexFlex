use crate::chat::settings::Verbosity;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct HeaderViewModel {
    pub title: String,
    pub mode: String,
    pub chat_language: String,
    pub source_lang: String,
    pub target_lang: String,
    pub verbosity: Verbosity,
    pub pending_label: Option<String>,
    pub pending_stage: Option<String>,
    pub pending_spinner: usize,
    pub backend_ready: bool,
}

pub fn render(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, vm: &HeaderViewModel) {
    let top = Line::from(vec![
        Span::styled(
            vm.title.clone(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            if vm.mode == "translate" {
                format!("translate {} -> {}", vm.source_lang, vm.target_lang)
            } else {
                format!("chat language {}", vm.chat_language)
            },
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            format!("verbosity {}", vm.verbosity.as_str()),
            Style::default().fg(Color::Magenta),
        ),
    ]);
    let bottom = if let Some(label) = vm.pending_label.as_deref() {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        Line::from(vec![
            Span::styled(frames[vm.pending_spinner % frames.len()], Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(label.to_string(), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled(vm.pending_stage.clone().unwrap_or_else(|| "queued".into()), Style::default().fg(Color::Gray)),
        ])
    } else if !vm.backend_ready {
        Line::from(vec![Span::styled("backend starting", Style::default().fg(Color::Gray))])
    } else {
        Line::from(vec![Span::styled("ready", Style::default().fg(Color::Gray))])
    };
    let paragraph = Paragraph::new(Text::from(vec![top, bottom]))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("session")
                .border_style(Style::default().fg(Color::Gray)),
        );
    frame.render_widget(paragraph, area);
}
