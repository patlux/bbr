//! Error-surface request dispatch (testkit feature only).
//!
//! One `Request` → `Response` arm per error-surface capability, split from
//! `super::dispatch` so no gateway module outgrows the ~500-line budget
//! (the same split pattern as `dispatch_find` / `dispatch_downloads`).
//!
//! - `InjectErrorIncident` drives the exact production `on_incident` seam
//!   the native delegate's incident sink uses, with a closed contract
//!   subreason (offline E2E cannot toggle the host network, so the
//!   *classified* incident is injected; the native-code → taxonomy mapping
//!   stays unit-proven in `bbr-web-macos::error_mapping`).
//! - `ClickErrorAction` / `ClickErrorDismiss` press the surface's buttons
//!   through the exact production mouse-down seams (`error_recover` /
//!   `error_dismiss`).
//! - `BenchmarkErrorSurfacePresent` arms the presentation measurement on
//!   the app, runs the production incident-presentation path, and forces a
//!   bounded main-window draw so the render records the surface observation
//!   (the surface is ordinary GPUI chrome — the main-window paint path
//!   commits the sample, exactly like `FlushFrame`).

// macOS-only: the classified-incident payload is produced by the native
// adapter's mapping (`bbr_web_macos::error_mapping`). The module itself is
// gated at the declaration site (`#[cfg(target_os = "macos")] mod
// dispatch_errors;` in `super`), so no duplicated inner attribute.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle `Request::InjectErrorIncident`, `Request::ClickErrorAction`,
/// `Request::ClickErrorDismiss`, and `Request::BenchmarkErrorSurfacePresent`
/// arms; `None` for every other request so `super::dispatch` falls through
/// to its own routing.
pub(crate) fn dispatch_errors(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::ClickErrorAction => with_app(window, app, cx, |view, window, cx| {
            view.error_recover(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickErrorDismiss => with_app(window, app, cx, |view, window, cx| {
            view.error_dismiss(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::InjectErrorIncident {
            profile,
            view: incident_view,
            reason,
            url,
        } => with_app(window, app, cx, |view, _window, cx| {
            // The exact classified payload the native adapter would produce
            // for this subreason (the same `classify` seam the delegate's
            // incident sink drives): the contract derives the category and
            // the privacy-safe context from the failing URL.
            let incident = bbr_web_macos::error_mapping::classify(
                *profile,
                *incident_view,
                bbr_web_macos::error_mapping::MapOutcome::new(*reason),
                url.as_deref(),
            );
            view.on_incident(incident, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::BenchmarkErrorSurfacePresent { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_error_surface_present(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "error surface did not present the classified incident",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        _ => return None,
    };

    // The error surface renders in the main window (ordinary GPUI chrome in
    // the content area), so the render-inclusive commit needs a bounded
    // main-window draw (the same seam `FlushFrame` uses): the paint path
    // records the surface observation and the explicit app-side commit
    // closes the pending sample. A frame that omitted the surface records
    // nothing and the commit cancels truthfully (the driver's deadline
    // catches a missing presentation).
    if matches!(
        req,
        Request::InjectErrorIncident { .. } | Request::BenchmarkErrorSurfacePresent { .. }
    ) {
        let _ = (**window).update(cx, |_root, window, cx| {
            let app: &mut App = cx;
            let _ = window.draw(app);
        });
        if matches!(req, Request::BenchmarkErrorSurfacePresent { .. }) {
            let committed = with_app(window, app, cx, |view, _window, _cx| {
                view.commit_error_surface_benchmark()
            })
            .unwrap_or(false);
            if !committed {
                return Some(Response::Error(Error::new(
                    "benchmark_noop",
                    "error surface render did not present the armed category",
                )));
            }
        }
    }
    Some(response)
}
