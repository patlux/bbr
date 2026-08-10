//! N1 web-content gesture benchmark-request dispatch (task packet
//! `web-content-navigation-gestures`).
//!
//! The `BenchmarkWebHistorySwipeBack` and `BenchmarkChildTabSwipeReturn`
//! arms drive the real production content-gesture path in the app (the
//! exact method the native content-gesture bridge reporter pump calls) and
//! commit synchronously. Split from `super::dispatch_benchmark` so no
//! gateway module outgrows the ~500-line budget, mirroring the
//! `dispatch_benchmark_swipe` router.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the N1 gesture benchmark arms; `None` for every other request so
/// `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_gestures(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkWebHistorySwipeBack { operation_id, view } => {
            with_app(window, app, cx, |app_view, window, cx| {
                app_view.benchmark_web_history_swipe_back(*operation_id, *view, window, cx)
            })
            .map(|committed| {
                if committed {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "history swipe-back did not present the correct state",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }
        Request::BenchmarkChildTabSwipeReturn {
            operation_id,
            child_view,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_child_tab_swipe_return(*operation_id, *child_view, window, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "child-tab swipe return did not present the exact opener",
                ))
            }
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
