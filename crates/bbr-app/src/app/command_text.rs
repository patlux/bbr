//! App-owned command text editing model (DESIGN.md §11.5).
//!
//! UTF-8-safe cursor/selection state: every mutation keeps the cursor and
//! the selected byte range on character boundaries, so a multi-byte
//! character is never split. The command field owns one of these; the
//! key handlers (`Cmd-L` select-all, replace-selection typing, backspace,
//! horizontal cursor movement) all route through this model, and the render
//! derives the block cursor position from `cursor`.

/// The command field's text plus its cursor/selection, in byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandText {
    pub text: String,
    /// Byte offset of the block cursor (always a UTF-8 char boundary).
    pub cursor: usize,
    /// Selected byte range `[start, end)` when non-empty; `None` when the
    /// field has no selection.
    pub selection: Option<(usize, usize)>,
}

impl Default for CommandText {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandText {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection: None,
        }
    }

    /// Replace the whole text, placing the cursor at the end with no
    /// selection (used when syncing the live URL and restoring origins).
    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        let cursor = text.len();
        self.text = text;
        self.cursor = cursor;
        self.selection = None;
    }

    /// Select the whole text, leaving the cursor at the end (`Cmd-L`).
    pub fn select_all(&mut self) {
        let len = self.text.len();
        self.selection = Some((0, len));
        self.cursor = len;
    }

    /// The selected byte range, or the cursor position as an empty range.
    pub fn selected_range(&self) -> (usize, usize) {
        self.selection.unwrap_or((self.cursor, self.cursor))
    }

    /// Replace the selection (or insert at the cursor) with `input`,
    /// UTF-8-safe: the replacement lands exactly on the selected byte
    /// range and the cursor moves to the end of the inserted text.
    pub fn replace_selection(&mut self, input: &str) {
        let (start, end) = self.selected_range();
        self.text.replace_range(start..end, input);
        let cursor = start + input.len();
        self.cursor = cursor;
        self.selection = None;
    }

    /// Backspace: delete the selection when non-empty, otherwise delete the
    /// character before the cursor. Never crosses a UTF-8 boundary.
    pub fn backspace(&mut self) {
        let (start, end) = self.selected_range();
        if start != end {
            self.text.replace_range(start..end, "");
            self.cursor = start;
            self.selection = None;
            return;
        }
        let previous = previous_char_boundary(&self.text, self.cursor);
        if previous != self.cursor {
            self.text.replace_range(previous..self.cursor, "");
            self.cursor = previous;
        }
    }

    /// Move the cursor one character left, collapsing any selection to its
    /// start. Stays on a UTF-8 char boundary; no-op at the start.
    pub fn move_left(&mut self) {
        let (start, end) = self.selected_range();
        self.selection = None;
        // A non-empty selection collapses to its start (already a char
        // boundary); without a selection the cursor moves one char left.
        self.cursor = if start != end {
            start
        } else {
            previous_char_boundary(&self.text, start)
        };
    }

    /// Move the cursor one character right, collapsing any selection to its
    /// end. Stays on a UTF-8 char boundary; no-op at the end.
    pub fn move_right(&mut self) {
        let (start, end) = self.selected_range();
        self.selection = None;
        // A non-empty selection collapses to its end (already a char
        // boundary); without a selection the cursor moves one char right.
        self.cursor = if start != end {
            end
        } else {
            next_char_boundary(&self.text, end)
        };
    }

    /// True when the field is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// The char-boundary index before `pos` (`0` at the start; `pos` unchanged
/// when `pos` is already at the start).
fn previous_char_boundary(text: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }
    let mut boundary = 0;
    for (index, _) in text.char_indices() {
        if index >= pos {
            break;
        }
        boundary = index;
    }
    boundary
}

/// The char-boundary index after `pos` (`len` at the end; `pos` unchanged
/// when `pos` is already at the end). A cursor inside a multi-byte
/// character (unreachable through the model's own mutations) moves to the
/// next boundary instead of jumping to the end.
fn next_char_boundary(text: &str, pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }
    for (index, ch) in text.char_indices() {
        if index == pos {
            return pos + ch.len_utf8();
        }
        if index > pos {
            return index;
        }
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_all_selects_the_whole_text() {
        let mut model = CommandText::new();
        model.set_text("https://example.com");
        model.select_all();
        assert_eq!(model.selection, Some((0, "https://example.com".len())));
        assert_eq!(model.cursor, "https://example.com".len());
    }

    #[test]
    fn typing_replaces_the_selection() {
        let mut model = CommandText::new();
        model.set_text("https://example.com");
        model.select_all();
        model.replace_selection("x");
        assert_eq!(model.text, "x");
        assert_eq!(model.cursor, 1);
        assert_eq!(model.selection, None);
    }

    #[test]
    fn typing_inserts_at_the_cursor_without_a_selection() {
        let mut model = CommandText::new();
        model.set_text("ab");
        model.move_left();
        model.replace_selection("X");
        assert_eq!(model.text, "aXb");
        assert_eq!(model.cursor, 2);
    }

    #[test]
    fn backspace_deletes_the_selection_first() {
        let mut model = CommandText::new();
        model.set_text("hello");
        model.selection = Some((1, 3));
        model.backspace();
        assert_eq!(model.text, "hlo");
        assert_eq!(model.cursor, 1);
        assert_eq!(model.selection, None);
    }

    #[test]
    fn backspace_deletes_the_char_before_the_cursor() {
        let mut model = CommandText::new();
        model.set_text("hello");
        model.cursor = 5;
        model.backspace();
        assert_eq!(model.text, "hell");
        assert_eq!(model.cursor, 4);
    }

    #[test]
    fn backspace_is_a_noop_at_the_start() {
        let mut model = CommandText::new();
        model.set_text("hello");
        model.cursor = 0;
        model.backspace();
        assert_eq!(model.text, "hello");
        assert_eq!(model.cursor, 0);
    }

    #[test]
    fn cursor_movement_is_utf8_safe_and_never_splits_characters() {
        // "h", a 4-byte emoji, and "x": byte offsets 0..1, 1..5, 5..6.
        let mut model = CommandText::new();
        model.set_text("h\u{1F600}x");
        model.cursor = 1; // start of the emoji
        model.move_left();
        assert_eq!(model.cursor, 0);
        model.move_right();
        assert_eq!(model.cursor, 1, "cursor stays on the char boundary");
        model.move_right();
        assert_eq!(model.cursor, 5, "the emoji is one step");
        model.backspace();
        assert_eq!(model.text, "hx");
        assert_eq!(model.cursor, 1);
    }

    #[test]
    fn backspace_with_multibyte_previous_char_removes_the_whole_char() {
        let mut model = CommandText::new();
        model.set_text("h\u{1F600}");
        model.cursor = 5;
        model.backspace();
        assert_eq!(model.text, "h");
        assert_eq!(model.cursor, 1);
    }

    #[test]
    fn selection_collapses_in_the_direction_of_movement() {
        let mut model = CommandText::new();
        model.set_text("abcd");
        model.selection = Some((1, 3));
        model.move_left();
        assert_eq!(model.cursor, 1, "left collapses to the selection start");
        assert_eq!(model.selection, None);
        model.selection = Some((1, 3));
        model.move_right();
        assert_eq!(model.cursor, 3, "right collapses to the selection end");
        assert_eq!(model.selection, None);
    }

    #[test]
    fn empty_boundaries_are_stable() {
        let mut model = CommandText::new();
        assert!(model.is_empty());
        model.backspace();
        model.move_left();
        model.move_right();
        assert_eq!(model.text, "");
        assert_eq!(model.cursor, 0);
        model.replace_selection("x");
        assert_eq!(model.text, "x");
    }

    #[test]
    fn set_text_resets_cursor_and_selection() {
        let mut model = CommandText::new();
        model.set_text("abc");
        model.select_all();
        model.set_text("de");
        assert_eq!(model.text, "de");
        assert_eq!(model.cursor, 2);
        assert_eq!(model.selection, None);
    }
}
