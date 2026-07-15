use crate::chat::session::ActivityState;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, activity: &ActivityState) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let active = !activity.label.is_empty();
    let spinner = if active { spinner(activity.spinner_index) } else { "·" };
    let label = if active { activity.label.as_str() } else { "idle" };
    let stage = if active { activity.current_stage.as_str() } else { "waiting for request" };
    let elapsed = activity
        .started_at
        .map(|started| format!("{}ms", started.elapsed().as_millis()))
        .unwrap_or_else(|| "0ms".into());
    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("{spinner} {label}"), Style::default().fg(if active { Color::Cyan } else { Color::Gray })),
            Span::raw(format!("  {}  {}", stage, elapsed)),
        ]),
    ];
    if !activity.current_stage_path.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            format!("  {}", activity.current_stage_path.replace('.', " › ")),
            Style::default().fg(Color::Gray),
        )]));
    }
    for detail in activity.details.iter().take(2) {
        lines.push(Line::from(vec![Span::styled(
            format!("  {detail}"),
            Style::default().fg(Color::White),
        )]));
    }
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("activity").border_style(Style::default().fg(if active { Color::Yellow } else { Color::Gray })));
    frame.render_widget(paragraph, area);
}

fn spinner(index: usize) -> &'static str {
    ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"][index % 10]
}
