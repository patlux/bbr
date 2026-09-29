//! App-owned command text editing model (DESIGN.md §11.5).
//!
//! UTF-8-safe cursor/selection state: every mutation keeps the cursor and
//! the selected byte range on character boundaries, so a multi-byte
//! character is never split. The command field owns one of these; the
//! key handlers (`Cmd-L` select-all, replace-selection typing, backspace,
//! horizontal cursor movement) all route through this model, and the render
//! derives the caret position from `cursor`. Blueprint D02: the model also
//! carries the IME composition (`marked`) and the platform text-input
//! protocol's UTF-16 view of all three ranges (`command_text_ime`).

/// The command field's text plus its cursor/selection, in byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandText {
    pub text: String,
    /// Byte offset of the block cursor (always a UTF-8 char boundary).
    pub cursor: usize,
    /// Selected byte range `[start, end)` when non-empty; `None` when the
    /// field has no selection.
    pub selection: Option<(usize, usize)>,
    /// The IME composition (marked text) byte range while an input method
    /// is composing; every non-IME mutation commits it as plain text.
    pub marked: Option<(usize, usize)>,
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
            marked: None,
        }
    }

    /// The presentation view of the caret, selection and composition.
    pub fn edit_state(&self) -> bbr_ui::components::EditState {
        bbr_ui::components::EditState {
            cursor: self.cursor,
            selection: self.selection,
            marked: self.marked,
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
        self.marked = None;
    }

    /// Select the whole text, leaving the cursor at the end (`Cmd-L`).
    pub fn select_all(&mut self) {
        let len = self.text.len();
        self.selection = Some((0, len));
        self.cursor = len;
        self.marked = None;
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
        self.marked = None;
    }

    /// Backspace: delete the selection when non-empty, otherwise delete the
    /// character before the cursor. Never crosses a UTF-8 boundary.
    pub fn backspace(&mut self) {
        self.marked = None;
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

    /// Move the cursor by `motion`. Without `extend` a non-empty selection
    /// collapses in the direction of travel (a character step stops at the
    /// selection edge); with `extend` the selection grows or shrinks from
    /// its anchor — the edge the cursor is not on.
    pub fn move_to(&mut self, motion: Motion, extend: bool) {
        self.marked = None;
        let (start, end) = self.selected_range();
        let selected = start != end;
        if extend {
            // The anchor is the selection edge the cursor is not on.
            let anchor = match (selected, self.cursor == start) {
                (true, true) => end,
                (true, false) => start,
                (false, _) => self.cursor,
            };
            let target = motion.target(&self.text, self.cursor);
            self.cursor = target;
            self.selection = (anchor != target).then(|| (anchor.min(target), anchor.max(target)));
            return;
        }
        self.selection = None;
        self.cursor = match (selected, motion) {
            // A character step collapses a selection to the edge it moves
            // toward (the selection edges are already char boundaries).
            (true, Motion::CharLeft) => start,
            (true, Motion::CharRight) => end,
            (true, motion) if motion.is_leftward() => motion.target(&self.text, start),
            (true, motion) => motion.target(&self.text, end),
            (false, motion) => motion.target(&self.text, self.cursor),
        };
    }

    /// True when the field is empty.
    ///
    /// Consumed by the test suite only since the G1a cutover moved the
    /// legacy inline field render (the only production caller) into
    /// `bbr_ui::components::address`; gated so the bin build stays clean.
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// A caret motion (§11.5 field keys): one character, one word (⌥), or to
/// the line's edge (⌘ / Home / End).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    CharLeft,
    CharRight,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
}

impl Motion {
    fn is_leftward(self) -> bool {
        matches!(
            self,
            Motion::CharLeft | Motion::WordLeft | Motion::LineStart
        )
    }

    fn target(self, text: &str, from: usize) -> usize {
        match self {
            Motion::CharLeft => previous_char_boundary(text, from),
            Motion::CharRight => next_char_boundary(text, from),
            Motion::WordLeft => previous_word_start(text, from),
            Motion::WordRight => next_word_end(text, from),
            Motion::LineStart => 0,
            Motion::LineEnd => text.len(),
        }
    }
}

/// The start of the word before `pos` (a word is a run of alphanumerics,
/// so `⌥←` stops at URL punctuation like a native text field).
fn previous_word_start(text: &str, pos: usize) -> usize {
    let before: Vec<(usize, char)> = text[..pos.min(text.len())].char_indices().collect();
    let mut index = before.len();
    while index > 0 && !before[index - 1].1.is_alphanumeric() {
        index -= 1;
    }
    while index > 0 && before[index - 1].1.is_alphanumeric() {
        index -= 1;
    }
    before.get(index).map_or(0, |(offset, _)| *offset)
}

/// The end of the word after `pos`.
fn next_word_end(text: &str, pos: usize) -> usize {
    let start = pos.min(text.len());
    let mut chars = text[start..].char_indices().peekable();
    while chars.next_if(|(_, ch)| !ch.is_alphanumeric()).is_some() {}
    while chars.next_if(|(_, ch)| ch.is_alphanumeric()).is_some() {}
    chars
        .peek()
        .map_or(text.len(), |(offset, _)| start + offset)
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
        model.move_to(Motion::CharLeft, false);
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
        model.move_to(Motion::CharLeft, false);
        assert_eq!(model.cursor, 0);
        model.move_to(Motion::CharRight, false);
        assert_eq!(model.cursor, 1, "cursor stays on the char boundary");
        model.move_to(Motion::CharRight, false);
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
        model.move_to(Motion::CharLeft, false);
        assert_eq!(model.cursor, 1, "left collapses to the selection start");
        assert_eq!(model.selection, None);
        model.selection = Some((1, 3));
        model.move_to(Motion::CharRight, false);
        assert_eq!(model.cursor, 3, "right collapses to the selection end");
        assert_eq!(model.selection, None);
    }

    #[test]
    fn empty_boundaries_are_stable() {
        let mut model = CommandText::new();
        assert!(model.is_empty());
        model.backspace();
        model.move_to(Motion::CharLeft, false);
        model.move_to(Motion::CharRight, false);
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
