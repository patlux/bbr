//! Testkit routing for the BBR context menu (queue item 12, part B).
//!
//! The `ContextMenuSelect`/`ContextMenuActivate`/`ContextMenuClose`
//! requests drive the exact production app methods the row `on_hover` and
//! mouse-down listeners and the backdrop/escape close paths call. The menu
//! itself is opened through the exact engine event transport the native
//! bridge uses (`Request::InjectWebEvent` with
//! [`WebEvent::ContextMenuRequested`]), so the functional suites exercise
//! the real event-driven open path; the benchmark arm routes through
//! `dispatch_benchmark_context_menu`.

use bbr_testkit::protocol::{Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

pub(super) fn dispatch_context_menu(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::ContextMenuSelect { index } => with_app(window, app, cx, |view, _window, cx| {
            view.menu_select(*index, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::ContextMenuActivate { index } => with_app(window, app, cx, |view, window, cx| {
            view.menu_activate(*index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::ContextMenuClose => with_app(window, app, cx, |view, window, cx| {
            view.menu_close(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
