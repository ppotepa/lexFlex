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
}

pub fn render(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, vm: &HeaderViewModel) {
    let top = Line::from(vec![
        Span::styled(
            vm.title.clone(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            if vm.mode == "translation" {
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
    let bottom = Line::from(vec![Span::styled(
        "activity below · Tab switches focus · Alt+M toggles mouse mode",
        Style::default().fg(Color::Gray),
    )]);
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
