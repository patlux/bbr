//! Saved Page gateway dispatch (B2 packet `saved-pages-actions-command`,
//! testkit feature only).
//!
//! One `Request` → `Response` arm per Saved Page capability, split from
//! `super::dispatch` so no gateway module outgrows the ~500-line budget.
//! Save/remove/open route through the one app-owned `SavedPageCommand`
//! authority (the same seam Cmd-D and the menu rows share); the flush arm
//! persists the in-memory provider deterministically; the benchmark arms
//! measure the production command-text path against the seeded provider and
//! the production fresh-tab open.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the Saved Page arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_saved_pages(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::SaveSavedPage { url, title } => with_app(window, app, cx, |view, _window, cx| {
            let command = view.saved_pages_command();
            let saved = command.save(view, url, title);
            if saved.is_some() {
                cx.notify();
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "saved_page_rejected",
                    "the URL is not navigable for a durable Saved Page",
                ))
            }
        })
        .unwrap_or_else(err_response),
        Request::SavedPageAvailability { url } => {
            with_app(window, app, cx, |view, _window, _cx| {
                let command = view.saved_pages_command();
                Response::SavedPageAvailability {
                    can_save: command.can_save(view, url),
                    can_remove: command.can_remove(view, url),
                }
            })
            .unwrap_or_else(err_response)
        }
        Request::RemoveSavedPage { id } => with_app(window, app, cx, |view, _window, cx| {
            let command = view.saved_pages_command();
            command.remove(view, bbr_command::id::BookmarkId(*id));
            cx.notify();
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::OpenSavedPage { id } => with_app(window, app, cx, |view, window, cx| {
            let command = view.saved_pages_command();
            let opened = command.open(view, bbr_command::id::BookmarkId(*id), window, cx);
            if opened {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "saved_page_missing",
                    "the Saved Page record is gone or the tab could not be opened",
                ))
            }
        })
        .unwrap_or_else(err_response),
        Request::FlushSavedPages => with_app(window, app, cx, |view, _window, _cx| {
            view.flush_saved_pages();
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::BenchmarkSavedPageSearchFirstResult {
            operation_id,
            text,
            expected_first_key,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_saved_page_search_first_result(
                *operation_id,
                text,
                expected_first_key,
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
                    "saved-page search did not present the expected first result",
                ))
            }
        })
        .unwrap_or_else(err_response),
        Request::BenchmarkSavedPageOpen {
            operation_id,
            bookmark_id,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_saved_page_open(*operation_id, *bookmark_id, window, cx)
        })
        .map(|committed| {
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "saved-page open did not present a fresh tab/session",
                ))
            }
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
