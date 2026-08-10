//! Saved Pages Library benchmark-request dispatch (B3 packet
//! `saved-pages-library`, testkit feature only).
//!
//! `BenchmarkSavedPagesLibrarySearch` / `BenchmarkSavedPagesLibraryScroll`
//! arm the presentation measurement on the app, run the production Library
//! search/scroll entry, force a bounded main-window draw so the frame that
//! renders the new bounded window records the observation, then commit the
//! pending sample only when the exact armed result count/window (search) or
//! window/anchor/selection (scroll) materialized. A failed arm (busy or
//! fast-but-wrong guard) returns the error immediately — never a draw/commit
//! of an unrelated pending sample. Split from `super::dispatch_benchmark` so
//! no gateway module outgrows the ~500-line budget, mirroring the
//! `dispatch_benchmark_u2` / `dispatch_errors` routers.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

/// Handle the Library `Request::Benchmark*` arms; `None` for every other
/// request so `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_library(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkSavedPagesLibrarySearch {
            operation_id,
            text,
            expected_total,
            expected_first,
            expected_last,
        } => {
            let started = with_app(window, app, cx, |view, window, cx| {
                view.benchmark_library_search(
                    *operation_id,
                    text,
                    *expected_total,
                    *expected_first,
                    *expected_last,
                    window,
                    cx,
                )
            })
            .unwrap_or(false);
            if !started {
                return Some(Response::Error(Error::new(
                    "benchmark_noop",
                    "library search did not present the exact result count/window",
                )));
            }
            Response::Ok
        }

        Request::BenchmarkSavedPagesLibraryScroll {
            operation_id,
            offset,
            expected_first,
            expected_last,
            expected_anchor,
        } => {
            let started = with_app(window, app, cx, |view, window, cx| {
                view.benchmark_library_scroll(
                    *operation_id,
                    *offset,
                    *expected_first,
                    *expected_last,
                    *expected_anchor,
                    window,
                    cx,
                )
            })
            .unwrap_or(false);
            if !started {
                return Some(Response::Error(Error::new(
                    "benchmark_noop",
                    "library scroll did not present the exact anchor/window/selection",
                )));
            }
            Response::Ok
        }

        _ => return None,
    };

    // The Library renders in the main window (ordinary GPUI chrome in the
    // content area), so the render-inclusive commit needs a bounded
    // main-window draw (the same seam `FlushFrame` and the error-surface
    // arm use): the paint path renders the new bounded window and records
    // the generic browser state, then the explicit app-side commit closes
    // the pending sample. A frame that omitted the armed Library
    // presentation records nothing and the commit cancels truthfully.
    if matches!(
        req,
        Request::BenchmarkSavedPagesLibrarySearch { .. }
            | Request::BenchmarkSavedPagesLibraryScroll { .. }
    ) {
        let _ = (**window).update(cx, |_root, window, cx| {
            let app: &mut App = cx;
            let _ = window.draw(app);
        });
        let committed = with_app(window, app, cx, |view, _window, _cx| {
            view.commit_library_benchmark()
        })
        .unwrap_or(false);
        if !committed {
            return Some(Response::Error(Error::new(
                "benchmark_noop",
                "library render did not present the armed search/scroll state",
            )));
        }
    }
    Some(response)
}
