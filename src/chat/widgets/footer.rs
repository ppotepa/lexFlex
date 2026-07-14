use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthChar;

pub struct FooterViewModel<'a> {
    pub input: &'a str,
    /// Byte offset into `input`, always on a UTF-8 character boundary.
    pub cursor: usize,
    pub status_line: String,
}

pub fn render(frame: &mut ratatui::Frame, area: Rect, vm: &FooterViewModel<'_>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .split(area);

    let status = Paragraph::new(vm.status_line.clone())
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(status, chunks[0]);

    let hints = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" send  "),
        Span::styled("Shift+Enter", Style::default().fg(Color::Green)),
        Span::raw(" newline  "),
        Span::styled("/", Style::default().fg(Color::Green)),
        Span::raw(" commands  "),
        Span::styled("Alt+V", Style::default().fg(Color::Green)),
        Span::raw(" verbosity  "),
        Span::styled("Ctrl+C", Style::default().fg(Color::Green)),
        Span::raw(" quit"),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[1]);

    render_input(frame, chunks[2], vm.input, vm.cursor);
}

fn render_input(frame: &mut ratatui::Frame, area: Rect, input: &str, cursor: usize) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let text_width = inner_width.saturating_sub(2).max(1);
    let inner_height = area.height.saturating_sub(2).max(1) as usize;
    let layout = input_layout(input, cursor, text_width);

    let start = layout
        .cursor_row
        .saturating_sub(inner_height.saturating_sub(1));
    let lines = layout
        .lines
        .iter()
        .skip(start)
        .take(inner_height)
        .enumerate()
        .map(|(index, text)| {
            Line::from(vec![
                Span::styled(
                    if index + start == 0 { "> " } else { "  " },
                    Style::default().fg(Color::Green),
                ),
                Span::styled(text.clone(), Style::default().fg(Color::White)),
            ])
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("input")
                .border_style(Style::default().fg(Color::Gray)),
        ),
        area,
    );

    let cursor_row = layout.cursor_row.saturating_sub(start) as u16;
    let cursor_x = area.x.saturating_add(1 + 2 + layout.cursor_col as u16);
    let cursor_y = area.y.saturating_add(1 + cursor_row);
    frame.set_cursor(cursor_x, cursor_y);
}

#[derive(Debug, PartialEq, Eq)]
struct InputLayout {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
}

fn input_layout(input: &str, cursor: usize, width: usize) -> InputLayout {
    let width = width.max(1);
    let mut lines = vec![String::new()];
    let mut row = 0usize;
    let mut col = 0usize;
    let mut cursor_row = 0usize;
    let mut cursor_col = 0usize;
    let cursor = cursor.min(input.len());

    for (byte_index, ch) in input.char_indices() {
        if byte_index == cursor {
            cursor_row = row;
            cursor_col = col;
        }
        if ch == '\n' {
            row += 1;
            lines.push(String::new());
            col = 0;
            continue;
        }
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if col > 0 && col + char_width > width {
            row += 1;
            lines.push(String::new());
            col = 0;
        }
        lines[row].push(ch);
        col += char_width;
    }
    if cursor == input.len() {
        cursor_row = row;
        cursor_col = col;
        if cursor_col >= width {
            cursor_row += 1;
            cursor_col = 0;
            lines.push(String::new());
        }
    }

    InputLayout {
        lines,
        cursor_row,
        cursor_col,
    }
}

#[cfg(test)]
mod tests {
    use super::{input_layout, render};
    use ratatui::backend::{Backend, TestBackend};
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn cursor_is_after_text() {
        let layout = input_layout("hello", 5, 10);
        assert_eq!(layout.cursor_col, 5);
        assert_eq!(layout.lines, vec!["hello"]);
    }

    #[test]
    fn full_line_moves_cursor_to_next_line() {
        let layout = input_layout("12345678", 8, 8);
        assert_eq!(layout.cursor_row, 1);
        assert_eq!(layout.cursor_col, 0);
        assert_eq!(layout.lines, vec!["12345678", ""]);
    }

    #[test]
    fn wraps_and_tracks_cursor_inside_text() {
        let layout = input_layout("abcdefghi", 7, 4);
        assert_eq!(layout.lines, vec!["abcd", "efgh", "i"]);
        assert_eq!((layout.cursor_row, layout.cursor_col), (1, 3));
    }

    #[test]
    fn handles_polish_and_wide_characters() {
        let layout = input_layout("ą界b", "ą界".len(), 4);
        assert_eq!((layout.cursor_row, layout.cursor_col), (0, 3));
        assert_eq!(UnicodeWidthStr::width(layout.lines[0].as_str()), 4);
    }

    #[test]
    fn rendered_cursor_is_after_prompt_and_text() {
        let backend = TestBackend::new(30, 6);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render(
                    frame,
                    Rect::new(0, 0, 30, 6),
                    &super::FooterViewModel {
                        input: "hello",
                        cursor: 5,
                        status_line: "ready".to_string(),
                    },
                )
            })
            .unwrap();
        assert_eq!(terminal.backend_mut().get_cursor().unwrap(), (8, 4));
    }
}
