//! Saved Pages Library gateway dispatch (B3 packet
//! `saved-pages-library`, testkit feature only).
//!
//! One `Request` → `Response` arm per Library capability, split from
//! `super::dispatch` so no gateway module outgrows the ~500-line budget.
//! Every mutation routes through the app's Library composition and the one
//! AppStore/SavedPageCommand authority — no second persistence, command, or
//! identity policy exists in the gateway.

use bbr_testkit::protocol::{Error, Request, Response, SavedPagesLibraryCollectionOp};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the Library arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_library(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::OpenSavedPagesLibrary => with_app(window, app, cx, |view, window, cx| {
            view.open_saved_pages_library(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::CloseSavedPagesLibrary => with_app(window, app, cx, |view, window, cx| {
            view.close_saved_pages_library(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::SetSavedPagesLibrarySearch { text } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_set_search(text, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::CycleSavedPagesLibrarySort => with_app(window, app, cx, |view, _window, cx| {
            view.library_cycle_sort(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::ToggleSavedPagesLibrarySelect { id } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_toggle_select(*id, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::ClearSavedPagesLibrarySelection => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_clear_selection(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::SetSavedPagesLibraryScroll { offset } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_set_scroll(*offset, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::BeginSavedPagesLibraryRemove { ids } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_begin_remove(ids, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::ConfirmSavedPagesLibraryRemove => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_confirm_remove(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::CancelSavedPagesLibraryRemove => with_app(window, app, cx, |view, _window, cx| {
            view.library_cancel_remove(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::RenameSavedPagesLibraryRow { id, label } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_rename(*id, label, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::BeginSavedPagesLibraryRowRename { id } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_begin_rename(*id, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::SetSavedPagesLibraryRenameDraft { text } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_rename_draft(text, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::ConfirmSavedPagesLibraryRename => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_confirm_rename(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::CancelSavedPagesLibraryRename => with_app(window, app, cx, |view, _window, cx| {
            view.library_cancel_rename(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::SavedPagesLibraryCollection { op } => {
            with_app(window, app, cx, |view, _window, cx| match op {
                SavedPagesLibraryCollectionOp::Create { title } => {
                    if view.library_create_collection(title, cx).is_some() {
                        Response::Ok
                    } else {
                        Response::Error(Error::new(
                            "library_collection_rejected",
                            "the collection title is invalid",
                        ))
                    }
                }
                SavedPagesLibraryCollectionOp::Rename { id, title } => {
                    view.library_rename_collection(*id, title, cx);
                    Response::Ok
                }
                SavedPagesLibraryCollectionOp::Remove { id } => {
                    view.library_remove_collection(*id, cx);
                    Response::Ok
                }
            })
            .unwrap_or_else(err_response)
        }
        Request::MoveSavedPagesLibrarySelection { collection } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_move_selection(*collection, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The pointer path of the selection toolbar Remove control: stage
        // the destructive remove for exactly the current selection.
        Request::ClickSavedPagesLibrarySelectionRemove => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_remove_selected(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The pointer path of the selection toolbar Move control: toggle
        // the move-to sheet.
        Request::ClickSavedPagesLibrarySelectionMove => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_toggle_move_sheet(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The pointer path of the selection toolbar Clear control: clear
        // the whole multi-select set.
        Request::ClickSavedPagesLibrarySelectionClear => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_clear_selection(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The pointer path of a move-to sheet option (per-collection and
        // "No collection"): move the current multi-select into (or out of)
        // one flat collection through the one AppStore authority.
        Request::ClickSavedPagesLibraryMoveOption { collection } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_move_selection(*collection, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The pointer path of a flat collection row: stage the
        // deterministic rename → remove → none cycle.
        Request::ClickSavedPagesLibraryCollectionRow { id } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_collection_click(*id, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The pointer path of the collection create entry: stage the
        // inline create input.
        Request::ClickSavedPagesLibraryCollectionCreate => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_begin_collection_create(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The staged collection action Confirm (create/rename/remove); an
        // empty/invalid create draft commits nothing (the store's invalid
        // title validation is a no-op, exactly like the direct create arm's
        // rejection).
        Request::ConfirmSavedPagesLibraryCollection => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_collection_confirm(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The staged collection action Cancel.
        Request::CancelSavedPagesLibraryCollection => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_collection_cancel(cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The staged collection draft setter (the typed seam of the staged
        // inline input).
        Request::SetSavedPagesLibraryCollectionDraft { text } => {
            with_app(window, app, cx, |view, _window, cx| {
                view.library_set_collection_draft(text, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        // The production wheel chain: the pure clamp math the list wheel
        // listener runs, then the one production scroll entry.
        Request::ScrollSavedPagesLibraryWheel { delta_rows } => {
            with_app(window, app, cx, |view, _window, cx| {
                let model = view.library_model();
                let next = bbr_ui::components::scroll_offset_for_wheel(
                    model.scroll_offset,
                    model.total,
                    model.viewport_rows,
                    *delta_rows,
                );
                view.library_set_scroll(next, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::ActivateSavedPagesLibraryRow { id } => {
            with_app(window, app, cx, |view, window, cx| {
                let opened = view.library_activate(*id, window, cx);
                if opened {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "saved_page_missing",
                        "the Saved Page record is gone or the tab could not be opened",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }
        _ => return None,
    };
    Some(response)
}
