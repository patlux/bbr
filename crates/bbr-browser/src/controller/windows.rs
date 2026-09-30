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

/// One window as persistence keeps it: its key, active space and the
/// selection of every space.
pub type WindowLayout = (WindowKey, SpaceId, Vec<(SpaceId, Option<TabId>)>);

/// A window's waiting context.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct WindowContext {
    pub(crate) active_space: Option<SpaceId>,
    /// Per space: the selected tab (`None`: the space's empty state).
    pub(crate) selections: BTreeMap<SpaceId, Option<TabId>>,
    /// The window's open Peek preview (a preview belongs to the window
    /// that opened it; its view sits in that window's root).
    pub(crate) preview: Option<crate::peek::PeekPreview>,
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
                preview: None,
            },
        );
        key
    }

    /// Blueprint D19: re-open a window saved at quit, on `active_space` with
    /// its per-space `selections` (a space or tab that is gone is dropped;
    /// the window then shows that space's empty state). The current window
    /// stays current, and no session moves until [`Self::refresh_sessions`]
    /// runs — the caller first gives the window its native root.
    pub fn restore_window(
        &mut self,
        active_space: SpaceId,
        selections: impl IntoIterator<Item = (SpaceId, Option<TabId>)>,
    ) -> WindowKey {
        let active_space = if self.browser.space(active_space).is_some() {
            active_space
        } else {
            self.active_space
        };
        let selections = selections
            .into_iter()
            .filter_map(|(space, tab)| {
                let state = self.browser.space(space)?;
                Some((space, tab.filter(|tab| state.tabs.contains(tab))))
            })
            .collect();
        self.windows.next = self.windows.next.max(self.windows.current.0) + 1;
        let key = WindowKey(self.windows.next);
        self.windows.waiting.insert(
            key,
            WindowContext {
                active_space: Some(active_space),
                selections,
                preview: None,
            },
        );
        key
    }

    /// How many tabs were closed so far (never decreases): a closed window
    /// is the most recently closed thing while this has not moved.
    pub fn closed_tab_count(&self) -> u64 {
        self.browser.closed_tab_count()
    }

    /// Reconcile every session with the windows (after a restored window
    /// got its native root).
    pub fn refresh_sessions(&mut self) {
        self.sync_views();
        self.check_invariants();
    }

    /// Every open window's active space and per-space selections (window
    /// order: by key), for persistence.
    pub fn window_layouts(&self) -> Vec<WindowLayout> {
        let mut out: Vec<_> = self
            .windows
            .waiting
            .iter()
            .map(|(key, context)| {
                let space = context.active_space.unwrap_or(self.active_space);
                let selections = self
                    .browser
                    .spaces()
                    .map(|state| {
                        let tab = context
                            .selections
                            .get(&state.id)
                            .copied()
                            .flatten()
                            .filter(|tab| state.tabs.contains(tab));
                        (state.id, tab)
                    })
                    .collect();
                (*key, space, selections)
            })
            .collect();
        out.push((
            self.windows.current,
            self.active_space,
            self.browser
                .spaces()
                .map(|space| (space.id, space.active_tab))
                .collect(),
        ));
        out.sort_by_key(|(key, _, _)| *key);
        out
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

    /// Load `window`'s context to *read and act in it* without touching
    /// the sessions: every access of a window's shell goes through this.
    /// Unlike [`Self::enter_window`] nothing is re-placed or re-shown; any
    /// operation run while it is loaded reconciles the sessions as usual.
    pub fn view_window(&mut self, window: WindowKey) -> Result<(), ControllerError> {
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
        if let Some(space) = context
            .active_space
            .filter(|space| self.browser.space(*space).is_some())
        {
            self.active_space = space;
        }
        self.load_selections(&context);
        self.preview = context.preview;
        // What the other windows select (model bookkeeping; a waiting
        // window's preview that lost its origin closes).
        let _ = self.reconcile_windows();
        Ok(())
    }

    /// Every open preview (the current window's first).
    pub(crate) fn previews(&self) -> impl Iterator<Item = &crate::peek::PeekPreview> {
        self.preview.iter().chain(
            self.windows
                .waiting
                .values()
                .filter_map(|context| context.preview.as_ref()),
        )
    }

    /// The window whose root holds the live session `view` (its events
    /// belong to that window's shell); `None` for an unknown view.
    pub fn window_of_view(&self, view: bbr_web::WebViewId) -> Option<WindowKey> {
        self.windows.placement.get(&view).copied()
    }

    /// The selected tab of `window` in its active space as its index in that
    /// space and its live session, if any.
    pub fn window_selected(
        &self,
        window: WindowKey,
    ) -> Option<(usize, Option<bbr_web::WebViewId>)> {
        let tab = self.window_selection(window)?;
        let space = self.window_space(window)?;
        let index = self
            .browser
            .space(space)?
            .tabs
            .iter()
            .position(|t| *t == tab)?;
        Some((index, self.registry.view_for_tab(tab)))
    }

    /// The selected tab of `window` in its active space (`None`: the
    /// space's empty state).
    pub fn window_selection(&self, window: WindowKey) -> Option<TabId> {
        if window == self.windows.current {
            return self.browser.space(self.active_space)?.active_tab;
        }
        let context = self.windows.waiting.get(&window)?;
        let space = context.active_space?;
        context.selections.get(&space).copied().flatten()
    }

    /// Close `window`: its context is dropped (its tabs stay in their
    /// spaces, warm). Closing the current window makes `next` current
    /// (another open window), or keeps the context without a window when it
    /// was the last one. Returns the closed context for a later reopen.
    pub fn close_window(&mut self, window: WindowKey) -> Result<(), ControllerError> {
        if window != self.windows.current {
            let closed = self
                .windows
                .waiting
                .remove(&window)
                .ok_or(ControllerError::UnknownWindow(window))?;
            // Its preview goes with it.
            if let Some(preview) = closed.preview {
                self.destroy_preview_session(preview.view);
            }
            self.sync_views();
            self.check_invariants();
            return Ok(());
        }
        let Some(&next) = self.windows.waiting.keys().next() else {
            // The last window: its context stays for the next window.
            return Ok(());
        };
        let context = self.windows.waiting.remove(&next).expect("listed");
        if let Some(preview) = self.preview.take() {
            self.destroy_preview_session(preview.view);
        }
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
            preview: self.preview.clone(),
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
        self.load_selections(&context);
        self.preview = context.preview;
        let spaces: Vec<SpaceId> = self.browser.spaces().map(|space| space.id).collect();
        for space in spaces {
            let _ = self.browser.wake_visible(space);
        }
        // The window's own preview stays only while its origin does.
        self.close_preview_if_stale();
    }

    /// Load `context`'s per-space selections (a selection that no longer
    /// fits falls back to the space's empty state).
    fn load_selections(&mut self, context: &WindowContext) {
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
            // The window's preview stays only while its origin is the
            // window's selection in its active space.
            let stale = kept.preview.as_ref().filter(|preview| {
                kept.active_space != Some(preview.source_space)
                    || kept
                        .selections
                        .get(&preview.source_space)
                        .copied()
                        .flatten()
                        != Some(preview.source_tab)
            });
            if let Some(view) = stale.map(|preview| preview.view) {
                kept.preview = None;
                self.destroy_preview_session(view);
            }
            self.windows.waiting.insert(key, kept);
        }
        self.browser.set_selected_elsewhere(elsewhere);
        shown
    }
}
