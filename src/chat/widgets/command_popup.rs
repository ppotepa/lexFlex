use crate::chat::commands::SlashSuggestion;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub struct CommandPopupViewModel<'a> {
    pub suggestions: &'a [SlashSuggestion],
    pub selected_index: usize,
    pub focused: bool,
}

pub fn render(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, vm: &CommandPopupViewModel<'_>) {
    if vm.suggestions.is_empty() {
        return;
    }
    frame.render_widget(Clear, area);
    let mut lines = vec![];
    for (idx, item) in vm.suggestions.iter().enumerate() {
        let style = if idx == vm.selected_index {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::styled(item.replacement.clone(), style),
            Span::raw("  "),
            Span::styled(item.description.clone(), Style::default().fg(Color::Gray)),
        ]));
    }
    let border_style = if vm.focused {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if vm.focused { "commands [FOCUS]" } else { "commands" })
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
