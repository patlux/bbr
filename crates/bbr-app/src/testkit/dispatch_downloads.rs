//! Downloads request dispatch (task packet `downloads-sidebar-panel`,
//! DESIGN.md §11.12).
//!
//! The download start/cancel/retry, row-action, and outside-dismiss requests
//! route through the same production app methods the element listeners call
//! (never a controller mutation behind the shell's back): `DownloadStart`
//! drives the production controller + native adapter path, cancel/retry are
//! the production request/confirmation paths, the row action click is the
//! exact app method the action's mouse-down listener runs, and the outside
//! press drives the §13 one-pointer-action dismissal. Split from
//! `super::dispatch` so no gateway module outgrows the ~500-line budget.

use bbr_testkit::protocol::{Request, Response};
use bbr_web::downloads::DestinationDecision;
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the downloads `Request` arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_downloads(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        // Start through the production controller + native adapter path:
        // mints the attempt id, records the optimistic `Waiting` state, and
        // starts the transfer on the active web view (the save-panel policy
        // stays open, DESIGN §17.9 — v1 uses the engine's suggested
        // destination, exactly like the production `download_start`).
        Request::DownloadStart { url, destination } => {
            with_app(window, app, cx, |view, _window, cx| {
                match crate::app::downloads_behavior::download_start(
                    view,
                    url,
                    DestinationDecision::Suggested(destination.clone()),
                    cx,
                ) {
                    Ok(_) => Response::Ok,
                    Err(error) => Response::Error(bbr_testkit::protocol::Error::new(
                        "download_start_failed",
                        format!("download start failed: {error:?}"),
                    )),
                }
            })
            .unwrap_or_else(err_response)
        }
        // Request cancellation (production `controller.cancel` path): the
        // row stays active until the engine's terminal event lands.
        Request::DownloadCancel { id } => with_app(window, app, cx, |view, _window, cx| {
            match crate::app::downloads_behavior::download_cancel(
                view,
                bbr_web::downloads::DownloadId(*id),
                cx,
            ) {
                Ok(()) => Response::Ok,
                Err(error) => Response::Error(bbr_testkit::protocol::Error::new(
                    "download_cancel_failed",
                    format!("download cancel failed: {error:?}"),
                )),
            }
        })
        .unwrap_or_else(err_response),
        // Retry the failed attempt (production `controller.retry` path): a
        // fresh attempt id linked via `retry_of`. The same seam restarts a
        // cancelled attempt (§17.8 settled).
        Request::DownloadRetry { id } => with_app(window, app, cx, |view, _window, cx| {
            match crate::app::downloads_behavior::download_retry(
                view,
                bbr_web::downloads::DownloadId(*id),
                cx,
            ) {
                Ok(_) => Response::Ok,
                Err(error) => Response::Error(bbr_testkit::protocol::Error::new(
                    "download_retry_failed",
                    format!("download retry failed: {error:?}"),
                )),
            }
        })
        .unwrap_or_else(err_response),
        // The settled row Delete action (production `download_delete_row`
        // path — the exact seam the row's Delete control mouse-down
        // listener calls): remove exactly that one terminal record durably.
        Request::ClickDownloadRowDelete { row } => {
            with_app(window, app, cx, |view, _window, cx| {
                let visuals =
                    crate::app::downloads::row_visuals(&view.downloads, Some(view.downloads.focus));
                match visuals.get(*row) {
                    Some(visual) if visual.delete_available => {
                        match crate::app::downloads_history::download_delete_row(
                            view,
                            bbr_web::downloads::DownloadId(visual.id),
                            cx,
                        ) {
                            Ok(()) => Response::Ok,
                            Err(error) => Response::Error(bbr_testkit::protocol::Error::new(
                                "download_delete_failed",
                                format!("download delete failed: {error:?}"),
                            )),
                        }
                    }
                    _ => Response::Error(bbr_testkit::protocol::Error::new(
                        "download_delete_unavailable",
                        format!("row {row} offers no Delete control"),
                    )),
                }
            })
            .unwrap_or_else(err_response)
        }
        // The settled Clear panel action (production `downloads_clear` path
        // — the exact seam the panel's Clear control mouse-down listener
        // calls): empty the controller history and the repository durably.
        Request::ClickDownloadsClear => with_app(window, app, cx, |view, _window, cx| {
            crate::app::downloads_history::downloads_clear(view, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // Activate a visible row's contextual action through the production
        // pointer path (the exact app method the action's mouse-down
        // listener calls): Cancel / Retry / Reveal-in-Finder. A disabled
        // action (a vanished reveal target) is a no-op.
        Request::ClickDownloadRowAction { row } => with_app(window, app, cx, |view, window, cx| {
            crate::app::downloads_behavior::download_click_row_action(view, *row, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // Drive the exact production outside-dismiss method the panel's
        // outside-press handler calls (DESIGN.md §13 one pointer action): a
        // pointer press outside the open panel dismisses it and returns
        // focus to the footer control.
        Request::ClickDownloadsOutside => with_app(window, app, cx, |view, window, cx| {
            view.begin_root_press();
            view.dismiss_downloads_on_outside_press(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // §11.12 bounded-window scroll in the native wheel listener's units
        // and sign (the same shared scroll state the keyboard traversal and
        // the render consume).
        Request::ScrollDownloads { delta_px } => with_app(window, app, cx, |view, _window, _cx| {
            view.scroll_downloads(*delta_px);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
