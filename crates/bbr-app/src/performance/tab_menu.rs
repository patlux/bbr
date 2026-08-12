//! Tab/Favorite context-menu presentation identity for the presentation
//! probe (task packet `tab-favorite-context-menus`, DESIGN.md §11.13).
//!
//! The `tab_context_menu_open` benchmark measures the production
//! event-driven open path (the exact seam the sidebar tab row's
//! right-button mouse-down listener routes to: `open_tab_menu_from_row`)
//! and commits only from a main-window render that drew the menu open,
//! focused, and bound to the exact source `TabId` at the armed pointer
//! anchor. The `tab_context_menu_submenu_commit` benchmark measures the
//! production hover path that opens the one-level submenu card
//! (`tab_menu_select`) and commits only from a render that drew the open
//! submenu card bound to the same source — a fast-but-wrong guard: a
//! render that omitted the menu, changed the bound tab, or lost the anchor
//! can never commit. The observation is recorded by the menu card render
//! itself (`BbrApp::tab_menu_overlay`), so a sample can never commit from
//! state the app did not genuinely present.

use super::{Pending, PerformanceProbe, State};

/// The exact menu presentation a pending sample must commit from.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum TabMenuExpectation {
    /// The menu is open, focused, bound to this source `TabId`, under the
    /// armed presentation source key, anchored at this pointer point
    /// (window content coordinates).
    Open {
        bound_tab: u64,
        point: [f32; 2],
        source_key: String,
    },
    /// The menu is open with the one-level submenu card drawn for this
    /// parent row, bound to the exact source `TabId` (the submenu commit
    /// kind; the cursor lands on the first enabled submenu row).
    SubmenuCommit { bound_tab: u64, parent_row: usize },
}

/// The menu presentation state the render actually drew.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct TabMenuObservation {
    pub open: bool,
    pub focused: bool,
    pub bound_tab: Option<u64>,
    pub source_key: String,
    pub selected: Option<usize>,
    pub open_submenu: Option<usize>,
    pub submenu_selected: Option<usize>,
    pub anchor: [f32; 4],
}

impl PerformanceProbe {
    /// Arm the exact menu presentation a pending sample must commit from.
    /// Arming clears any prior observation: a render that showed an earlier
    /// open (or nothing) can never satisfy a newer expectation.
    pub fn expect_tab_menu_open(&self, bound_tab: u64, point: [f32; 2], source_key: String) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.tab_menu = Some(TabMenuExpectation::Open {
                bound_tab,
                point,
                source_key,
            });
            state.latest_tab_menu = None;
        }
    }

    /// Arm the exact open-submenu presentation the submenu-commit kind must
    /// commit from. Arming clears any prior observation.
    pub fn expect_tab_menu_submenu_commit(&self, bound_tab: u64, parent_row: usize) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.tab_menu = Some(TabMenuExpectation::SubmenuCommit {
                bound_tab,
                parent_row,
            });
            state.latest_tab_menu = None;
        }
    }

    /// Record the menu state the main-window render actually drew (the
    /// milestone). The tab-menu benchmarks commit only from this
    /// observation: the open card with the bound tab, source key, the
    /// keyboard/pointer cursor, the open submenu, and the clamped anchor.
    #[allow(clippy::too_many_arguments)] // the flat observed menu identity
    pub fn observe_tab_menu_state(
        &self,
        open: bool,
        focused: bool,
        bound_tab: Option<u64>,
        source_key: &str,
        selected: Option<usize>,
        open_submenu: Option<usize>,
        submenu_selected: Option<usize>,
        anchor: [f32; 4],
    ) {
        self.inner.borrow_mut().latest_tab_menu = Some(TabMenuObservation {
            open,
            focused,
            bound_tab,
            source_key: source_key.to_string(),
            selected,
            open_submenu,
            submenu_selected,
            anchor,
        });
    }
}

/// The tab-menu part of the presentation predicate: true when no tab-menu
/// expectation is armed, or when the observed state carries exactly the
/// armed open/submenu identity with the bound source tab and the armed
/// anchor. An armed expectation with no observation yet can never commit.
pub(super) fn tab_menu_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.tab_menu.as_ref() else {
        return true;
    };
    let observed = state.latest_tab_menu.as_ref();
    match (expected, observed) {
        (
            TabMenuExpectation::Open {
                bound_tab,
                point,
                source_key,
            },
            Some(observed),
        ) => {
            // The drawn anchor is the clamped placement; in-window anchors
            // equal the armed pointer within the half-point tolerance (the
            // same close rule the window-geometry waits use), so a render
            // drawn at a different position rejects.
            observed.open
                && observed.focused
                && observed.bound_tab == Some(*bound_tab)
                && observed.source_key == *source_key
                && (observed.anchor[0] - point[0]).abs() <= 0.5
                && (observed.anchor[1] - point[1]).abs() <= 0.5
        }
        (
            TabMenuExpectation::SubmenuCommit {
                bound_tab,
                parent_row,
            },
            Some(observed),
        ) => {
            observed.open
                && observed.focused
                && observed.bound_tab == Some(*bound_tab)
                && observed.open_submenu == Some(*parent_row)
                && observed.submenu_selected.is_some()
        }
        (_, None) => false,
    }
}
