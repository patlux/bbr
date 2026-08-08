//! Testkit routing for the per-site zoom commands (queue item 11, part A).
//!
//! The `ZoomIn`/`ZoomOut`/`ZoomReset` requests drive the exact production
//! seams the `Cmd-Plus`/`Cmd-Minus`/`Cmd-0` key bindings route to. Zoom
//! mutations are synchronous (the lifecycle change, the persistence write,
//! and the native `pageZoom` property set all complete on the main thread),
//! so no portal-frame flush or async completion is involved — the functional
//! suites additionally exercise the real key path through
//! `Request::DispatchKeystroke`.

use bbr_testkit::protocol::{Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

pub(super) fn dispatch_zoom(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::ZoomIn => with_app(window, app, cx, |view, _window, cx| {
            view.zoom_in(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::ZoomOut => with_app(window, app, cx, |view, _window, cx| {
            view.zoom_out(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::ZoomReset => with_app(window, app, cx, |view, _window, cx| {
            view.zoom_reset(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
