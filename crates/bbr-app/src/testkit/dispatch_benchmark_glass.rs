//! Settings transparency benchmark-request dispatch (testkit feature only),
//! split like `dispatch_benchmark_zoom`. The arm arms the render-gated
//! measurement and sends the slider's production event; the sample commits
//! asynchronously from the browser-window frame that painted the new tint.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::BenchmarkChromeTransparency`; `None` for every other
/// request so `super::dispatch_benchmark` falls through.
pub(crate) fn dispatch_benchmark_glass(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let Request::BenchmarkChromeTransparency {
        operation_id,
        percent,
    } = req
    else {
        return None;
    };
    Some(
        with_app(window, app, cx, |view, window, cx| {
            view.benchmark_chrome_transparency(*operation_id, *percent, window, cx)
        })
        .map(|started| {
            if started {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "glass unavailable, value unchanged, or a measurement is pending",
                ))
            }
        })
        .unwrap_or_else(err_response),
    )
}
