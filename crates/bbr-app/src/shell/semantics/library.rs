//! The §11.16 Saved Pages Library semantic node (B3 packet
//! `saved-pages-library`, DESIGN.md §15): the typed mirror of the `bbr-ui`
//! surface anatomy — the header (search field, sort control, close), the
//! scroll list (count line, flat collection rows, the bounded row window or
//! the decided-state row), and the destructive remove confirmation. Split
//! from `content` so no semantics module outgrows the ~500-line budget; the
//! node mirrors the same model the surface render and the typed snapshot
//! expose, and only advertises ids the render actually draws.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};
use bbr_ui::components::{
    library_collection_id, library_collection_remove_cancel_id,
    library_collection_remove_confirm_id, library_collection_remove_ok_id,
    library_collection_rename_cancel_id, library_collection_rename_confirm_id,
    library_collection_rename_input_id, library_move_id, LibraryCollectionAction,
    SavedPagesLibraryModel, LIBRARY_CLOSE_ID, LIBRARY_COLLECTION_CREATE_CANCEL_ID,
    LIBRARY_COLLECTION_CREATE_CONFIRM_ID, LIBRARY_COLLECTION_CREATE_ID,
    LIBRARY_COLLECTION_CREATE_INPUT_ID, LIBRARY_COUNT_ID, LIBRARY_EMPTY_ID, LIBRARY_ID,
    LIBRARY_LOADING_ID, LIBRARY_MOVE_NONE_ID, LIBRARY_NO_RESULTS_ID, LIBRARY_REMOVE_CONFIRM_ID,
    LIBRARY_SEARCH_ID, LIBRARY_SELECTION_CLEAR_ID, LIBRARY_SELECTION_ID, LIBRARY_SELECTION_MOVE_ID,
    LIBRARY_SELECTION_REMOVE_ID, LIBRARY_SORT_ID,
};

use super::node;
mod row;

use row::{library_row_node, state_row};

/// The §15 semantic node of the open Saved Pages Library, mirroring the
/// `bbr-ui` surface anatomy: the surface node, the header controls (search
/// field, sort control, close), the selection toolbar + move sheet (while
/// any record is selected), and the scroll list (count line, flat
/// collection rows with the create entry, the staged collection action
/// surface, and the bounded row window or the decided-state row), plus the
/// destructive remove confirmation (only while staged). The `focused` flag
/// mirrors the Library surface's real GPUI focus on the search field.
pub fn library_node(model: &SavedPagesLibraryModel, focused: bool) -> SemanticNode {
    let mut children: Vec<SemanticNode> = vec![
        node(
            LIBRARY_SEARCH_ID,
            SemanticRole::Field,
            Some("Search Saved Pages".into()),
            Some(model.search.clone()),
            false,
            // While a row rename or a collection input is staged, the
            // inline input owns surface focus; the search field mirrors the
            // real focus only at rest.
            focused && model.editing_row.is_none() && !model.collection_input_staged(),
            vec![],
        ),
        node(
            LIBRARY_SORT_ID,
            SemanticRole::Button,
            Some(model.sort.label().into()),
            None,
            false,
            false,
            vec![],
        ),
        node(
            LIBRARY_CLOSE_ID,
            SemanticRole::Button,
            Some("Close Library".into()),
            None,
            false,
            false,
            vec![],
        ),
    ];
    // The selection toolbar + move sheet render only while the multi-select
    // set is non-empty; the sheet only while open.
    if !model.selected.is_empty() {
        children.push(selection_toolbar_node(model));
        if model.move_sheet_open {
            children.push(move_sheet_node(model));
        }
    }
    let mut list_children: Vec<SemanticNode> = vec![node(
        LIBRARY_COUNT_ID,
        SemanticRole::Count,
        Some(format!("{}", model.total)),
        None,
        false,
        false,
        vec![],
    )];
    // The flat collections strip renders inside the list, one row per
    // collection (the member count rides the row's `value`), plus the
    // create entry.
    list_children.extend(model.collections.iter().map(|collection| {
        node(
            library_collection_id(collection.id),
            SemanticRole::Row,
            Some(collection.title.clone()),
            Some(format!("{}", collection.member_count)),
            false,
            false,
            vec![],
        )
    }));
    list_children.push(collection_create_node(model));
    // The staged collection action surface (rename input / remove confirm)
    // renders below the collections strip while the pointer cycle stages
    // an action.
    if let Some((id, action)) = model.collection_action {
        match action {
            LibraryCollectionAction::Rename => list_children.push(node(
                library_collection_rename_input_id(id),
                SemanticRole::Field,
                Some(if model.collection_draft.is_empty() {
                    "Rename collection".into()
                } else {
                    model.collection_draft.clone()
                }),
                None,
                false,
                focused && model.editing_row.is_none(),
                vec![
                    node(
                        library_collection_rename_confirm_id(id),
                        SemanticRole::Button,
                        Some("Confirm rename collection".into()),
                        None,
                        false,
                        false,
                        vec![],
                    ),
                    node(
                        library_collection_rename_cancel_id(id),
                        SemanticRole::Button,
                        Some("Cancel rename collection".into()),
                        None,
                        false,
                        false,
                        vec![],
                    ),
                ],
            )),
            LibraryCollectionAction::Remove => list_children.push(node(
                library_collection_remove_confirm_id(id),
                SemanticRole::Section,
                Some(
                    "Remove collection? Members move out first — records are never lost."
                        .to_string(),
                ),
                None,
                false,
                false,
                vec![
                    node(
                        library_collection_remove_ok_id(id),
                        SemanticRole::Button,
                        Some("Remove".into()),
                        None,
                        false,
                        false,
                        vec![],
                    ),
                    node(
                        library_collection_remove_cancel_id(id),
                        SemanticRole::Button,
                        Some("Cancel".into()),
                        None,
                        false,
                        false,
                        vec![],
                    ),
                ],
            )),
        }
    }
    // The decided states mirror the surface render: loading / empty / no
    // results state rows, or the bounded row window (never the full
    // collection).
    if model.is_loading() {
        list_children.push(state_row(LIBRARY_LOADING_ID, "Loading Saved Pages…"));
    } else if model.is_empty() {
        list_children.push(state_row(
            LIBRARY_EMPTY_ID,
            "No Saved Pages yet — press Cmd-D on any page to save it.",
        ));
    } else if model.has_no_results() {
        list_children.push(state_row(
            LIBRARY_NO_RESULTS_ID,
            &format!("No Saved Pages match “{}”.", model.search),
        ));
    } else {
        list_children.extend(model.rows.iter().map(|row| {
            let editing = model.editing_row == Some(row.bookmark);
            library_row_node(row, editing, &model.rename_draft, focused && editing)
        }));
    }
    children.push(node(
        "bbr-saved-pages-list",
        SemanticRole::List,
        Some("Saved Pages".into()),
        None,
        false,
        false,
        list_children,
    ));
    // The destructive remove confirmation renders only while staged.
    if model.remove_confirm {
        let noun = if model.remove_targets.len() == 1 {
            "1 Saved Page".to_string()
        } else {
            format!("{} Saved Pages", model.remove_targets.len())
        };
        children.push(node(
            LIBRARY_REMOVE_CONFIRM_ID,
            SemanticRole::Section,
            Some(format!("Remove {noun}? Open tabs are never touched.")),
            None,
            false,
            false,
            vec![
                node(
                    "bbr-saved-pages-remove-confirm-ok",
                    SemanticRole::Button,
                    Some("Remove".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    "bbr-saved-pages-remove-confirm-cancel",
                    SemanticRole::Button,
                    Some("Cancel".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
            ],
        ));
    }
    node(
        LIBRARY_ID,
        SemanticRole::Content,
        Some("Saved Pages Library".into()),
        None,
        false,
        focused,
        children,
    )
}

/// The §15 selection toolbar node: the count plus the Clear / Remove /
/// Move controls.
fn selection_toolbar_node(model: &SavedPagesLibraryModel) -> SemanticNode {
    let noun = if model.selected.len() == 1 {
        "1 selected".to_string()
    } else {
        format!("{} selected", model.selected.len())
    };
    node(
        LIBRARY_SELECTION_ID,
        SemanticRole::Section,
        Some(noun),
        None,
        false,
        false,
        vec![
            node(
                LIBRARY_SELECTION_CLEAR_ID,
                SemanticRole::Button,
                Some("Clear".into()),
                None,
                false,
                false,
                vec![],
            ),
            node(
                LIBRARY_SELECTION_REMOVE_ID,
                SemanticRole::Button,
                Some("Remove".into()),
                None,
                false,
                false,
                vec![],
            ),
            node(
                LIBRARY_SELECTION_MOVE_ID,
                SemanticRole::Button,
                Some(if model.move_sheet_open {
                    "Done".into()
                } else {
                    "Move to…".into()
                }),
                None,
                false,
                false,
                vec![],
            ),
        ],
    )
}

/// The §15 move-to sheet node: one option per flat collection (move the
/// selection into it) plus the "No collection" option (move it out).
fn move_sheet_node(model: &SavedPagesLibraryModel) -> SemanticNode {
    let mut options: Vec<SemanticNode> = model
        .collections
        .iter()
        .map(|collection| {
            node(
                library_move_id(collection.id),
                SemanticRole::Button,
                Some(format!("Move into {}", collection.title)),
                None,
                false,
                false,
                vec![],
            )
        })
        .collect();
    options.push(node(
        LIBRARY_MOVE_NONE_ID,
        SemanticRole::Button,
        Some("Move out of collection".into()),
        None,
        false,
        false,
        vec![],
    ));
    node(
        "bbr-saved-pages-move-sheet",
        SemanticRole::Section,
        Some("Move selected to".into()),
        None,
        false,
        false,
        options,
    )
}

/// The collection create entry node: the `+ New collection` row, or the
/// staged create input with its Confirm/Cancel while staged.
fn collection_create_node(model: &SavedPagesLibraryModel) -> SemanticNode {
    if model.collection_create_staged {
        return node(
            LIBRARY_COLLECTION_CREATE_INPUT_ID,
            SemanticRole::Field,
            Some(if model.collection_draft.is_empty() {
                "New collection".into()
            } else {
                model.collection_draft.clone()
            }),
            None,
            false,
            false,
            vec![
                node(
                    LIBRARY_COLLECTION_CREATE_CONFIRM_ID,
                    SemanticRole::Button,
                    Some("Confirm new collection".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    LIBRARY_COLLECTION_CREATE_CANCEL_ID,
                    SemanticRole::Button,
                    Some("Cancel new collection".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
            ],
        );
    }
    node(
        LIBRARY_COLLECTION_CREATE_ID,
        SemanticRole::Button,
        Some("New collection".into()),
        None,
        false,
        false,
        vec![],
    )
}
