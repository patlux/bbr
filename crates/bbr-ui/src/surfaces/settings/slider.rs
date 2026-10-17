//! The Settings slider: a track, its accent fill, a thumb and the value.
//! A press sets the value under the pointer, a drag follows it anywhere in
//! the window, and the arrow keys step it while it is the active control
//! (the app owns that state). Values are whole percents in `step` steps.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{canvas, div, prelude::*, px, AnyElement, Bounds, MouseButton, Pixels, SharedString};

use crate::theme::Theme;

use super::{SettingsCallback, SettingsEvent};

/// The track length.
pub const SLIDER_TRACK_WIDTH: f32 = 168.0;
const THUMB: f32 = 16.0;

/// The value at `x` on a track starting at `left`, `width` long: clamped to
/// 0–100 and rounded to the nearest `step`.
pub fn slider_value_at(x: f32, left: f32, width: f32, step: u8) -> u8 {
    let step = f32::from(step.max(1));
    let share = if width > 0.0 {
        ((x - left) / width).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((share * 100.0 / step).round() * step).clamp(0.0, 100.0) as u8
}

/// The drag payload: the row whose slider follows the pointer.
#[derive(Clone)]
struct SliderDrag(SharedString);

/// No drag image: the thumb itself is the feedback.
struct SliderDragPreview;

impl Render for SliderDragPreview {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

fn value_from(bounds: Bounds<Pixels>, x: Pixels, step: u8) -> u8 {
    slider_value_at(
        f32::from(x),
        f32::from(bounds.origin.x),
        f32::from(bounds.size.width),
        step,
    )
}

pub(super) fn slider(
    theme: &Theme,
    row: &SharedString,
    value: u8,
    step: u8,
    label: &SharedString,
    focused: bool,
    events: &SettingsCallback,
) -> AnyElement {
    let p = theme.palette;
    let share = f32::from(value.min(100)) / 100.0;
    // The track's laid-out bounds of this frame, for the press position.
    let track_bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::default();
    let press = {
        let events = events.clone();
        let row = row.clone();
        let bounds = track_bounds.clone();
        move |event: &gpui::MouseDownEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
            cx.stop_propagation();
            if let Some(bounds) = bounds.get() {
                let value = value_from(bounds, event.position.x, step);
                events(SettingsEvent::Slide(row.clone(), value), window, cx);
            }
        }
    };
    let follow = {
        let events = events.clone();
        let row = row.clone();
        move |event: &gpui::DragMoveEvent<SliderDrag>,
              window: &mut gpui::Window,
              cx: &mut gpui::App| {
            if event.drag(cx).0 != row {
                return;
            }
            let value = value_from(event.bounds, event.event.position.x, step);
            events(SettingsEvent::Slide(row.clone(), value), window, cx);
        }
    };
    let record = {
        let bounds = track_bounds.clone();
        move |frame: Bounds<Pixels>, _: &mut gpui::Window, _: &mut gpui::App| {
            bounds.set(Some(frame))
        }
    };
    let track = div()
        .id(SharedString::from(format!("{row}-slider")))
        .relative()
        .w(px(SLIDER_TRACK_WIDTH))
        .h(px(THUMB + 4.0))
        .flex()
        .items_center()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, press)
        .on_drag(SliderDrag(row.clone()), |_, _, _, cx| {
            cx.new(|_| SliderDragPreview)
        })
        .on_drag_move(follow)
        .child(canvas(record, |_, _, _, _| {}).absolute().size_full())
        .child(
            div()
                .w_full()
                .h(px(4.0))
                .rounded(px(2.0))
                // `hairline` vanishes on the dark field; the stronger border
                // keeps the unfilled track readable in both modes.
                .bg(p.border_strong)
                .child(
                    div()
                        .h_full()
                        .w(px(SLIDER_TRACK_WIDTH * share))
                        .rounded(px(2.0))
                        .bg(p.accent),
                ),
        )
        .child(
            div()
                .id(SharedString::from(format!("{row}-slider-thumb")))
                .absolute()
                .left(px((SLIDER_TRACK_WIDTH - THUMB) * share))
                .size(px(THUMB))
                .rounded(px(THUMB / 2.0))
                .bg(p.surface_field)
                .border_1()
                .border_color(if focused { p.accent } else { p.border_strong })
                .when(focused, |el| el.border_2())
                .shadow(crate::theme::elevation::lift(theme)),
        );
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(track)
        .child(
            div()
                .min_w(px(36.0))
                .flex()
                .justify_end()
                .text_size(theme.type_.label.size)
                .text_color(p.ink_secondary)
                .child(label.clone()),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::slider_value_at;

    #[test]
    fn pointer_positions_map_to_clamped_stepped_percents() {
        assert_eq!(slider_value_at(100.0, 100.0, 200.0, 5), 0);
        assert_eq!(slider_value_at(300.0, 100.0, 200.0, 5), 100);
        assert_eq!(slider_value_at(200.0, 100.0, 200.0, 5), 50);
        // Outside the track the value pins to the end it left from.
        assert_eq!(slider_value_at(-40.0, 100.0, 200.0, 5), 0);
        assert_eq!(slider_value_at(900.0, 100.0, 200.0, 5), 100);
        // Rounded to the nearest step: 51.5 % → 50, 53 % → 55.
        assert_eq!(slider_value_at(203.0, 100.0, 200.0, 5), 50);
        assert_eq!(slider_value_at(206.0, 100.0, 200.0, 5), 55);
        // A degenerate track never divides by zero.
        assert_eq!(slider_value_at(10.0, 10.0, 0.0, 5), 0);
        assert_eq!(slider_value_at(150.0, 100.0, 200.0, 1), 25);
    }
}
