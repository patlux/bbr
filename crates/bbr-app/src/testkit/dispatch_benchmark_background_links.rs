//! Background-link benchmark-request dispatch (testkit feature only).
//!
//! One `Request` → `Response` arm for the background-link performance
//! capability, split from `super::dispatch_benchmark` so no gateway module
//! outgrows the ~500-line budget (the same split pattern as
//! `dispatch_benchmark_peek`). The arm arms the presentation measurement on
//! the app and starts the production background-child mutation; the sample
//! commits synchronously from the authoritative browser state.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::BenchmarkBackgroundLinkOpen`; `None` for every other
/// request so `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_background_links(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkBackgroundLinkOpen { operation_id, url } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_background_link_open(*operation_id, url.clone(), window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "the background disposition did not open a hidden child after the opener",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }
        _ => return None,
    };
    Some(response)
}
