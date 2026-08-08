//! Downloads benchmark-request dispatch (testkit feature only).
//!
//! One `Request` → `Response` arm per downloads performance capability,
//! split from `super::dispatch_benchmark` so no gateway module outgrows the
//! ~500-line budget (the same split pattern as `dispatch_benchmark_find`).
//! Every arm starts the presentation measurement on the app, runs the
//! production downloads mutation, arms the exact panel presentation the
//! sample must commit from, and forces a bounded main-window draw so the
//! render records the observation (the panel is ordinary GPUI sidebar
//! chrome — the main-window paint path commits the sample, exactly like
//! `FlushFrame`). The `SeedDownloadsAttempt` arm seeds a deterministic
//! non-transferring running attempt for the progress scenario.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::BenchmarkDownloads*` and `SeedDownloadsAttempt` arms;
/// `None` for every other request so `super::dispatch_benchmark` falls
/// through to its own routing.
pub(crate) fn dispatch_benchmark_downloads(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::BenchmarkDownloadsPanelOpen { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_downloads_panel_open(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "downloads panel open did not present the anchored panel",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkDownloadsProgress {
            operation_id,
            id,
            received,
            total,
        } => with_app(window, app, cx, |view, window, cx| {
            view.benchmark_downloads_progress_commit(
                *operation_id,
                *id,
                *received,
                *total,
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
                    "download progress did not apply at the armed received/total",
                ))
            }
        })
        .unwrap_or_else(err_response),

        Request::BenchmarkDownloadsRowRetry { operation_id, row } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_downloads_row_retry(*operation_id, *row, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "download row retry did not present the fresh active attempt",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkDownloadsRowRestart { operation_id, row } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_downloads_row_restart(*operation_id, *row, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "download row restart did not present the fresh active attempt",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkDownloadsRowDelete { operation_id, row } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_downloads_row_delete(*operation_id, *row, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "download row delete did not remove exactly one terminal row",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::BenchmarkDownloadsClear { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_downloads_clear(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "downloads clear did not present the empty panel",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        Request::SeedDownloadsAttempt {
            url,
            destination,
            total,
        } => with_app(window, app, cx, |view, _window, cx| {
            view.seed_downloads_attempt(url, destination, *total, cx)
                .map(|_| Response::Ok)
                .unwrap_or_else(|| {
                    Response::Error(Error::new(
                        "seed_failed",
                        "could not seed the deterministic download attempt",
                    ))
                })
        })
        .unwrap_or_else(err_response),

        // Seed a deterministic non-transferring **cancelled** terminal
        // attempt for the restart/delete scenarios (the same in-process
        // seeding as `SeedDownloadsAttempt`, then the engine's terminal
        // `Cancelled` event through the production event path — no network
        // transfer runs). The driver reads the minted id from the snapshot.
        Request::SeedDownloadsCancelled {
            url,
            destination,
            total,
        } => with_app(window, app, cx, |view, _window, cx| {
            view.seed_downloads_cancelled(url, destination, *total, cx)
                .map(|_| Response::Ok)
                .unwrap_or_else(|| {
                    Response::Error(Error::new(
                        "seed_failed",
                        "could not seed the deterministic cancelled attempt",
                    ))
                })
        })
        .unwrap_or_else(err_response),

        _ => return None,
    };

    // The downloads panel renders in the main window, so the render-inclusive
    // commit needs a bounded main-window draw (the same seam `FlushFrame`
    // uses): the paint path records the panel observation and commits the
    // pending sample. A frame that omitted the panel records nothing and
    // the sample simply never commits (the driver's deadline catches it).
    if matches!(
        req,
        Request::BenchmarkDownloadsPanelOpen { .. }
            | Request::BenchmarkDownloadsProgress { .. }
            | Request::BenchmarkDownloadsRowRetry { .. }
            | Request::BenchmarkDownloadsRowRestart { .. }
            | Request::BenchmarkDownloadsRowDelete { .. }
            | Request::BenchmarkDownloadsClear { .. }
    ) {
        let _ = (**window).update(cx, |_root, window, cx| {
            let app: &mut App = cx;
            let _ = window.draw(app);
        });
    }
    Some(response)
}
