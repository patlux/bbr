//! Tab/Favorite context-menu benchmark-request dispatch (testkit feature
//! only).
//!
//! One `Request` → `Response` arm pair for the tab-menu performance
//! capability, split from `super::dispatch_benchmark` so no gateway module
//! outgrows the ~500-line budget (the same split pattern as
//! `dispatch_benchmark_context_menu`). Each arm starts the production
//! mutation on the app entity, draws one real main-window frame through the
//! untyped path (the render records the drawn menu state into the probe),
//! then commits the sample only when the armed presentation identity was
//! genuinely drawn — the fast-but-wrong guard.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::flush_dense_frame;

/// Handle `Request::BenchmarkTabMenuOpen` / `BenchmarkTabMenuSubmenuCommit`;
/// `None` for every other request so `super::dispatch_benchmark` falls
/// through to its own routing.
pub(crate) fn dispatch_benchmark_tab_menu(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkTabMenuOpen {
            operation_id,
            row,
            point,
        } => {
            // Begin on the entity: the probe starts at the exact production
            // right-button open seam and arms the drawn-menu identity.
            let started = with_app(window, app, cx, |view, window, cx| {
                view.begin_tab_menu_open_benchmark(*operation_id, *row, *point, window, cx)
            })
            .unwrap_or(false);
            if !started {
                return Some(Response::Error(Error::new(
                    "benchmark_busy",
                    "another presentation measurement is pending",
                )));
            }
            // Draw the frame that renders the open menu (untyped: the draw
            // re-borrows the root entity); the main-window render records
            // the drawn menu state into the probe.
            flush_dense_frame(window, cx);
            let committed = with_app(window, app, cx, |view, window, _cx| {
                view.commit_tab_menu_open_benchmark(window)
            })
            .unwrap_or(false);
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "tab menu open did not present the armed identity",
                ))
            }
        }
        Request::BenchmarkTabMenuSubmenuCommit {
            operation_id,
            parent_row,
        } => {
            // Begin on the entity: the menu must already be open bound to
            // the source `TabId`; the sample starts at the production hover
            // seam that opens the one-level submenu card for the parent row.
            let started = with_app(window, app, cx, |view, window, cx| {
                view.begin_tab_menu_submenu_commit_benchmark(*operation_id, *parent_row, window, cx)
            })
            .unwrap_or(false);
            if !started {
                return Some(Response::Error(Error::new(
                    "benchmark_busy",
                    "another presentation measurement is pending",
                )));
            }
            flush_dense_frame(window, cx);
            let committed = with_app(window, app, cx, |view, window, _cx| {
                view.commit_tab_menu_submenu_commit_benchmark(window)
            })
            .unwrap_or(false);
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "tab menu submenu commit did not present the armed identity",
                ))
            }
        }
        _ => return None,
    };
    Some(response)
}
