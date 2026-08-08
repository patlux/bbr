//! Testkit routing for the one native portal window.
//!
//! Portal draws happen only after the mutable `BbrApp` lease is released:
//! U1's `PortalRoot` reads that same entity during render, so drawing inside
//! `with_app` would re-enter GPUI's entity map.

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{AnyWindowHandle, App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

pub(super) fn dispatch_portal(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::PortalOpenProbe => {
            match with_app(window, app, cx, |view, window, cx| {
                view.portal_present(crate::portal::probe::probe_request(), window, cx);
                view.portal_frame_handle()
            }) {
                Ok(handle) => {
                    flush_portal_frame(handle, cx);
                    Response::Ok
                }
                Err(error) => err_response(error),
            }
        }
        Request::PortalCloseProbe => {
            match with_app(window, app, cx, |view, _window, cx| {
                let dismissed = view.portal_dismiss(cx);
                (dismissed, view.portal_frame_handle())
            }) {
                Ok((dismissed, handle)) => {
                    flush_portal_frame(handle, cx);
                    portal_dismiss_response(dismissed, "no active portal presentation")
                }
                Err(error) => err_response(error),
            }
        }
        Request::PortalOutsideDismiss => {
            match with_app(window, app, cx, |view, _window, cx| {
                let dismissed = view.portal_outside_dismiss(cx);
                (dismissed, view.portal_frame_handle())
            }) {
                Ok((dismissed, handle)) => {
                    flush_portal_frame(handle, cx);
                    portal_dismiss_response(dismissed, "outside press dismissed nothing")
                }
                Err(error) => err_response(error),
            }
        }
        Request::PortalTeardown => with_app(window, app, cx, |view, _window, _cx| {
            view.portal_teardown();
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // Native-portal-hardening drivers: real AppKit lifecycle mutations
        // on the parent browser window (fullscreen, minimize/restore,
        // Window Server space assignment, and the real close path). Each is
        // idempotent and main-thread-only.
        Request::PortalSetFullscreen { fullscreen } => {
            with_app(window, app, cx, |view, _window, _cx| {
                view.portal_set_fullscreen(*fullscreen);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::PortalSetMinimized { minimized } => {
            with_app(window, app, cx, |view, _window, _cx| {
                view.portal_set_minimized(*minimized);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::PortalSetAllSpaces { all_spaces } => {
            with_app(window, app, cx, |view, _window, _cx| {
                view.portal_set_all_spaces(*all_spaces);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::PortalCloseWindow => with_app(window, app, cx, |view, _window, _cx| {
            view.portal_close_parent();
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::FlushFrame => {
            let main = (**window)
                .update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _clear = window.draw(app);
                })
                .map(|_| Response::Ok)
                .unwrap_or_else(err_response);
            let portal = with_app(window, app, cx, |view, _window, _cx| {
                view.portal_frame_handle()
            })
            .ok()
            .flatten();
            flush_portal_frame(portal, cx);
            main
        }
        _ => return None,
    };
    Some(response)
}

fn portal_dismiss_response(dismissed: bool, message: &str) -> Response {
    if dismissed {
        Response::Ok
    } else {
        Response::Error(Error::new("portal_inactive", message))
    }
}

pub(super) fn flush_portal_frame(handle: Option<AnyWindowHandle>, cx: &mut App) {
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, window, cx| {
            let app: &mut App = cx;
            let _ = window.draw(app);
        });
    }
}
