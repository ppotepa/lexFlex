use crate::chat::session::{ChatSession, OverlayKind};
use crate::chat::widgets::activity_panel;
use crate::chat::widgets::chat_window::{self, ChatWindowViewModel};
use crate::chat::widgets::command_popup::{self, CommandPopupViewModel};
use crate::chat::widgets::footer::{self, FooterViewModel};
use crate::chat::widgets::header::{self, HeaderViewModel};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn draw(frame: &mut ratatui::Frame, session: &ChatSession) {
    if frame.size().width < 24 || frame.size().height < 16 {
        let message = ratatui::widgets::Paragraph::new("Terminal too small. Resize to at least 24x16.")
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        frame.render_widget(message, frame.size());
        return;
    }
    let layout = layout_for(frame.size());

    header::render(
        frame,
        layout.header,
        &HeaderViewModel {
            title: "lexFlex chat".to_string(),
            mode: session.context.mode.as_str().to_string(),
            chat_language: session.chat_language_label(),
            source_lang: session.context.source_lang.clone(),
            target_lang: session.context.target_lang.clone(),
            verbosity: session.settings.verbosity,
        },
    );
    activity_panel::render(frame, layout.activity, &session.activity);
    chat_window::render(
        frame,
        layout.chat,
        &ChatWindowViewModel {
            messages: &session.transcript.messages,
            scroll: session.viewport.scroll_offset,
            selected_turn: session.selected_turn,
            expanded_nodes: &session.expanded_nodes,
            focused: session.overlays.focus.is_transcript(),
        },
    );
    footer::render(
        frame,
        layout.footer,
        &FooterViewModel {
            input: &session.composer.input,
            cursor: session.composer.cursor,
            status_line: session.status_line(),
            focused: session.overlays.focus.is_composer(),
        },
    );

    match session.overlays.active {
        Some(OverlayKind::CommandPalette) => {
            let area = command_popup_rect(layout.chat, layout.footer, session.overlays.command_popup.suggestions.len() as u16);
            command_popup::render(
                frame,
                area,
                &CommandPopupViewModel {
                    suggestions: &session.overlays.command_popup.suggestions,
                    selected_index: session.overlays.command_popup.selected_index,
                    focused: session.overlays.focus.is_popup(),
                },
            );
        }
        None => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatLayout {
    pub header: Rect,
    pub activity: Rect,
    pub chat: Rect,
    pub footer: Rect,
}

pub fn layout_for(area: Rect) -> ChatLayout {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Min(1),
            Constraint::Length(6),
        ])
        .split(area);
    ChatLayout {
        header: regions[0],
        activity: regions[1],
        chat: regions[2],
        footer: regions[3],
    }
}

pub fn command_popup_rect(chat: Rect, footer: Rect, suggestions_len: u16) -> Rect {
    let height = suggestions_len.saturating_add(2).clamp(3, 6);
    let width = footer.width.saturating_sub(2).min(72);
    Rect {
        x: footer.x.saturating_add(1),
        y: chat.y.saturating_add(chat.height.saturating_sub(height.saturating_add(1))),
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::{command_popup_rect, draw, layout_for};
    use crate::chat::{ChatOptions, ChatSession, TraceMode};
    use crate::chat::session::FocusTarget;
    use crate::engine::SourceFetchPolicy;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    #[test]
    fn layout_uses_fixed_regions() {
        let layout = layout_for(Rect::new(0, 0, 120, 30));
        assert_eq!(layout.header.height, 4);
        assert_eq!(layout.activity.height, 5);
        assert_eq!(layout.footer.height, 6);
        assert_eq!(layout.activity.y, 4);
        assert_eq!(layout.chat.y, 9);
        assert_eq!(layout.footer.y, 24);
    }

    #[test]
    fn command_popup_stays_above_footer() {
        let layout = layout_for(Rect::new(0, 0, 120, 30));
        let popup = command_popup_rect(layout.chat, layout.footer, 3);
        assert!(popup.y + popup.height <= layout.footer.y + 1);
        assert!(popup.y >= layout.chat.y);
    }

    #[test]
    fn standard_layout_has_non_overlapping_regions() {
        let layout = layout_for(Rect::new(0, 0, 80, 24));
        assert_eq!(layout.header.bottom(), layout.activity.y);
        assert_eq!(layout.activity.bottom(), layout.chat.y);
        assert_eq!(layout.chat.bottom(), layout.footer.y);
        assert_eq!(layout.footer.bottom(), 24);
    }

    #[test]
    fn focused_widget_has_white_bold_border_and_inactive_widget_is_dark_gray() {
        let mut session = ChatSession::new(ChatOptions {
            from: "pl".into(),
            to: "en".into(),
            data_dir: "data".into(),
            offline: true,
            trace_mode: TraceMode::Full,
            source_policy: SourceFetchPolicy::SnapshotOnly,
        });
        session.overlays.focus = FocusTarget::Composer;
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &session)).unwrap();
        let layout = layout_for(Rect::new(0, 0, 80, 24));
        assert_eq!(terminal.backend().buffer().get(layout.footer.x, layout.footer.y + 3).fg, ratatui::style::Color::White);
        assert_eq!(terminal.backend().buffer().get(layout.chat.x, layout.chat.y).fg, ratatui::style::Color::DarkGray);

        session.overlays.focus = FocusTarget::Transcript;
        terminal.draw(|frame| draw(frame, &session)).unwrap();
        assert_eq!(terminal.backend().buffer().get(layout.chat.x, layout.chat.y).fg, ratatui::style::Color::White);
        assert_eq!(terminal.backend().buffer().get(layout.footer.x, layout.footer.y + 3).fg, ratatui::style::Color::DarkGray);
    }
}
