//! Blueprint D02 prerequisite (DESIGN §11.4/§11.5): the one-line editable
//! text of the command field — the query shaped once, the selection washed
//! behind it, the IME composition (marked text) underlined, and the caret at
//! the real insertion point. The line scrolls horizontally so the caret
//! stays visible in a long URL. Presentation only: the app owns the text,
//! the cursor, the selection, the marked range and the platform input
//! handler; this element reports where it laid the text out (through an
//! [`EditLineSink`]) so IME candidate windows and pointer hits map to exact
//! characters.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    canvas, fill, point, prelude::*, px, size, Bounds, ContentMask, Font, Hsla, Pixels, Rgba,
    SharedString, TextAlign, TextRun, UnderlineStyle, Window,
};

/// The editing state drawn over the text (byte offsets on UTF-8 char
/// boundaries).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditState {
    pub cursor: usize,
    /// Selected byte range `[start, end)` when non-empty.
    pub selection: Option<(usize, usize)>,
    /// The IME composition byte range, drawn underlined.
    pub marked: Option<(usize, usize)>,
}

impl EditState {
    /// The caret after `text` with nothing selected or composed.
    pub fn end_of(text: &str) -> Self {
        Self {
            cursor: text.len(),
            selection: None,
            marked: None,
        }
    }
}

/// Where the line was laid out (window points): the line box, the scroll
/// applied to keep the caret visible, and the x of every UTF-8 boundary.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditLineLayout {
    /// `[x, y, width, height]` of the line box.
    pub bounds: [f32; 4],
    pub scroll: f32,
    /// `(byte offset, x relative to the unscrolled text origin)` per char
    /// boundary, ascending (includes the end of the text).
    pub boundaries: Vec<(usize, f32)>,
    /// The font and size the line shapes with.
    pub font: Font,
    pub font_size: Pixels,
}

impl EditLineLayout {
    /// The window x of byte offset `index` (a byte inside a character maps
    /// to that character's start).
    pub fn x_for(&self, index: usize) -> f32 {
        let x = self
            .boundaries
            .iter()
            .take_while(|(offset, _)| *offset <= index)
            .last()
            .map_or(0.0, |(_, x)| *x);
        self.bounds[0] + x - self.scroll
    }

    /// The window x of byte offset `index` in `text` shaped now with the
    /// line's font — current even before the line repaints (an input method
    /// asks for the composition's rect right after it edits the text).
    pub fn shaped_x_for(&self, window: &Window, text: &str, index: usize) -> f32 {
        let run = plain_run(text.len(), &self.font, Rgba::default());
        let shaped = window.text_system().shape_line(
            SharedString::from(text.to_string()),
            self.font_size,
            &[run],
            None,
        );
        let index = (0..=index.min(text.len()))
            .rev()
            .find(|offset| text.is_char_boundary(*offset))
            .unwrap_or(0);
        self.bounds[0] + f32::from(shaped.x_for_index(index)) - self.scroll
    }

    /// The byte offset whose boundary is closest to window x `x`.
    pub fn index_for_x(&self, x: f32) -> usize {
        let local = x - self.bounds[0] + self.scroll;
        self.boundaries
            .iter()
            .min_by(|a, b| (a.1 - local).abs().total_cmp(&(b.1 - local).abs()))
            .map_or(0, |(offset, _)| *offset)
    }
}

/// The app's handle on the last laid-out line (shared with the paint
/// closure; the render thread is the main thread).
#[derive(Clone, Default)]
pub struct EditLineSink(Rc<RefCell<Option<EditLineLayout>>>);

impl EditLineSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// The last reported layout, if the line has painted.
    pub fn layout(&self) -> Option<EditLineLayout> {
        self.0.borrow().clone()
    }

    /// Forget the layout (the field closed).
    pub fn clear(&self) {
        self.0.borrow_mut().take();
    }

    fn store(&self, layout: EditLineLayout) {
        *self.0.borrow_mut() = Some(layout);
    }

    fn scroll(&self) -> f32 {
        self.0.borrow().as_ref().map_or(0.0, |layout| layout.scroll)
    }
}

impl std::fmt::Debug for EditLineSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EditLineSink")
    }
}

/// The horizontal scroll that keeps `caret_x` inside `width`, starting from
/// `current`, with `margin` of room on the caret's side.
pub fn scroll_to_caret(
    current: f32,
    caret_x: f32,
    width: f32,
    text_width: f32,
    margin: f32,
) -> f32 {
    let max_scroll = (text_width + margin - width).max(0.0);
    let mut scroll = current.clamp(0.0, max_scroll);
    if caret_x - scroll > width - margin {
        scroll = caret_x - width + margin;
    }
    if caret_x - scroll < 0.0 {
        scroll = caret_x;
    }
    scroll.clamp(0.0, max_scroll)
}

/// The colours the line paints with.
#[derive(Debug, Clone, Copy)]
pub struct EditLineColors {
    pub text: Rgba,
    pub placeholder: Rgba,
    pub selection: Rgba,
    pub caret: Rgba,
}

/// The editable line, sized by its parent (which also sets the text size).
/// `placeholder` shows while `text` is empty; `show_caret` is false while
/// the field does not own the keyboard.
pub fn edit_line(
    text: &str,
    state: EditState,
    placeholder: SharedString,
    colors: EditLineColors,
    show_caret: bool,
    sink: Option<EditLineSink>,
) -> impl IntoElement {
    let text = text.to_string();
    let prepaint_text = text.clone();
    let prepaint_sink = sink.clone();
    canvas(
        move |bounds, window, _cx| {
            let style = window.text_style();
            let font_size = style.font_size.to_pixels(window.rem_size());
            let font = style.font();
            let empty = prepaint_text.is_empty();
            let (shown, runs) = if empty {
                let run = plain_run(placeholder.len(), &font, colors.placeholder);
                (placeholder, vec![run])
            } else {
                let runs = runs_for(&prepaint_text, state.marked, &font, colors.text);
                (SharedString::from(prepaint_text.clone()), runs)
            };
            let shaped = window
                .text_system()
                .shape_line(shown, font_size, &runs, None);
            let boundaries: Vec<(usize, f32)> = if empty {
                vec![(0, 0.0)]
            } else {
                prepaint_text
                    .char_indices()
                    .map(|(offset, _)| offset)
                    .chain(std::iter::once(prepaint_text.len()))
                    .map(|offset| (offset, f32::from(shaped.x_for_index(offset))))
                    .collect()
            };
            let caret_x = if empty {
                0.0
            } else {
                f32::from(shaped.x_for_index(state.cursor.min(prepaint_text.len())))
            };
            let scroll = if empty {
                0.0
            } else {
                let previous = prepaint_sink.as_ref().map_or(0.0, EditLineSink::scroll);
                scroll_to_caret(
                    previous,
                    caret_x,
                    f32::from(bounds.size.width),
                    f32::from(shaped.width()),
                    4.0,
                )
            };
            (shaped, boundaries, caret_x, scroll, font, font_size)
        },
        move |bounds: Bounds<Pixels>,
              (shaped, boundaries, caret_x, scroll, font, font_size),
              window,
              cx| {
            let height = bounds.size.height;
            let origin = point(bounds.origin.x - px(scroll), bounds.origin.y);
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                let selection = state
                    .selection
                    .filter(|(start, end)| start < end && *end <= text.len());
                if let Some((start, end)) = selection {
                    let x0 = shaped.x_for_index(start);
                    let x1 = shaped.x_for_index(end);
                    window.paint_quad(fill(
                        Bounds::new(point(origin.x + x0, origin.y), size(x1 - x0, height)),
                        colors.selection,
                    ));
                }
                let _ = shaped.paint(origin, height, TextAlign::Left, None, window, cx);
                if show_caret && selection.is_none() {
                    let caret_height = font_size * 1.25;
                    let top = origin.y + (height - caret_height) / 2.0;
                    window.paint_quad(fill(
                        Bounds::new(
                            point(origin.x + px(caret_x), top),
                            size(px(2.0), caret_height),
                        ),
                        colors.caret,
                    ));
                }
            });
            if let Some(sink) = sink.as_ref() {
                sink.store(EditLineLayout {
                    bounds: [
                        f32::from(bounds.origin.x),
                        f32::from(bounds.origin.y),
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                    ],
                    scroll,
                    boundaries,
                    font,
                    font_size,
                });
            }
        },
    )
    .size_full()
}

fn plain_run(len: usize, font: &Font, color: Rgba) -> TextRun {
    TextRun {
        len,
        font: font.clone(),
        color: Hsla::from(color),
        background_color: None,
        underline: None,
        strikethrough: None,
    }
}

/// The text runs: the marked (IME composition) range underlined.
fn runs_for(text: &str, marked: Option<(usize, usize)>, font: &Font, color: Rgba) -> Vec<TextRun> {
    let len = text.len();
    let Some((start, end)) = marked.filter(|(start, end)| start < end && *end <= len) else {
        return vec![plain_run(len, font, color)];
    };
    [(start, false), (end - start, true), (len - end, false)]
        .into_iter()
        .filter(|(run_len, _)| *run_len > 0)
        .map(|(run_len, underline)| TextRun {
            underline: underline.then(|| UnderlineStyle {
                thickness: px(1.5),
                color: Some(Hsla::from(color)),
                wavy: false,
            }),
            ..plain_run(run_len, font, color)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caret_stays_visible_in_a_long_line() {
        // Fits: no scroll.
        assert_eq!(scroll_to_caret(0.0, 50.0, 200.0, 120.0, 4.0), 0.0);
        // Caret past the right edge: scroll so it sits `margin` inside.
        assert_eq!(scroll_to_caret(0.0, 500.0, 200.0, 520.0, 4.0), 304.0);
        // Caret moved left of the scrolled window: scroll back to it.
        assert_eq!(scroll_to_caret(304.0, 100.0, 200.0, 520.0, 4.0), 100.0);
        // Caret still inside the scrolled window: keep the scroll.
        assert_eq!(scroll_to_caret(304.0, 400.0, 200.0, 520.0, 4.0), 304.0);
    }

    #[test]
    fn layout_maps_offsets_and_points_both_ways() {
        let layout = EditLineLayout {
            bounds: [100.0, 10.0, 200.0, 20.0],
            scroll: 20.0,
            boundaries: vec![(0, 0.0), (1, 8.0), (3, 16.0), (4, 24.0)],
            ..EditLineLayout::default()
        };
        assert_eq!(layout.x_for(0), 80.0);
        assert_eq!(layout.x_for(3), 96.0);
        // A byte inside a multi-byte char maps to its start.
        assert_eq!(layout.x_for(2), 88.0);
        assert_eq!(layout.index_for_x(97.0), 3);
        assert_eq!(layout.index_for_x(0.0), 0);
    }

    #[test]
    fn the_composition_is_its_own_underlined_run() {
        let font = gpui::font("Helvetica");
        let black = Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let runs = runs_for("abにほc", Some((2, 8)), &font, black);
        let lens: Vec<(usize, bool)> = runs
            .iter()
            .map(|run| (run.len, run.underline.is_some()))
            .collect();
        assert_eq!(lens, vec![(2, false), (6, true), (1, false)]);
        assert_eq!(runs_for("abc", None, &font, black).len(), 1);
    }
}
