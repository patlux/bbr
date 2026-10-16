//! Blueprint D19 (several windows, task packet `multi-window.md`): a shell's
//! part in the window lifecycle — preparing a further window, following the
//! other windows' changes, becoming the current window, closing.

use bbr_browser::WindowKey;
use gpui::{Context, Entity};

use super::BbrApp;
use crate::browser_windows::{Registry, Target};
use crate::shared::Shared;

impl BbrApp {
    /// The window this shell renders.
    pub(crate) fn window_key(&self) -> WindowKey {
        self.controller.window()
    }

    /// The window whose root holds the live session `view`.
    pub(crate) fn window_of_view(&self, view: bbr_web::WebViewId) -> Option<WindowKey> {
        self.controller.read().window_of_view(view)
    }

    /// Every open browser window.
    pub(crate) fn window_registry(&self) -> Shared<Registry> {
        self.windows.clone()
    }

    /// ⌘N: a new window's context on this window's space (nothing
    /// selected).
    pub(crate) fn prepare_window(&mut self) -> WindowKey {
        self.controller.get().open_window()
    }

    /// A window saved at quit: its context on its saved space and
    /// selections (its sessions move once it has its root).
    pub(crate) fn restore_window_context(
        &mut self,
        layout: &crate::window_layouts::WindowLayout,
    ) -> WindowKey {
        self.controller
            .get()
            .restore_window(layout.active_space, layout.selections.iter().copied())
    }

    /// Reconcile the sessions with every window (a restored window got its
    /// native root).
    pub(crate) fn refresh_window_sessions(&mut self) {
        self.controller.get().refresh_sessions();
    }

    /// Re-render when `other` changed shared state this window draws (its
    /// own re-renders — hover, animation — leave this window alone).
    pub(crate) fn watch_window(&mut self, other: &Entity<BbrApp>, cx: &mut Context<Self>) {
        let watch = cx.observe(other, |this, _other, cx| {
            let generation = crate::shared::generation();
            if generation != this.seen_generation {
                this.seen_generation = generation;
                #[cfg(feature = "testkit")]
                {
                    this.window_refreshes += 1;
                }
                cx.notify();
            }
        });
        self.window_watch.push(watch);
    }

    /// The generation this window's last frame drew.
    pub(crate) fn mark_generation_drawn(&mut self) {
        self.seen_generation = crate::shared::generation();
    }

    /// This window took input (activation, the testkit): engine events,
    /// timers and commands act in it, and the engine hands the keyboard back
    /// to its chrome.
    pub(crate) fn window_activated(&mut self, _cx: &mut Context<Self>) {
        let key = self.window_key();
        if self.windows.read().current() != key {
            let _ = self.windows.bookkeep().set_current(key);
        }
        #[cfg(target_os = "macos")]
        if let Some(native) = self.find_engine.as_ref() {
            native.borrow_mut().set_key_window(key.0);
        }
    }

    /// Attach this window's GPUI root view to the engine.
    ///
    /// # Safety
    ///
    /// `ns_view` must be the window's live GPUI root `NSView*`, captured on
    /// the main thread.
    #[cfg(target_os = "macos")]
    pub(crate) unsafe fn attach_window_root(
        &mut self,
        key: WindowKey,
        ns_view: *mut std::ffi::c_void,
    ) {
        let Some(native) = self.find_engine.clone() else {
            return;
        };
        let Some(mtm) = objc2::MainThreadMarker::new() else {
            return;
        };
        // SAFETY: forwarded from the caller.
        unsafe { native.borrow_mut().attach_window(mtm, key.0, ns_view) };
    }

    /// This window closed: its context goes (its tabs stay in their
    /// spaces) and is kept with its presentation `row` for Reopen, the
    /// window that takes over says so with a toast, its engine root is
    /// released, and a Settings window opened from it follows `successor`.
    pub(crate) fn window_closed(
        &mut self,
        successor: Option<Target>,
        row: Option<crate::browser_windows::WindowRow>,
        cx: &mut Context<Self>,
    ) {
        #[cfg(target_os = "macos")]
        self.chrome_material.borrow_mut().take();
        let key = self.window_key();
        let layout = self
            .controller
            .read()
            .window_layouts()
            .into_iter()
            .find(|(window, _, _)| *window == key)
            .map(
                |(_, active_space, selections)| crate::window_layouts::WindowLayout {
                    active_space,
                    selections,
                },
            );
        let tab_closes = self.controller.read().closed_tab_count();
        let _ = self.controller.get().close_window(key);
        if let Some(layout) = layout {
            let id = self.windows.get().push_closed(layout, row, tab_closes);
            if let Some((handle, app)) = successor.clone() {
                cx.defer(move |cx| {
                    let _ = crate::window_seam::with_app(&handle, &app, cx, |view, _w, cx| {
                        view.show_window_closed_toast(id, cx)
                    });
                });
            }
        }
        // Window 0 is the engine's base root; the others were attached.
        #[cfg(target_os = "macos")]
        if key != WindowKey::default() {
            if let Some(native) = self.find_engine.as_ref() {
                native.borrow_mut().detach_window(key.0);
            }
        }
        let settings = self.settings.read().window;
        if let (Some(settings), Some((_, successor))) = (settings, successor) {
            cx.defer(move |cx| crate::settings_window::rebind(settings, successor, cx));
        }
        self.persist_structure();
    }

    /// ⇧⌘T: the closed window to bring back instead of a tab — the most
    /// recently closed one, when no tab was closed after it.
    pub(crate) fn window_to_reopen(&self) -> Option<u64> {
        let tab_closes = self.controller.read().closed_tab_count();
        let registry = self.windows.read();
        let last = registry.last_closed()?;
        (last.tab_closes == tab_closes).then_some(last.id)
    }

    /// A window that could not be opened: drop the context prepared for it.
    pub(crate) fn abandon_window(&mut self, key: WindowKey) {
        let _ = self.controller.get().close_window(key);
    }

    /// Blueprint D19 testkit evidence: how often another window's change
    /// re-rendered this one.
    #[cfg(feature = "testkit")]
    pub(crate) fn window_refreshes(&self) -> u64 {
        self.window_refreshes
    }

    /// Blueprint D19 testkit evidence: every open window.
    #[cfg(feature = "testkit")]
    pub(crate) fn window_entries(&self) -> Vec<bbr_testkit::protocol::WindowEntry> {
        let current = self.windows.read().current();
        let keys = self.windows.read().keys();
        keys.into_iter()
            .map(|key| {
                let controller = self.controller.read();
                let selected = controller.window_selected(key);
                let selected_view = selected.and_then(|(_, view)| view);
                #[cfg(target_os = "macos")]
                let selected_root = selected_view.and_then(|view| {
                    let native = self.find_engine.as_ref()?;
                    Some(native.borrow().window_of_view(view))
                });
                #[cfg(not(target_os = "macos"))]
                let selected_root = None;
                let row = if key == self.window_key() {
                    Some(self.window_row())
                } else {
                    self.windows.read().row(key)
                };
                bbr_testkit::protocol::WindowEntry {
                    window: key.0,
                    current: key == current,
                    active_space: controller.window_space(key).map(|space| space.0),
                    selected_index: selected.map(|(index, _)| index),
                    selected_view,
                    selected_root,
                    content_rect: row.and_then(|row| row.geometry),
                    sidebar_visible: row.is_some_and(|row| !row.sidebar.collapsed),
                    sidebar_width: row.map_or(0.0, |row| row.sidebar.width),
                    frame: self.windows.read().frame(key),
                    incoming_drop: self.windows.read().incoming(key),
                }
            })
            .collect()
    }
}
