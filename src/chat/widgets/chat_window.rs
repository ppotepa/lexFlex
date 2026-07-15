use crate::chat::transcript::{MessageBlock, MessageRole, TranscriptMessage, TranscriptNode};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::BTreeMap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct ChatWindowViewModel<'a> {
    pub messages: &'a [TranscriptMessage],
    pub scroll: u16,
    pub selected_turn: Option<usize>,
    pub expanded_nodes: &'a BTreeMap<(usize, String), bool>,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatHitTarget {
    Message { turn: usize },
    Node { turn: usize, key: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MessageRowSpan {
    turn: usize,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HitRow {
    target: Option<ChatHitTarget>,
    toggle_width: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowKind {
    Header(MessageRole, bool),
    Body(MessageRole),
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedRow {
    kind: RowKind,
    text: String,
    hit: Option<HitRow>,
}

pub fn render(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, vm: &ChatWindowViewModel<'_>) {
    let content_width = area.width.saturating_sub(2) as usize;
    let rendered = rendered_transcript_rows(vm.messages, content_width, vm.selected_turn, vm.expanded_nodes);
    let lines: Vec<Line> = rendered.iter().map(|row| render_row(row, content_width)).collect();
    let max_scroll = max_scroll_for_height(vm.messages, area, vm.expanded_nodes);
    let scroll = if vm.scroll == u16::MAX { max_scroll } else { vm.scroll.min(max_scroll) };
    let border_style = if vm.focused {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title(if vm.focused { "chat [FOCUS]" } else { "chat" }).border_style(border_style))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_row(row: &RenderedRow, content_width: usize) -> Line<'static> {
    match row.kind {
        RowKind::Blank => Line::from(""),
        RowKind::Header(role, selected) => styled_header(&row.text, role, selected, content_width),
        RowKind::Body(role) => styled_body(&row.text, role, content_width),
    }
}

fn styled_header(line: &str, role: MessageRole, selected: bool, content_width: usize) -> Line<'static> {
    let color = match role {
        MessageRole::User => Color::Green,
        MessageRole::Assistant => Color::Cyan,
        MessageRole::EngineFacts => Color::Magenta,
        MessageRole::EngineEvidence => Color::LightCyan,
        MessageRole::EngineInterlingua => Color::LightBlue,
        MessageRole::EngineEntities => Color::Yellow,
        MessageRole::EngineQuery => Color::Blue,
        MessageRole::EnginePipeline => Color::LightBlue,
        MessageRole::EngineSources => Color::Green,
        MessageRole::EngineSnapshot => Color::Gray,
        MessageRole::EngineTrace => Color::Cyan,
        MessageRole::System => Color::Yellow,
        MessageRole::Error => Color::Red,
    };
    let bg = if selected { Color::Rgb(52, 52, 52) } else { row_background(role) };
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
        MessageRole::EngineFacts => Color::Rgb(28, 18, 32),
        MessageRole::EngineEvidence => Color::Rgb(16, 28, 32),
        MessageRole::EngineInterlingua => Color::Rgb(18, 20, 36),
        MessageRole::EngineEntities => Color::Rgb(34, 28, 14),
        MessageRole::EngineQuery => Color::Rgb(18, 20, 34),
        MessageRole::EnginePipeline => Color::Rgb(16, 18, 28),
        MessageRole::EngineSources => Color::Rgb(16, 28, 18),
        MessageRole::EngineSnapshot => Color::Rgb(24, 24, 24),
        MessageRole::EngineTrace => Color::Rgb(16, 24, 30),
        _ => Color::Reset,
    }
}

fn rendered_transcript_rows(
    messages: &[TranscriptMessage],
    content_width: usize,
    selected_turn: Option<usize>,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Vec<RenderedRow> {
    let mut lines = vec![];
    if messages.is_empty() {
        lines.push(RenderedRow {
            kind: RowKind::Body(MessageRole::System),
            text: "No messages yet. Type text or use slash commands.".to_string(),
            hit: None,
        });
        return lines;
    }
    for msg in messages {
        let header = match msg.meta.latency_ms {
            Some(latency) => format!("{} [{} #{}]  {}ms", if msg.collapsed { "+" } else { "-" }, msg.title, msg.turn, latency),
            None => format!("{} [{} #{}]", if msg.collapsed { "+" } else { "-" }, msg.title, msg.turn),
        };
        lines.push(RenderedRow {
            kind: RowKind::Header(msg.role, selected_turn == Some(msg.turn)),
            text: fit_width(&header, content_width),
            hit: Some(HitRow {
                target: Some(ChatHitTarget::Message { turn: msg.turn }),
                toggle_width: 3,
            }),
        });
        if msg.collapsed {
            lines.push(RenderedRow {
                kind: RowKind::Body(msg.role),
                text: "  (collapsed)".into(),
                hit: None,
            });
            lines.push(RenderedRow { kind: RowKind::Blank, text: String::new(), hit: None });
            continue;
        }
        for block in &msg.blocks {
            lines.extend(render_block_lines(block, msg.role, msg.turn, content_width, expanded_nodes, 0));
        }
        if !lines.last().is_some_and(|line| matches!(line.kind, RowKind::Blank)) {
            lines.push(RenderedRow { kind: RowKind::Blank, text: String::new(), hit: None });
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

pub fn max_scroll_for_height(messages: &[TranscriptMessage], area: ratatui::layout::Rect, expanded_nodes: &BTreeMap<(usize, String), bool>) -> u16 {
    let viewport_height = area.height.saturating_sub(2) as usize;
    rendered_transcript_rows(messages, area.width.saturating_sub(2) as usize, None, expanded_nodes)
        .len()
        .saturating_sub(viewport_height) as u16
}

pub fn turn_at_row_offset(
    messages: &[TranscriptMessage],
    area: ratatui::layout::Rect,
    scroll: u16,
    row_offset: u16,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Option<usize> {
    let content_width = area.width.saturating_sub(2) as usize;
    let absolute_row = scroll as usize + row_offset as usize;
    message_row_spans(messages, content_width, expanded_nodes)
        .into_iter()
        .find(|span| absolute_row >= span.start && absolute_row < span.end)
        .map(|span| span.turn)
}

pub fn hit_target_at_row_offset(
    messages: &[TranscriptMessage],
    area: ratatui::layout::Rect,
    scroll: u16,
    row_offset: u16,
    column_offset: u16,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Option<ChatHitTarget> {
    let rendered = rendered_transcript_rows(messages, area.width.saturating_sub(2) as usize, None, expanded_nodes);
    let absolute_row = scroll as usize + row_offset as usize;
    let row = rendered.get(absolute_row)?;
    let hit = row.hit.clone()?;
    if usize::from(column_offset) <= hit.toggle_width {
        hit.target
    } else {
        None
    }
}

fn render_block_lines(
    block: &MessageBlock,
    role: MessageRole,
    turn: usize,
    content_width: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    depth: usize,
) -> Vec<RenderedRow> {
    match block {
        MessageBlock::Section(text) => wrap_prefixed(text, "  ", role, content_width),
        MessageBlock::Paragraph(text) => wrap_prefixed(text, "  ", role, content_width),
        MessageBlock::KeyValue { key, value } => wrap_prefixed(&format!("{}: {}", key, value), "  ", role, content_width),
        MessageBlock::BulletList(items) => items.iter().flat_map(|item| wrap_prefixed(item, "  - ", role, content_width)).collect(),
        MessageBlock::Notice { tone, text } => wrap_prefixed(&format!("[{tone}] {text}"), "  ! ", role, content_width),
        MessageBlock::Timeline(items) => items.iter().flat_map(|item| wrap_prefixed(item, "  > ", role, content_width)).collect(),
        MessageBlock::TechnicalList(items) => items.iter().flat_map(|item| wrap_prefixed(item, "  # ", role, content_width)).collect(),
        MessageBlock::Trace(lines) => lines.iter().flat_map(|line| wrap_prefixed(line, "  ", role, content_width)).collect(),
        MessageBlock::Tree(nodes) => nodes.iter().flat_map(|node| render_node(node, role, turn, content_width, expanded_nodes, depth)).collect(),
    }
}

fn render_node(
    node: &TranscriptNode,
    role: MessageRole,
    turn: usize,
    content_width: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    depth: usize,
) -> Vec<RenderedRow> {
    let expanded = expanded_nodes
        .get(&(turn, node.key.clone()))
        .copied()
        .unwrap_or(node.expanded);
    let indent = "  ".repeat(depth);
    let prefix = if expanded { "[-]" } else { "[+]" };
    let summary = node.summary.as_ref().map(|summary| format!(": {summary}")).unwrap_or_default();
    let tone = node.tone.as_deref().unwrap_or("info");
    let line = format!("{indent}{prefix} {}{} ({tone})", node.label, summary);
    let mut rows = vec![RenderedRow {
        kind: RowKind::Body(role),
        text: fit_width(&line, content_width),
        hit: Some(HitRow {
            target: Some(ChatHitTarget::Node {
                turn,
                key: node.key.clone(),
            }),
            toggle_width: indent.len() + 3,
        }),
    }];
    if expanded {
        for block in &node.blocks {
            rows.extend(render_block_lines(block, role, turn, content_width, expanded_nodes, depth + 1));
        }
        for child in &node.children {
            rows.extend(render_node(child, role, turn, content_width, expanded_nodes, depth + 1));
        }
    }
    rows
}

fn message_row_spans(messages: &[TranscriptMessage], content_width: usize, expanded_nodes: &BTreeMap<(usize, String), bool>) -> Vec<MessageRowSpan> {
    let mut spans = Vec::new();
    let mut row = 0usize;
    for msg in messages {
        let start = row;
        row += 1;
        if msg.collapsed {
            row += 2;
        } else {
            for block in &msg.blocks {
                row += render_block_lines(block, msg.role, msg.turn, content_width, expanded_nodes, 0).len();
            }
            row += 1;
        }
        spans.push(MessageRowSpan { turn: msg.turn, start, end: row });
    }
    spans
}

fn wrap_prefixed(text: &str, prefix: &str, role: MessageRole, content_width: usize) -> Vec<RenderedRow> {
    let width = content_width.max(1);
    let prefix_width = UnicodeWidthStr::width(prefix).min(width);
    let continuation = " ".repeat(prefix_width);
    let mut rows = Vec::new();

    let source_lines: Vec<&str> = if text.is_empty() { vec![""] } else { text.split('\n').collect() };
    for source_line in source_lines {
        let mut current = prefix.to_string();
        let mut current_width = prefix_width;
        for ch in source_line.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_width > prefix_width && current_width + ch_width > width {
                rows.push(RenderedRow { kind: RowKind::Body(role), text: current, hit: None });
                current = continuation.clone();
                current_width = prefix_width;
            }
            current.push(ch);
            current_width += ch_width;
        }
        rows.push(RenderedRow { kind: RowKind::Body(role), text: current, hit: None });
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::{hit_target_at_row_offset, max_scroll_for_height, rendered_transcript_rows, turn_at_row_offset, ChatHitTarget, RowKind};
    use crate::chat::transcript::{MessageBlock, MessageMeta, MessageRole, Transcript, TranscriptMessage, TranscriptNode};
    use std::collections::BTreeMap;

    #[test]
    fn transcript_projection_is_compact() {
        let message = TranscriptMessage {
            turn: 1,
            role: MessageRole::Assistant,
            title: "assistant".to_string(),
            collapsed: false,
            blocks: vec![
                MessageBlock::Paragraph("hello".to_string()),
                MessageBlock::KeyValue { key: "direction".to_string(), value: "pl -> en".to_string() },
            ],
            meta: MessageMeta { latency_ms: Some(42), command: None },
        };
        let lines = rendered_transcript_rows(&[message], 40, None, &BTreeMap::new());
        assert_eq!(lines[0].text, "- [assistant #1]  42ms");
        assert!(matches!(lines[0].kind, RowKind::Header(MessageRole::Assistant, _)));
        assert_eq!(lines[1].text, "  hello");
        assert_eq!(lines[2].text, "  direction: pl -> en");
    }

    #[test]
    fn engine_debug_messages_keep_a_distinct_role() {
        let message = TranscriptMessage {
            turn: 1,
            role: MessageRole::EngineFacts,
            title: "engine debug".into(),
            collapsed: false,
            blocks: vec![MessageBlock::Paragraph("one fact".into())],
            meta: MessageMeta::default(),
        };
        let lines = rendered_transcript_rows(&[message], 80, None, &BTreeMap::new());
        assert!(matches!(lines[0].kind, RowKind::Header(MessageRole::EngineFacts, _)));
        assert!(lines.iter().any(|line| matches!(line.kind, RowKind::Body(MessageRole::EngineFacts))));
    }

    #[test]
    fn max_scroll_matches_line_count() {
        let mut transcript = Transcript::new();
        transcript.push(MessageRole::User, "you", vec![MessageBlock::Paragraph("hello".to_string())], MessageMeta::default());
        assert_eq!(max_scroll_for_height(&transcript.messages, ratatui::layout::Rect::new(20, 0, 20, 4), &BTreeMap::new()), 1);
    }

    #[test]
    fn node_rows_expose_toggle_targets() {
        let node = TranscriptNode {
            key: "facts.0".into(),
            label: "Capital of France".into(),
            summary: Some("Paris".into()),
            tone: None,
            expanded: false,
            blocks: vec![],
            children: vec![],
        };
        let message = TranscriptMessage {
            turn: 1,
            role: MessageRole::EngineFacts,
            title: "facts".into(),
            collapsed: false,
            blocks: vec![MessageBlock::Tree(vec![node])],
            meta: MessageMeta::default(),
        };
        let mut expanded = BTreeMap::new();
        assert!(matches!(
            hit_target_at_row_offset(&[message.clone()], ratatui::layout::Rect::new(0, 0, 40, 10), 0, 1, 1, &expanded),
            Some(ChatHitTarget::Node { turn: 1, .. })
        ));
        expanded.insert((1, "facts.0".into()), true);
        let lines = rendered_transcript_rows(&[message], 40, None, &expanded);
        assert!(lines.iter().any(|line| line.text.contains("Capital of France")));
    }

    #[test]
    fn click_row_maps_back_to_message_turn() {
        let mut transcript = Transcript::new();
        transcript.push(MessageRole::User, "you", vec![MessageBlock::Paragraph("first".to_string())], MessageMeta::default());
        transcript.push(MessageRole::Assistant, "assistant", vec![MessageBlock::Paragraph("second".to_string())], MessageMeta::default());
        let area = ratatui::layout::Rect::new(0, 0, 40, 8);
        assert_eq!(turn_at_row_offset(&transcript.messages, area, 0, 0, &BTreeMap::new()), Some(1));
        assert_eq!(turn_at_row_offset(&transcript.messages, area, 0, 3, &BTreeMap::new()), Some(2));
    }
}
