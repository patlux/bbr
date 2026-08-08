//! §11.3 swipe-commit benchmark-request dispatch (DESIGN.md §11.3, task
//! packet `space-footer-strip-appearance-ui`, `swipe_commit` performance
//! layer).
//!
//! The `BenchmarkSwipeCommit` arm drives the real production gesture path in
//! the app (`Started`/`Moved` wheel updates plus a zero-delta `Ended`
//! resolving into exactly one existing `activate_space` commit) and commits
//! synchronously. Split from `super::dispatch_benchmark` so no gateway
//! module outgrows the ~500-line budget, mirroring the
//! `dispatch_benchmark_downloads`/`dispatch_benchmark_u2` routers.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the §11.3 `Request::BenchmarkSwipeCommit` arm; `None` for every
/// other request so `super::dispatch_benchmark` falls through to its own
/// routing.
pub(crate) fn dispatch_benchmark_swipe(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkSwipeCommit {
            operation_id,
            destination,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_swipe_commit(
                *operation_id,
                bbr_browser::SpaceId(destination.0),
                window,
                cx,
            )
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "swipe gesture did not commit the adjacent destination",
                ))
            }
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
