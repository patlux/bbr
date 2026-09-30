//! The B3 Saved Pages Library pointer rename entry (packet
//! `saved-pages-library`): the staged inline rename input — the row being
//! edited plus the draft label — and the app methods that enter, type,
//! confirm, and cancel it. Split from `mod` so no composition file
//! outgrows the ~500-line budget.
//!
//! The pointer cycle is deterministic and testable without GPUI: the row
//! rename control stages the input (`begin_rename`, seeded with the
//! record's current user label), keyboard/pointer input edits the draft,
//! Confirm commits a non-empty draft through the one existing
//! `library_rename` → AppStore authority (no second rename or command
//! policy exists), and Cancel leaves the input without touching the
//! record.

use gpui::Context;

use super::{BbrApp, LibraryState};

impl LibraryState {
    /// Enter the pointer rename entry for `id`: stage the inline rename
    /// input seeded with the record's current user label (`current_label`;
    /// empty when the record has none — a fresh label).
    pub(crate) fn begin_rename(&mut self, id: u64, current_label: &str) {
        self.editing_row = Some(id);
        self.rename_draft = current_label.to_string();
    }

    /// Leave the staged rename input without committing: the row display
    /// and the stored label are untouched.
    pub(crate) fn cancel_rename(&mut self) {
        self.editing_row = None;
        self.rename_draft.clear();
    }

    /// Commit the staged rename: returns `(id, draft)` when a row is
    /// staged and the draft is non-empty — the app routes the commit
    /// through the one `library_rename` authority — and clears the staging
    /// either way (an empty draft is a no-op at the store, so the commit
    /// never writes one).
    pub(crate) fn confirm_rename(&mut self) -> Option<(u64, String)> {
        let id = self.editing_row.take()?;
        let draft = std::mem::take(&mut self.rename_draft);
        if draft.trim().is_empty() {
            None
        } else {
            Some((id, draft))
        }
    }
}

impl BbrApp {
    /// The pointer rename entry for one row: stage the inline rename input
    /// seeded with the record's current user label (empty when none), so
    /// the row renders the staged draft with Confirm/Cancel. Any staged
    /// collection action surface yields to the row rename (the two inline
    /// inputs are mutually exclusive).
    pub fn library_begin_rename(&mut self, id: u64, cx: &mut Context<Self>) {
        self.library.collection_action = None;
        self.library.collection_create_staged = false;
        self.library.collection_draft.clear();
        let current = self
            .store
            .read()
            .saved_page_by_id(id)
            .and_then(|page| page.user_label)
            .unwrap_or_default();
        self.library.begin_rename(id, &current);
        cx.notify();
    }

    /// The direct staged-rename draft setter (the testkit seam the staged
    /// inline input's typed requests share; the production keyboard routes
    /// printable input into the draft through the same app state).
    #[cfg(feature = "testkit")]
    pub fn library_rename_draft(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.library.editing_row.is_none() {
            return;
        }
        self.library.rename_draft = text.to_string();
        cx.notify();
    }

    /// Confirm the staged rename: commit the non-empty draft through the
    /// one existing `library_rename` → AppStore authority and leave the
    /// rename input. No second command or persistence authority exists.
    pub fn library_confirm_rename(&mut self, cx: &mut Context<Self>) {
        if let Some((id, draft)) = self.library.confirm_rename() {
            self.library_rename(id, &draft, cx);
        } else {
            cx.notify();
        }
    }

    /// Cancel the staged rename: leave the inline input without committing
    /// (the row display and the stored label are untouched).
    pub fn library_cancel_rename(&mut self, cx: &mut Context<Self>) {
        if self.library.editing_row.is_none() {
            return;
        }
        self.library.cancel_rename();
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_stages_the_row_with_the_seeded_label() {
        let mut state = LibraryState::default();
        assert_eq!(state.editing_row, None);
        state.begin_rename(7, "Work");
        assert_eq!(state.editing_row, Some(7));
        assert_eq!(state.rename_draft, "Work");
    }

    #[test]
    fn confirm_commits_a_nonempty_draft_once_and_clears_the_staging() {
        let mut state = LibraryState::default();
        state.begin_rename(7, "Work");
        state.rename_draft = "Renamed".into();
        assert_eq!(state.confirm_rename(), Some((7, "Renamed".to_string())));
        assert_eq!(state.editing_row, None);
        assert!(state.rename_draft.is_empty());
        // A second confirm has nothing staged.
        assert_eq!(state.confirm_rename(), None);
    }

    #[test]
    fn confirm_never_commits_an_empty_draft() {
        let mut state = LibraryState::default();
        state.begin_rename(7, "Work");
        state.rename_draft.clear();
        assert_eq!(state.confirm_rename(), None, "empty draft is a no-op");
        assert_eq!(state.editing_row, None, "the staging still clears");
        assert!(state.rename_draft.is_empty());
        // Whitespace-only drafts are equally rejected.
        state.begin_rename(7, "Work");
        state.rename_draft = "   ".into();
        assert_eq!(state.confirm_rename(), None);
    }

    #[test]
    fn cancel_restores_without_committing() {
        let mut state = LibraryState::default();
        state.begin_rename(7, "Work");
        state.rename_draft = "Never committed".into();
        state.cancel_rename();
        assert_eq!(state.editing_row, None);
        assert!(state.rename_draft.is_empty());
    }
}
