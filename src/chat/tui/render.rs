use crate::chat::session::{ChatSession, OverlayKind};
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
    let root = main_layout(frame.size());

    header::render(
        frame,
        root[0],
        &HeaderViewModel {
            title: "lexFlex chat".to_string(),
            mode: session.context.mode.as_str().to_string(),
            chat_language: session.chat_language_label(),
            source_lang: session.context.source_lang.clone(),
            target_lang: session.context.target_lang.clone(),
            verbosity: session.settings.verbosity,
            pending_label: session.pending.as_ref().map(|item| item.label.clone()),
            pending_stage: session.pending.as_ref().map(|item| item.current_stage.clone()),
            pending_spinner: session.pending.as_ref().map(|item| item.spinner_index).unwrap_or(0),
            backend_ready: session.backend_ready,
        },
    );
    chat_window::render(
        frame,
        root[1],
        &ChatWindowViewModel {
            messages: &session.transcript.messages,
            scroll: session.viewport.scroll_offset,
        },
    );
    footer::render(
        frame,
        root[2],
        &FooterViewModel {
            input: &session.composer.input,
            cursor: session.composer.cursor,
            status_line: session.status_line(),
        },
    );

    match session.overlays.active {
        Some(OverlayKind::CommandPalette) => {
            let area = command_popup_rect(root[1], root[2], session.overlays.command_popup.suggestions.len() as u16);
            command_popup::render(
                frame,
                area,
                &CommandPopupViewModel {
                    suggestions: &session.overlays.command_popup.suggestions,
                    selected_index: session.overlays.command_popup.selected_index,
                },
            );
        }
        None => {}
    }
}

pub fn main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(6),
            Constraint::Length(6),
        ])
        .split(area)
        .to_vec()
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
    use super::{command_popup_rect, main_layout};
    use ratatui::layout::Rect;

    #[test]
    fn main_layout_uses_fixed_header_and_footer() {
        let parts = main_layout(Rect::new(0, 0, 120, 30));
        assert_eq!(parts[0].height, 4);
        assert_eq!(parts[2].height, 6);
        assert_eq!(parts[1].y, 4);
        assert_eq!(parts[2].y, 24);
    }

    #[test]
    fn command_popup_stays_above_footer() {
        let parts = main_layout(Rect::new(0, 0, 120, 30));
        let popup = command_popup_rect(parts[1], parts[2], 3);
        assert!(popup.y + popup.height <= parts[2].y + 1);
        assert!(popup.y >= parts[1].y);
    }

    #[test]
    fn standard_layout_has_non_overlapping_regions() {
        let parts = main_layout(Rect::new(0, 0, 80, 24));
        assert_eq!(parts[0].bottom(), parts[1].y);
        assert_eq!(parts[1].bottom(), parts[2].y);
        assert_eq!(parts[2].bottom(), 24);
    }
}
