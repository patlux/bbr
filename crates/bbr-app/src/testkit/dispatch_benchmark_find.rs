//! Find benchmark-request dispatch (testkit feature only).
//!
//! One `Request` → `Response` arm per find performance capability, split
//! from `super::dispatch_benchmark` so no gateway module outgrows the
//! ~500-line budget (the same split pattern as `dispatch_find`). Every arm
//! arms the presentation measurement on the app and starts the production
//! find mutation; the sample commits asynchronously when the portal render
//! drew the armed open/report identity (fast-but-wrong guard).

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::BenchmarkFind*` arms; `None` for every other request so
/// `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_find(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkFindOpen { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_find_open(*operation_id, window, cx)
            })
            .map(|committed| {
                if committed {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "find open did not present the find bar",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkFindQueryResult {
            operation_id,
            query,
            expected_count,
            expected_ordinal,
        } => with_app(window, app, cx, |view, _window, cx| {
            view.benchmark_find_query_result(
                *operation_id,
                query,
                *expected_count,
                *expected_ordinal,
                cx,
            )
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "find query did not report the expected count/ordinal",
                ))
            }
        })
        .unwrap_or_else(err_response),

        Request::BenchmarkFindStep {
            operation_id,
            next,
            expected_count,
            expected_ordinal,
        } => with_app(window, app, cx, |view, _window, cx| {
            view.benchmark_find_step(*operation_id, *next, *expected_count, *expected_ordinal, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "find step did not report the expected ordinal",
                ))
            }
        })
        .unwrap_or_else(err_response),

        _ => return None,
    };
    Some(response)
}
