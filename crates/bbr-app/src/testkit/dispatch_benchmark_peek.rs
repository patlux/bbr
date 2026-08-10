//! Peek benchmark-request dispatch (testkit feature only).
//!
//! One `Request` → `Response` arm per Peek performance capability, split
//! from `super::dispatch_benchmark` so no gateway module outgrows the
//! ~500-line budget (the same split pattern as
//! `dispatch_benchmark_find`). Every arm arms the presentation measurement
//! on the app and starts the production Peek mutation; the
//! `peek_request_to_present` sample commits asynchronously when the portal
//! render drew the armed Peek bar identity (fast-but-wrong guard), the
//! dismissal/promotion samples commit synchronously from the authoritative
//! browser state.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::BenchmarkPeek*` arms; `None` for every other request so
/// `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_peek(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkPeekRequestToPresent { operation_id, url } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_peek_request_to_present(*operation_id, url.clone(), window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "the eligible request did not present a Peek preview",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkPeekDismissToOrigin { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_peek_dismiss_to_origin(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "Peek dismissal did not restore the warm origin",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkPeekPromoteToTab { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_peek_promote_to_tab(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "Peek promotion did not adopt the same session as a regular tab",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        _ => return None,
    };
    Some(response)
}
