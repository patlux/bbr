//! Blueprint D19 (several windows, task packet `multi-window.md`): the
//! browser windows and their lifecycle.
//!
//! Every browser window is a GPUI `Root` over its own `BbrApp` shell; the
//! shells share the browser state (see `crate::shared`). The [`Registry`]
//! maps each window's key to its handle and shell and remembers the window
//! that last took input — engine events, timers and the testkit act there.
//! ⌘N opens a window on the current space with New Tab ready; closing a
//! window (its close button or ⇧⌘W) keeps every tab; closing the last one
//! saves everything and quits, and a relaunch restores it.

use std::collections::BTreeMap;

use bbr_browser::WindowKey;
use gpui::{
    prelude::*, px, size, App, Bounds, Entity, TitlebarOptions, WindowBounds, WindowHandle,
    WindowId, WindowOptions,
};

use crate::app::BbrApp;
use crate::shared::Shared;
use crate::window_seam::with_app;

/// One open browser window.
struct BrowserWindow {
    handle: WindowHandle<gpui_component::Root>,
    app: Entity<BbrApp>,
}

/// One window's presentation as the store keeps it: its content rect
/// (`[x, y, width, height]`, points) and its rail.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WindowRow {
    pub(crate) geometry: Option<[f64; 4]>,
    pub(crate) sidebar: bbr_persistence::SidebarState,
}

/// Every open browser window, the one that last took input, and each
/// window's presentation row (published when it draws).
#[derive(Default)]
pub(crate) struct Registry {
    windows: BTreeMap<WindowKey, BrowserWindow>,
    current: WindowKey,
    rows: BTreeMap<WindowKey, WindowRow>,
}

/// A window's handle and shell.
pub(crate) type Target = (WindowHandle<gpui_component::Root>, Entity<BbrApp>);

impl Registry {
    pub(crate) fn insert(
        &mut self,
        key: WindowKey,
        handle: WindowHandle<gpui_component::Root>,
        app: Entity<BbrApp>,
    ) {
        self.windows.insert(key, BrowserWindow { handle, app });
    }

    fn remove(&mut self, key: WindowKey) -> Option<BrowserWindow> {
        self.rows.remove(&key);
        let removed = self.windows.remove(&key);
        if self.current == key {
            if let Some(next) = self.windows.keys().next() {
                self.current = *next;
            }
        }
        removed
    }

    /// The window that last took input.
    pub(crate) fn current(&self) -> WindowKey {
        self.current
    }

    /// Make `key` the window engine events, timers and the testkit act in
    /// (false for an unknown window).
    pub(crate) fn set_current(&mut self, key: WindowKey) -> bool {
        if !self.windows.contains_key(&key) {
            return false;
        }
        self.current = key;
        true
    }

    pub(crate) fn target(&self, key: WindowKey) -> Option<Target> {
        let window = self.windows.get(&key)?;
        Some((window.handle, window.app.clone()))
    }

    /// The current window (the first open one if the current one is gone).
    pub(crate) fn current_target(&self) -> Option<Target> {
        self.target(self.current).or_else(|| {
            let window = self.windows.values().next()?;
            Some((window.handle, window.app.clone()))
        })
    }

    pub(crate) fn key_of(&self, id: WindowId) -> Option<WindowKey> {
        self.windows
            .iter()
            .find(|(_, window)| window.handle.window_id() == id)
            .map(|(key, _)| *key)
    }

    /// Every open window, in key order.
    pub(crate) fn keys(&self) -> Vec<WindowKey> {
        self.windows.keys().copied().collect()
    }

    /// `key`'s presentation row as it last drew.
    pub(crate) fn row(&self, key: WindowKey) -> Option<WindowRow> {
        self.rows.get(&key).copied()
    }

    pub(crate) fn set_row(&mut self, key: WindowKey, row: WindowRow) {
        self.rows.insert(key, row);
    }

    /// Every window's presentation row, in window order.
    pub(crate) fn rows(&self) -> Vec<WindowRow> {
        self.rows.values().copied().collect()
    }

    /// Every open window's shell.
    pub(crate) fn shells(&self) -> Vec<Entity<BbrApp>> {
        self.windows
            .values()
            .map(|window| window.app.clone())
            .collect()
    }
}

/// A browser window's options at `bounds` (its content rect).
pub(crate) fn browser_window_options(bounds: Bounds<gpui::Pixels>, hidden: bool) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        // The production chrome owns the complete top rail. GPUI's
        // public full-size-content titlebar keeps the native traffic
        // lights but lets the light sidebar material continue behind
        // them instead of exposing a separate white system strip.
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: None,
        }),
        focus: !hidden,
        // `--bbr-hidden` (every E2E harness) must never order the window
        // front: a shown window is visible to the host's window manager
        // (yabai BSP on this host tiles it to the current space's tile
        // frame — 1213x1063 here — so the frame is no longer the
        // requested 1080x700 and geometry suites flake depending on the
        // manager's state). A never-shown window keeps the requested
        // bounds deterministically; WebKit and GPUI rendering work
        // without display (frames are driven by the testkit FlushFrame
        // path).
        show: !hidden,
        ..Default::default()
    }
}

/// Register the first window and install the app-wide lifecycle: a closed
/// browser window's shell and context go (its tabs stay in their spaces);
/// closing the last one saves everything and quits.
///
/// Only registered browser windows count: the native Portal host, the
/// Settings and About windows are never browser windows, so closing them
/// never quits (and the Portal never keeps the process alive).
pub(crate) fn install_lifecycle(
    cx: &mut App,
    handle: WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
) {
    let windows = app.read(cx).window_registry();
    let key = app.read(cx).window_key();
    windows.get().insert(key, handle, app.clone());
    install_window_hooks(cx, handle, app);
    cx.on_window_closed(move |cx, closed| {
        let key = windows.read().key_of(closed);
        let Some(key) = key else {
            return;
        };
        if windows.read().keys() == [key] {
            // The last browser window: it stays registered for the save
            // (its row and selections are written), the app entity outlives
            // its window (held here), and the structural model and the
            // Saved Page provider are each persisted in one synchronous
            // transaction before the quit.
            let target = windows.read().target(key);
            if let Some((_, app)) = target {
                app.update(cx, |view, _cx| {
                    view.flush_browser_state();
                    view.flush_saved_pages();
                });
            }
            cx.quit();
            return;
        }
        let removed = windows.get().remove(key);
        let Some(closed_window) = removed else {
            return;
        };
        let remaining = windows.read().shells();
        let successor = windows.read().current_target();
        closed_window
            .app
            .update(cx, |view, cx| view.window_closed(successor, cx));
        for shell in remaining {
            shell.update(cx, |_view, cx| cx.notify());
        }
    })
    .detach();
}

/// One browser window's hooks: activating it makes it the current window.
fn install_window_hooks(
    cx: &mut App,
    handle: WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
) {
    let _ = with_app(&handle, app, cx, |_view, window, cx| {
        cx.observe_window_activation(window, |view, window, cx| {
            if window.is_window_active() {
                view.window_activated(cx);
            }
        })
        .detach();
    });
}

/// ⌘N / New Window (menu, Dock): a browser window on the current window's
/// space, nothing selected, with the ⌘T palette open.
pub(crate) fn new_window(cx: &mut App, windows: &Shared<Registry>) {
    let source = windows.read().current_target();
    let Some((_, source)) = source else {
        return;
    };
    let key = source.update(cx, |view, _cx| view.prepare_window());
    let rect = source
        .read(cx)
        .window_row()
        .geometry
        .map(|[x, y, width, height]| [x + 24.0, y + 24.0, width, height]);
    let Some((handle, app)) = open_shell(cx, windows, &source, key, rect) else {
        source.update(cx, |view, _cx| view.abandon_window(key));
        return;
    };
    let _ = with_app(&handle, &app, cx, |view, window, cx| {
        view.window_activated(cx);
        view.open_palette(window, cx);
    });
}

/// Blueprint D19: bring back the windows after the first one saved at
/// quit (`windows.txt` with their store rows by position): each on its
/// saved space and selections, at its saved content rect with its saved
/// rail. The first window stays the current one.
pub(crate) fn restore_windows(
    cx: &mut App,
    windows: &Shared<Registry>,
    saved: &[(crate::window_layouts::WindowLayout, Option<WindowRow>)],
) {
    let source = windows.read().current_target();
    let Some((first_handle, source)) = source else {
        return;
    };
    let first = windows.read().current();
    for (layout, row) in saved {
        let key = source.update(cx, |view, _cx| view.restore_window_context(layout));
        let rect = row.and_then(|row| row.geometry);
        let Some((handle, app)) = open_shell(cx, windows, &source, key, rect) else {
            source.update(cx, |view, _cx| view.abandon_window(key));
            continue;
        };
        let _ = with_app(&handle, &app, cx, |view, _window, cx| {
            if let Some(row) = row {
                view.restore_window_state(&row.sidebar);
            }
            // Its root is attached: its pages move in now.
            view.refresh_window_sessions();
            cx.notify();
        });
    }
    let _ = windows.bookkeep().set_current(first);
    let hidden = source.read(cx).windows_hidden;
    let _ = with_app(&first_handle, &source, cx, |view, window, cx| {
        if !hidden {
            window.activate_window();
        }
        view.window_activated(cx);
    });
}

/// Open window `key`'s shell (its context already exists in the
/// controller) at the content rect `rect` (centered when unknown): its
/// Root, its native root, Portal and pointer observer, its find pump, and
/// the watches between it and every other window. The new window becomes
/// the current one.
fn open_shell(
    cx: &mut App,
    windows: &Shared<Registry>,
    source: &Entity<BbrApp>,
    key: WindowKey,
    rect: Option<[f64; 4]>,
) -> Option<Target> {
    #[cfg(target_os = "macos")]
    let (find_tx, find_rx) =
        smol::channel::unbounded::<bbr_web_macos::find_mapping::FindCompletion>();
    let parts = source.read(cx).sibling_parts(
        key,
        #[cfg(target_os = "macos")]
        find_tx,
    );
    let hidden = source.read(cx).windows_hidden;
    let bounds = match rect {
        Some([x, y, width, height]) if width > 0.0 && height > 0.0 => Bounds {
            origin: gpui::point(px(x as f32), px(y as f32)),
            size: size(px(width as f32), px(height as f32)),
        },
        _ => {
            let (width, height) = crate::app::DEFAULT_WINDOW_SIZE;
            Bounds::centered(None, size(px(width), px(height)), cx)
        }
    };
    let mut shell = None;
    let opened = cx.open_window(browser_window_options(bounds, hidden), |window, cx| {
        let app = cx.new(|cx| BbrApp::assemble(parts, cx));
        shell = Some(app.clone());
        cx.new(|cx| gpui_component::Root::new(app, window, cx))
    });
    let (Ok(handle), Some(app)) = (opened, shell) else {
        log::error!("could not open a browser window");
        return None;
    };
    let others = windows.read().shells();
    windows.get().insert(key, handle, app.clone());
    let _ = windows.get().set_current(key);
    for other in others {
        app.update(cx, |view, cx| view.watch_window(&other, cx));
        other.update(cx, |view, cx| view.watch_window(&app, cx));
    }
    let _ = with_app(&handle, &app, cx, |view, _window, _cx| {
        view.browser_window = Some(handle);
    });
    attach_native(cx, &app, key, handle);
    install_window_hooks(cx, handle, &app);
    #[cfg(target_os = "macos")]
    spawn_find_pump(cx, handle, &app, find_rx);
    let _ = with_app(&handle, &app, cx, |view, window, cx| {
        view.observe_system_appearance(window, cx);
        view.observe_system_accessibility(cx);
    });
    Some((handle, app))
}

/// The new window's native parts: its root in the engine, its Portal host
/// and its pointer observer.
#[cfg(target_os = "macos")]
fn attach_native(
    cx: &mut App,
    app: &Entity<BbrApp>,
    key: WindowKey,
    handle: WindowHandle<gpui_component::Root>,
) {
    let Ok(ns_view) = handle.update(cx, |_root, window, _cx| crate::extract_ns_view(window)) else {
        return;
    };
    if ns_view.is_null() {
        return;
    }
    app.update(cx, |view, _cx| {
        // SAFETY: `ns_view` is the new window's live GPUI root view,
        // captured on the main thread.
        unsafe { view.attach_window_root(key, ns_view) }
    });
    let probe = app.read(cx).portal_probe_handle();
    // SAFETY: as above; the bridge retains the view.
    match unsafe {
        crate::portal::native::NativePortalBridge::open(cx, ns_view, handle, app.clone(), probe)
    } {
        Ok(bridge) => {
            let _ = with_app(&handle, app, cx, |view, _window, _cx| {
                view.set_native_portal(bridge)
            });
        }
        Err(error) => log::error!("portal for a new window: {error}"),
    }
    crate::rail_observer::install(cx, &handle, app);
}

#[cfg(not(target_os = "macos"))]
fn attach_native(
    _cx: &mut App,
    _app: &Entity<BbrApp>,
    _key: WindowKey,
    _handle: WindowHandle<gpui_component::Root>,
) {
}

/// The new window's find completions reach its own shell (each window's
/// find session numbers its own requests).
#[cfg(target_os = "macos")]
fn spawn_find_pump(
    cx: &mut App,
    handle: WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    find_rx: smol::channel::Receiver<bbr_web_macos::find_mapping::FindCompletion>,
) {
    let app = app.downgrade();
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        while let Ok(completion) = find_rx.recv().await {
            cx.update(|cx| {
                if let Some(app) = app.upgrade() {
                    let _ = with_app(&handle, &app, cx, |view, _window, cx| {
                        view.on_find_completion(completion, cx)
                    });
                }
            });
        }
    })
    .detach();
}

/// ⇧⌘W: close the key browser window (its tabs stay; the last window
/// quits).
pub(crate) fn close_window(cx: &mut App, windows: &Shared<Registry>) {
    let target = windows.read().current_target();
    if let Some((handle, _)) = target {
        let _ = handle.update(cx, |_root, window, _cx| window.remove_window());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_registry_has_no_window_to_act_in() {
        let registry = Registry::default();
        assert!(registry.current_target().is_none());
        assert!(registry.keys().is_empty());
        assert!(registry.key_of(WindowId::from(7)).is_none());
    }
}
