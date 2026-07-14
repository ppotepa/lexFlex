use crate::chat::session::ComposerState;

pub fn insert_char(composer: &mut ComposerState, ch: char) {
    composer.input.insert(composer.cursor, ch);
    composer.cursor += ch.len_utf8();
}

pub fn insert_newline(composer: &mut ComposerState) {
    insert_char(composer, '\n');
}

pub fn insert_text(composer: &mut ComposerState, text: &str) {
    composer.input.insert_str(composer.cursor, text);
    composer.cursor += text.len();
}

pub fn backspace(composer: &mut ComposerState) {
    if composer.cursor == 0 {
        return;
    }
    let prev = prev_char_boundary(&composer.input, composer.cursor);
    composer.input.drain(prev..composer.cursor);
    composer.cursor = prev;
}

pub fn delete_forward(composer: &mut ComposerState) {
    if composer.cursor >= composer.input.len() {
        return;
    }
    let next = next_char_boundary(&composer.input, composer.cursor);
    composer.input.drain(composer.cursor..next);
}

pub fn move_left(composer: &mut ComposerState) {
    composer.cursor = prev_char_boundary(&composer.input, composer.cursor);
}

pub fn move_right(composer: &mut ComposerState) {
    composer.cursor = next_char_boundary(&composer.input, composer.cursor);
}

pub fn move_up(composer: &mut ComposerState) {
    let (line_start, _) = line_bounds(&composer.input, composer.cursor);
    if line_start == 0 {
        return;
    }
    let column = composer.input[line_start..composer.cursor].chars().count();
    let previous_end = line_start - 1;
    let previous_start = composer.input[..previous_end]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    composer.cursor = char_boundary_at(&composer.input, previous_start, column);
}

pub fn move_down(composer: &mut ComposerState) {
    let (_, line_end) = line_bounds(&composer.input, composer.cursor);
    if line_end >= composer.input.len() {
        return;
    }
    let (line_start, _) = line_bounds(&composer.input, composer.cursor);
    let column = composer.input[line_start..composer.cursor].chars().count();
    let next_start = line_end + 1;
    composer.cursor = char_boundary_at(&composer.input, next_start, column);
}

pub fn recall_previous_input(composer: &mut ComposerState) {
    if composer.history.is_empty() {
        return;
    }
    match composer.history_cursor {
        None => {
            composer.history_draft = Some(composer.input.clone());
            composer.history_cursor = Some(composer.history.len().saturating_sub(1));
        }
        Some(0) => {}
        Some(idx) => composer.history_cursor = Some(idx.saturating_sub(1)),
    }
    if let Some(idx) = composer.history_cursor {
        if let Some(item) = composer.history.get(idx) {
            composer.input = item.clone();
            composer.cursor = composer.input.len();
        }
    }
}

pub fn recall_next_input(composer: &mut ComposerState) {
    match composer.history_cursor {
        None => {}
        Some(idx) if idx + 1 >= composer.history.len() => {
            composer.history_cursor = None;
            composer.input = composer.history_draft.take().unwrap_or_default();
            composer.cursor = composer.input.len();
        }
        Some(idx) => {
            composer.history_cursor = Some(idx + 1);
            if let Some(item) = composer.history.get(idx + 1) {
                composer.input = item.clone();
                composer.cursor = composer.input.len();
            }
        }
    }
}

fn prev_char_boundary(s: &str, idx: usize) -> usize {
    if idx == 0 {
        return 0;
    }
    s[..idx]
        .char_indices()
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn next_char_boundary(s: &str, idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    s[idx..]
        .chars()
        .next()
        .map(|ch| idx + ch.len_utf8())
        .unwrap_or(s.len())
}

fn line_bounds(s: &str, cursor: usize) -> (usize, usize) {
    let start = s[..cursor]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let end = s[cursor..]
        .find('\n')
        .map(|index| cursor + index)
        .unwrap_or(s.len());
    (start, end)
}

fn char_boundary_at(s: &str, start: usize, column: usize) -> usize {
    s[start..]
        .char_indices()
        .nth(column)
        .map(|(index, _)| start + index)
        .unwrap_or_else(|| {
            s[start..]
                .find('\n')
                .map(|index| start + index)
                .unwrap_or(s.len())
        })
}

#[cfg(test)]
mod tests {
    use super::{move_down, move_up, recall_next_input, recall_previous_input};
    use crate::chat::session::ComposerState;

    #[test]
    fn history_restores_draft_after_navigation() {
        let mut composer = ComposerState {
            input: "draft".to_string(),
            history: vec!["first".to_string(), "second".to_string()],
            ..ComposerState::default()
        };
        recall_previous_input(&mut composer);
        assert_eq!(composer.input, "second");
        recall_previous_input(&mut composer);
        assert_eq!(composer.input, "first");
        recall_next_input(&mut composer);
        assert_eq!(composer.input, "second");
        recall_next_input(&mut composer);
        assert_eq!(composer.input, "draft");
    }

    #[test]
    fn vertical_movement_stays_inside_multiline_input() {
        let mut composer = ComposerState {
            input: "one\ntwo\nthree".to_string(),
            cursor: "one\ntw".len(),
            ..ComposerState::default()
        };
        move_up(&mut composer);
        assert_eq!(&composer.input[..composer.cursor], "on");
        move_down(&mut composer);
        assert_eq!(&composer.input[..composer.cursor], "one\ntw");
    }
}
