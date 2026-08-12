//! §11.14 titlebar pointer-path dispatch (task packet
//! `titlebar-navigation-controls`): the toggle/Back/Forward/Reload click
//! seams and the hover/press/release presentation seams route through the
//! exact production app methods the element listeners call, so E2E
//! exercises the production listener path rather than a controller call.
//! The control wire names are the §15 id suffixes (`toggle` / `back` /
//! `forward` / `reload`).

use bbr_testkit::protocol::{Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::testkit::dispatch_util::err_response;
use crate::window_seam::with_app;

use crate::app::BbrApp;

/// Resolve a wire control name to the production [`TitlebarControl`].
/// Unknown names reject the request.
pub(crate) fn control_from_name(name: &str) -> Option<bbr_ui::titlebar::TitlebarControl> {
    match name {
        "toggle" => Some(bbr_ui::titlebar::TitlebarControl::Toggle),
        "back" => Some(bbr_ui::titlebar::TitlebarControl::Back),
        "forward" => Some(bbr_ui::titlebar::TitlebarControl::Forward),
        "reload" => Some(bbr_ui::titlebar::TitlebarControl::Reload),
        _ => None,
    }
}

pub(crate) fn dispatch_titlebar(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    match req {
        Request::ClickTitlebarToggle => Some(
            with_app(window, app, cx, |view, window, cx| {
                view.toggle_sidebar(window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        Request::ClickTitlebarBack => Some(
            with_app(window, app, cx, |view, _window, _cx| {
                // The production Back authority (the same dispatch as
                // Cmd-[); a disabled control's listener never fires, and
                // the app method is a no-op when history has no back
                // entry — the E2E asserts the visible disabled no-op.
                view.go_back();
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        Request::ClickTitlebarForward => Some(
            with_app(window, app, cx, |view, _window, _cx| {
                view.go_forward();
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        Request::ClickTitlebarReload => Some(
            with_app(window, app, cx, |view, _window, _cx| {
                view.reload();
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        Request::HoverTitlebarControl { control } => {
            let Some(control) = control_from_name(control) else {
                return Some(Response::Error(bbr_testkit::protocol::Error::new(
                    "unknown_titlebar_control",
                    "control must be toggle/back/forward/reload",
                )));
            };
            Some(
                with_app(window, app, cx, |view, _window, cx| {
                    view.set_titlebar_hover(control, true, cx);
                    Response::Ok
                })
                .unwrap_or_else(err_response),
            )
        }
        Request::UnhoverTitlebarControl => Some(
            with_app(window, app, cx, |view, _window, cx| {
                view.clear_titlebar_hover(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        Request::PressTitlebarControl { control } => {
            let Some(control) = control_from_name(control) else {
                return Some(Response::Error(bbr_testkit::protocol::Error::new(
                    "unknown_titlebar_control",
                    "control must be toggle/back/forward/reload",
                )));
            };
            Some(
                with_app(window, app, cx, |view, _window, cx| {
                    view.set_titlebar_pressed(control, cx);
                    Response::Ok
                })
                .unwrap_or_else(err_response),
            )
        }
        Request::ReleaseTitlebarControl => Some(
            with_app(window, app, cx, |view, _window, cx| {
                view.clear_titlebar_pressed(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),
        ),
        _ => None,
    }
}
