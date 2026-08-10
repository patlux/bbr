//! Saved Pages Library §15 ids (DESIGN.md §11.16, B3 packet
//! `saved-pages-library`): the stable surface ids and the deterministic
//! per-row/per-collection id builders. The ids are the snapshot/visible-id
//! contract, kept in one focused module so the render modules and the
//! `model` stay under the ~500-line budget.

use gpui::SharedString;

/// The §15 id of the Library surface.
pub const LIBRARY_ID: &str = "bbr-saved-pages-library";
/// The §15 id of the Library search field.
pub const LIBRARY_SEARCH_ID: &str = "bbr-saved-pages-search";
/// The §15 id of the Library sort control.
pub const LIBRARY_SORT_ID: &str = "bbr-saved-pages-sort";
/// The §15 id of the destructive remove confirmation.
pub const LIBRARY_REMOVE_CONFIRM_ID: &str = "bbr-saved-pages-remove-confirm";
/// The §15 id of the empty Library state.
pub const LIBRARY_EMPTY_ID: &str = "bbr-saved-pages-empty";
/// The §15 id of the loading state.
pub const LIBRARY_LOADING_ID: &str = "bbr-saved-pages-loading";
/// The §15 id of the no-search-results state.
pub const LIBRARY_NO_RESULTS_ID: &str = "bbr-saved-pages-no-results";
/// The §15 id of the count line.
pub const LIBRARY_COUNT_ID: &str = "bbr-saved-pages-count";
/// The §15 id of the close control.
pub const LIBRARY_CLOSE_ID: &str = "bbr-saved-pages-close";
/// The §15 id of the Collections section header.
pub const LIBRARY_COLLECTIONS_ID: &str = "bbr-saved-pages-collections";
/// The §15 id of the selection toolbar (present while any record is
/// selected): the count plus the Clear / Remove / Move controls.
pub const LIBRARY_SELECTION_ID: &str = "bbr-saved-pages-selection";
/// The §15 id of the selection toolbar Clear control (clears the whole
/// multi-select set).
pub const LIBRARY_SELECTION_CLEAR_ID: &str = "bbr-saved-pages-selection-clear";
/// The §15 id of the selection toolbar Remove control (stages the
/// destructive remove confirmation for the selected records).
pub const LIBRARY_SELECTION_REMOVE_ID: &str = "bbr-saved-pages-selection-remove";
/// The §15 id of the selection toolbar Move control (opens/closes the
/// move-to sheet).
pub const LIBRARY_SELECTION_MOVE_ID: &str = "bbr-saved-pages-selection-move";
/// The §15 id of the move-to sheet's "No collection" option (moves the
/// selected records out of any collection).
pub const LIBRARY_MOVE_NONE_ID: &str = "bbr-saved-pages-move-none";
/// The §15 id of the collection create entry ("+ New collection" row at
/// the end of the flat collections strip).
pub const LIBRARY_COLLECTION_CREATE_ID: &str = "bbr-saved-pages-collection-create";
/// The §15 id of the staged collection-create input field.
pub const LIBRARY_COLLECTION_CREATE_INPUT_ID: &str = "bbr-saved-pages-collection-create-input";
/// The §15 id of the staged collection-create Confirm control.
pub const LIBRARY_COLLECTION_CREATE_CONFIRM_ID: &str = "bbr-saved-pages-collection-create-confirm";
/// The §15 id of the staged collection-create Cancel control.
pub const LIBRARY_COLLECTION_CREATE_CANCEL_ID: &str = "bbr-saved-pages-collection-create-cancel";

/// The §15 move-to sheet option id `bbr-saved-pages-move-<collection-id>`
/// (moves the selected records into this flat collection).
pub fn library_move_id(collection: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-move-{collection}"))
}

/// The §15 staged collection-rename input id
/// `bbr-saved-pages-collection-rename-input-<collection-id>`.
pub fn library_collection_rename_input_id(collection: u64) -> SharedString {
    SharedString::from(format!(
        "bbr-saved-pages-collection-rename-input-{collection}"
    ))
}

/// The §15 staged collection-rename Confirm id
/// `bbr-saved-pages-collection-rename-confirm-<collection-id>`.
pub fn library_collection_rename_confirm_id(collection: u64) -> SharedString {
    SharedString::from(format!(
        "bbr-saved-pages-collection-rename-confirm-{collection}"
    ))
}

/// The §15 staged collection-rename Cancel id
/// `bbr-saved-pages-collection-rename-cancel-<collection-id>`.
pub fn library_collection_rename_cancel_id(collection: u64) -> SharedString {
    SharedString::from(format!(
        "bbr-saved-pages-collection-rename-cancel-{collection}"
    ))
}

/// The §15 staged collection-remove confirmation id
/// `bbr-saved-pages-collection-remove-confirm-<collection-id>`.
pub fn library_collection_remove_confirm_id(collection: u64) -> SharedString {
    SharedString::from(format!(
        "bbr-saved-pages-collection-remove-confirm-{collection}"
    ))
}

/// The §15 staged collection-remove Confirm id
/// `bbr-saved-pages-collection-remove-ok-<collection-id>`.
pub fn library_collection_remove_ok_id(collection: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-collection-remove-ok-{collection}"))
}

/// The §15 staged collection-remove Cancel id
/// `bbr-saved-pages-collection-remove-cancel-<collection-id>`.
pub fn library_collection_remove_cancel_id(collection: u64) -> SharedString {
    SharedString::from(format!(
        "bbr-saved-pages-collection-remove-cancel-{collection}"
    ))
}

/// The §15 result-row id `bbr-saved-page-result-<bookmark-id>` (DESIGN.md
/// §11.16 — shared by the Palette result and the Library row).
pub fn saved_page_result_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-page-result-{bookmark}"))
}

/// The §15 result-title id `bbr-saved-page-result-title-<bookmark-id>`.
pub fn saved_page_result_title_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-page-result-title-{bookmark}"))
}

/// The §15 result-url id `bbr-saved-page-result-url-<bookmark-id>`.
pub fn saved_page_result_url_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-page-result-url-{bookmark}"))
}

/// The §15 Library-row id `bbr-saved-pages-row-<bookmark-id>`.
pub fn library_row_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-row-{bookmark}"))
}

/// The §15 collection-row id `bbr-saved-pages-collection-<collection-id>`.
pub fn library_collection_id(collection: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-collection-{collection}"))
}

/// The §15 id of the row rename control (the pointer rename entry; the
/// button that stages the inline rename input for this row).
pub fn saved_page_row_rename_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-rename-{bookmark}"))
}

/// The §15 id of the staged rename input field (rendered while the row's
/// rename is staged, replacing the rename button).
pub fn saved_page_row_rename_input_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-rename-input-{bookmark}"))
}

/// The §15 id of the staged rename Confirm control.
pub fn saved_page_row_rename_confirm_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-rename-confirm-{bookmark}"))
}

/// The §15 id of the staged rename Cancel control.
pub fn saved_page_row_rename_cancel_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-rename-cancel-{bookmark}"))
}

/// The §15 id of the row multi-select toggle.
pub fn library_row_select_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-select-{bookmark}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_ids_are_stable_and_scoped_per_row() {
        // The pointer rename entry stages three deterministic ids: the
        // input field, Confirm, and Cancel — each scoped to the row's
        // `BookmarkId` (DESIGN §15), distinct from the entry button id.
        assert_eq!(
            saved_page_row_rename_id(7).as_ref(),
            "bbr-saved-pages-rename-7"
        );
        assert_eq!(
            saved_page_row_rename_input_id(7).as_ref(),
            "bbr-saved-pages-rename-input-7"
        );
        assert_eq!(
            saved_page_row_rename_confirm_id(7).as_ref(),
            "bbr-saved-pages-rename-confirm-7"
        );
        assert_eq!(
            saved_page_row_rename_cancel_id(7).as_ref(),
            "bbr-saved-pages-rename-cancel-7"
        );
        // A different row never shares an id.
        assert_ne!(
            saved_page_row_rename_input_id(7).as_ref(),
            saved_page_row_rename_input_id(8).as_ref()
        );
    }

    #[test]
    fn collection_surface_ids_are_stable_and_scoped_per_collection() {
        // The move sheet and the staged collection action surface derive
        // deterministic ids scoped to the collection id, distinct from the
        // row id and the create entry.
        assert_eq!(library_move_id(7).as_ref(), "bbr-saved-pages-move-7");
        assert_eq!(
            library_collection_rename_input_id(7).as_ref(),
            "bbr-saved-pages-collection-rename-input-7"
        );
        assert_eq!(
            library_collection_rename_confirm_id(7).as_ref(),
            "bbr-saved-pages-collection-rename-confirm-7"
        );
        assert_eq!(
            library_collection_rename_cancel_id(7).as_ref(),
            "bbr-saved-pages-collection-rename-cancel-7"
        );
        assert_eq!(
            library_collection_remove_confirm_id(7).as_ref(),
            "bbr-saved-pages-collection-remove-confirm-7"
        );
        assert_eq!(
            library_collection_remove_ok_id(7).as_ref(),
            "bbr-saved-pages-collection-remove-ok-7"
        );
        assert_eq!(
            library_collection_remove_cancel_id(7).as_ref(),
            "bbr-saved-pages-collection-remove-cancel-7"
        );
        assert_ne!(library_move_id(7).as_ref(), library_move_id(8).as_ref());
        assert_eq!(
            LIBRARY_COLLECTION_CREATE_ID,
            "bbr-saved-pages-collection-create"
        );
        assert_eq!(
            LIBRARY_COLLECTION_CREATE_INPUT_ID,
            "bbr-saved-pages-collection-create-input"
        );
        assert_eq!(LIBRARY_SELECTION_ID, "bbr-saved-pages-selection");
        assert_eq!(
            LIBRARY_SELECTION_CLEAR_ID,
            "bbr-saved-pages-selection-clear"
        );
        assert_eq!(
            LIBRARY_SELECTION_REMOVE_ID,
            "bbr-saved-pages-selection-remove"
        );
        assert_eq!(LIBRARY_SELECTION_MOVE_ID, "bbr-saved-pages-selection-move");
        assert_eq!(LIBRARY_MOVE_NONE_ID, "bbr-saved-pages-move-none");
    }
}
