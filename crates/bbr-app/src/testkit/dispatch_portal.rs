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
        // Blueprint D08: the hit-region decision at a window point, and the
        // toast action through its production method.
        Request::PortalRoutePointer { x, y } => with_app(window, app, cx, |view, _window, _cx| {
            view.portal_route_pointer(*x, *y);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // Blueprint D02: one pointer sample into the floating rail's reveal
        // model, the method the native observer feeds.
        Request::RailPointer {
            x,
            y,
            time_ms,
            pressed,
        } => with_app(window, app, cx, |view, window, cx| {
            view.on_rail_pointer(
                crate::app::rail_reveal::PointerSample {
                    x: *x,
                    y: *y,
                    time_ms: *time_ms,
                    pressed: *pressed,
                },
                window,
                cx,
            );
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // Blueprint D10: History and Clear Browsing Data.
        Request::OpenHistory => with_app(window, app, cx, |view, window, cx| {
            view.open_history(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::HistoryInput { input } => with_app(window, app, cx, |view, window, cx| {
            if view.history_input(input.clone(), window, cx) {
                Response::Ok
            } else {
                Response::Error(Error::new("no_history", "History is not open"))
            }
        })
        .unwrap_or_else(err_response),
        // Blueprint D11: the permission prompt and site settings.
        Request::PermissionInput { input } => with_app(window, app, cx, |view, _window, cx| {
            view.permission_input(input.clone(), cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // Blueprint D16: the Settings window.
        Request::OpenSettings => with_app(window, app, cx, |view, _window, cx| {
            view.show_settings(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::SettingsInput { input } => with_app(window, app, cx, |view, window, cx| {
            if view.settings_input(input.clone(), window, cx) {
                Response::Ok
            } else {
                Response::Error(Error::new("bad_settings_input", "unknown pane"))
            }
        })
        .unwrap_or_else(err_response),
        Request::CloseSettings => {
            let handle = with_app(window, app, cx, |view, _window, _cx| view.settings.window)
                .ok()
                .flatten();
            match handle {
                Some(handle) => {
                    let _ = handle.update(cx, |_view, window, _cx| window.remove_window());
                    Response::Ok
                }
                None => Response::Error(Error::new("no_settings", "Settings is not open")),
            }
        }
        // Blueprint D17: the recovery pane's action.
        Request::RecoveryAction => with_app(window, app, cx, |view, _window, cx| {
            if view.recovery_action(cx) {
                Response::Ok
            } else {
                Response::Error(Error::new("no_recovery", "no recovery pane action"))
            }
        })
        .unwrap_or_else(err_response),
        Request::DismissRestoreNotice => match with_app(window, app, cx, |view, _window, cx| {
            (view.dismiss_restore_notice(cx), view.portal_frame_handle())
        }) {
            Ok((dismissed, handle)) => {
                flush_portal_frame(handle, cx);
                if dismissed {
                    Response::Ok
                } else {
                    Response::Error(Error::new("no_notice", "no restore notice"))
                }
            }
            Err(error) => err_response(error),
        },
        // Blueprint D19: a menu item through the NSMenuItem dispatch path.
        Request::InvokeMenuItem { menu, item } => {
            let spaces = app.read(cx).menu_spaces().to_vec();
            match crate::menus::menu_action(&spaces, menu, item) {
                Some(action) => {
                    cx.dispatch_action(&*action);
                    Response::Ok
                }
                None => Response::Error(Error::new("no_menu_item", format!("{menu} › {item}"))),
            }
        }
        Request::ToastAction => match with_app(window, app, cx, |view, _window, cx| {
            (view.toast_action(cx), view.portal_frame_handle())
        }) {
            Ok((acted, handle)) => {
                flush_portal_frame(handle, cx);
                if acted {
                    Response::Ok
                } else {
                    Response::Error(Error::new("no_toast_action", "no toast action to press"))
                }
            }
            Err(error) => err_response(error),
        },
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
