//! Blueprint D19: the registry of browser windows — every open window's
//! handle and shell, the window that last took input, each window's
//! presentation row and frame, and the closed windows kept for Reopen.

use std::collections::BTreeMap;

use bbr_browser::WindowKey;
use gpui::{Entity, WindowHandle, WindowId};

use crate::app::BbrApp;

/// One open browser window.
pub(crate) struct BrowserWindow {
    pub(crate) handle: WindowHandle<gpui_component::Root>,
    pub(crate) app: Entity<BbrApp>,
}

/// One window's presentation as the store keeps it: its content rect
/// (`[x, y, width, height]`, points) and its rail.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WindowRow {
    pub(crate) geometry: Option<[f64; 4]>,
    pub(crate) sidebar: bbr_persistence::SidebarState,
}

/// A window closed while others stayed open, kept for Reopen and ⇧⌘T.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClosedWindow {
    pub(crate) id: u64,
    pub(crate) layout: crate::window_layouts::WindowLayout,
    pub(crate) row: Option<WindowRow>,
    /// The controller's closed-tab count when it closed.
    pub(crate) tab_closes: u64,
}

/// Every open browser window, the one that last took input, each window's
/// presentation row (published when it draws), and the closed windows,
/// most recent last.
#[derive(Default)]
pub(crate) struct Registry {
    windows: BTreeMap<WindowKey, BrowserWindow>,
    current: WindowKey,
    rows: BTreeMap<WindowKey, WindowRow>,
    closed: Vec<ClosedWindow>,
    next_closed: u64,
    /// The windows by the input they last took, most recent first (the
    /// stacking order when the window server has none, e.g. for the hidden
    /// test harness).
    recency: Vec<WindowKey>,
    /// Each window's frame in screen points as it last drew.
    frames: BTreeMap<WindowKey, [f32; 4]>,
    /// What a tab dragged in from another window would do in each window
    /// (testkit evidence).
    incoming: BTreeMap<WindowKey, String>,
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

    pub(crate) fn remove(&mut self, key: WindowKey) -> Option<BrowserWindow> {
        self.rows.remove(&key);
        self.frames.remove(&key);
        self.incoming.remove(&key);
        self.recency.retain(|window| *window != key);
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
        self.recency.retain(|window| *window != key);
        self.recency.insert(0, key);
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

    /// Keep a closed window for Reopen; returns its id.
    pub(crate) fn push_closed(
        &mut self,
        layout: crate::window_layouts::WindowLayout,
        row: Option<WindowRow>,
        tab_closes: u64,
    ) -> u64 {
        self.next_closed += 1;
        let id = self.next_closed;
        self.closed.push(ClosedWindow {
            id,
            layout,
            row,
            tab_closes,
        });
        id
    }

    /// The most recently closed window.
    pub(crate) fn last_closed(&self) -> Option<&ClosedWindow> {
        self.closed.last()
    }

    /// Take closed window `id` (the most recent one for `None`).
    pub(crate) fn take_closed(&mut self, id: Option<u64>) -> Option<ClosedWindow> {
        let index = match id {
            Some(id) => self.closed.iter().position(|closed| closed.id == id)?,
            None => self.closed.len().checked_sub(1)?,
        };
        Some(self.closed.remove(index))
    }

    /// `key`'s frame as it last drew.
    pub(crate) fn frame(&self, key: WindowKey) -> Option<[f32; 4]> {
        self.frames.get(&key).copied()
    }

    pub(crate) fn set_frame(&mut self, key: WindowKey, frame: [f32; 4]) {
        self.frames.insert(key, frame);
    }

    /// What a tab dragged in would do in `key` (testkit evidence).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub(crate) fn incoming(&self, key: WindowKey) -> Option<String> {
        self.incoming.get(&key).cloned()
    }

    pub(crate) fn set_incoming(&mut self, key: WindowKey, preview: Option<String>) {
        match preview {
            Some(preview) => self.incoming.insert(key, preview),
            None => self.incoming.remove(&key),
        };
    }

    /// `windows` ordered front to back: by the window server's stack
    /// where it lists them (`stack`, window ids front to back), the rest by
    /// the input they last took.
    pub(crate) fn front_to_back(
        &self,
        windows: &[WindowKey],
        stack: &[WindowId],
    ) -> Vec<WindowKey> {
        let rank = |key: &WindowKey| {
            let listed = self
                .windows
                .get(key)
                .and_then(|window| stack.iter().position(|id| *id == window.handle.window_id()));
            let recent = self
                .recency
                .iter()
                .position(|window| window == key)
                .unwrap_or(usize::MAX / 2);
            match listed {
                Some(position) => (0, position),
                None => (1, recent),
            }
        };
        let mut ordered = windows.to_vec();
        ordered.sort_by_key(rank);
        ordered
    }

    /// Every open window's shell.
    pub(crate) fn shells(&self) -> Vec<Entity<BbrApp>> {
        self.windows
            .values()
            .map(|window| window.app.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(space: u64) -> crate::window_layouts::WindowLayout {
        crate::window_layouts::WindowLayout {
            active_space: bbr_domain::SpaceId(space),
            selections: Vec::new(),
        }
    }

    #[test]
    fn closed_windows_come_back_most_recent_first_or_by_id() {
        let mut registry = Registry::default();
        let first = registry.push_closed(layout(1), None, 0);
        let second = registry.push_closed(layout(2), None, 3);
        assert_eq!(registry.last_closed().map(|closed| closed.id), Some(second));
        assert_eq!(
            registry
                .take_closed(Some(first))
                .map(|closed| closed.layout),
            Some(layout(1))
        );
        assert_eq!(registry.take_closed(Some(first)), None, "taken once");
        assert_eq!(
            registry.take_closed(None).map(|closed| closed.id),
            Some(second)
        );
        assert_eq!(registry.take_closed(None), None);
    }

    #[test]
    fn an_empty_registry_has_no_window_to_act_in() {
        let registry = Registry::default();
        assert!(registry.current_target().is_none());
        assert!(registry.keys().is_empty());
        assert!(registry.key_of(WindowId::from(7)).is_none());
    }
}
