//! The Library observer builder (B3 packet `saved-pages-library`, Gallery
//! G2/G3): builds the `LibraryObserver` callback bundle that re-enters the
//! app entity through the production `observe_library` path. Kept in its
//! own focused module so `app.rs` stays under the ~500-line budget.

use std::rc::Rc;

use gpui::{App, WeakEntity, Window};

use crate::app::GalleryApp;
use crate::library_recorder::{LibraryObservation, LibraryObserver};

/// Build the Library observer: every callback re-enters the app entity
/// (the same downgraded handle seam the tree and the mode/state controls
/// use), so each real Library event resolves through the app's production
/// `observe_library` method and re-renders the canvas.
pub fn build_library_observer(entity: WeakEntity<GalleryApp>) -> LibraryObserver {
    let on_library = move |event: LibraryObservation, _window: &mut Window, cx: &mut App| {
        if let Some(entity) = entity.upgrade() {
            entity.update(cx, |app, cx| app.observe_library(event, cx));
        }
    };
    let on_close = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| on(LibraryObservation::Close, window, cx))
    };
    let on_cycle_sort = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::CycleSort, window, cx)
        })
    };
    let on_activate = {
        let on = on_library.clone();
        Rc::new(move |id: u64, window: &mut Window, cx: &mut App| {
            on(LibraryObservation::Activate(id), window, cx)
        })
    };
    let on_toggle_select = {
        let on = on_library.clone();
        Rc::new(move |id: u64, window: &mut Window, cx: &mut App| {
            on(LibraryObservation::ToggleSelect(id), window, cx)
        })
    };
    let on_rename = {
        let on = on_library.clone();
        Rc::new(move |id: u64, window: &mut Window, cx: &mut App| {
            on(LibraryObservation::Rename(id), window, cx)
        })
    };
    let on_rename_confirm = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::RenameConfirm, window, cx)
        })
    };
    let on_rename_cancel = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::RenameCancel, window, cx)
        })
    };
    let on_collection_action = {
        let on = on_library.clone();
        Rc::new(move |id: u64, window: &mut Window, cx: &mut App| {
            on(LibraryObservation::CollectionAction(id), window, cx)
        })
    };
    let on_collection_create = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::CollectionCreate, window, cx)
        })
    };
    let on_collection_confirm = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::CollectionConfirm, window, cx)
        })
    };
    let on_collection_cancel = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::CollectionCancel, window, cx)
        })
    };
    let on_clear_selection = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::ClearSelection, window, cx)
        })
    };
    let on_remove_selection = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::RemoveSelection, window, cx)
        })
    };
    let on_toggle_move_sheet = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::ToggleMoveSheet, window, cx)
        })
    };
    let on_move_into = {
        let on = on_library.clone();
        Rc::new(move |id: u64, window: &mut Window, cx: &mut App| {
            on(LibraryObservation::MoveInto(id), window, cx)
        })
    };
    let on_move_out = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::MoveOut, window, cx)
        })
    };
    let on_scroll = {
        let on = on_library.clone();
        Rc::new(move |offset: usize, window: &mut Window, cx: &mut App| {
            on(LibraryObservation::Scroll(offset), window, cx)
        })
    };
    let on_confirm_remove = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::ConfirmRemove, window, cx)
        })
    };
    let on_cancel_remove = {
        let on = on_library.clone();
        Rc::new(move |window: &mut Window, cx: &mut App| {
            on(LibraryObservation::CancelRemove, window, cx)
        })
    };
    LibraryObserver {
        on_close,
        on_cycle_sort,
        on_activate,
        on_toggle_select,
        on_rename,
        on_rename_confirm,
        on_rename_cancel,
        on_collection_action,
        on_collection_create,
        on_collection_confirm,
        on_collection_cancel,
        on_clear_selection,
        on_remove_selection,
        on_toggle_move_sheet,
        on_move_into,
        on_move_out,
        on_scroll,
        on_confirm_remove,
        on_cancel_remove,
    }
}
