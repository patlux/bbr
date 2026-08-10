//! The §15 visible-id mirror of the rendered Library surface (B3 packet
//! `saved-pages-library`), split from `model.rs` so no file outgrows the
//! ~500-line budget. A genuine test/testkit mirror helper (the normal
//! binary never renders the ids), so the module compiles only for the test
//! harness and the testkit feature.

use bbr_ui::components::SavedPagesLibraryModel;

/// The §15 visible-id set of the rendered Library surface, mirroring the
/// `bbr-ui` surface exactly (the ids the render draws when the model is
/// presented): the surface/header ids, the selection toolbar + move sheet,
/// the count line, the flat collection rows + create entry, the staged
/// collection action surface, the bounded row window, the decided-state
/// row, and the remove confirmation. Never advertises an id the surface
/// did not render.
pub(crate) fn library_visible_ids(model: &SavedPagesLibraryModel) -> Vec<String> {
    use bbr_ui::components::{
        library_collection_id, library_collection_remove_cancel_id,
        library_collection_remove_confirm_id, library_collection_remove_ok_id,
        library_collection_rename_cancel_id, library_collection_rename_confirm_id,
        library_collection_rename_input_id, library_move_id, library_row_ids,
        LibraryCollectionAction, LIBRARY_CLOSE_ID, LIBRARY_COLLECTION_CREATE_CANCEL_ID,
        LIBRARY_COLLECTION_CREATE_CONFIRM_ID, LIBRARY_COLLECTION_CREATE_ID,
        LIBRARY_COLLECTION_CREATE_INPUT_ID, LIBRARY_COUNT_ID, LIBRARY_EMPTY_ID, LIBRARY_ID,
        LIBRARY_LOADING_ID, LIBRARY_MOVE_NONE_ID, LIBRARY_NO_RESULTS_ID, LIBRARY_REMOVE_CONFIRM_ID,
        LIBRARY_SEARCH_ID, LIBRARY_SELECTION_CLEAR_ID, LIBRARY_SELECTION_ID,
        LIBRARY_SELECTION_MOVE_ID, LIBRARY_SELECTION_REMOVE_ID, LIBRARY_SORT_ID,
    };
    let mut ids = vec![
        LIBRARY_ID.to_string(),
        LIBRARY_SEARCH_ID.to_string(),
        LIBRARY_SORT_ID.to_string(),
        LIBRARY_CLOSE_ID.to_string(),
        "bbr-saved-pages-list".to_string(),
        LIBRARY_COUNT_ID.to_string(),
    ];
    // The selection toolbar + move sheet render only while the multi-select
    // set is non-empty; the sheet only while open.
    if !model.selected.is_empty() {
        ids.push(LIBRARY_SELECTION_ID.to_string());
        ids.push(LIBRARY_SELECTION_CLEAR_ID.to_string());
        ids.push(LIBRARY_SELECTION_REMOVE_ID.to_string());
        ids.push(LIBRARY_SELECTION_MOVE_ID.to_string());
        if model.move_sheet_open {
            ids.push(LIBRARY_MOVE_NONE_ID.to_string());
            ids.extend(
                model
                    .collections
                    .iter()
                    .map(|collection| library_move_id(collection.id).to_string()),
            );
        }
    }
    ids.extend(
        model
            .collections
            .iter()
            .map(|collection| library_collection_id(collection.id).to_string()),
    );
    // The create entry keeps one stable slot id; the staged create form
    // replaces it with the input/confirm/cancel ids.
    if model.collection_create_staged {
        ids.push(LIBRARY_COLLECTION_CREATE_INPUT_ID.to_string());
        ids.push(LIBRARY_COLLECTION_CREATE_CONFIRM_ID.to_string());
        ids.push(LIBRARY_COLLECTION_CREATE_CANCEL_ID.to_string());
    } else {
        ids.push(LIBRARY_COLLECTION_CREATE_ID.to_string());
    }
    // The staged collection action surface.
    if let Some((id, action)) = model.collection_action {
        match action {
            LibraryCollectionAction::Rename => {
                ids.push(library_collection_rename_input_id(id).to_string());
                ids.push(library_collection_rename_confirm_id(id).to_string());
                ids.push(library_collection_rename_cancel_id(id).to_string());
            }
            LibraryCollectionAction::Remove => {
                ids.push(library_collection_remove_confirm_id(id).to_string());
                ids.push(library_collection_remove_ok_id(id).to_string());
                ids.push(library_collection_remove_cancel_id(id).to_string());
            }
        }
    }
    if model.is_loading() {
        ids.push(LIBRARY_LOADING_ID.to_string());
    } else if model.is_empty() {
        ids.push(LIBRARY_EMPTY_ID.to_string());
    } else if model.has_no_results() {
        ids.push(LIBRARY_NO_RESULTS_ID.to_string());
    } else {
        for row in &model.rows {
            ids.extend(
                library_row_ids(row.bookmark, model.editing_row == Some(row.bookmark))
                    .into_iter()
                    .map(|id| id.to_string()),
            );
        }
    }
    if model.remove_confirm {
        ids.push(LIBRARY_REMOVE_CONFIRM_ID.to_string());
        ids.push("bbr-saved-pages-remove-confirm-ok".to_string());
        ids.push("bbr-saved-pages-remove-confirm-cancel".to_string());
    }
    ids
}
