//! Blueprint D08 (DESIGN §6, §18.4): the transient feedback surfaces over
//! the content card — the toast that confirms a reversible action (4 s,
//! bottom-centre of the card, one action such as Undo) and the zoom readout
//! that confirms each ⌘+ / ⌘− step (top-centre, fades after 1.2 s).
//!
//! Both overlap web content, so the app renders them in the one native
//! Portal in its hit-region mode: the panel captures the pointer only over
//! these surfaces and lets it through everywhere else, so the page is never
//! blocked. Presentation only: the app owns the lifecycle and the callbacks.

use std::time::Duration;

use gpui::{
    div, prelude::*, px, AnimationExt, AnyElement, App, MouseButton, MouseDownEvent, Rgba,
    SharedString, Window,
};

use crate::icons::Icon;
use crate::theme::elevation::elev2;
use crate::theme::motion::Easing;
use crate::theme::{Motion, MotionMode, Theme};

pub const TOAST_ID: &str = "bbr-toast";
pub const TOAST_MESSAGE_ID: &str = "bbr-toast-message";
pub const TOAST_ACTION_ID: &str = "bbr-toast-action";
pub const ZOOM_READOUT_ID: &str = "bbr-zoom-readout";
pub const ZOOM_READOUT_LEVEL_ID: &str = "bbr-zoom-readout-level";
pub const ZOOM_READOUT_MINUS_ID: &str = "bbr-zoom-readout-minus";
pub const ZOOM_READOUT_PLUS_ID: &str = "bbr-zoom-readout-plus";

/// How long a toast stays before it leaves.
pub const TOAST_DURATION: Duration = Duration::from_secs(4);
/// How long the zoom readout stays after the last step.
pub const ZOOM_READOUT_DURATION: Duration = Duration::from_millis(1200);
/// The toast's height and its distance above the card's bottom edge.
pub const TOAST_HEIGHT: f32 = 40.0;
pub const TOAST_BOTTOM: f32 = 24.0;
/// The zoom readout's size and its distance below the card's top edge.
pub const ZOOM_READOUT_SIZE: [f32; 2] = [120.0, 32.0];
pub const ZOOM_READOUT_TOP: f32 = 24.0;
/// The rise of the toast's entrance (full motion only).
const TOAST_RISE: f32 = 4.0;

/// The toast's leading glyph tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastTone {
    /// `status.ok` — a completed action.
    Ok,
    /// `accent.ink` — a saved/marked page.
    Accent,
    /// `ink.secondary` — a neutral notice.
    Neutral,
}

/// The presentation model of one toast.
#[derive(Debug, Clone, PartialEq)]
pub struct ToastModel {
    /// Identity of this appearance (a new toast replays its entrance).
    pub generation: u64,
    pub icon: Icon,
    pub tone: ToastTone,
    pub message: SharedString,
    /// The one action (e.g. "Undo"), if the confirmed action is reversible.
    pub action: Option<SharedString>,
    /// The toast is leaving (its exit fade plays; it no longer takes input).
    pub leaving: bool,
}

fn tone_color(theme: &Theme, tone: ToastTone) -> Rgba {
    let p = theme.palette;
    match tone {
        ToastTone::Ok => p.status_ok,
        ToastTone::Accent => p.accent_ink,
        ToastTone::Neutral => p.ink_secondary,
    }
}

/// The toast's rect inside the card inner box `[x, y, w, h]` for a toast of
/// `width` (bottom-centre, [`TOAST_BOTTOM`] above the card's bottom edge).
pub fn toast_rect(card: [f32; 4], width: f32) -> [f32; 4] {
    let [x, y, w, h] = card;
    let width = width.min(w).max(0.0);
    [
        x + (w - width) / 2.0,
        y + h - TOAST_BOTTOM - TOAST_HEIGHT,
        width,
        TOAST_HEIGHT,
    ]
}

/// The zoom readout's rect inside the card inner box (top-centre).
pub fn zoom_readout_rect(card: [f32; 4]) -> [f32; 4] {
    let [x, y, w, _] = card;
    let [width, height] = ZOOM_READOUT_SIZE;
    [x + (w - width) / 2.0, y + ZOOM_READOUT_TOP, width, height]
}

/// A pointer-press callback.
type PressFn = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// The quiet small action button (text in `accent.ink`, hover wash).
fn quiet_button(
    theme: &Theme,
    label: SharedString,
    on_press: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let p = theme.palette;
    div()
        .id(TOAST_ACTION_ID)
        .h(px(28.0))
        .px(crate::theme::Spacing::sp2())
        .flex()
        .items_center()
        .rounded(theme.radius.small())
        .text_size(theme.type_.label.size)
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(p.accent_ink)
        .hover(move |style| style.bg(p.wash_hover))
        .active(move |style| style.bg(p.wash_active))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            cx.stop_propagation();
            on_press(event, window, cx);
        })
        .child(label)
}

/// The toast card (`material:overlay`, `radius:lg`, hairline + `elev:2`),
/// hugging its content. The caller places it at [`toast_rect`].
pub fn toast(
    theme: &Theme,
    model: &ToastModel,
    on_action: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let p = theme.palette;
    let card = div()
        .id(TOAST_ID)
        .h(px(TOAST_HEIGHT))
        .pl(px(14.0))
        .pr(px(if model.action.is_some() { 8.0 } else { 14.0 }))
        .gap(px(10.0))
        .flex()
        .flex_row()
        .items_center()
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .child(
            model
                .icon
                .canvas_with(theme, px(16.0), tone_color(theme, model.tone)),
        )
        .child(
            div()
                .id(TOAST_MESSAGE_ID)
                .text_size(theme.type_.label.size)
                .text_color(p.ink)
                .whitespace_nowrap()
                .child(model.message.clone()),
        )
        .children(
            model
                .action
                .clone()
                .map(|label| quiet_button(theme, label, on_action)),
        );
    // V-6: dur:fine fade plus a 4 pt rise; Reduce Motion keeps the fade.
    // Leaving: a dur:fine fade out.
    if model.leaving {
        return card
            .with_animation(
                SharedString::from(format!("bbr-toast-out-{}", model.generation)),
                gpui::Animation::new(Motion::fine()),
                |el, t| el.opacity(1.0 - t),
            )
            .into_any_element();
    }
    let reduced = theme.motion_mode == MotionMode::Reduced;
    card.with_animation(
        SharedString::from(format!("bbr-toast-in-{}", model.generation)),
        gpui::Animation::new(Motion::fine()).with_easing(|t| Easing::Standard.apply(t)),
        move |el, t| {
            let el = el.opacity(t);
            if reduced {
                el
            } else {
                el.mt(px(TOAST_RISE * (1.0 - t)))
            }
        },
    )
    .into_any_element()
}

/// The zoom readout `[−] 140 % [+]` (`material:overlay`, `radius:lg`,
/// hairline + `elev:2`); the minus and plus step the zoom through the same
/// app methods as ⌘− / ⌘+. The caller places it at [`zoom_readout_rect`].
pub fn zoom_readout(
    theme: &Theme,
    percent: u32,
    generation: u64,
    on_minus: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_plus: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let p = theme.palette;
    let step = |id: &'static str, icon: Icon, on_press: PressFn| {
        div()
            .id(id)
            .size(px(24.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(theme.radius.small())
            .hover(move |style| style.bg(p.wash_hover))
            .active(move |style| style.bg(p.wash_active))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                cx.stop_propagation();
                on_press(event, window, cx);
            })
            .child(icon.canvas_with(theme, px(14.0), p.ink_secondary))
    };
    let [width, height] = ZOOM_READOUT_SIZE;
    let readout = div()
        .id(ZOOM_READOUT_ID)
        .w(px(width))
        .h(px(height))
        .px(px(4.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .child(step(ZOOM_READOUT_MINUS_ID, Icon::Minus, Box::new(on_minus)))
        .child(
            div()
                .id(ZOOM_READOUT_LEVEL_ID)
                .text_size(theme.type_.key.size)
                .text_color(p.ink)
                .child(format!("{percent}%")),
        )
        .child(step(ZOOM_READOUT_PLUS_ID, Icon::Plus, Box::new(on_plus)));
    // V-6: dur:fine in (the first step of a burst only); Reduce Motion: none.
    if theme.motion_mode == MotionMode::Reduced {
        return readout.into_any_element();
    }
    readout
        .with_animation(
            SharedString::from(format!("bbr-zoom-readout-in-{generation}")),
            gpui::Animation::new(Motion::fine()),
            |el, t| el.opacity(t),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_sits_bottom_centre_and_the_readout_top_centre_of_the_card() {
        let card = [260.0, 10.0, 900.0, 700.0];
        assert_eq!(toast_rect(card, 300.0), [560.0, 646.0, 300.0, 40.0]);
        // A toast wider than the card clamps to it.
        assert_eq!(toast_rect(card, 2000.0)[2], 900.0);
        assert_eq!(zoom_readout_rect(card), [650.0, 34.0, 120.0, 32.0]);
        assert_eq!(TOAST_DURATION, Duration::from_secs(4));
        assert_eq!(ZOOM_READOUT_DURATION, Duration::from_millis(1200));
    }
}
