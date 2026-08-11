//! Gateway routing for the real WebKit content probe (testkit-only).
//!
//! The probe request issues one client-world DOM + native url/title
//! observation against a live WKWebView through the concrete engine handle;
//! the completion lands on the next snapshot's `web_content_probe`. Routed
//! before the main match (like find/portal/downloads) so the main router
//! stays under its size budget.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};
use gpui_component::Root;

use crate::app::BbrApp;
use crate::testkit::dispatch_util::err_response;
use crate::window_seam::with_app;

/// Route the content-probe request; `None` for every other request.
pub(crate) fn dispatch_probe(
    req: &Request,
    window: &WindowHandle<Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    match req {
        Request::ProbeRendererResponsiveness { view } => Some(
            with_app(window, app, cx, |view_app, _window, _cx| {
                view_app.probe_renderer_responsiveness(*view)
            })
            .map(|issued| {
                if issued {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "probe_error",
                        "responsiveness probe rejected: unknown or destroyed view",
                    ))
                }
            })
            .unwrap_or_else(err_response),
        ),
        Request::ClickUnresponsiveWait => Some(
            with_app(window, app, cx, |view_app, window, cx| {
                view_app.wait_unresponsive_active(window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        Request::ClickUnresponsiveReload => Some(
            with_app(window, app, cx, |view_app, window, cx| {
                view_app.reload_unresponsive_active(window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        Request::ClickUnresponsiveClose => Some(
            with_app(window, app, cx, |view_app, window, cx| {
                let _ = view_app.close_active_tab(window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        #[cfg(all(target_os = "macos", feature = "testkit"))]
        Request::ProbeWebContent { view } => Some(
            with_app(window, app, cx, |view_app, _window, _cx| {
                view_app.probe_web_content(*view)
            })
            .map(|issued| {
                if issued {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "probe_error",
                        "content probe rejected: unknown view or missing native engine",
                    ))
                }
            })
            .unwrap_or_else(err_response),
        ),
        // Non-macOS or non-testkit builds cannot probe the real engine; the
        // fall-through match must never see the variant.
        #[cfg(not(all(target_os = "macos", feature = "testkit")))]
        Request::ProbeWebContent { .. } => Some(Response::Error(Error::new(
            "unsupported",
            "web content probe requires the macOS testkit engine",
        ))),
        _ => None,
    }
}
