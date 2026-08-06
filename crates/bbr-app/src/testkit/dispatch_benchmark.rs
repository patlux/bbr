//! Benchmark-request dispatch (testkit feature only).
//!
//! One `Request` → `Response` arm per performance capability, split from
//! `super::dispatch` so no gateway module outgrows the ~500-line budget.
//! Every arm arms the presentation measurement on the app and commits
//! synchronously through the production mutation path, exactly as the
//! typed benchmark requests in the E2E protocol describe.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Keystroke, WindowHandle};

use crate::app::BbrApp;

use super::dispatch::{browser_mutation, err_response};

/// Handle `Request::Benchmark*` arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_benchmark(
    req: &Request,
    window: &WindowHandle<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkSelectTab {
            operation_id,
            index,
        } => browser_mutation(window.update(cx, |view, window, cx| {
            view.benchmark_select_tab(*operation_id, *index, window, cx)
        })),

        Request::BenchmarkClickTab {
            operation_id,
            index,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_click_tab(*operation_id, *index, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkReopenTab { operation_id } => window
            .update(cx, |view, window, cx| {
                view.benchmark_reopen_tab(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkActivateSpace {
            operation_id,
            space,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_activate_space(
                    *operation_id,
                    bbr_browser::SpaceId(space.0),
                    window,
                    cx,
                )
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkPageReady {
            operation_id,
            expected_view,
            expected_title,
        } => window
            .update(cx, |view, _window, _cx| {
                view.benchmark_page_ready(*operation_id, *expected_view, expected_title.clone())
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new("benchmark_busy", "page-ready already pending"))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkToggleSidebar { operation_id } => window
            .update(cx, |view, window, cx| {
                view.benchmark_toggle_sidebar(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkDispatchKeystroke {
            operation_id,
            kind,
            keystroke,
        } => {
            let started = window
                .update(cx, |view, _window, _cx| {
                    view.begin_benchmark(*operation_id, *kind)
                })
                .unwrap_or(false);
            if !started {
                return Some(Response::Error(Error::new(
                    "benchmark_busy",
                    "another presentation measurement is pending",
                )));
            }
            let ks = match Keystroke::parse(keystroke) {
                Ok(ks) => ks,
                Err(error) => {
                    let _ = window.update(cx, |view, _window, _cx| view.cancel_benchmark());
                    return Some(Response::Error(Error::new(
                        "bad_keystroke",
                        error.to_string(),
                    )));
                }
            };
            (**window)
                .update(cx, |_root, window, cx| window.dispatch_keystroke(ks, cx))
                .map(|_| Response::Ok)
                .unwrap_or_else(|error| {
                    let _ = window.update(cx, |view, _window, _cx| view.cancel_benchmark());
                    Response::Error(Error::new("app_error", error.to_string()))
                })
        }

        Request::BenchmarkResizeSidebar {
            operation_id,
            width,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_resize_sidebar(*operation_id, *width, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkRestoreSidebar { operation_id } => window
            .update(cx, |view, window, cx| {
                view.benchmark_restore_sidebar(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkPinTab { operation_id } => window
            .update(cx, |view, window, cx| {
                view.benchmark_pin_tab(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkReorderTab {
            operation_id,
            delta,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_reorder_tab(*operation_id, *delta, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkMoveTab {
            operation_id,
            to_space,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_move_tab(*operation_id, bbr_browser::SpaceId(to_space.0), window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        _ => return None,
    };
    Some(response)
}
