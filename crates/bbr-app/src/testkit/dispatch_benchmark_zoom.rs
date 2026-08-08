//! Zoom benchmark-request dispatch (testkit feature only; queue item 11,
//! part A).
//!
//! One `Request` → `Response` arm per zoom performance capability, split
//! from `super::dispatch_benchmark` so no gateway module outgrows the
//! ~500-line budget (the same split pattern as `dispatch_benchmark_find`).
//! The arm arms the presentation measurement on the app and starts the
//! production `Cmd-Plus` mutation; the sample commits synchronously when the
//! app's observed zoom state matches the armed level (fast-but-wrong guard).

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::BenchmarkZoomStep`; `None` for every other request so
/// `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_zoom(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkZoomStep {
            operation_id,
            level,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_zoom_step(*operation_id, *level, window, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "zoom step did not reach the armed level on the active view",
                ))
            }
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
