//! Blueprint D02 prerequisite: the platform text-input protocol's view of
//! the command field (`NSTextInputClient` through GPUI's input handler).
//!
//! The protocol speaks UTF-16 code-unit ranges; the model stores UTF-8 byte
//! offsets. Every conversion clamps to a char boundary, so an input method
//! can never split a character, and a composition (marked text) is always a
//! whole-character range that the next commit replaces.

use std::ops::Range;

use super::command_text::CommandText;

impl CommandText {
    /// The byte offset of UTF-16 offset `utf16` (clamped to the text; a
    /// position inside a surrogate pair lands on that character's start).
    pub fn offset_from_utf16(&self, utf16: usize) -> usize {
        let mut units = 0;
        for (offset, ch) in self.text.char_indices() {
            if units + ch.len_utf16() > utf16 {
                return offset;
            }
            units += ch.len_utf16();
        }
        self.text.len()
    }

    /// The UTF-16 offset of byte offset `offset`.
    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        self.text[..offset.min(self.text.len())]
            .chars()
            .map(char::len_utf16)
            .sum()
    }

    /// A UTF-16 range as an ordered byte range.
    pub fn range_from_utf16(&self, range: &Range<usize>) -> (usize, usize) {
        let start = self.offset_from_utf16(range.start);
        let end = self.offset_from_utf16(range.end);
        (start.min(end), start.max(end))
    }

    /// A byte range as a UTF-16 range.
    pub fn range_to_utf16(&self, (start, end): (usize, usize)) -> Range<usize> {
        self.offset_to_utf16(start)..self.offset_to_utf16(end)
    }

    /// The selection (or the caret as an empty range) in UTF-16, and whether
    /// the caret sits at the selection's start.
    pub fn selected_utf16(&self) -> (Range<usize>, bool) {
        let (start, end) = self.selected_range();
        (
            self.range_to_utf16((start, end)),
            start != end && self.cursor == start,
        )
    }

    /// The composition in UTF-16.
    pub fn marked_utf16(&self) -> Option<Range<usize>> {
        self.marked.map(|range| self.range_to_utf16(range))
    }

    /// The range an input-method edit replaces: the explicit protocol range,
    /// else the composition, else the selection (or the caret).
    fn edit_range(&self, range: Option<&Range<usize>>) -> (usize, usize) {
        match range {
            Some(range) => self.range_from_utf16(range),
            None => self.marked.unwrap_or_else(|| self.selected_range()),
        }
    }

    /// `insertText:` — commit `text` over the edit range; the composition
    /// ends and the caret follows the inserted text.
    pub fn commit_text(&mut self, range: Option<&Range<usize>>, text: &str) {
        let (start, end) = self.edit_range(range);
        self.text.replace_range(start..end, text);
        self.cursor = start + text.len();
        self.selection = None;
        self.marked = None;
    }

    /// `setMarkedText:` — replace the edit range with the composition
    /// `text`; `selected` (UTF-16, relative to `text`) places the caret or
    /// the input method's highlighted segment inside it. An empty `text`
    /// cancels the composition.
    pub fn compose_text(
        &mut self,
        range: Option<&Range<usize>>,
        text: &str,
        selected: Option<&Range<usize>>,
    ) {
        let (start, end) = self.edit_range(range);
        self.text.replace_range(start..end, text);
        self.marked = (!text.is_empty()).then_some((start, start + text.len()));
        let local = |utf16: usize| {
            let mut units = 0;
            for (offset, ch) in text.char_indices() {
                if units + ch.len_utf16() > utf16 {
                    return start + offset;
                }
                units += ch.len_utf16();
            }
            start + text.len()
        };
        match selected {
            Some(selected) if selected.start != selected.end => {
                let (a, b) = (local(selected.start), local(selected.end));
                self.selection = Some((a.min(b), a.max(b)));
                self.cursor = a.max(b);
            }
            Some(selected) => {
                self.selection = None;
                self.cursor = local(selected.start);
            }
            None => {
                self.selection = None;
                self.cursor = start + text.len();
            }
        }
    }

    /// `unmarkText` — keep the composed text as typed text.
    pub fn unmark(&mut self) {
        self.marked = None;
    }
}

#[cfg(test)]
mod tests {
    use super::super::command_text::Motion;
    use super::*;

    fn field(text: &str) -> CommandText {
        let mut model = CommandText::new();
        model.set_text(text);
        model
    }

    #[test]
    fn utf16_offsets_round_trip_over_multi_unit_characters() {
        // "a" (1 unit, 1 byte), "日" (1 unit, 3 bytes), "😀" (2 units,
        // 4 bytes), "b".
        let model = field("a日😀b");
        assert_eq!(model.offset_to_utf16(0), 0);
        assert_eq!(model.offset_to_utf16(1), 1);
        assert_eq!(model.offset_to_utf16(4), 2);
        assert_eq!(model.offset_to_utf16(8), 4);
        assert_eq!(model.offset_to_utf16(9), 5);
        assert_eq!(model.offset_from_utf16(2), 4);
        // Inside the surrogate pair: the emoji's start, never a split.
        assert_eq!(model.offset_from_utf16(3), 4);
        assert_eq!(model.offset_from_utf16(4), 8);
        assert_eq!(model.offset_from_utf16(99), 9);
        assert_eq!(model.range_to_utf16((1, 8)), 1..4);
        assert_eq!(model.range_from_utf16(&(1..4)), (1, 8));
        // A reversed protocol range is ordered, never inverted.
        let reversed = Range { start: 4, end: 1 };
        assert_eq!(model.range_from_utf16(&reversed), (1, 8));
    }

    #[test]
    fn a_composition_is_marked_then_committed_over_itself() {
        let mut model = field("ab");
        // The input method composes "にほ" at the caret, caret at its end.
        model.compose_text(None, "にほ", Some(&(2..2)));
        assert_eq!(model.text, "abにほ");
        assert_eq!(model.marked, Some((2, 8)));
        assert_eq!(model.marked_utf16(), Some(2..4));
        assert_eq!(model.cursor, 8);
        // It refines the composition: the marked range is replaced.
        model.compose_text(None, "にほん", Some(&(3..3)));
        assert_eq!(model.text, "abにほん");
        assert_eq!(model.marked_utf16(), Some(2..5));
        // Commit: the conversion replaces the composition and ends it.
        model.commit_text(None, "日本");
        assert_eq!(model.text, "ab日本");
        assert_eq!(model.marked, None);
        assert_eq!(model.cursor, 8);
        assert_eq!(model.selected_utf16(), (4..4, false));
    }

    #[test]
    fn a_composition_replaces_the_selection_and_can_be_cancelled() {
        let mut model = field("https://example.com");
        model.select_all();
        model.compose_text(None, "ka", None);
        assert_eq!(model.text, "ka");
        assert_eq!(model.marked, Some((0, 2)));
        // An empty composition cancels it (the input method's Escape).
        model.compose_text(None, "", None);
        assert_eq!(model.text, "");
        assert_eq!(model.marked, None);
    }

    #[test]
    fn the_segment_the_input_method_highlights_is_the_selection() {
        let mut model = field("");
        model.compose_text(None, "にほんご", Some(&(2..4)));
        assert_eq!(model.selection, Some((6, 12)));
        assert_eq!(model.selected_utf16(), (2..4, false));
        model.unmark();
        assert_eq!(model.marked, None);
        assert_eq!(model.text, "にほんご");
    }

    #[test]
    fn an_explicit_replacement_range_wins() {
        let mut model = field("abcd");
        model.commit_text(Some(&(1..3)), "X");
        assert_eq!(model.text, "aXd");
        assert_eq!(model.cursor, 2);
    }

    #[test]
    fn any_caret_motion_commits_the_composition_as_text() {
        let mut model = field("");
        model.compose_text(None, "に", None);
        model.move_to(Motion::CharLeft, false);
        assert_eq!(model.marked, None);
        assert_eq!(model.text, "に");
    }

    #[test]
    fn shift_motions_grow_and_shrink_the_selection_from_its_anchor() {
        let mut model = field("abcd");
        model.move_to(Motion::CharLeft, true);
        model.move_to(Motion::CharLeft, true);
        assert_eq!(model.selection, Some((2, 4)));
        assert_eq!(model.cursor, 2);
        assert_eq!(model.selected_utf16(), (2..4, true));
        model.move_to(Motion::CharRight, true);
        assert_eq!(model.selection, Some((3, 4)));
        model.move_to(Motion::CharRight, true);
        assert_eq!(model.selection, None, "back at the anchor");
        model.move_to(Motion::LineStart, true);
        assert_eq!(model.selection, Some((0, 4)));
        // A plain line motion collapses and jumps.
        model.move_to(Motion::LineEnd, false);
        assert_eq!((model.cursor, model.selection), (4, None));
    }

    #[test]
    fn word_motions_stop_at_url_punctuation() {
        let mut model = field("https://example.com/path");
        model.move_to(Motion::WordLeft, false);
        assert_eq!(model.cursor, "https://example.com/".len());
        model.move_to(Motion::WordLeft, false);
        assert_eq!(model.cursor, "https://example.".len());
        model.move_to(Motion::WordRight, false);
        assert_eq!(model.cursor, "https://example.com".len());
        model.move_to(Motion::LineStart, false);
        model.move_to(Motion::WordRight, false);
        assert_eq!(model.cursor, "https".len());
    }
}
