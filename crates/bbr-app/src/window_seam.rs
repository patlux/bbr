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
    AuthSheetFieldTab, Back, CloseTab, FindInPage, FocusCommandBar, FocusRail, FocusWeb, Forward,
    MoveTab, NewSpace, NewTab, OpenOverlay, OpenPalette, OpenSavedPagesLibrary, PinTab, Reload,
    ReopenTab, ReorderTabDown, ReorderTabUp, ToggleBookmark, ToggleSidebar, ZoomIn, ZoomOut,
    ZoomReset,
};
use crate::app::BbrApp;
/// Root-window + app-entity seam: run an update against the separately
/// retained `Entity<BbrApp>` with `&mut Window` access through the
/// `gpui_component::Root` window. Every action, pump, dispatch, and
/// presentation hook routes app mutation through this single contract.
/// Terminate BBR when its browser window closes.
///
/// GPUI follows the native macOS convention by keeping an application alive
/// after its last window closes. BBR is intentionally a single-browser-window
/// process, so closing that owning window is a full application shutdown even
/// while implementation-owned child windows (such as the native portal host)
/// still exist.
///
/// Normal-quit persistence correctness: the owning window-close seam flushes
/// the latest structural browser model and the current Saved Page provider
/// before the app quits (each one synchronous transaction through the
/// identical repository write the background writer uses). This is a narrow
/// persistence seam, not a general deterministic process-shutdown system
/// (queue item 20 stays out of scope); the force-quit and crash paths
/// recover exactly the last committed state.
///
/// True only when the closed window is BBR's owning browser window.
///
/// The one implementation-owned child window (the native portal host)
/// carries a distinct [`gpui::WindowId`] and must never satisfy the
/// process-exit rule. The find bar and auth sheet are in-window elements
/// and never produce an independent window-closed event. Closing a child
/// is a dismissal, closing the owning browser window is full application
/// shutdown. The predicate is pure so the owning-window rule is
/// unit-testable without a display server.
pub(crate) fn is_owning_window(closed: gpui::WindowId, owning: gpui::WindowId) -> bool {
    closed == owning
}

pub(crate) fn quit_when_browser_window_closes(
    cx: &App,
    browser_window: gpui::WindowHandle<gpui_component::Root>,
    app: Entity<BbrApp>,
) {
    let browser_window_id = browser_window.window_id();
    cx.on_window_closed(move |cx, closed_window_id| {
        if is_owning_window(closed_window_id, browser_window_id) {
            // The app entity outlives its window (retained by the
            // composition root), so both flushes run before the quit in the
            // same update: the structural browser model and the Saved Page
            // provider are each persisted in one synchronous transaction.
            app.update(cx, |view, _cx| {
                view.flush_browser_state();
                view.flush_saved_pages();
            });
            cx.quit();
        }
    })
    .detach();
}

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
        move |_: &ZoomIn, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, cx| view.zoom_in(cx));
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &ZoomOut, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, cx| view.zoom_out(cx));
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &ZoomReset, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, _w, cx| view.zoom_reset(cx));
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
        move |_: &OpenPalette, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.open_palette(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &OpenOverlay, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.open_overlay(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &FocusRail, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.focus_rail(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &FindInPage, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.find_open(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &AuthSheetFieldTab, cx| {
            // Read the presented state synchronously: the keystroke
            // dispatch holds only the window (the app entity is not
            // borrowed here), so the read decides whether Tab is consumed
            // before the deferred mutation runs. When no sheet is
            // presented the handler is a no-op and leaves propagation
            // open, so gpui-component's `Root` focus-traversal binding
            // keeps handling Tab exactly as before.
            // Blueprint D05: the Delete Space sheet shares the binding.
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
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.auth_field_tab(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &ToggleBookmark, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.toggle_bookmark(window, cx)
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |_: &OpenSavedPagesLibrary, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    view.open_saved_pages_library(window, cx)
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

    // Blueprint D19: the menu-bar-only commands and the space activation
    // the Spaces and Dock menus dispatch.
    cx.on_action({
        let app = app.clone();
        move |_: &crate::menus::Quit, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                app.update(cx, |view, _cx| {
                    view.flush_browser_state();
                    view.flush_saved_pages();
                });
                cx.quit();
            });
        }
    });
    cx.on_action(|_: &crate::menus::HideApp, cx| cx.hide());
    cx.on_action({
        let app = app.clone();
        move |_: &crate::menus::MinimizeWindow, cx| {
            let app = app.clone();
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |_view, window, _cx| {
                    window.minimize_window();
                });
            });
        }
    });
    cx.on_action({
        let app = app.clone();
        move |action: &crate::menus::ActivateSpace, cx| {
            let app = app.clone();
            let space = bbr_browser::SpaceId(action.id);
            cx.defer(move |cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    let _ = view.activate_space(space, window, cx);
                });
            });
        }
    });

    cx.bind_keys([
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
        KeyBinding::new("cmd-b", ToggleSidebar, None),
        KeyBinding::new("cmd-t", OpenPalette, None),
        KeyBinding::new("cmd-shift-n", NewSpace, None),
        KeyBinding::new("cmd-shift-p", PinTab, None),
        KeyBinding::new("cmd-shift-up", ReorderTabUp, None),
        KeyBinding::new("cmd-shift-down", ReorderTabDown, None),
        KeyBinding::new("cmd-shift-m", MoveTab, None),
        KeyBinding::new("cmd-w", CloseTab, None),
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
        // App-wide (context-less) bindings resolve at the deepest depth, so
        // they outrank gpui-component `Root`'s element-scoped `tab`/`shift-tab`
        // focus-traversal bindings: Tab moves the credential sheet's field
        // cursor while the sheet is open (the handler consumes it) and falls
        // through to focus traversal otherwise (the no-op contract).
        KeyBinding::new("tab", AuthSheetFieldTab, None),
        KeyBinding::new("shift-tab", AuthSheetFieldTab, None),
    ]);
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
pub(crate) fn spawn_responsiveness_cadence(
    cx: &App,
    window: &gpui::WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
) {
    // The monitor's re-arm cadence is ~1 s (probe_interval) and the
    // fail-closed deadline 2 s; a 250 ms tick detects a due probe or an
    // expired deadline within a quarter second while emitting nothing on an
    // idle window.
    const TICK: std::time::Duration = std::time::Duration::from_millis(250);
    let window = *window;
    let app = app.clone();
    cx.spawn(async move |async_cx: &mut gpui::AsyncApp| loop {
        async_cx.background_executor().timer(TICK).await;
        // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
        async_cx.update(|cx| {
            let _ = with_app(&window, &app, cx, |view, window, cx| {
                view.pump_responsiveness_cadence(window, cx)
            });
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
pub(crate) fn spawn_event_pump(
    cx: &App,
    window: &gpui::WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    channels: EventPumpChannels,
) {
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
    let window = *window;
    let app = app.clone();
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
                    let _ = with_app(&window, &app, cx, |view, window, cx| {
                        view.on_web_event(ev, window, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Find(Ok(completion)) => {
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        view.on_find_completion(completion, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Download(Ok(event)) => {
                // L6 download events from the native adapter: route through
                // the same main-thread app update path as engine events.
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
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
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        view.on_external_url(url, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Incident(Ok(incident)) => {
                // Classified web-error incidents from the native delegate's
                // incident sink: the same main-thread app update path as
                // engine/find/downloads events.
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        view.on_incident(incident, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::NewWindow(Ok(delivery)) => {
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, window, cx| {
                        view.on_new_window_request(delivery, window, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::ContentGesture(Ok(commit)) => {
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::WindowId;

    #[test]
    fn owning_window_close_is_a_process_quit_signal() {
        let owning = WindowId::from(7);
        assert!(
            is_owning_window(owning, owning),
            "closing the owning browser window must satisfy the quit rule"
        );
    }

    #[test]
    fn portal_child_window_never_satisfies_the_quit_rule() {
        let owning = WindowId::from(7);
        // The one native portal host (and any other implementation-owned
        // child window) has its own WindowId: closing it is a dismissal,
        // never a process quit.
        let portal_child = WindowId::from(8);
        assert!(
            !is_owning_window(portal_child, owning),
            "a portal child window id must not terminate the process"
        );
        assert!(
            !is_owning_window(WindowId::from(9), owning),
            "an unrelated window id must not terminate the process"
        );
    }

    #[test]
    fn distinct_window_ids_never_match() {
        let a = WindowId::from(1);
        let b = WindowId::from(2);
        assert_ne!(a, b);
        assert!(!is_owning_window(b, a));
        assert!(!is_owning_window(a, b));
    }
}
