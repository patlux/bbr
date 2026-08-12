//! Context-menu benchmark-request dispatch (testkit feature only).
//!
//! One `Request` → `Response` arm for the context-menu performance
//! capability, split from `super::dispatch_benchmark` so no gateway module
//! outgrows the ~500-line budget (the same split pattern as
//! `dispatch_benchmark_find`). The arm starts the production event-driven
//! open; the sample commits asynchronously when the portal render drew the
//! armed open identity (fast-but-wrong guard).

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::BenchmarkContextMenuOpen`; `None` for every other
/// request so `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_context_menu(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkContextMenuOpen {
            operation_id,
            view,
            point,
        } => with_app(window, app, cx, |view_app, window, cx| {
            view_app.benchmark_context_menu_open(*operation_id, *view, *point, window, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "context menu open did not present the menu",
                ))
            }
        })
        .unwrap_or_else(err_response),
        Request::BenchmarkContextMenuBookmark {
            operation_id,
            view,
            point,
        } => with_app(window, app, cx, |view_app, window, cx| {
            view_app.benchmark_context_menu_bookmark(*operation_id, *view, *point, window, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "context menu bookmark commit did not activate the bookmark row",
                ))
            }
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
