//! Blueprint D02 prerequisite: the command field as a platform text-input
//! client (DESIGN §11.5).
//!
//! GPUI routes the macOS `NSTextInputClient` protocol — input methods
//! (Japanese, Chinese, Korean), dead keys, the accent menu and the
//! character viewer — to the input handler registered during paint. While
//! the command field owns the keyboard the root registers one for the
//! chrome focus handle; its edits go through the same UTF-8-safe
//! [`CommandText`](super::command_text::CommandText) model the key handler
//! uses and restart the ranking debounce exactly like typing. Plain
//! printable keys still arrive as key-down events first (synchronous and
//! deterministic); the platform sends a key to the input method first only
//! while it composes or when an input-method source is active.

use std::ops::Range;

use gpui::{
    canvas, point, prelude::*, px, size, Bounds, Context, ElementInputHandler, EntityInputHandler,
    Pixels, Point, UTF16Selection, Window,
};

use super::BbrApp;

impl BbrApp {
    /// Whether the command field is the keyboard owner: a command mode is
    /// active and no sheet, menu, find bar or other surface that claims the
    /// keyboard first (the precedence `on_chrome_key_down` applies) is open.
    pub(crate) fn command_field_owns_keyboard(&self) -> bool {
        self.command_bar_mode.is_active()
            && !self.auth_sheet_is_presented()
            && !self.space_delete_open()
            && !self.context_menu.state.open
            && !self.nav_history_open()
            && !self.tab_menu.state.open
            && !self.find.state.open
            && !self.space_config.open
            && !self.library.open
    }

    /// The element that registers the command field's input handler for
    /// this frame (nothing while another surface owns the keyboard). The
    /// platform only consults it while the chrome focus handle is focused.
    pub(crate) fn command_input_registration(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        if !self.command_field_owns_keyboard() {
            self.command_line.clear();
            return None;
        }
        let entity = cx.entity();
        let focus = self.chrome_handle.clone();
        Some(
            canvas(
                |_, _, _| (),
                move |bounds, (), window, cx| {
                    window.handle_input(&focus, ElementInputHandler::new(bounds, entity), cx);
                },
            )
            .absolute()
            .size_0()
            .into_any_element(),
        )
    }

    /// The window rect of the byte range `(start, end)` in the field line,
    /// shaped from the current text (an empty range is the caret's
    /// zero-width rect). `None` until the line has painted once.
    pub(crate) fn command_range_bounds(
        &self,
        window: &Window,
        (start, end): (usize, usize),
    ) -> Option<[f32; 4]> {
        let layout = self.command_line.layout()?;
        let text = &self.command_text.text;
        let x0 = layout.shaped_x_for(window, text, start);
        let x1 = layout.shaped_x_for(window, text, end);
        Some([x0, layout.bounds[1], (x1 - x0).max(0.0), layout.bounds[3]])
    }

    fn after_input_edit(&mut self, cx: &mut Context<Self>) {
        self.on_command_text_changed(cx);
        cx.notify();
    }
}

impl EntityInputHandler for BbrApp {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        if !self.command_field_owns_keyboard() {
            return None;
        }
        let (start, end) = self.command_text.range_from_utf16(&range);
        *adjusted_range = Some(self.command_text.range_to_utf16((start, end)));
        Some(self.command_text.text[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if !self.command_field_owns_keyboard() {
            return None;
        }
        let (range, reversed) = self.command_text.selected_utf16();
        Some(UTF16Selection { range, reversed })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        if !self.command_field_owns_keyboard() {
            return None;
        }
        self.command_text.marked_utf16()
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.command_field_owns_keyboard() {
            self.command_text.unmark();
            cx.notify();
        }
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.command_field_owns_keyboard() {
            return;
        }
        self.command_text.commit_text(range.as_ref(), text);
        self.after_input_edit(cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.command_field_owns_keyboard() {
            return;
        }
        self.command_text
            .compose_text(range.as_ref(), new_text, new_selected_range.as_ref());
        self.after_input_edit(cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _element_bounds: Bounds<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        if !self.command_field_owns_keyboard() {
            return None;
        }
        let range = self.command_text.range_from_utf16(&range_utf16);
        let [x, y, width, height] = self.command_range_bounds(window, range)?;
        Some(Bounds::new(
            point(px(x), px(y)),
            size(px(width), px(height)),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        if !self.command_field_owns_keyboard() {
            return None;
        }
        let layout = self.command_line.layout()?;
        let index = layout.index_for_x(f32::from(point.x));
        Some(self.command_text.offset_to_utf16(index))
    }

    fn accepts_text_input(&self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        self.command_field_owns_keyboard()
    }
}

#[cfg(feature = "testkit")]
impl BbrApp {
    /// The testkit view of the text-input client (`None` while the field
    /// does not own the keyboard).
    pub(crate) fn command_input_snapshot(
        &self,
        window: &Window,
    ) -> Option<bbr_testkit::protocol::CommandInputSnapshot> {
        if !self.command_field_owns_keyboard() {
            return None;
        }
        let text = &self.command_text;
        let pair = |(start, end): (usize, usize)| {
            let range = text.range_to_utf16((start, end));
            [range.start, range.end]
        };
        let probe = text.marked.unwrap_or_else(|| text.selected_range());
        Some(bbr_testkit::protocol::CommandInputSnapshot {
            text: text.text.clone(),
            cursor: text.offset_to_utf16(text.cursor),
            selection: text.selection.map(pair),
            marked: text.marked.map(pair),
            line: self.command_line.layout().map(|layout| layout.bounds),
            range_rect: self.command_range_bounds(window, probe),
        })
    }
}
