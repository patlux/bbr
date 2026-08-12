//! Concrete forwarding closures for the titlebar [`TitlebarCallbacks`]
//! slots.
//!
//! GPUI's element listeners require concrete `Fn` closures (they are never
//! `Rc<dyn Fn>`-callable), so the row wraps each `Rc<dyn Fn>` slot in a
//! small forwarding closure at the composition boundary. The product
//! authority stays in the caller: `bbr-app` fills every slot with the
//! production app methods (re-entering the app entity), and the gallery
//! fills recorder callbacks. This module only adapts the invocation shape —
//! it never invents behavior. The control mouse-down additionally reports
//! the §10.3 press through the caller's press slot before the activation
//! fires (the row never owns the state).

use gpui::{App, MouseDownEvent, Window};

use super::model::{MouseCallback, TitlebarCallbacks, TitlebarControl};

/// Forward a `MouseDownEvent` slot as a concrete `Fn` closure.
pub fn mouse(cb: &MouseCallback) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let cb = cb.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| cb(event, window, cx)
}

/// A control mouse-down: report the §10.3 press, then fire the activation
/// through the caller's callback.
pub fn control_mouse_down(
    activation: &MouseCallback,
    control: TitlebarControl,
    callbacks: &TitlebarCallbacks,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let activation = activation.clone();
    let press = callbacks.on_control_press.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
        press(control, window, cx);
        activation(event, window, cx);
    }
}

/// Forward a control hover slot as a concrete `Fn` closure bound to
/// `control`.
pub fn control_hover(
    callbacks: &TitlebarCallbacks,
    control: TitlebarControl,
) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
    let hover = callbacks.on_control_hover.clone();
    move |hovered: &bool, window: &mut Window, cx: &mut App| hover(control, *hovered, window, cx)
}
