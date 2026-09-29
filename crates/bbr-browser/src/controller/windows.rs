//! Blueprint D19 (several windows, task packet `multi-window.md`): the
//! controller's window contexts.
//!
//! One controller owns the model and every session; each browser window
//! has its own active space and its own selected tab per space. The model's
//! selections (`Space::active_tab`) and `active_space` always belong to the
//! **current** window; the other windows' contexts wait in `windows` and
//! are swapped in by [`BrowserController::enter_window`]. Every existing
//! operation therefore works unchanged for the window it runs in.
//!
//! The windows share the pages: a tab another window selects (or a pane of
//! the split it selects) counts as visible for lifecycle and session
//! policy, stays shown in that window's root, and is never selected by two
//! windows at once — the window that selects it last takes it, and the
//! other window's selection of that space falls back to its empty state.

use std::collections::{BTreeMap, BTreeSet};

use bbr_domain::{SpaceId, TabId};

use super::BrowserController;
use crate::ControllerError;

/// A browser window's identity in the controller (the app maps its native
/// windows onto these).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowKey(pub u64);

/// A window's waiting context.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WindowContext {
    pub(crate) active_space: Option<SpaceId>,
    /// Per space: the selected tab (`None`: the space's empty state).
    pub(crate) selections: BTreeMap<SpaceId, Option<TabId>>,
}

/// The controller's window bookkeeping.
#[derive(Debug, Clone, Default)]
pub(crate) struct Windows {
    pub(crate) current: WindowKey,
    /// Every other open window's context.
    pub(crate) waiting: BTreeMap<WindowKey, WindowContext>,
    /// The root each live session sits in (only recorded once a second
    /// window exists; absent means the first window's root).
    pub(crate) placement: BTreeMap<bbr_web::WebViewId, WindowKey>,
    pub(crate) next: u64,
}

impl BrowserController {
    /// The window operations currently run in.
    pub fn current_window(&self) -> WindowKey {
        self.windows.current
    }

    /// Every open window, the current one first.
    pub fn window_keys(&self) -> Vec<WindowKey> {
        std::iter::once(self.windows.current)
            .chain(self.windows.waiting.keys().copied())
            .collect()
    }

    /// The active space of `window`.
    pub fn window_space(&self, window: WindowKey) -> Option<SpaceId> {
        if window == self.windows.current {
            return Some(self.active_space);
        }
        self.windows.waiting.get(&window)?.active_space
    }

    /// Open a window on the current window's active space with nothing
    /// selected (its empty state, New Tab ready). The current window stays
    /// current.
    pub fn open_window(&mut self) -> WindowKey {
        self.windows.next = self.windows.next.max(self.windows.current.0) + 1;
        let key = WindowKey(self.windows.next);
        let mut selections = BTreeMap::new();
        selections.insert(self.active_space, None);
        self.windows.waiting.insert(
            key,
            WindowContext {
                active_space: Some(self.active_space),
                selections,
            },
        );
        key
    }

    /// Make `window` current: stash the current context, load its own.
    /// A selection that no longer fits (a closed or moved tab, or a tab
    /// another window took) falls back to the space's empty state.
    pub fn enter_window(&mut self, window: WindowKey) -> Result<(), ControllerError> {
        if window == self.windows.current {
            return Ok(());
        }
        let context = self
            .windows
            .waiting
            .remove(&window)
            .ok_or(ControllerError::UnknownWindow(window))?;
        let outgoing = self.current_context();
        self.windows.waiting.insert(self.windows.current, outgoing);
        self.windows.current = window;
        self.load_context(context);
        self.sync_views();
        self.check_invariants();
        Ok(())
    }

    /// Close `window`: its context is dropped (its tabs stay in their
    /// spaces, warm). Closing the current window makes `next` current
    /// (another open window), or keeps the context without a window when it
    /// was the last one. Returns the closed context for a later reopen.
    pub fn close_window(&mut self, window: WindowKey) -> Result<(), ControllerError> {
        if window != self.windows.current {
            self.windows
                .waiting
                .remove(&window)
                .ok_or(ControllerError::UnknownWindow(window))?;
            self.sync_views();
            self.check_invariants();
            return Ok(());
        }
        let Some(&next) = self.windows.waiting.keys().next() else {
            // The last window: its context stays for the next window.
            return Ok(());
        };
        let context = self.windows.waiting.remove(&next).expect("listed");
        self.windows.current = next;
        self.load_context(context);
        self.sync_views();
        self.check_invariants();
        Ok(())
    }

    /// The current window's context as a waiting one.
    fn current_context(&self) -> WindowContext {
        WindowContext {
            active_space: Some(self.active_space),
            selections: self
                .browser
                .spaces()
                .map(|space| (space.id, space.active_tab))
                .collect(),
        }
    }

    /// Load `context` into the model as the current window's.
    fn load_context(&mut self, context: WindowContext) {
        if let Some(space) = context
            .active_space
            .filter(|space| self.browser.space(*space).is_some())
        {
            self.active_space = space;
        }
        let spaces: Vec<SpaceId> = self.browser.spaces().map(|space| space.id).collect();
        for space in spaces {
            let selection = context
                .selections
                .get(&space)
                .copied()
                .flatten()
                .filter(|tab| {
                    self.browser
                        .space(space)
                        .is_some_and(|state| state.tabs.contains(tab))
                });
            let _ = self.browser.restore_active_tab(space, selection);
        }
        let spaces: Vec<SpaceId> = self.browser.spaces().map(|space| space.id).collect();
        for space in spaces {
            let _ = self.browser.wake_visible(space);
        }
        // A preview belongs to the window that opened it.
        self.close_preview_if_stale();
    }

    /// The tabs a waiting context shows or keeps (its selections and their
    /// split panes), per space.
    fn context_tabs(&self, context: &WindowContext) -> BTreeMap<SpaceId, BTreeSet<TabId>> {
        let mut out: BTreeMap<SpaceId, BTreeSet<TabId>> = BTreeMap::new();
        for (space, selection) in &context.selections {
            let (Some(tab), Some(state)) = (selection, self.browser.space(*space)) else {
                continue;
            };
            if !state.tabs.contains(tab) {
                continue;
            }
            let set = out.entry(*space).or_default();
            match state.group_of(*tab) {
                Some(group) => set.extend(group.members()),
                None => {
                    set.insert(*tab);
                }
            }
        }
        out
    }

    /// Before every session reconciliation: the waiting windows give up a
    /// tab the current window selects (the last selection wins), their
    /// stale selections fall back to empty, and what they still select is
    /// published to the model as selected elsewhere. Returns the views the
    /// waiting windows show, with their window.
    pub(crate) fn reconcile_windows(&mut self) -> BTreeMap<TabId, WindowKey> {
        if self.windows.waiting.is_empty() {
            if !self.browser.selected_elsewhere().is_empty() {
                self.browser.set_selected_elsewhere(BTreeSet::new());
            }
            return BTreeMap::new();
        }
        let mine: BTreeMap<SpaceId, BTreeSet<TabId>> = self
            .browser
            .spaces()
            .map(|space| {
                let mut set = BTreeSet::new();
                if let Some(tab) = space.active_tab {
                    match space.group_of(tab) {
                        Some(group) => set.extend(group.members()),
                        None => {
                            set.insert(tab);
                        }
                    }
                }
                (space.id, set)
            })
            .collect();
        let keys: Vec<WindowKey> = self.windows.waiting.keys().copied().collect();
        let mut elsewhere = BTreeSet::new();
        let mut shown = BTreeMap::new();
        for key in keys {
            let context = self.windows.waiting[&key].clone();
            let tabs = self.context_tabs(&context);
            let mut kept = context.clone();
            for (space, selection) in &context.selections {
                let taken = tabs.get(space).is_none_or(|set| {
                    set.is_empty()
                        || mine
                            .get(space)
                            .is_some_and(|current| !current.is_disjoint(set))
                });
                if selection.is_some() && taken {
                    kept.selections.insert(*space, None);
                }
            }
            let tabs = self.context_tabs(&kept);
            for (space, set) in &tabs {
                elsewhere.extend(set.iter().copied());
                if kept.active_space == Some(*space) {
                    for tab in set {
                        shown.insert(*tab, key);
                    }
                }
            }
            if kept
                .active_space
                .is_some_and(|space| self.browser.space(space).is_none())
            {
                kept.active_space = Some(self.active_space);
            }
            self.windows.waiting.insert(key, kept);
        }
        self.browser.set_selected_elsewhere(elsewhere);
        shown
    }
}
