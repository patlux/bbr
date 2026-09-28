//! Blueprint D20 (DESIGN.md §11.11): the inline folder-rename field that
//! replaces a folder row's name slot while the rename is open.

use gpui::{div, prelude::*, Div, MouseButton, Stateful};

use super::{folder_name_id, FOLDER_RENAME_ID};
use crate::theme::{Radius, Theme};

/// The inline rename field's rendered state: the draft text and whether
/// the whole draft is selected (a new folder opens with its default title
/// selected, so the first typed character replaces it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRenameField {
    pub draft: String,
    pub selected: bool,
}

/// The rename field's name slot (`bbr-folder-rename`): the draft in the
/// folder-name type, highlighted while wholly selected, followed by the
/// 1.5 pt accent caret. The name id stays in the tree on the slot so the
/// row's §15 id set does not change while renaming.
pub(super) fn rename_slot(theme: &Theme, index: usize, field: &FolderRenameField) -> Stateful<Div> {
    let p = theme.palette;
    let text = div()
        .text_size(theme.type_.title.size)
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(p.ink)
        .overflow_hidden()
        .whitespace_nowrap()
        .when(field.selected && !field.draft.is_empty(), |d| {
            d.bg(p.tint_accent).rounded(Radius.small())
        })
        .child(field.draft.clone());
    div()
        .id(folder_name_id(index))
        .debug_selector(|| FOLDER_RENAME_ID.to_string())
        .flex_1()
        .min_w_0()
        .flex()
        .flex_row()
        .items_center()
        .overflow_hidden()
        // A press on the field keeps it open: stop the bubble before the
        // root's outside-press commit sees it (§13 one pointer action).
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(text)
        .child(div().w(gpui::px(1.5)).h(gpui::px(16.0)).bg(p.accent))
}
