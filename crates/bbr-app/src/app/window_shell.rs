//! Blueprint D19 (several windows, task packet `multi-window.md`): a shell's
//! part in the window lifecycle — preparing a further window, following the
//! other windows' changes, becoming the current window, closing.

use bbr_browser::WindowKey;
use gpui::{Context, Entity};

use super::construct::SharedParts;
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
    /// selected), its shell's shared parts, and where to open it (this
    /// window's content rect, cascaded).
    pub(crate) fn prepare_window(
        &mut self,
        #[cfg(target_os = "macos")] find_tx: smol::channel::Sender<
            bbr_web_macos::find_mapping::FindCompletion,
        >,
    ) -> (WindowKey, SharedParts, Option<[f64; 4]>) {
        let key = self.controller.get().open_window();
        let parts = self.sibling_parts(
            key,
            #[cfg(target_os = "macos")]
            find_tx,
        );
        let rect = self
            .window_geometry
            .borrow()
            .map(|[x, y, width, height]| [x + 24.0, y + 24.0, width, height]);
        (key, parts, rect)
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
    /// spaces), its engine root is released, and a Settings window opened
    /// from it follows `successor`.
    pub(crate) fn window_closed(&mut self, successor: Option<Target>, cx: &mut Context<Self>) {
        let key = self.window_key();
        let _ = self.controller.get().close_window(key);
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
                bbr_testkit::protocol::WindowEntry {
                    window: key.0,
                    current: key == current,
                    active_space: controller.window_space(key).map(|space| space.0),
                    selected_index: selected.map(|(index, _)| index),
                    selected_view,
                    selected_root,
                }
            })
            .collect()
    }
}
