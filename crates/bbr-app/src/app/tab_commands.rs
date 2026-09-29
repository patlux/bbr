//! Blueprint D19 (board L-13): the Tab menu's Duplicate Tab and Select
//! Next / Previous Tab (`Ctrl-Tab` / `Ctrl-Shift-Tab`). Both run through
//! the existing production tab paths (`open_tab`, `reorder_tab_at`,
//! `select_tab`), so the browser events, persistence and chrome sync are
//! the ones every other tab command takes.

use bbr_browser::ControllerEvent;
use gpui::Context;

use super::BbrApp;

/// The index `delta` steps from `active` in a list of `len`, wrapping.
pub(crate) fn adjacent_index(active: usize, len: usize, delta: isize) -> Option<usize> {
    if len < 2 {
        return None;
    }
    Some((active as isize + delta).rem_euclid(len as isize) as usize)
}

/// Where a duplicate of the tab at `source` lands: directly below it, or at
/// the top of the regular tabs when the source is pinned (a new tab is
/// never pinned).
pub(crate) fn duplicate_slot(source: usize, pinned: usize) -> usize {
    if source < pinned {
        pinned
    } else {
        source + 1
    }
}

impl BbrApp {
    /// Select the tab `delta` steps from the active one (wrapping through
    /// pinned and regular tabs alike).
    pub fn select_adjacent_tab(
        &mut self,
        delta: isize,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let snapshot = self.controller.snapshot();
        let index = adjacent_index(snapshot.active_index?, snapshot.tabs.len(), delta)?;
        self.select_tab(index, cx)
    }

    /// Open the active tab's page in a new tab directly below it (the new
    /// tab is selected; the source keeps its own history and session).
    pub fn duplicate_active_tab(&mut self, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let snapshot = self.controller.snapshot();
        let source = snapshot.active_index?;
        let url = snapshot.tabs.get(source)?.url.clone();
        let pinned = crate::shell::pinned_count(&snapshot.tabs);
        let opened = self.open_tab(&url, cx)?;
        let from = self.controller.snapshot().active_index?;
        let to = duplicate_slot(source, pinned);
        if from != to {
            self.reorder_tab_at(from, to, cx);
        }
        Some(opened)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_selection_wraps_both_ways() {
        assert_eq!(adjacent_index(0, 3, 1), Some(1));
        assert_eq!(adjacent_index(2, 3, 1), Some(0));
        assert_eq!(adjacent_index(0, 3, -1), Some(2));
        assert_eq!(adjacent_index(0, 1, 1), None, "one tab: nothing to select");
    }

    #[test]
    fn duplicates_land_below_their_source() {
        assert_eq!(duplicate_slot(3, 2), 4);
        assert_eq!(
            duplicate_slot(0, 2),
            2,
            "a pinned source: first regular slot"
        );
        assert_eq!(duplicate_slot(1, 0), 2);
    }
}
