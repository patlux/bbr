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
#[cfg(unix)]
use smol::prelude::*;

use crate::actions::{
    AuthSheetFieldTab, Back, ClearBrowsingData, CloseSplit, CloseTab, CloseWindow, DuplicateTab,
    FindInPage, FocusCommandBar, FocusPaneDown, FocusPaneLeft, FocusPaneRight, FocusPaneUp,
    FocusRail, FocusWeb, Forward, MoveTab, NewSpace, NewTab, NewWindow, OpenOverlay, OpenPalette,
    OpenSavedPagesLibrary, PinTab, Reload, ReopenTab, ReorderTabDown, ReorderTabUp, SelectNextTab,
    SelectPreviousTab, ShowHistory, SplitBelow, SplitRight, ToggleBookmark, ToggleSidebar, ZoomIn,
    ZoomOut, ZoomReset,
};
use crate::app::BbrApp;
use crate::browser_windows::Registry;
use crate::shared::Shared;

/// Convert terminal Ctrl-C (`SIGINT`) into GPUI's graceful shutdown path.
///
/// The signal listener lives on GPUI's foreground executor, so the signal
/// handler itself remains async-signal-safe and every native object tears down
/// on the application/main thread through `App::quit`.
#[cfg(unix)]
pub(crate) fn install_sigint_shutdown(cx: &App) {
    let mut signals =
        async_signal::Signals::new([async_signal::Signal::Int]).expect("install SIGINT listener");
    cx.spawn(async move |async_cx| {
        if signals.next().await.transpose().is_ok() {
            async_cx.update(|cx| cx.quit());
        }
    })
    .detach();
}

#[cfg(not(unix))]
pub(crate) fn install_sigint_shutdown(_cx: &App) {}

/// Root-window + app-entity seam: run an update against a window's
/// separately retained `Entity<BbrApp>` shell with `&mut Window` access
/// through its `gpui_component::Root` window. Every action, pump, dispatch,
/// and presentation hook routes app mutation through this single contract.
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

/// Blueprint D19: run an update in window `key` (`None`: the current
/// window — the one that last took input).
pub(crate) fn with_window<R>(
    windows: &Shared<Registry>,
    key: Option<bbr_browser::WindowKey>,
    cx: &mut App,
    f: impl FnOnce(&mut BbrApp, &mut Window, &mut gpui::Context<BbrApp>) -> R,
) -> anyhow::Result<R> {
    let target = match key {
        Some(key) => windows.read().target(key),
        None => windows.read().current_target(),
    };
    let (handle, app) = target.ok_or_else(|| anyhow::anyhow!("no such browser window"))?;
    with_app(&handle, &app, cx, f)
}

/// Blueprint D19: run an update in the current window.
pub(crate) fn with_current<R>(
    windows: &Shared<Registry>,
    cx: &mut App,
    f: impl FnOnce(&mut BbrApp, &mut Window, &mut gpui::Context<BbrApp>) -> R,
) -> anyhow::Result<R> {
    with_window(windows, None, cx, f)
}

/// Blueprint D19: run a keyboard or menu command in the key browser window
/// (the current one when no browser window is key, e.g. the hidden test
/// harness); the key window becomes current, as any input in it does.
pub(crate) fn with_key_window<R>(
    windows: &Shared<Registry>,
    cx: &mut App,
    f: impl FnOnce(&mut BbrApp, &mut Window, &mut gpui::Context<BbrApp>) -> R,
) -> anyhow::Result<R> {
    let key = cx
        .active_window()
        .and_then(|active| windows.read().key_of(active.window_id()));
    if let Some(key) = key {
        if windows.read().current() != key {
            let _ = windows.bookkeep().set_current(key);
        }
    }
    with_current(windows, cx, f)
}

/// Blueprint D19: run an engine event's update in the window whose root
/// holds `view` (the current window for an unknown view).
pub(crate) fn with_view_window<R>(
    windows: &Shared<Registry>,
    view: Option<bbr_web::WebViewId>,
    cx: &mut App,
    f: impl FnOnce(&mut BbrApp, &mut Window, &mut gpui::Context<BbrApp>) -> R,
) -> anyhow::Result<R> {
    let key = view.and_then(|view| {
        let (_, app) = windows.read().current_target()?;
        app.read(cx).window_of_view(view)
    });
    let key = key.filter(|key| windows.read().target(*key).is_some());
    with_window(windows, key, cx, f)
}

/// Register global action handlers (bubble phase) that reach the app view.
///
/// These run while the keystroke dispatch already holds the window borrowed
/// (see [`gpui::Window::dispatch_keystroke`]), so a direct `win.update` would
/// re-enter the window update and fail with "window not found". The update is
/// therefore deferred to the effect flush — the same pattern GPUI's own
/// [`gpui::Window::dispatch_action`] uses — so the view is updated once the
/// keystroke dispatch has released the window.
pub(crate) fn install_actions(cx: &mut App, windows: Shared<Registry>) {
    // The registry handle is cloned per listener so every `cx.on_action`
    // closure is `'static`; each command runs in the key browser window.
    cx.on_action({
        let windows = windows.clone();
        move |_: &FocusCommandBar, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.activate_command_bar(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &FocusWeb, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.cancel_command_bar(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &Back, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, _cx| view.go_back());
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &Forward, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, _cx| view.go_forward());
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &Reload, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, _cx| view.reload());
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ZoomIn, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| view.zoom_in(cx));
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ZoomOut, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| view.zoom_out(cx));
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ZoomReset, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| view.zoom_reset(cx));
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ToggleSidebar, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.toggle_sidebar(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &OpenPalette, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.open_palette(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &OpenOverlay, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.open_overlay(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &FocusRail, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ =
                    with_key_window(&windows, cx, |view, window, cx| view.focus_rail(window, cx));
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &FindInPage, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ =
                    with_key_window(&windows, cx, |view, window, cx| view.find_open(window, cx));
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &AuthSheetFieldTab, cx| {
            // Read the presented state synchronously: the keystroke
            // dispatch holds only the window (the app entity is not
            // borrowed here), so the read decides whether Tab is consumed
            // before the deferred mutation runs. When no sheet is
            // presented the handler is a no-op and leaves propagation
            // open, so gpui-component's `Root` focus-traversal binding
            // keeps handling Tab exactly as before.
            // Blueprint D05: the Delete Space sheet shares the binding.
            let key = cx
                .active_window()
                .and_then(|active| windows.read().key_of(active.window_id()));
            let target = key
                .and_then(|key| windows.read().target(key))
                .or_else(|| windows.read().current_target());
            let Some((_, app)) = target else {
                return;
            };
            let view = app.read(cx);
            if !view.auth_sheet_is_presented() && !view.space_delete_open() {
                return;
            }
            // The sheet owns the keyboard while open: consume Tab so the
            // gpui-component `Root` focus-traversal binding cannot move
            // focus out of the sheet, then move the sheet's field cursor
            // on the effect flush (the keystroke dispatch still holds the
            // window borrowed, so a direct `with_app` would re-enter the
            // window update).
            cx.stop_propagation();
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.auth_field_tab(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ToggleBookmark, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.toggle_bookmark(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &OpenSavedPagesLibrary, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.open_saved_pages_library(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ShowHistory, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.open_history(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ClearBrowsingData, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.open_clear_browsing_data(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &NewTab, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.activate_new_tab(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &NewSpace, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.activate_new_space(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &PinTab, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| {
                    let _ = view.pin_active_tab(cx);
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ReorderTabUp, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| {
                    let _ = view.reorder_active_tab(-1, cx);
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ReorderTabDown, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| {
                    let _ = view.reorder_active_tab(1, cx);
                });
            });
        }
    });
    // Blueprint D23: split commands.
    for (edge, below) in [
        (bbr_domain::tiling::Edge::Right, false),
        (bbr_domain::tiling::Edge::Bottom, true),
    ] {
        let windows = windows.clone();
        let run = move |cx: &mut App| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.begin_split_palette(edge, window, cx);
                });
            });
        };
        if below {
            cx.on_action(move |_: &SplitBelow, cx| run(cx));
        } else {
            cx.on_action(move |_: &SplitRight, cx| run(cx));
        }
    }
    cx.on_action({
        let windows = windows.clone();
        move |_: &CloseSplit, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| {
                    view.dissolve_shown_split(cx);
                });
            });
        }
    });
    for edge in [
        bbr_domain::tiling::Edge::Left,
        bbr_domain::tiling::Edge::Right,
        bbr_domain::tiling::Edge::Top,
        bbr_domain::tiling::Edge::Bottom,
    ] {
        let windows = windows.clone();
        let run = move |cx: &mut App| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| {
                    view.split_focus_neighbour(edge, cx);
                });
            });
        };
        let _ = match edge {
            bbr_domain::tiling::Edge::Left => cx.on_action(move |_: &FocusPaneLeft, cx| run(cx)),
            bbr_domain::tiling::Edge::Right => cx.on_action(move |_: &FocusPaneRight, cx| run(cx)),
            bbr_domain::tiling::Edge::Top => cx.on_action(move |_: &FocusPaneUp, cx| run(cx)),
            bbr_domain::tiling::Edge::Bottom => cx.on_action(move |_: &FocusPaneDown, cx| run(cx)),
        };
    }
    // Blueprint D19: Duplicate Tab and Select Next/Previous Tab.
    cx.on_action({
        let windows = windows.clone();
        move |_: &DuplicateTab, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| {
                    let _ = view.duplicate_active_tab(cx);
                });
            });
        }
    });
    for (delta, next) in [(1isize, true), (-1, false)] {
        let windows = windows.clone();
        let run = move |cx: &mut App| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _w, cx| {
                    let _ = view.select_adjacent_tab(delta, cx);
                });
            });
        };
        if next {
            cx.on_action(move |_: &SelectNextTab, cx| run(cx));
        } else {
            cx.on_action(move |_: &SelectPreviousTab, cx| run(cx));
        }
    }
    cx.on_action({
        let windows = windows.clone();
        move |_: &MoveTab, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    view.activate_move_tab(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &CloseTab, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    let _ = view.close_active_tab(window, cx);
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &ReopenTab, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    let _ = view.reopen_tab(window, cx);
                });
            });
        }
    });

    // Blueprint D19: the menu-bar-only commands and the space activation
    // the Spaces and Dock menus dispatch.
    cx.on_action({
        let windows = windows.clone();
        move |_: &crate::menus::Quit, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_current(&windows, cx, |view, _window, _cx| {
                    view.flush_browser_state();
                    view.flush_saved_pages();
                });
                cx.quit();
            });
        }
    });
    cx.on_action(|_: &crate::menus::HideApp, cx| cx.hide());
    // Blueprint D19: several windows.
    cx.on_action({
        let windows = windows.clone();
        move |_: &NewWindow, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| crate::browser_windows::new_window(cx, &windows));
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &CloseWindow, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                // The key window closes (the current one under the hidden
                // harness).
                let key = cx
                    .active_window()
                    .and_then(|active| windows.read().key_of(active.window_id()));
                if let Some(key) = key {
                    if windows.read().current() != key {
                        let _ = windows.bookkeep().set_current(key);
                    }
                }
                crate::browser_windows::close_window(cx, &windows);
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &crate::menus::ShowSettings, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _window, cx| view.show_settings(cx));
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &crate::menus::ShowAbout, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, _window, cx| view.show_about(cx));
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |_: &crate::menus::MinimizeWindow, cx| {
            let windows = windows.clone();
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |_view, window, _cx| {
                    window.minimize_window();
                });
            });
        }
    });
    cx.on_action({
        let windows = windows.clone();
        move |action: &crate::menus::ActivateSpace, cx| {
            let windows = windows.clone();
            let space = bbr_browser::SpaceId(action.id);
            cx.defer(move |cx| {
                let _ = with_key_window(&windows, cx, |view, window, cx| {
                    let _ = view.activate_space(space, window, cx);
                });
            });
        }
    });

    cx.bind_keys(keybindings());
}

/// The bounded renderer-availability cadence (renderer-isolation lane): a
/// main-thread periodic task that advances the portable liveness probes of
/// the visible sessions (`BrowserController::pump_responsiveness` — arm
/// due probes, apply the fail-closed deadline, report transitions) and
/// re-asserts the §11.8a pane visibility. Never blocks on the renderer:
/// the task sleeps on the platform dispatcher between ticks and the pump
/// itself is a bounded registry sweep that arms nothing on an idle window.
/// GPUI's platform-driven `background_executor().timer` is required (a
/// foreground-spawned task is polled by the main-thread executor, which
/// never drives smol's reactor — see the command-debounce precedent).
pub(crate) fn spawn_responsiveness_cadence(cx: &App, windows: &Shared<Registry>) {
    // The monitor's re-arm cadence is ~1 s (probe_interval) and the
    // fail-closed deadline 2 s; a 250 ms tick detects a due probe or an
    // expired deadline within a quarter second while emitting nothing on an
    // idle window.
    const TICK: std::time::Duration = std::time::Duration::from_millis(250);
    let windows = windows.clone();
    cx.spawn(async move |async_cx: &mut gpui::AsyncApp| loop {
        async_cx.background_executor().timer(TICK).await;
        // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
        async_cx.update(|cx| {
            // Blueprint D19: the shared probes run once, in the current
            // window; every other window re-asserts its own pane.
            let _ = with_current(&windows, cx, |view, window, cx| {
                view.pump_responsiveness_cadence(window, cx)
            });
            let current = windows.read().current();
            let others: Vec<_> = windows
                .read()
                .keys()
                .into_iter()
                .filter(|key| *key != current)
                .collect();
            for key in others {
                let _ = with_window(&windows, Some(key), cx, |view, _window, _cx| {
                    view.sync_unresponsive_visibility()
                });
            }
        });
    })
    .detach();
}

/// The channels the production event pump drains.
///
/// Grouped rather than passed individually: every capability that reaches the
/// app from a native delegate adds a receiver here, so an argument list would
/// grow without bound and each addition would touch the same signature. The
/// struct keeps the additions additive and names each source at the call site.
#[cfg(not(feature = "testkit"))]
pub(crate) struct EventPumpChannels {
    pub(crate) events: Receiver<WebEvent>,
    #[cfg(target_os = "macos")]
    pub(crate) find: Receiver<bbr_web_macos::find_mapping::FindCompletion>,
    #[cfg(target_os = "macos")]
    pub(crate) downloads: Receiver<bbr_web::downloads::DownloadEvent>,
    /// External-URL open deliveries (queue item 13): buffered URLs from the
    /// AppKit delegate path, drained on the main thread into
    /// `BbrApp::on_external_url`.
    #[cfg(target_os = "macos")]
    pub(crate) external_urls: Receiver<String>,
    /// Classified web-error incidents from the native delegate's incident
    /// sink, drained into `BbrApp::on_incident`.
    #[cfg(target_os = "macos")]
    pub(crate) incidents: Receiver<bbr_web_macos::error_mapping::ClassifiedIncident>,
    /// Fully decided new-window deliveries from the WKUI delegate.
    #[cfg(target_os = "macos")]
    pub(crate) new_windows: Receiver<bbr_web::NewWindowDelivery>,
    /// Committed logical content gestures from the native content-gesture
    /// bridge (N1), drained into `BbrApp::on_content_gesture_commit`.
    #[cfg(target_os = "macos")]
    pub(crate) gestures: Receiver<bbr_web::ContentGestureCommit>,
}

/// Non-testkit builds still need the event pump to keep the shell live.
#[cfg(not(feature = "testkit"))]
pub(crate) fn spawn_event_pump(cx: &App, windows: &Shared<Registry>, channels: EventPumpChannels) {
    let evt_rx = channels.events;
    #[cfg(target_os = "macos")]
    let find_rx = channels.find;
    #[cfg(target_os = "macos")]
    let downloads_rx = channels.downloads;
    #[cfg(target_os = "macos")]
    let external_url_rx = channels.external_urls;
    #[cfg(target_os = "macos")]
    let incidents_rx = channels.incidents;
    #[cfg(target_os = "macos")]
    let new_windows_rx = channels.new_windows;
    #[cfg(target_os = "macos")]
    let gestures_rx = channels.gestures;
    let windows = windows.clone();
    cx.spawn(async move |async_cx: &mut gpui::AsyncApp| loop {
        enum Incoming {
            Event(Result<WebEvent, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            Find(Result<bbr_web_macos::find_mapping::FindCompletion, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            Download(Result<bbr_web::downloads::DownloadEvent, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            ExternalUrl(Result<String, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            Incident(
                Result<bbr_web_macos::error_mapping::ClassifiedIncident, smol::channel::RecvError>,
            ),
            #[cfg(target_os = "macos")]
            NewWindow(Result<bbr_web::NewWindowDelivery, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            ContentGesture(Result<bbr_web::ContentGestureCommit, smol::channel::RecvError>),
        }
        let incoming = smol::future::race(
            async { Incoming::Event(evt_rx.recv().await) },
            smol::future::race(
                smol::future::race(
                    #[cfg(target_os = "macos")]
                    async {
                        Incoming::Find(find_rx.recv().await)
                    },
                    #[cfg(target_os = "macos")]
                    async {
                        Incoming::Download(downloads_rx.recv().await)
                    },
                ),
                smol::future::race(
                    #[cfg(target_os = "macos")]
                    async {
                        Incoming::ExternalUrl(external_url_rx.recv().await)
                    },
                    smol::future::race(
                        #[cfg(target_os = "macos")]
                        async {
                            Incoming::Incident(incidents_rx.recv().await)
                        },
                        smol::future::race(
                            #[cfg(target_os = "macos")]
                            async {
                                Incoming::NewWindow(new_windows_rx.recv().await)
                            },
                            #[cfg(target_os = "macos")]
                            async {
                                Incoming::ContentGesture(gestures_rx.recv().await)
                            },
                        ),
                    ),
                ),
            ),
        )
        .await;
        match incoming {
            Incoming::Event(Ok(ev)) => {
                async_cx.update(|cx| {
                    let _ = with_view_window(&windows, Some(ev.view()), cx, |view, window, cx| {
                        view.on_web_event(ev, window, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Find(Ok(completion)) => {
                async_cx.update(|cx| {
                    // The first window's find channel (every further window
                    // pumps its own).
                    let _ = with_window(
                        &windows,
                        Some(bbr_browser::WindowKey::default()),
                        cx,
                        |view, _w, cx| view.on_find_completion(completion, cx),
                    );
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Download(Ok(event)) => {
                // L6 download events from the native adapter: route through
                // the same main-thread app update path as engine events.
                async_cx.update(|cx| {
                    let _ = with_current(&windows, cx, |view, _w, cx| {
                        crate::app::downloads_behavior::on_download_event(view, event, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::ExternalUrl(Ok(url)) => {
                // External-URL open from the AppKit delegate path: the same
                // main-thread app update path as engine events. URLs buffered
                // before the app finished launching are drained here, in
                // order, so a cold-start link lands exactly once.
                async_cx.update(|cx| {
                    let _ = with_key_window(&windows, cx, |view, w, cx| {
                        view.on_external_url(url, w, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Incident(Ok(incident)) => {
                // Classified web-error incidents from the native delegate's
                // incident sink: the same main-thread app update path as
                // engine/find/downloads events.
                async_cx.update(|cx| {
                    let view = Some(incident.web_view);
                    let _ = with_view_window(&windows, view, cx, |view, _w, cx| {
                        view.on_incident(incident, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::NewWindow(Ok(delivery)) => {
                async_cx.update(|cx| {
                    let view = Some(delivery.request.source);
                    let _ = with_view_window(&windows, view, cx, |view, window, cx| {
                        view.on_new_window_request(delivery, window, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::ContentGesture(Ok(commit)) => {
                async_cx.update(|cx| {
                    let view = Some(commit.view);
                    let _ = with_view_window(&windows, view, cx, |view, _w, cx| {
                        view.on_content_gesture_commit(commit, cx)
                    });
                });
            }
            Incoming::Event(Err(_)) => break,
            #[cfg(target_os = "macos")]
            Incoming::Find(Err(_))
            | Incoming::Download(Err(_))
            | Incoming::ExternalUrl(Err(_))
            | Incoming::Incident(Err(_))
            | Incoming::NewWindow(Err(_))
            | Incoming::ContentGesture(Err(_)) => break,
        }
    })
    .detach();
}

/// The browser's key bindings (one list: the keymap and the Settings ›
/// Shortcuts pane's check read it).
pub(crate) fn keybindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("cmd-q", crate::menus::Quit, None),
        KeyBinding::new("cmd-h", crate::menus::HideApp, None),
        KeyBinding::new("cmd-m", crate::menus::MinimizeWindow, None),
        KeyBinding::new("cmd-f", FindInPage, None),
        KeyBinding::new("alt-cmd-s", FocusRail, None),
        KeyBinding::new("cmd-l", OpenOverlay, None),
        KeyBinding::new("escape", FocusWeb, None),
        KeyBinding::new("cmd-[", Back, None),
        KeyBinding::new("cmd-]", Forward, None),
        KeyBinding::new("cmd-r", Reload, None),
        KeyBinding::new("cmd-s", ToggleSidebar, None),
        KeyBinding::new("cmd-t", OpenPalette, None),
        KeyBinding::new("cmd-shift-n", NewSpace, None),
        KeyBinding::new("cmd-shift-p", PinTab, None),
        KeyBinding::new("cmd-shift-up", ReorderTabUp, None),
        KeyBinding::new("cmd-shift-down", ReorderTabDown, None),
        KeyBinding::new("ctrl-tab", SelectNextTab, None),
        KeyBinding::new("ctrl-shift-tab", SelectPreviousTab, None),
        // Blueprint D23: split commands.
        KeyBinding::new("ctrl-cmd-s", SplitRight, None),
        KeyBinding::new("ctrl-alt-cmd-s", SplitBelow, None),
        KeyBinding::new("ctrl-shift-cmd-s", CloseSplit, None),
        KeyBinding::new("ctrl-cmd-left", FocusPaneLeft, None),
        KeyBinding::new("ctrl-cmd-right", FocusPaneRight, None),
        KeyBinding::new("ctrl-cmd-up", FocusPaneUp, None),
        KeyBinding::new("ctrl-cmd-down", FocusPaneDown, None),
        KeyBinding::new("cmd-shift-m", MoveTab, None),
        KeyBinding::new("cmd-w", CloseTab, None),
        // Blueprint D19: several windows.
        KeyBinding::new("cmd-n", NewWindow, None),
        KeyBinding::new("cmd-shift-w", CloseWindow, None),
        KeyBinding::new("cmd-shift-t", ReopenTab, None),
        // Zoom (queue item 11, part A): `cmd-+` is the literal Cmd-Plus on
        // the US layout (and the shifted-plus layouts GPUI resolves to the
        // `+` key); `cmd-=` is the equals-key alias browsers ship; `cmd--`
        // is Cmd-Minus; `cmd-0` resets.
        KeyBinding::new("cmd-+", ZoomIn, None),
        KeyBinding::new("cmd-=", ZoomIn, None),
        KeyBinding::new("cmd--", ZoomOut, None),
        KeyBinding::new("cmd-0", ZoomReset, None),
        // Cmd-D toggles the active page's saved state (B2 packet
        // `saved-pages-actions-command`; conventional macOS bookmark
        // shortcut).
        KeyBinding::new("cmd-d", ToggleBookmark, None),
        // Cmd-Shift-L opens the Saved Pages Library (B3 packet
        // `saved-pages-library`; DESIGN.md §11.16).
        KeyBinding::new("cmd-shift-l", OpenSavedPagesLibrary, None),
        // Blueprint D10: History and Clear Browsing Data.
        KeyBinding::new("cmd-y", ShowHistory, None),
        KeyBinding::new("cmd-shift-backspace", ClearBrowsingData, None),
        // App-wide (context-less) bindings resolve at the deepest depth, so
        // they outrank gpui-component `Root`'s element-scoped `tab`/`shift-tab`
        // focus-traversal bindings: Tab moves the credential sheet's field
        // cursor while the sheet is open (the handler consumes it) and falls
        // through to focus traversal otherwise (the no-op contract).
        KeyBinding::new("tab", AuthSheetFieldTab, None),
        KeyBinding::new("shift-tab", AuthSheetFieldTab, None),
        KeyBinding::new("cmd-,", crate::menus::ShowSettings, None),
    ]
}
