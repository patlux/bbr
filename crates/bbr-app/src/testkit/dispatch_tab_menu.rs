//! Testkit routing for the Tab/Favorite context menu (task packet
//! `tab-favorite-context-menus`).
//!
//! The `RightClickRow`/`RightClickFavorite` requests drive the exact
//! production app methods the sidebar row/favorite right-button mouse-down
//! listeners call (with a synthetic `MouseDownEvent` anchored at the given
//! pointer position), and the `TabMenu*` requests drive the exact app
//! methods the card's row hover/press listeners and the Escape/outside-press
//! close paths call. Every seam re-enters the app entity on the real event —
//! never a direct controller mutation.

use bbr_testkit::protocol::{Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the `Request::*TabMenu` arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_tab_menu(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::RightClickRow { index, x, y } => with_app(window, app, cx, |view, window, cx| {
            use gpui::{point, px, Modifiers, MouseButton, MouseDownEvent};
            view.open_tab_menu_from_row(
                *index,
                &MouseDownEvent {
                    button: MouseButton::Right,
                    position: point(px(*x), px(*y)),
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    first_mouse: true,
                },
                window,
                cx,
            );
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::RightClickFavorite { index, x, y } => {
            with_app(window, app, cx, |view, window, cx| {
                use gpui::{point, px, Modifiers, MouseButton, MouseDownEvent};
                view.open_tab_menu_from_favorite(
                    *index,
                    &MouseDownEvent {
                        button: MouseButton::Right,
                        position: point(px(*x), px(*y)),
                        modifiers: Modifiers::default(),
                        click_count: 1,
                        first_mouse: true,
                    },
                    window,
                    cx,
                );
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::RightClickSpaceChip { space, x, y } => {
            with_app(window, app, cx, |view, window, cx| {
                use gpui::{point, px, Modifiers, MouseButton, MouseDownEvent};
                view.open_space_menu(
                    bbr_domain::SpaceId(space.0),
                    &MouseDownEvent {
                        button: MouseButton::Right,
                        position: point(px(*x), px(*y)),
                        modifiers: Modifiers::default(),
                        click_count: 1,
                        first_mouse: true,
                    },
                    window,
                    cx,
                );
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::TabMenuSelect { index } => with_app(window, app, cx, |view, window, cx| {
            view.tab_menu_select(*index, true, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::TabMenuSubmenuSelect { index } => with_app(window, app, cx, |view, window, cx| {
            view.tab_menu_submenu_select(*index, true, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::TabMenuActivate { index } => with_app(window, app, cx, |view, window, cx| {
            view.on_tab_menu_row_press(*index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::TabMenuSubmenuActivate { index } => {
            with_app(window, app, cx, |view, window, cx| {
                view.on_tab_menu_submenu_press(*index, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::TabMenuClose => with_app(window, app, cx, |view, window, cx| {
            view.tab_menu_close(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::TabMenuOutsideClick => with_app(window, app, cx, |view, window, cx| {
            view.dismiss_tab_menu_on_outside_press(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}
