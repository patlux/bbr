//! The §11.16 Library row-level semantic nodes (B3 packet
//! `saved-pages-library`): the per-row node folding in the §15 result ids
//! and the inline rename input while the row's rename is staged, plus the
//! typed state row (loading / empty / no results). Split from `library.rs`
//! so no semantics module outgrows the ~500-line budget.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};
use bbr_ui::components::{
    library_row_id, library_row_select_id, saved_page_result_id, saved_page_result_title_id,
    saved_page_result_url_id, saved_page_row_rename_cancel_id, saved_page_row_rename_confirm_id,
    saved_page_row_rename_id, saved_page_row_rename_input_id, LibraryRowSpec,
};

use super::node;

/// One §11.16 Library row node: the stable row id plus the §15 result ids
/// the `bbr-ui` row folds in (the shared derivation the visible-id mirror
/// uses), with the selection state the row renders. While the row's rename
/// is staged (`editing`), the node mirrors the inline rename input the
/// render draws instead — the input Field (the staged draft), its
/// Confirm/Cancel controls, and the selection toggle — with the input
/// owning surface focus while editing.
pub(super) fn library_row_node(
    row: &LibraryRowSpec,
    editing: bool,
    draft: &str,
    focused: bool,
) -> SemanticNode {
    let mut row_node = node(
        library_row_id(row.bookmark),
        SemanticRole::Row,
        Some(row.title.clone()),
        Some(row.url.clone()),
        row.selected,
        false,
        vec![],
    );
    if editing {
        row_node.children.push(node(
            saved_page_row_rename_input_id(row.bookmark),
            SemanticRole::Field,
            Some(if draft.is_empty() {
                "Rename".to_string()
            } else {
                draft.to_string()
            }),
            None,
            false,
            focused,
            vec![],
        ));
        row_node.children.push(node(
            saved_page_row_rename_confirm_id(row.bookmark),
            SemanticRole::Button,
            Some("Confirm rename".into()),
            None,
            false,
            false,
            vec![],
        ));
        row_node.children.push(node(
            saved_page_row_rename_cancel_id(row.bookmark),
            SemanticRole::Button,
            Some("Cancel rename".into()),
            None,
            false,
            false,
            vec![],
        ));
        row_node.children.push(node(
            library_row_select_id(row.bookmark),
            SemanticRole::Button,
            Some(if row.selected {
                "Deselect".to_string()
            } else {
                "Select".to_string()
            }),
            None,
            false,
            false,
            vec![],
        ));
    } else {
        row_node.children.push(node(
            saved_page_result_id(row.bookmark),
            SemanticRole::Result,
            Some(row.title.clone()),
            None,
            row.selected,
            false,
            vec![],
        ));
        row_node.children.push(node(
            saved_page_result_title_id(row.bookmark),
            SemanticRole::Title,
            Some(row.title.clone()),
            None,
            row.selected,
            false,
            vec![],
        ));
        row_node.children.push(node(
            saved_page_result_url_id(row.bookmark),
            SemanticRole::State,
            Some(row.url.clone()),
            None,
            false,
            false,
            vec![],
        ));
        row_node.children.push(node(
            library_row_select_id(row.bookmark),
            SemanticRole::Button,
            Some(if row.selected {
                "Deselect".to_string()
            } else {
                "Select".to_string()
            }),
            None,
            false,
            false,
            vec![],
        ));
        row_node.children.push(node(
            saved_page_row_rename_id(row.bookmark),
            SemanticRole::Button,
            Some("Rename".into()),
            None,
            false,
            false,
            vec![],
        ));
    }
    row_node
}

/// One typed state row (loading / empty / no results) with its stable §15
/// id, mirroring `bbr-ui`'s `library_state_row`.
pub(super) fn state_row(id: &str, text: &str) -> SemanticNode {
    node(
        id,
        SemanticRole::State,
        Some(text.to_string()),
        None,
        false,
        false,
        vec![],
    )
}
