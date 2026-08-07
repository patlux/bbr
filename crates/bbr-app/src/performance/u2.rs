//! U2 presentation-probe additions (DESIGN.md §11.4/§11.10/§11.11): the
//! observed/expected semantic state the plus-menu, favorites, and
//! folder-toggle benchmarks commit against. The latest observed values ride
//! [`super::State`], the pending expectations ride [`super::Pending`]; the
//! commit guard in `super` rejects a fast-but-wrong presentation (missing
//! menu, wrong tile count, wrong expanded value). Split so `super` stays
//! under the ~500-line budget.

use super::PerformanceProbe;

/// The U2 rail state the plus-menu/favorites/folder benchmarks observed at
/// the mutation site (embedded in [`State`]).
#[derive(Default)]
pub struct U2Observed {
    /// §11.4 plus-menu open state of the latest observed presentation.
    pub plus_menu_open: bool,
    /// §11.10 favorite tile count of the latest observed presentation.
    pub favorites_len: usize,
    /// §11.11 folder expanded state of the latest observed presentation.
    pub folder_expanded: bool,
}

/// The U2 rail state the pending sample must present (embedded in
/// [`Pending`]); `None` means no expectation for that benchmark class.
#[derive(Default)]
pub struct U2Expected {
    /// The §11.4 plus-menu open state the presentation must show.
    pub plus_menu_open: Option<bool>,
    /// The §11.10 favorite tile count the presentation must show.
    pub favorites_len: Option<usize>,
    /// The §11.11 folder expanded state the presentation must show.
    pub folder_expanded: Option<bool>,
}

impl U2Expected {
    /// Whether any expected value differs from the observed rail state (the
    /// fast-but-wrong guard: a presentation without the expected menu,
    /// tile count, or disclosure is rejected before it leaves the app).
    pub fn mismatches(&self, observed: &U2Observed) -> bool {
        self.plus_menu_open
            .is_some_and(|e| e != observed.plus_menu_open)
            || self
                .favorites_len
                .is_some_and(|e| e != observed.favorites_len)
            || self
                .folder_expanded
                .is_some_and(|e| e != observed.folder_expanded)
    }
}

impl PerformanceProbe {
    /// Record the §11.4 plus-menu open state observed at the mutation site,
    /// so the plus-menu benchmark commits only when the presented rail shows
    /// the open menu surface (fast-but-wrong guard).
    pub fn observe_plus_menu(&self, open: bool) {
        self.inner.borrow_mut().u2.plus_menu_open = open;
    }

    /// Require the pending presentation to show the §11.4 plus menu in the
    /// given open state.
    pub fn expect_plus_menu_open(&self, open: bool) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.u2.plus_menu_open = Some(open);
        }
    }

    /// Record the §11.10 favorites tile count observed at the mutation site,
    /// so the favorites benchmark commits only when the presented grid shows
    /// the expected tiles.
    pub fn observe_favorites(&self, len: usize) {
        self.inner.borrow_mut().u2.favorites_len = len;
    }

    /// Require the pending presentation to show exactly `len` favorite tiles.
    pub fn expect_favorites_len(&self, len: usize) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.u2.favorites_len = Some(len);
        }
    }

    /// Record the §11.11 folder expanded state observed at the mutation site,
    /// so the folder-toggle benchmark commits only when the presented nested
    /// rows match the disclosure.
    pub fn observe_folder_expanded(&self, expanded: bool) {
        self.inner.borrow_mut().u2.folder_expanded = expanded;
    }

    /// Require the pending presentation to show the §11.11 folder in the
    /// given expanded state.
    pub fn expect_folder_expanded(&self, expanded: bool) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.u2.folder_expanded = Some(expanded);
        }
    }
}
