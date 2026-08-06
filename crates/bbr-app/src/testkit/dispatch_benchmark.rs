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

use super::dispatch::{browser_mutation, err_response, flush_dense_frame};

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

        // Pointer-state benchmarks: the production hover/press listener
        // methods (`set_row_hover` / `set_row_pressed`) render the §10.2
        // hover wash + close reveal and the §10.3 active wash. Each sample
        // commits only when the derived row state actually shows the wash;
        // a broken path cancels with an explicit no-op error instead of
        // committing fast-but-wrong.
        Request::BenchmarkHoverTab {
            operation_id,
            index,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_hover_tab(*operation_id, *index, window, cx)
            })
            .map(|committed| {
                if committed {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "hover did not render the hover state",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkPressTab {
            operation_id,
            index,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_press_tab(*operation_id, *index, window, cx)
            })
            .map(|committed| {
                if committed {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "press did not render the active state",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        // Dense scroll: begin the measurement on the entity, drive the
        // production dense scroll state through the app's tracked handle,
        // draw the resulting frame through the untyped path, then commit
        // only if the scroll moved the rendered anchor (a no-op cancels).
        Request::BenchmarkDenseScroll {
            operation_id,
            delta_px,
        } => {
            let started = window
                .update(cx, |view, _window, _cx| {
                    view.begin_dense_scroll_benchmark(*operation_id)
                })
                .unwrap_or(false);
            if !started {
                return Some(Response::Error(Error::new(
                    "benchmark_busy",
                    "another presentation measurement is pending",
                )));
            }
            let anchor_before = window
                .update(cx, |view, _window, _cx| {
                    view.dense_window.get().scroll_anchor
                })
                .unwrap_or(0);
            let scrolled = window
                .update(cx, |view, _window, _cx| view.scroll_dense_list(*delta_px))
                .map(|_| true)
                .unwrap_or(false);
            if !scrolled {
                let _ = window.update(cx, |view, _window, _cx| view.cancel_benchmark());
                return Some(Response::Error(Error::new(
                    "app_error",
                    "dense scroll failed",
                )));
            }
            // Draw the frame that renders the bounded window for the new
            // anchor (untyped: the draw re-borrows the root entity).
            flush_dense_frame(window, cx);
            let committed = window
                .update(cx, |view, window, _cx| {
                    view.commit_dense_scroll_benchmark(window, anchor_before)
                })
                .unwrap_or(false);
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "dense scroll did not move the rendered anchor",
                ))
            }
        }

        // Dense select: begin on the entity, run the production selection
        // path (pointer row activation), draw the resulting frame through the
        // untyped path so `dense_window` reflects the instant
        // scroll-into-view, then commit only when the selected row lies
        // inside the bounded window the render built.
        Request::BenchmarkDenseSelect {
            operation_id,
            index,
        } => {
            let started = window
                .update(cx, |view, _window, _cx| {
                    view.begin_dense_select_benchmark(*operation_id)
                })
                .unwrap_or(false);
            if !started {
                return Some(Response::Error(Error::new(
                    "benchmark_busy",
                    "another presentation measurement is pending",
                )));
            }
            let selected = window
                .update(cx, |view, window, cx| {
                    view.select_row_via_pointer(*index, window, cx);
                    true
                })
                .unwrap_or(false);
            if !selected {
                let _ = window.update(cx, |view, _window, _cx| view.cancel_benchmark());
                return Some(Response::Error(Error::new(
                    "app_error",
                    "dense selection failed",
                )));
            }
            // Draw the frame that renders the selection with the bounded
            // window (untyped: the draw re-borrows the root entity).
            flush_dense_frame(window, cx);
            let committed = window
                .update(cx, |view, window, _cx| {
                    view.commit_dense_select_benchmark(window, *index)
                })
                .unwrap_or(false);
            if committed {
                Response::Ok
            } else {
                Response::Error(Error::new(
                    "benchmark_noop",
                    "selected row is not inside the rendered window",
                ))
            }
        }

        _ => return None,
    };
    Some(response)
}
