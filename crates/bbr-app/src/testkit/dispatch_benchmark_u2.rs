//! U2 sidebar benchmark-request dispatch (DESIGN.md §11.4/§11.10/§11.11).
//!
//! The plus-menu, favorites-toggle, and folder-toggle presentation arms
//! commit synchronously through the same production app methods the element
//! listeners call, exactly like the other benchmark arms. Split from
//! `super::dispatch_benchmark` so no gateway module outgrows the ~500-line
//! budget, mirroring the `dispatch_u2`/`dispatch_drag` routers.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the U2 sidebar `Request::Benchmark*` arms; `None` for every other
/// request so `super::dispatch_benchmark` falls through to its own routing.
pub(crate) fn dispatch_benchmark_u2(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkPlusMenu { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_plus_menu(*operation_id, window, cx)
            })
            .map(|committed| {
                if committed {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "plus menu did not present the expected open state",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkFavoriteToggle {
            operation_id,
            index,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_favorite_toggle(*operation_id, *index, window, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "favorites grid did not present the expected tile count",
                ))
            }
        })
        .unwrap_or_else(err_response),

        Request::BenchmarkFolderToggle {
            operation_id,
            index,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_folder_toggle(*operation_id, *index, window, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "folder disclosure did not present the expected expanded state",
                ))
            }
        })
        .unwrap_or_else(err_response),

        _ => return None,
    };
    Some(response)
}
