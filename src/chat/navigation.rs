use crate::chat::session::ChatSession;

pub fn page_up(session: &mut ChatSession, amount: u16, max_scroll: u16) {
    session.viewport.page_up(amount, max_scroll);
}

pub fn page_down(session: &mut ChatSession, amount: u16, max_scroll: u16) {
    session.viewport.page_down(amount, max_scroll);
}

pub fn jump_to_bottom(session: &mut ChatSession) {
    session.viewport.jump_to_bottom();
}
