use crate::chat::transcript::{MessageBlock, MessageRole, TranscriptMessage};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct ChatWindowViewModel<'a> {
    pub messages: &'a [TranscriptMessage],
    pub scroll: u16,
}

pub fn render(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, vm: &ChatWindowViewModel<'_>) {
    let content_width = area.width.saturating_sub(2) as usize;
    let rendered = rendered_transcript_rows(vm.messages, content_width);
    let lines: Vec<Line> = rendered
        .iter()
        .map(|row| render_row(row, content_width))
        .collect();
    let max_scroll = max_scroll_for_height(vm.messages, area);
    let scroll = if vm.scroll == u16::MAX {
        max_scroll
    } else {
        vm.scroll.min(max_scroll)
    };
    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("chat")
                .border_style(Style::default().fg(Color::Gray)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowKind {
    Header(MessageRole),
    Body(MessageRole),
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedRow {
    kind: RowKind,
    text: String,
}

fn render_row(row: &RenderedRow, content_width: usize) -> Line<'static> {
    match row.kind {
        RowKind::Blank => Line::from(""),
        RowKind::Header(role) => styled_header(&row.text, role, content_width),
        RowKind::Body(role) => styled_body(&row.text, role, content_width),
    }
}

fn styled_header(line: &str, role: MessageRole, content_width: usize) -> Line<'static> {
    let color = match role {
        MessageRole::User => Color::Green,
        MessageRole::Assistant => Color::Cyan,
        MessageRole::System => Color::Yellow,
        MessageRole::Error => Color::Red,
    };
    let bg = row_background(role);
    let split = line.find("]  ").unwrap_or(line.len());
    let (left, right) = if split < line.len() { (&line[..=split], &line[split + 3..]) } else { (line, "") };
    let consumed = left.chars().count() + 1 + right.chars().count();
    let padding = " ".repeat(content_width.saturating_sub(consumed));
    Line::from(vec![
        Span::styled(left.to_string(), Style::default().fg(color).bg(bg)),
        Span::styled(" ".to_string(), Style::default().bg(bg)),
        Span::styled(right.to_string(), Style::default().fg(Color::Gray).bg(bg)),
        Span::styled(padding, Style::default().bg(bg)),
    ])
}

fn styled_body(line: &str, role: MessageRole, content_width: usize) -> Line<'static> {
    let bg = row_background(role);
    let fg = if line.starts_with("  - ") {
        Color::White
    } else if matches!(role, MessageRole::System) {
        Color::Gray
    } else {
        Color::White
    };
    let text_len = UnicodeWidthStr::width(line);
    let padding = " ".repeat(content_width.saturating_sub(text_len));
    Line::from(vec![
        Span::styled(line.to_string(), Style::default().fg(fg).bg(bg)),
        Span::styled(padding, Style::default().bg(bg)),
    ])
}

fn row_background(role: MessageRole) -> Color {
    match role {
        MessageRole::User => Color::Rgb(20, 28, 20),
        _ => Color::Reset,
    }
}

fn rendered_transcript_rows(messages: &[TranscriptMessage], content_width: usize) -> Vec<RenderedRow> {
    let mut lines = vec![];
    if messages.is_empty() {
        lines.push(RenderedRow {
            kind: RowKind::Body(MessageRole::System),
            text: "No messages yet. Type text or use slash commands.".to_string(),
        });
        return lines;
    }
    for msg in messages {
        let header = match msg.meta.latency_ms {
            Some(latency) => format!("[{} #{}]  {}ms", msg.title, msg.turn, latency),
            None => format!("[{} #{}]", msg.title, msg.turn),
        };
        lines.push(RenderedRow {
            kind: RowKind::Header(msg.role),
            text: fit_width(&header, content_width),
        });
        for block in &msg.blocks {
            lines.extend(render_block_lines(block, msg.role, content_width));
        }
        if !lines
            .last()
            .is_some_and(|line| matches!(line.kind, RowKind::Blank))
        {
            lines.push(RenderedRow {
                kind: RowKind::Blank,
                text: String::new(),
            });
        }
    }
    lines
}

fn fit_width(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + char_width > width {
            break;
        }
        result.push(ch);
        used += char_width;
    }
    result
}

pub fn max_scroll_for_height(messages: &[TranscriptMessage], area: ratatui::layout::Rect) -> u16 {
    let viewport_height = area.height.saturating_sub(2) as usize;
    rendered_transcript_rows(messages, area.width.saturating_sub(2) as usize)
        .len()
        .saturating_sub(viewport_height) as u16
}

fn render_block_lines(
    block: &MessageBlock,
    role: MessageRole,
    content_width: usize,
) -> Vec<RenderedRow> {
    match block {
        MessageBlock::Paragraph(text) => wrap_prefixed(text, "  ", role, content_width),
        MessageBlock::KeyValue { key, value } => {
            wrap_prefixed(&format!("{}: {}", key, value), "  ", role, content_width)
        }
        MessageBlock::BulletList(items) => items
            .iter()
            .flat_map(|item| wrap_prefixed(item, "  - ", role, content_width))
            .collect(),
        MessageBlock::Trace(lines) => lines
            .iter()
            .flat_map(|line| wrap_prefixed(line, "  ", role, content_width))
            .collect(),
    }
}

fn wrap_prefixed(
    text: &str,
    prefix: &str,
    role: MessageRole,
    content_width: usize,
) -> Vec<RenderedRow> {
    let width = content_width.max(1);
    let prefix_width = UnicodeWidthStr::width(prefix).min(width);
    let continuation = " ".repeat(prefix_width);
    let mut rows = Vec::new();

    let source_lines: Vec<&str> = if text.is_empty() {
        vec![""]
    } else {
        text.split('\n').collect()
    };
    for source_line in source_lines {
        let mut current = prefix.to_string();
        let mut current_width = prefix_width;
        for ch in source_line.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_width > prefix_width && current_width + ch_width > width {
                rows.push(RenderedRow {
                    kind: RowKind::Body(role),
                    text: current,
                });
                current = continuation.clone();
                current_width = prefix_width;
            }
            current.push(ch);
            current_width += ch_width;
        }
        rows.push(RenderedRow {
            kind: RowKind::Body(role),
            text: current,
        });
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::{max_scroll_for_height, rendered_transcript_rows, RowKind};
    use crate::chat::transcript::{
        MessageBlock, MessageMeta, MessageRole, Transcript, TranscriptMessage,
    };

    #[test]
    fn transcript_projection_is_compact() {
        let message = TranscriptMessage {
            turn: 1,
            role: MessageRole::Assistant,
            title: "assistant".to_string(),
            blocks: vec![
                MessageBlock::Paragraph("hello".to_string()),
                MessageBlock::KeyValue {
                    key: "direction".to_string(),
                    value: "pl -> en".to_string(),
                },
            ],
            meta: MessageMeta {
                latency_ms: Some(42),
                command: None,
            },
        };
        let lines = rendered_transcript_rows(&[message], 40);
        assert_eq!(lines[0].text, "[assistant #1]  42ms");
        assert!(matches!(lines[0].kind, RowKind::Header(MessageRole::Assistant)));
        assert_eq!(lines[1].text, "  hello");
        assert_eq!(lines[2].text, "  direction: pl -> en");
    }

    #[test]
    fn max_scroll_matches_line_count() {
        let mut transcript = Transcript::new();
        transcript.push(
            MessageRole::User,
            "you",
            vec![MessageBlock::Paragraph("hello".to_string())],
            MessageMeta::default(),
        );
        assert_eq!(
            max_scroll_for_height(&transcript.messages, ratatui::layout::Rect::new(20, 0, 20, 4)),
            1
        );
    }

    #[test]
    fn user_background_fills_wrapped_rows() {
        use super::render;
        use ratatui::backend::TestBackend;
        use ratatui::layout::Rect;
        use ratatui::style::Color;
        use ratatui::Terminal;

        let mut transcript = Transcript::new();
        transcript.push(
            MessageRole::User,
            "you",
            vec![MessageBlock::Paragraph("abcdefghijklmnopqrstuvwxyz".to_string())],
            MessageMeta::default(),
        );
        let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();
        terminal
            .draw(|frame| {
                render(
                    frame,
                    Rect::new(0, 0, 20, 8),
                    &super::ChatWindowViewModel {
                        messages: &transcript.messages,
                        scroll: u16::MAX,
                    },
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        for y in [1, 2, 3] {
            for x in 1..19 {
                assert_eq!(buffer.get(x, y).bg, Color::Rgb(20, 28, 20));
            }
        }
    }
}
