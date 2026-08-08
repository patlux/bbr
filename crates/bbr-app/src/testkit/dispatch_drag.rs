//! Tab-drag request dispatch (testkit feature only).
//!
//! Pointer drag on a tab row through the production listener methods (the
//! same begin/move/end handlers the element listeners drive), exactly like
//! the `DragSidebarDivider` seam: never a direct controller mutation. The
//! press is anchored at `y` with the press `press_in_row` within the source
//! row; moves are displacements from it; the app resolves typed targets and
//! legality. Split from `super::dispatch` so no gateway module outgrows the
//! ~500-line budget, mirroring the `dispatch_benchmark` router.

use bbr_testkit::protocol::{Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the `Request::*TabDrag` arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_drag(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BeginTabDrag {
            index,
            x,
            y,
            press_in_row,
        } => with_app(window, app, cx, |view, window, cx| {
            use gpui::{point, px, Modifiers, MouseButton, MouseDownEvent};
            view.begin_tab_drag(
                &MouseDownEvent {
                    button: MouseButton::Left,
                    position: point(px(*x), px(*y)),
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    first_mouse: true,
                },
                *index,
                window,
                cx,
            );
            // The seam records the true press offset on the drag state so
            // subsequent moves resolve targets from the same anchor the
            // scenario simulated.
            view.set_tab_drag_press_in_row(*press_in_row);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::MoveTabDrag { dx, dy } => with_app(window, app, cx, |view, window, cx| {
            use gpui::{point, px, Modifiers, MouseButton, MouseMoveEvent};
            // The app only ever uses the displacement from the press
            // position, so the seam's positions stay consistent by
            // construction (the drag-to-chip path also enters the chip
            // spans through the absolute pointer x).
            let x = view.tab_drag_start_x().unwrap_or(0.0) + dx;
            let y = view.tab_drag_start_y().unwrap_or(0.0) + dy;
            view.on_tab_drag_move(
                &MouseMoveEvent {
                    position: point(px(x), px(y)),
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Modifiers::default(),
                },
                window,
                cx,
            );
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::EndTabDrag => with_app(window, app, cx, |view, window, cx| {
            use gpui::{point, px, Modifiers, MouseButton, MouseUpEvent};
            let y = view.tab_drag_start_y().unwrap_or(0.0);
            view.end_tab_drag(
                &MouseUpEvent {
                    button: MouseButton::Left,
                    position: point(px(130.0), px(y)),
                    modifiers: Modifiers::default(),
                    click_count: 1,
                },
                window,
                cx,
            );
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::CancelTabDrag => with_app(window, app, cx, |view, window, cx| {
            view.cancel_tab_drag(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        _ => return None,
    };
    Some(response)
}
