use crate::chat::session::TranscriptSelection;
use crate::chat::transcript::{MessageBlock, MessageRole, TranscriptMessage, TranscriptNode};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::BTreeMap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct ChatWindowViewModel<'a> {
    pub messages: &'a [TranscriptMessage],
    pub scroll: u16,
    pub selection: Option<&'a TranscriptSelection>,
    pub expanded_nodes: &'a BTreeMap<(usize, String), bool>,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowKind {
    Header(MessageRole),
    Body(MessageRole),
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HitRow {
    target: TranscriptSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedRow {
    kind: RowKind,
    text: String,
    hit: Option<HitRow>,
    selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MessageRowSpan {
    turn: usize,
    start: usize,
    end: usize,
}

pub fn render(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, vm: &ChatWindowViewModel<'_>) {
    let content_width = area.width.saturating_sub(2) as usize;
    let rendered = rendered_transcript_rows(vm.messages, content_width, vm.selection, vm.expanded_nodes);
    let lines: Vec<Line> = rendered.iter().map(|row| render_row(row, content_width)).collect();
    let max_scroll = max_scroll_for_height(vm.messages, area, vm.expanded_nodes);
    let scroll = if vm.scroll == u16::MAX {
        max_scroll
    } else {
        vm.scroll.min(max_scroll)
    };
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
                .title(if vm.focused { "chat [FOCUS]" } else { "chat" })
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_row(row: &RenderedRow, content_width: usize) -> Line<'static> {
    match row.kind {
        RowKind::Blank => Line::from(""),
        RowKind::Header(role) => styled_header(&row.text, role, row.selected, content_width),
        RowKind::Body(role) => styled_body(&row.text, role, row.selected, content_width),
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
    let bg = if selected {
        Color::Rgb(52, 52, 52)
    } else {
        row_background(role)
    };
    let split = line.find("]  ").unwrap_or(line.len());
    let (left, right) = if split < line.len() {
        (&line[..=split], &line[split + 3..])
    } else {
        (line, "")
    };
    let consumed = left.chars().count() + 1 + right.chars().count();
    let padding = " ".repeat(content_width.saturating_sub(consumed));
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };
    Line::from(vec![
        Span::styled(left.to_string(), Style::default().fg(color).bg(bg).add_modifier(modifier)),
        Span::styled(" ".to_string(), Style::default().bg(bg)),
        Span::styled(right.to_string(), Style::default().fg(Color::Gray).bg(bg).add_modifier(modifier)),
        Span::styled(padding, Style::default().bg(bg)),
    ])
}

fn styled_body(line: &str, role: MessageRole, selected: bool, content_width: usize) -> Line<'static> {
    let bg = if selected {
        Color::Rgb(52, 52, 52)
    } else {
        row_background(role)
    };
    let fg = if line.starts_with("  - ") {
        Color::White
    } else if matches!(role, MessageRole::System) {
        Color::Gray
    } else {
        Color::White
    };
    let text_len = UnicodeWidthStr::width(line);
    let padding = " ".repeat(content_width.saturating_sub(text_len));
    let modifier = if selected { Modifier::BOLD } else { Modifier::empty() };
    Line::from(vec![
        Span::styled(line.to_string(), Style::default().fg(fg).bg(bg).add_modifier(modifier)),
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
    selection: Option<&TranscriptSelection>,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Vec<RenderedRow> {
    let mut lines = vec![];
    if messages.is_empty() {
        lines.push(RenderedRow {
            kind: RowKind::Body(MessageRole::System),
            text: "No messages yet. Type text or use slash commands.".to_string(),
            hit: None,
            selected: false,
        });
        return lines;
    }
    for msg in messages {
        let message_selected = matches!(
            selection,
            Some(TranscriptSelection::Message { turn }) if *turn == msg.turn
        );
        let header = match msg.meta.latency_ms {
            Some(latency) => format!("{} [{} #{}]  {}ms", if msg.collapsed { "+" } else { "-" }, msg.title, msg.turn, latency),
            None => format!("{} [{} #{}]", if msg.collapsed { "+" } else { "-" }, msg.title, msg.turn),
        };
        lines.push(RenderedRow {
            kind: RowKind::Header(msg.role),
            text: fit_width(&header, content_width),
            hit: Some(HitRow {
                target: TranscriptSelection::Message { turn: msg.turn },
            }),
            selected: message_selected,
        });
        if msg.collapsed {
            lines.push(RenderedRow {
                kind: RowKind::Body(msg.role),
                text: "  (collapsed)".into(),
                hit: None,
                selected: message_selected,
            });
            lines.push(RenderedRow {
                kind: RowKind::Blank,
                text: String::new(),
                hit: None,
                selected: false,
            });
            continue;
        }
        for block in &msg.blocks {
            lines.extend(render_block_lines(block, msg.role, msg.turn, content_width, expanded_nodes, 0, selection));
        }
        if !lines.last().is_some_and(|line| matches!(line.kind, RowKind::Blank)) {
            lines.push(RenderedRow {
                kind: RowKind::Blank,
                text: String::new(),
                hit: None,
                selected: false,
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

pub fn max_scroll_for_height(
    messages: &[TranscriptMessage],
    area: ratatui::layout::Rect,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> u16 {
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
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Option<TranscriptSelection> {
    let rendered = rendered_transcript_rows(messages, area.width.saturating_sub(2) as usize, None, expanded_nodes);
    let absolute_row = scroll as usize + row_offset as usize;
    let row = rendered.get(absolute_row)?;
    row.hit.as_ref().map(|hit| hit.target.clone())
}

pub fn selection_at_row_offset(
    messages: &[TranscriptMessage],
    area: ratatui::layout::Rect,
    scroll: u16,
    row_offset: u16,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Option<TranscriptSelection> {
    hit_target_at_row_offset(messages, area, scroll, row_offset, expanded_nodes)
}

pub fn visible_selections(
    messages: &[TranscriptMessage],
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Vec<TranscriptSelection> {
    let mut selections = Vec::new();
    for message in messages {
        selections.push(TranscriptSelection::Message { turn: message.turn });
        if message.collapsed {
            continue;
        }
        collect_visible_node_selections(&message.blocks, message.turn, expanded_nodes, &mut selections);
    }
    selections
}

pub fn next_selection(
    messages: &[TranscriptMessage],
    current: Option<&TranscriptSelection>,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Option<TranscriptSelection> {
    let selections = visible_selections(messages, expanded_nodes);
    if selections.is_empty() {
        return None;
    }
    match current {
        Some(current) => {
            let index = selections.iter().position(|selection| selection == current)?;
            selections.get(index + 1).cloned().or_else(|| selections.last().cloned())
        }
        None => selections.first().cloned(),
    }
}

pub fn previous_selection(
    messages: &[TranscriptMessage],
    current: Option<&TranscriptSelection>,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Option<TranscriptSelection> {
    let selections = visible_selections(messages, expanded_nodes);
    if selections.is_empty() {
        return None;
    }
    match current {
        Some(current) => {
            let index = selections.iter().position(|selection| selection == current)?;
            index.checked_sub(1).and_then(|index| selections.get(index).cloned()).or_else(|| selections.first().cloned())
        }
        None => selections.last().cloned(),
    }
}

fn render_block_lines(
    block: &MessageBlock,
    role: MessageRole,
    turn: usize,
    content_width: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    depth: usize,
    selection: Option<&TranscriptSelection>,
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
        MessageBlock::Tree(nodes) => nodes.iter().flat_map(|node| render_node(node, role, turn, content_width, expanded_nodes, depth, selection)).collect(),
    }
}

fn render_node(
    node: &TranscriptNode,
    role: MessageRole,
    turn: usize,
    content_width: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    depth: usize,
    selection: Option<&TranscriptSelection>,
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
    let selected = matches!(
        selection,
        Some(TranscriptSelection::Node { turn: selected_turn, key })
            if *selected_turn == turn && key == &node.key
    );
    let mut rows = vec![RenderedRow {
        kind: RowKind::Body(role),
        text: fit_width(&line, content_width),
        hit: Some(HitRow {
            target: TranscriptSelection::Node {
                turn,
                key: node.key.clone(),
            },
        }),
        selected,
    }];
    if expanded {
        for block in &node.blocks {
            rows.extend(render_block_lines(block, role, turn, content_width, expanded_nodes, depth + 1, selection));
        }
        for child in &node.children {
            rows.extend(render_node(child, role, turn, content_width, expanded_nodes, depth + 1, selection));
        }
    }
    rows
}

fn message_row_spans(
    messages: &[TranscriptMessage],
    content_width: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
) -> Vec<MessageRowSpan> {
    let mut spans = Vec::new();
    let mut row = 0usize;
    for msg in messages {
        let start = row;
        row += 1;
        if msg.collapsed {
            row += 2;
        } else {
            for block in &msg.blocks {
                row += render_block_lines(block, msg.role, msg.turn, content_width, expanded_nodes, 0, None).len();
            }
            row += 1;
        }
        spans.push(MessageRowSpan { turn: msg.turn, start, end: row });
    }
    spans
}

fn collect_visible_node_selections(
    blocks: &[MessageBlock],
    turn: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    selections: &mut Vec<TranscriptSelection>,
) {
    for block in blocks {
        if let MessageBlock::Tree(nodes) = block {
            collect_visible_node_selections_in_nodes(nodes, turn, expanded_nodes, selections);
        }
    }
}

fn collect_visible_node_selections_in_nodes(
    nodes: &[TranscriptNode],
    turn: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    selections: &mut Vec<TranscriptSelection>,
) {
    for node in nodes {
        selections.push(TranscriptSelection::Node {
            turn,
            key: node.key.clone(),
        });
        let expanded = expanded_nodes
            .get(&(turn, node.key.clone()))
            .copied()
            .unwrap_or(node.expanded);
        if expanded {
            collect_visible_node_selections(&node.blocks, turn, expanded_nodes, selections);
            collect_visible_node_selections_in_nodes(&node.children, turn, expanded_nodes, selections);
        }
    }
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
                rows.push(RenderedRow {
                    kind: RowKind::Body(role),
                    text: current,
                    hit: None,
                    selected: false,
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
            hit: None,
            selected: false,
        });
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::{
        hit_target_at_row_offset, max_scroll_for_height, previous_selection, rendered_transcript_rows,
        selection_at_row_offset, turn_at_row_offset, visible_selections, RowKind,
    };
    use crate::chat::session::TranscriptSelection;
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
                MessageBlock::KeyValue {
                    key: "direction".to_string(),
                    value: "pl -> en".to_string(),
                },
            ],
            meta: MessageMeta { latency_ms: Some(42), command: None },
        };
        let lines = rendered_transcript_rows(&[message], 40, None, &BTreeMap::new());
        assert_eq!(lines[0].text, "- [assistant #1]  42ms");
        assert!(matches!(lines[0].kind, RowKind::Header(MessageRole::Assistant)));
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
        assert!(matches!(lines[0].kind, RowKind::Header(MessageRole::EngineFacts)));
        assert!(lines.iter().any(|line| matches!(line.kind, RowKind::Body(MessageRole::EngineFacts))));
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
            max_scroll_for_height(&transcript.messages, ratatui::layout::Rect::new(20, 0, 20, 4), &BTreeMap::new()),
            1
        );
    }

    #[test]
    fn node_rows_expose_selection_targets() {
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
        let expanded = BTreeMap::new();
        assert!(matches!(
            hit_target_at_row_offset(&[message.clone()], ratatui::layout::Rect::new(0, 0, 40, 10), 0, 1, &expanded),
            Some(TranscriptSelection::Node { turn: 1, .. })
        ));
        let lines = rendered_transcript_rows(&[message], 40, Some(&TranscriptSelection::Message { turn: 1 }), &expanded);
        assert!(lines[0].selected);
    }

    #[test]
    fn click_row_maps_back_to_message_turn() {
        let mut transcript = Transcript::new();
        transcript.push(
            MessageRole::User,
            "you",
            vec![MessageBlock::Paragraph("first".to_string())],
            MessageMeta::default(),
        );
        transcript.push(
            MessageRole::Assistant,
            "assistant",
            vec![MessageBlock::Paragraph("second".to_string())],
            MessageMeta::default(),
        );
        let area = ratatui::layout::Rect::new(0, 0, 40, 8);
        assert_eq!(turn_at_row_offset(&transcript.messages, area, 0, 0, &BTreeMap::new()), Some(1));
        assert_eq!(turn_at_row_offset(&transcript.messages, area, 0, 3, &BTreeMap::new()), Some(2));
    }

    #[test]
    fn selection_order_includes_visible_nodes() {
        let node = TranscriptNode {
            key: "facts.0".into(),
            label: "Facts".into(),
            summary: None,
            tone: None,
            expanded: true,
            blocks: vec![],
            children: vec![TranscriptNode {
                key: "facts.0.child".into(),
                label: "Child".into(),
                summary: None,
                tone: None,
                expanded: false,
                blocks: vec![],
                children: vec![],
            }],
        };
        let message = TranscriptMessage {
            turn: 1,
            role: MessageRole::EngineFacts,
            title: "facts".into(),
            collapsed: false,
            blocks: vec![MessageBlock::Tree(vec![node])],
            meta: MessageMeta::default(),
        };
        let selections = visible_selections(&[message], &BTreeMap::new());
        assert!(matches!(selections[0], TranscriptSelection::Message { turn: 1 }));
        assert!(matches!(selections[1], TranscriptSelection::Node { ref key, .. } if key == "facts.0"));
        assert!(matches!(selections[2], TranscriptSelection::Node { ref key, .. } if key == "facts.0.child"));
    }

    #[test]
    fn selection_lookup_returns_exact_node_target() {
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
        assert!(matches!(
            selection_at_row_offset(&[message], ratatui::layout::Rect::new(0, 0, 40, 10), 0, 1, &BTreeMap::new()),
            Some(TranscriptSelection::Node { turn: 1, .. })
        ));
    }

    #[test]
    fn node_selection_highlights_only_the_exact_node() {
        let nodes = ["facts.0", "facts.1"].map(|key| TranscriptNode {
            key: key.into(),
            label: key.into(),
            summary: None,
            tone: None,
            expanded: false,
            blocks: vec![],
            children: vec![],
        });
        let message = TranscriptMessage {
            turn: 1,
            role: MessageRole::EngineFacts,
            title: "facts".into(),
            collapsed: false,
            blocks: vec![MessageBlock::Tree(nodes.into())],
            meta: MessageMeta::default(),
        };
        let selection = TranscriptSelection::Node {
            turn: 1,
            key: "facts.1".into(),
        };

        let rows = rendered_transcript_rows(&[message], 40, Some(&selection), &BTreeMap::new());

        assert!(!rows[0].selected);
        assert!(!rows[1].selected);
        assert!(rows[2].selected);
    }

    #[test]
    fn previous_selection_moves_backwards() {
        let mut transcript = Transcript::new();
        transcript.push(
            MessageRole::User,
            "you",
            vec![MessageBlock::Paragraph("first".to_string())],
            MessageMeta::default(),
        );
        transcript.push(
            MessageRole::Assistant,
            "assistant",
            vec![MessageBlock::Paragraph("second".to_string())],
            MessageMeta::default(),
        );
        let selection = previous_selection(
            &transcript.messages,
            Some(&TranscriptSelection::Message { turn: 2 }),
            &BTreeMap::new(),
        );
        assert!(matches!(selection, Some(TranscriptSelection::Message { turn: 1 })));
    }
}
