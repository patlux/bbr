//! Gallery-owned observation state for the Saved Pages Library stories
//! (B3 packet `saved-pages-library`, Gallery G2/G3).
//!
//! The Library stories render the **exact production `bbr-ui` rows,
//! controls, and surface** (`library_row`, `library_collection_row`,
//! `library_search_field`, `library_sort_control`, `library_count_line`,
//! `library_remove_confirm`, `library_state_row`, `library_surface`) with
//! plain fixture models and recorder callbacks — never a gallery-local
//! wrapper, reimplementation, restyle, or copy. The gallery's own
//! contribution is limited to this observation state: what the real
//! Library controls report back so the E2E can assert real interactions
//! and the canvas can reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real Library
//! events re-enter the app through the same seam the mode/state controls
//! use, so the app's snapshot can read the observed values without a
//! second authority. Search is proven at the model level (the search
//! field renders the query and the no-results state presents); the
//! collection action is the move entry (moving records into/out of a
//! collection).

use bbr_ui::components::{LibraryAction, LibraryItemAction, LibraryScrollAction};

/// What a Saved Pages Library interaction reports back to the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryObservation {
    /// The header close control.
    Close,
    /// The deterministic sort cycle control.
    CycleSort,
    /// A row activation (open): carries the `BookmarkId`.
    Activate(u64),
    /// A row multi-select toggle: carries the `BookmarkId`.
    ToggleSelect(u64),
    /// A row rename stage (the pointer rename entry): carries the
    /// `BookmarkId`.
    Rename(u64),
    /// The staged rename Confirm.
    RenameConfirm,
    /// The staged rename Cancel.
    RenameCancel,
    /// A flat collection row action (the rename/remove cycle entry):
    /// carries the `CollectionId`.
    CollectionAction(u64),
    /// The collection create entry (stage the inline create input).
    CollectionCreate,
    /// The staged collection action Confirm (create/rename/remove).
    CollectionConfirm,
    /// The staged collection action Cancel.
    CollectionCancel,
    /// The selection toolbar Clear control.
    ClearSelection,
    /// The selection toolbar Remove control (stage the destructive remove
    /// for the selection).
    RemoveSelection,
    /// The selection toolbar Move control (toggle the move-to sheet).
    ToggleMoveSheet,
    /// A move-to sheet option (move the selection into this collection):
    /// carries the `CollectionId`.
    MoveInto(u64),
    /// The move-to sheet "No collection" option (move the selection out).
    MoveOut,
    /// The bounded list wheel: carries the new clamped scroll offset in
    /// row units.
    Scroll(usize),
    /// The destructive remove Confirm.
    ConfirmRemove,
    /// The destructive remove Cancel.
    CancelRemove,
}

/// The shared Library recorder: plain fixture observation state. Every
/// field is written only through the app's production `observe_library`
/// path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryRecorder {
    pub closes: u32,
    pub sort_cycles: u32,
    /// The `BookmarkId`s opened, in dispatch order.
    pub opens: Vec<u64>,
    /// The `BookmarkId`s toggled, in dispatch order.
    pub select_toggles: Vec<u64>,
    /// The `BookmarkId`s whose rename was staged, in dispatch order.
    pub renames: Vec<u64>,
    pub rename_confirms: u32,
    pub rename_cancels: u32,
    /// The `CollectionId`s acted on, in dispatch order.
    pub collection_actions: Vec<u64>,
    pub collection_creates: u32,
    pub collection_confirms: u32,
    pub collection_cancels: u32,
    pub clear_selections: u32,
    pub remove_selections: u32,
    pub move_sheet_toggles: u32,
    /// The `CollectionId`s the selection was moved into, in dispatch order.
    pub move_ins: Vec<u64>,
    pub move_outs: u32,
    /// The wheel-dispatched scroll offsets, in dispatch order.
    pub scrolls: Vec<usize>,
    pub remove_confirms: u32,
    pub remove_cancels: u32,
}

impl LibraryRecorder {
    /// Apply one observed Library event.
    pub fn observe(&mut self, event: LibraryObservation) {
        match event {
            LibraryObservation::Close => self.closes += 1,
            LibraryObservation::CycleSort => self.sort_cycles += 1,
            LibraryObservation::Activate(id) => self.opens.push(id),
            LibraryObservation::ToggleSelect(id) => self.select_toggles.push(id),
            LibraryObservation::Rename(id) => self.renames.push(id),
            LibraryObservation::RenameConfirm => self.rename_confirms += 1,
            LibraryObservation::RenameCancel => self.rename_cancels += 1,
            LibraryObservation::CollectionAction(id) => self.collection_actions.push(id),
            LibraryObservation::CollectionCreate => self.collection_creates += 1,
            LibraryObservation::CollectionConfirm => self.collection_confirms += 1,
            LibraryObservation::CollectionCancel => self.collection_cancels += 1,
            LibraryObservation::ClearSelection => self.clear_selections += 1,
            LibraryObservation::RemoveSelection => self.remove_selections += 1,
            LibraryObservation::ToggleMoveSheet => self.move_sheet_toggles += 1,
            LibraryObservation::MoveInto(id) => self.move_ins.push(id),
            LibraryObservation::MoveOut => self.move_outs += 1,
            LibraryObservation::Scroll(offset) => self.scrolls.push(offset),
            LibraryObservation::ConfirmRemove => self.remove_confirms += 1,
            LibraryObservation::CancelRemove => self.remove_cancels += 1,
        }
    }
}

/// The callback bundle the Saved Pages Library stories route real Library
/// events through. Each callback re-enters the app entity (the same
/// downgraded handle seam the tree and the mode/state controls use), so
/// every interaction resolves through the app's production
/// `observe_library` method and re-renders the canvas.
pub struct LibraryObserver {
    pub on_close: LibraryAction,
    pub on_cycle_sort: LibraryAction,
    pub on_activate: LibraryItemAction,
    pub on_toggle_select: LibraryItemAction,
    pub on_rename: LibraryItemAction,
    pub on_rename_confirm: LibraryAction,
    pub on_rename_cancel: LibraryAction,
    pub on_collection_action: LibraryItemAction,
    pub on_collection_create: LibraryAction,
    pub on_collection_confirm: LibraryAction,
    pub on_collection_cancel: LibraryAction,
    pub on_confirm_remove: LibraryAction,
    pub on_cancel_remove: LibraryAction,
    pub on_clear_selection: LibraryAction,
    pub on_remove_selection: LibraryAction,
    pub on_toggle_move_sheet: LibraryAction,
    pub on_move_into: LibraryItemAction,
    pub on_move_out: LibraryAction,
    pub on_scroll: LibraryScrollAction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_updates_the_expected_field_only() {
        let mut recorder = LibraryRecorder::default();
        recorder.observe(LibraryObservation::Close);
        recorder.observe(LibraryObservation::CycleSort);
        recorder.observe(LibraryObservation::Activate(1));
        recorder.observe(LibraryObservation::ToggleSelect(2));
        recorder.observe(LibraryObservation::Rename(3));
        recorder.observe(LibraryObservation::RenameConfirm);
        recorder.observe(LibraryObservation::RenameCancel);
        recorder.observe(LibraryObservation::CollectionAction(10));
        recorder.observe(LibraryObservation::CollectionCreate);
        recorder.observe(LibraryObservation::CollectionConfirm);
        recorder.observe(LibraryObservation::CollectionCancel);
        recorder.observe(LibraryObservation::ClearSelection);
        recorder.observe(LibraryObservation::RemoveSelection);
        recorder.observe(LibraryObservation::ToggleMoveSheet);
        recorder.observe(LibraryObservation::MoveInto(11));
        recorder.observe(LibraryObservation::MoveOut);
        recorder.observe(LibraryObservation::Scroll(24));
        recorder.observe(LibraryObservation::ConfirmRemove);
        recorder.observe(LibraryObservation::CancelRemove);
        assert_eq!(recorder.closes, 1);
        assert_eq!(recorder.sort_cycles, 1);
        assert_eq!(recorder.opens, vec![1]);
        assert_eq!(recorder.select_toggles, vec![2]);
        assert_eq!(recorder.renames, vec![3]);
        assert_eq!(recorder.rename_confirms, 1);
        assert_eq!(recorder.rename_cancels, 1);
        assert_eq!(recorder.collection_actions, vec![10]);
        assert_eq!(recorder.collection_creates, 1);
        assert_eq!(recorder.collection_confirms, 1);
        assert_eq!(recorder.collection_cancels, 1);
        assert_eq!(recorder.clear_selections, 1);
        assert_eq!(recorder.remove_selections, 1);
        assert_eq!(recorder.move_sheet_toggles, 1);
        assert_eq!(recorder.move_ins, vec![11]);
        assert_eq!(recorder.move_outs, 1);
        assert_eq!(recorder.scrolls, vec![24]);
        assert_eq!(recorder.remove_confirms, 1);
        assert_eq!(recorder.remove_cancels, 1);
    }

    #[test]
    fn each_interaction_dispatches_exactly_once_per_observe() {
        // One observe call appends/increments exactly one field — the
        // "dispatch exactly once" contract the E2E asserts per real click.
        let mut recorder = LibraryRecorder::default();
        recorder.observe(LibraryObservation::Activate(7));
        recorder.observe(LibraryObservation::Activate(7));
        assert_eq!(recorder.opens, vec![7, 7], "one observe per dispatch");
        assert!(recorder.select_toggles.is_empty());
        assert!(recorder.renames.is_empty());
        assert_eq!(recorder.closes, 0);
        recorder.observe(LibraryObservation::ToggleSelect(7));
        assert_eq!(recorder.select_toggles, vec![7]);
        assert_eq!(recorder.opens, vec![7, 7], "other fields untouched");
        recorder.observe(LibraryObservation::MoveInto(3));
        recorder.observe(LibraryObservation::MoveInto(3));
        assert_eq!(recorder.move_ins, vec![3, 3]);
        assert_eq!(recorder.move_sheet_toggles, 0, "other fields untouched");
        recorder.observe(LibraryObservation::Scroll(48));
        assert_eq!(recorder.scrolls, vec![48]);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = LibraryRecorder::default();
        assert_eq!(recorder.closes, 0);
        assert_eq!(recorder.sort_cycles, 0);
        assert!(recorder.opens.is_empty());
        assert!(recorder.select_toggles.is_empty());
        assert!(recorder.renames.is_empty());
        assert_eq!(recorder.rename_confirms, 0);
        assert_eq!(recorder.rename_cancels, 0);
        assert!(recorder.collection_actions.is_empty());
        assert_eq!(recorder.collection_creates, 0);
        assert_eq!(recorder.collection_confirms, 0);
        assert_eq!(recorder.collection_cancels, 0);
        assert_eq!(recorder.clear_selections, 0);
        assert_eq!(recorder.remove_selections, 0);
        assert_eq!(recorder.move_sheet_toggles, 0);
        assert!(recorder.move_ins.is_empty());
        assert_eq!(recorder.move_outs, 0);
        assert!(recorder.scrolls.is_empty());
        assert_eq!(recorder.remove_confirms, 0);
        assert_eq!(recorder.remove_cancels, 0);
    }
}
