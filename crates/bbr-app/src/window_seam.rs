//! Root-window + app-entity seam for the `gpui_component::Root` window.
//!
//! The browser window's root view is `gpui_component::Root`; the `BbrApp`
//! entity is retained separately so actions, pumps, dispatch, and
//! presentation hooks address the app directly. This module owns the single
//! `with_app` contract and the keyboard action/event wiring that routes
//! through it, keeping the composition root and the gateway router focused.

#[cfg(not(feature = "testkit"))]
use bbr_web::WebEvent;
use gpui::{App, Entity, KeyBinding, Window};
#[cfg(not(feature = "testkit"))]
use smol::channel::Receiver;

use crate::actions::{
    Back, CloseTab, FocusCommandBar, FocusWeb, Forward, MoveTab, NewSpace, NewTab, PinTab, Reload,
    ReopenTab, ReorderTabDown, ReorderTabUp, ToggleSidebar,
};
use crate::app::BbrApp;

/// Root-window + app-entity seam: run an update against the separately
/// retained `Entity<BbrApp>` with `&mut Window` access through the
/// `gpui_component::Root` window. Every action, pump, dispatch, and
/// presentation hook routes app mutation through this single contract.
pub(crate) fn with_app<R>(
    window: &gpui::WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
    f: impl FnOnce(&mut BbrApp, &mut Window, &mut gpui::Context<BbrApp>) -> R,
) -> anyhow::Result<R> {
    window.update(cx, |_root, window, cx| {
        app.update(cx, |view, cx| f(view, window, cx))
    })
}

/// Register global action handlers (bubble phase) that reach the app view.
///
/// These run while the keystroke dispatch already holds the window borrowed
/// (see [`gpui::Window::dispatch_keystroke`]), so a direct `win.update` would
/// re-enter the window update and fail with "window not found". The update is
/// therefore deferred to the effect flush — the same pattern GPUI's own
/// [`gpui::Window::dispatch_action`] uses — so the view is updated once the
/// keystroke dispatch has released the window.
pub(crate) fn install_actions(
    cx: &mut App,
    window: gpui::WindowHandle<gpui_component::Root>,
    app: Entity<BbrApp>,
) {
    // The window handle is `Copy`; the app entity is cloned per listener so
    // every `cx.on_action` closure is `'static`.
    cx.on_action({
        let app = app.clone();
        move |_: &FocusCommandBar, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.activate_command_bar(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &FocusWeb, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.cancel_command_bar(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &Back, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, _cx| view.go_back());
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &Forward, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, _cx| view.go_forward());
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &Reload, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, _cx| view.reload());
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &ToggleSidebar, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.toggle_sidebar(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &NewTab, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.activate_new_tab(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &NewSpace, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.activate_new_space(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &PinTab, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, cx| {
                    let _ = view.pin_active_tab(cx);
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &ReorderTabUp, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, cx| {
                    let _ = view.reorder_active_tab(-1, cx);
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &ReorderTabDown, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, cx| {
                    let _ = view.reorder_active_tab(1, cx);
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &MoveTab, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.activate_move_tab(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &CloseTab, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    let _ = view.close_active_tab(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &ReopenTab, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    let _ = view.reopen_tab(window, cx);
                });
            });
        }
    });

    cx.bind_keys([
        KeyBinding::new("cmd-l", FocusCommandBar, None),
        KeyBinding::new("escape", FocusWeb, None),
        KeyBinding::new("cmd-[", Back, None),
        KeyBinding::new("cmd-]", Forward, None),
        KeyBinding::new("cmd-r", Reload, None),
        KeyBinding::new("cmd-b", ToggleSidebar, None),
        KeyBinding::new("cmd-t", NewTab, None),
        KeyBinding::new("cmd-shift-n", NewSpace, None),
        KeyBinding::new("cmd-shift-p", PinTab, None),
        KeyBinding::new("cmd-shift-up", ReorderTabUp, None),
        KeyBinding::new("cmd-shift-down", ReorderTabDown, None),
        KeyBinding::new("cmd-shift-m", MoveTab, None),
        KeyBinding::new("cmd-w", CloseTab, None),
        KeyBinding::new("cmd-shift-t", ReopenTab, None),
    ]);
}

/// Non-testkit builds still need the event pump to keep the shell live.
#[cfg(not(feature = "testkit"))]
pub(crate) fn spawn_event_pump(
    cx: &App,
    window: &gpui::WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    evt_rx: Receiver<WebEvent>,
) {
    let window = *window;
    let app = app.clone();
    cx.spawn(async move |async_cx: &mut gpui::AsyncApp| loop {
        let ev = match evt_rx.recv().await {
            Ok(ev) => ev,
            Err(_) => break,
        };
        // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
        async_cx.update(|cx| {
            let _ = with_app(&window, &app, cx, |view, _w, cx| view.on_web_event(ev, cx));
        });
    })
    .detach();
}
