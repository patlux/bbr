//! Renderer-responsiveness performance request routing (testkit-only).

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};
use gpui_component::Root;

use crate::app::BbrApp;
use crate::testkit::dispatch_util::err_response;
use crate::window_seam::with_app;

pub(crate) fn dispatch_benchmark_responsiveness(
    req: &Request,
    window: &WindowHandle<Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkUnresponsiveChrome {
            operation_id,
            source,
            neighbor_index,
            expected_neighbor,
            expected_sessions,
            close,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_unresponsive_chrome(
                *operation_id,
                *source,
                *neighbor_index,
                *expected_neighbor,
                *expected_sessions,
                *close,
                window,
                cx,
            )
        })
        .map(result),
        Request::BenchmarkProbeTick {
            operation_id,
            view: expected_view,
            expected_sessions,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_probe_tick(
                *operation_id,
                *expected_view,
                *expected_sessions,
                window,
                cx,
            )
        })
        .map(result),
        _ => return None,
    };
    Some(response.unwrap_or_else(err_response))
}

fn result(committed: bool) -> Response {
    if committed {
        Response::Ok
    } else {
        Response::Error(Error::new(
            "benchmark_noop",
            "renderer responsiveness benchmark guard rejected the sample",
        ))
    }
}
