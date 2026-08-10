//! Saved Pages Library callbacks (DESIGN.md §11.16, B3 packet
//! `saved-pages-library`): the typed listener bundle the app wires onto
//! the `bbr-ui` surface. Kept in its own focused module so the `model`
//! stays under the ~500-line budget.
//!
//! Every callback is a plain GPUI handler the app composes through its one
//! AppStore/SavedPageCommand authority — the surface never holds a
//! repository, controller, or network client.

use std::rc::Rc;

use gpui::{App, Window};

/// A Library callback carrying a stable `BookmarkId`/`CollectionId` (the
/// per-row/per-collection listeners the app wires through its production
/// entity handle; `Rc` so the surface clones one handler into every row).
pub type LibraryItemAction = Rc<dyn Fn(u64, &mut Window, &mut App)>;
/// A Library callback with no payload (close, sort cycle, remove confirm,
/// selection clear/remove, move sheet, collection create).
pub type LibraryAction = Rc<dyn Fn(&mut Window, &mut App)>;
/// A Library scroll callback carrying the new clamped absolute scroll
/// offset in row units (the `bbr-ui` list wheel listener computes the
/// offset; the app stores it through its one `library_set_scroll`
/// authority).
pub type LibraryScrollAction = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The full production listener bundle of the §11.16 Library surface:
/// every control the surface draws routes through one of these callbacks,
/// so the normal binary reaches every packet operation through the same
/// composition the E2E asserts.
pub struct LibraryCallbacks {
    /// The header close control.
    pub on_close: LibraryAction,
    /// The deterministic sort cycle control.
    pub on_cycle_sort: LibraryAction,
    /// A row activation (open exactly one fresh regular tab through B2).
    pub on_activate: LibraryItemAction,
    /// A row multi-select toggle.
    pub on_toggle_select: LibraryItemAction,
    /// The row rename entry (stage the inline rename input).
    pub on_rename: LibraryItemAction,
    /// The staged row rename Confirm.
    pub on_rename_confirm: LibraryAction,
    /// The staged row rename Cancel.
    pub on_rename_cancel: LibraryAction,
    /// A flat collection row click (stage the rename → remove → none
    /// pointer cycle).
    pub on_collection_action: LibraryItemAction,
    /// The collection create entry (stage the inline create input).
    pub on_collection_create: LibraryAction,
    /// Commit whatever collection action is staged (create / rename /
    /// remove) through the one AppStore authority.
    pub on_collection_confirm: LibraryAction,
    /// Leave the staged collection action without committing.
    pub on_collection_cancel: LibraryAction,
    /// The destructive remove Confirm.
    pub on_confirm_remove: LibraryAction,
    /// The destructive remove Cancel.
    pub on_cancel_remove: LibraryAction,
    /// The selection toolbar Clear control (clear the multi-select set).
    pub on_clear_selection: LibraryAction,
    /// The selection toolbar Remove control (stage the destructive remove
    /// for the selected records).
    pub on_remove_selection: LibraryAction,
    /// The selection toolbar Move control (toggle the move-to sheet).
    pub on_toggle_move_sheet: LibraryAction,
    /// A move-to sheet option: move the selection into this collection.
    pub on_move_into: LibraryItemAction,
    /// The move-to sheet "No collection" option: move the selection out of
    /// any collection.
    pub on_move_out: LibraryAction,
    /// The bounded list wheel listener: the new clamped absolute scroll
    /// offset in row units.
    pub on_scroll: LibraryScrollAction,
}
