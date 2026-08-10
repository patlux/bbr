//! App-owned Saved Pages Library composition (B3 packet
//! `saved-pages-library`): the open/close entry, the local search/sort/
//! selection/remove-confirm/scroll state, the B1/B2 binding for every
//! Library operation, the native-view hide/restore following the existing
//! error-surface visibility authority, and the render wrapper that builds
//! the `bbr-ui` model and wires the production listener bundles.
//!
//! The Library is one ordinary GPUI internal chrome surface in the content
//! area — never a permanent Sidebar region, a WebKit page, a Portal, or a
//! second Saved Page authority. It consumes the one loaded in-memory
//! provider snapshot (B1/B2) through [`BbrApp::store`], and every mutation
//! persists through the same [`crate::store::AppStore`] operations the B2
//! command authority uses; no second persistence, command, or identity
//! policy exists here.
//!
//! # Native-view hide/restore
//!
//! While the Library is open the active view's native slot is hidden (never
//! destroyed — identity untouched) and the surface renders in its place;
//! on close the exact view the Library hid is restored. This mirrors the
//! error-surface visibility authority (`sync_error_surface_visibility`):
//! the Library is the single visibility owner while open, and the error
//! sync yields to it.

use std::collections::HashSet;
use std::rc::Rc;

use gpui::{Context, KeyDownEvent, Stateful, Window};

use bbr_command::id::BookmarkId;
use bbr_ui::components::{LibraryAction, LibraryCallbacks, LibraryItemAction, LibrarySort};
use bbr_ui::theme::Theme;

use super::BbrApp;
use crate::app::library::model::{build_library_model, LibraryStateView};
use crate::command_bar::CommandBarMode;
use bbr_web::WebViewId;

#[cfg(feature = "testkit")]
pub(crate) mod benchmarks;
pub(crate) mod collections;
pub(crate) mod model;
pub(crate) mod rename;
#[cfg(test)]
mod tests;

/// The default bounded viewport (rows rendered) for the composition slice:
/// the deterministic production window the model derives from the tracked
/// scroll offset. The E2E/performance slice wires the measured viewport;
/// this constant keeps the model building deterministic here.
pub(crate) const LIBRARY_VIEWPORT_ROWS: usize = 24;

/// App-local Library presentation state: open flag, the search query, the
/// deterministic sort, the multi-select set, the destructive remove
/// confirmation, the tracked scroll offset, the pending collection action
/// surface, the move-to sheet, and the set of native views hidden by the
/// open Library (restored on close; identity untouched).
#[derive(Debug, Default)]
pub(crate) struct LibraryState {
    pub open: bool,
    pub search: String,
    pub sort: LibrarySort,
    pub selected: Vec<u64>,
    pub remove_confirm: bool,
    pub remove_targets: Vec<u64>,
    pub scroll_offset: usize,
    /// The collection row whose pointer action is staged (rename → remove
    /// cycle); the action surface renders the staged inline form.
    pub collection_action: Option<(u64, bbr_ui::components::LibraryCollectionAction)>,
    /// The staged collection rename/create draft label (empty at rest).
    pub collection_draft: String,
    /// True while the collection create input is staged.
    pub collection_create_staged: bool,
    /// True while the selection move-to sheet is open.
    pub move_sheet_open: bool,
    /// The row whose inline/staged rename input is presented (the pointer
    /// rename entry, B3); `None` at rest.
    pub editing_row: Option<u64>,
    /// The staged rename draft label (empty at rest).
    pub rename_draft: String,
    /// Native views hidden by the open Library (exact restore on close).
    hidden: HashSet<WebViewId>,
}

impl LibraryState {
    /// True when the Library hid this native view (bookkeeping mirror of
    /// the error-surface `hidden` set).
    pub(crate) fn is_hidden(&self, view: WebViewId) -> bool {
        self.hidden.contains(&view)
    }

    /// Records that the open Library hid this native view.
    pub(crate) fn mark_hidden(&mut self, view: WebViewId) {
        self.hidden.insert(view);
    }

    /// Removes `view` from the hidden set and reports whether the Library
    /// had hidden it (the restore decision).
    pub(crate) fn take_hidden(&mut self, view: WebViewId) -> bool {
        self.hidden.remove(&view)
    }

    /// Drops a closed view's hide bookkeeping (the view id is released with
    /// the tab; no native restore is needed).
    pub(crate) fn forget_view(&mut self, view: WebViewId) {
        self.hidden.remove(&view);
    }
}

impl BbrApp {
    /// Open the Library through its integrated entry (Cmd-Shift-L and the
    /// testkit request): close any open command surface, reset the local
    /// state, hide the active view's native slot (never destroyed), focus
    /// the Library, and present.
    pub fn open_saved_pages_library(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.library.open {
            return;
        }
        // §11.4 mutual exclusion (the settled find/download surface shape):
        // opening the Library dismisses any open command portal, cancels
        // the ranking generation, and resets the mode and the captured
        // return point — the Library is the only internal chrome surface
        // while open, and Escape later closes it without re-presenting a
        // trapped palette or overlay.
        self.dismiss_command_portal(cx);
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.command_ranking.close();
        self.library.open = true;
        self.library.search.clear();
        self.library.sort = LibrarySort::default();
        self.library.selected.clear();
        self.library.remove_confirm = false;
        self.library.remove_targets.clear();
        self.library.scroll_offset = 0;
        self.library.collection_action = None;
        self.library.collection_draft.clear();
        self.library.collection_create_staged = false;
        self.library.move_sheet_open = false;
        self.library.editing_row = None;
        self.library.rename_draft.clear();
        self.sync_library_visibility();
        window.focus(&self.library_handle, cx);
        cx.notify();
        self.emit_focus(window);
    }

    /// Close the Library (Escape, the close control, or the testkit
    /// request): restore the exact native view the Library hid, hand focus
    /// back to the page, and present.
    pub fn close_saved_pages_library(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.library.open {
            return;
        }
        self.library.open = false;
        self.library.remove_confirm = false;
        self.library.remove_targets.clear();
        self.library.collection_action = None;
        self.library.collection_draft.clear();
        self.library.collection_create_staged = false;
        self.library.move_sheet_open = false;
        self.library.editing_row = None;
        self.library.rename_draft.clear();
        self.sync_library_visibility();
        self.focus_web(window, cx);
    }

    /// The typed search entry: sets the query, resets the scroll offset
    /// (a fresh window over the new matches), and presents.
    pub fn library_set_search(&mut self, text: &str, cx: &mut Context<Self>) {
        self.library.search = text.to_string();
        self.library.scroll_offset = 0;
        cx.notify();
    }

    /// Cycle the deterministic sort order and reset the scroll offset.
    pub fn library_cycle_sort(&mut self, cx: &mut Context<Self>) {
        self.library.sort = self.library.sort.next();
        self.library.scroll_offset = 0;
        cx.notify();
    }

    /// Toggle one row's multi-select membership.
    pub fn library_toggle_select(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(index) = self.library.selected.iter().position(|s| *s == id) {
            self.library.selected.remove(index);
        } else {
            self.library.selected.push(id);
        }
        cx.notify();
    }

    /// Stage the destructive remove confirmation for the given target ids
    /// (the multi-select removal contract: a clear confirmation before any
    /// record is removed; never touches an open tab).
    pub fn library_begin_remove(&mut self, ids: &[u64], cx: &mut Context<Self>) {
        if ids.is_empty() {
            return;
        }
        self.library.remove_targets = ids.to_vec();
        self.library.remove_confirm = true;
        cx.notify();
    }

    /// Commit the staged destructive remove through the one AppStore
    /// authority (one poke) and clear the confirmation and the removed
    /// selection entries.
    pub fn library_confirm_remove(&mut self, cx: &mut Context<Self>) {
        let targets = std::mem::take(&mut self.library.remove_targets);
        self.library.remove_confirm = false;
        if targets.is_empty() {
            cx.notify();
            return;
        }
        self.store.remove_saved_pages(&targets);
        self.library.selected.retain(|id| !targets.contains(id));
        cx.notify();
    }

    /// Cancel the staged destructive remove.
    pub fn library_cancel_remove(&mut self, cx: &mut Context<Self>) {
        self.library.remove_confirm = false;
        self.library.remove_targets.clear();
        cx.notify();
    }

    /// Rename one record's user label through the one AppStore authority
    /// (the B3 Library rename; invalid labels are a no-op at the store).
    pub fn library_rename(&mut self, id: u64, label: &str, cx: &mut Context<Self>) {
        self.store.rename_saved_page(id, label);
        cx.notify();
    }

    /// Set the tracked production scroll offset (rows scrolled past); the
    /// model derives the bounded window from it.
    pub fn library_set_scroll(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.library.scroll_offset = offset;
        cx.notify();
    }

    /// Open one Library row through the B2 Saved Page command authority:
    /// the record is revalidated in the in-memory provider, then exactly
    /// one fresh regular tab opens in the currently active Space/Profile —
    /// never a silent selection of an existing matching tab. The Library
    /// stays open (a bookmarks-manager surface); the fresh active view is
    /// hidden by the Library while it covers the content area.
    pub fn library_activate(
        &mut self,
        id: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.saved_pages_command()
            .open(self, BookmarkId(id), window, cx)
    }

    /// Re-assert native-view visibility against the Library: while the
    /// Library is open the active view stays hidden (the WKWebView is a
    /// platform subview above the GPUI content layer); on close the exact
    /// view the Library hid is restored. Called after every open/close and
    /// every controller mutation that re-syncs view visibility — the same
    /// authority shape as [`super::error_surface_behavior::BbrApp::sync_error_surface_visibility`].
    pub(crate) fn sync_library_visibility(&mut self) {
        let Some(view) = self.controller.snapshot().active_view else {
            return;
        };
        if self.library.open {
            if !self.library.is_hidden(view) {
                self.engine.borrow_mut().show(view, false);
                self.library.mark_hidden(view);
            }
        } else if self.library.take_hidden(view) {
            self.engine.borrow_mut().show(view, true);
        }
    }

    /// The §11.16 presentation model built from the loaded in-memory
    /// snapshot (B1/B2 authority) plus the app-local search/sort/
    /// selection/scroll/rename state — the single model source the render,
    /// the typed snapshot, and the visible-id mirror share, so the surface
    /// never drifts from what E2E reads.
    pub(crate) fn library_model(&self) -> bbr_ui::components::SavedPagesLibraryModel {
        let (pages, collections) = self
            .store
            .with_saved_pages(|pages, collections| (pages.to_vec(), collections.to_vec()));
        let view = LibraryStateView {
            search: &self.library.search,
            sort: self.library.sort,
            selected: &self.library.selected,
            remove_confirm: self.library.remove_confirm,
            remove_targets: &self.library.remove_targets,
            scroll_offset: self.library.scroll_offset,
            viewport_rows: LIBRARY_VIEWPORT_ROWS,
            editing_row: self.library.editing_row,
            rename_draft: &self.library.rename_draft,
            collection_action: self.library.collection_action,
            collection_draft: &self.library.collection_draft,
            collection_create_staged: self.library.collection_create_staged,
            move_sheet_open: self.library.move_sheet_open,
        };
        build_library_model(&pages, &collections, &view)
    }

    /// The §11.16 render wrapper: builds the presentation model from the
    /// loaded in-memory snapshot plus the local state and wires the
    /// production listener bundle onto the `bbr-ui` surface. Every control
    /// the surface draws routes through this one bundle, so the normal
    /// binary reaches every packet operation (selection clear/remove /
    /// move, collection create/rename/remove, bounded scroll).
    pub fn library_surface(
        &mut self,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<gpui::Div> {
        let model = self.library_model();
        let focused = self.library_handle.is_focused(window);
        let entity = cx.entity();
        let on_close: LibraryAction = Rc::new(move |window, cx| {
            entity.update(cx, |view, cx| view.close_saved_pages_library(window, cx));
        });
        let entity = cx.entity();
        let on_cycle_sort: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_cycle_sort(cx));
        });
        let entity = cx.entity();
        let on_activate: LibraryItemAction = Rc::new(move |id, window, cx| {
            entity.update(cx, |view, cx| {
                view.library_activate(id, window, cx);
            });
        });
        let entity = cx.entity();
        let on_toggle_select: LibraryItemAction = Rc::new(move |id, _window, cx| {
            entity.update(cx, |view, cx| view.library_toggle_select(id, cx));
        });
        let entity = cx.entity();
        let on_rename: LibraryItemAction = Rc::new(move |id, _window, cx| {
            entity.update(cx, |view, cx| view.library_begin_rename(id, cx));
        });
        let entity = cx.entity();
        let on_rename_confirm: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_confirm_rename(cx));
        });
        let entity = cx.entity();
        let on_rename_cancel: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_cancel_rename(cx));
        });
        let entity = cx.entity();
        let on_collection_action: LibraryItemAction = Rc::new(move |id, _window, cx| {
            entity.update(cx, |view, cx| view.library_collection_click(id, cx));
        });
        let entity = cx.entity();
        let on_collection_create: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_begin_collection_create(cx));
        });
        let entity = cx.entity();
        let on_collection_confirm: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| {
                view.library_collection_confirm(cx);
            });
        });
        let entity = cx.entity();
        let on_collection_cancel: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_collection_cancel(cx));
        });
        let entity = cx.entity();
        let on_confirm_remove: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_confirm_remove(cx));
        });
        let entity = cx.entity();
        let on_cancel_remove: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_cancel_remove(cx));
        });
        let entity = cx.entity();
        let on_clear_selection: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_clear_selection(cx));
        });
        let entity = cx.entity();
        let on_remove_selection: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_remove_selected(cx));
        });
        let entity = cx.entity();
        let on_toggle_move_sheet: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.library_toggle_move_sheet(cx));
        });
        let entity = cx.entity();
        let on_move_into: LibraryItemAction = Rc::new(move |id, _window, cx| {
            entity.update(cx, |view, cx| {
                view.library_move_selection(Some(id), cx);
            });
        });
        let entity = cx.entity();
        let on_move_out: LibraryAction = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| {
                view.library_move_selection(None, cx);
            });
        });
        let entity = cx.entity();
        let on_scroll: bbr_ui::components::LibraryScrollAction =
            Rc::new(move |offset, _window, cx| {
                entity.update(cx, |view, cx| view.library_set_scroll(offset, cx));
            });
        let callbacks = LibraryCallbacks {
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
            on_confirm_remove,
            on_cancel_remove,
            on_clear_selection,
            on_remove_selection,
            on_toggle_move_sheet,
            on_move_into,
            on_move_out,
            on_scroll,
        };
        bbr_ui::components::library_surface(theme, &model, focused, &callbacks)
    }

    /// The Library owns the keyboard while open: printable characters edit
    /// the search query and backspace deletes from it; while a row rename
    /// is staged, they edit the staged draft instead and Enter confirms it
    /// (the pointer rename entry, B3); while a collection rename/create
    /// input is staged, they edit the collection draft and Enter commits
    /// the staged collection action (create/rename/remove). The search
    /// entry is the one production `library_set_search` authority. Escape
    /// is handled by the global `escape → FocusWeb` binding (dispatched
    /// before this handler), so it never reaches here.
    pub(super) fn on_library_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ks = &event.keystroke;
        let collection_editing = self.library.collection_input_staged();
        match ks.key.as_str() {
            // Enter confirms the staged rename or the staged collection
            // action through the one authority; the search field owns no
            // submit.
            "enter" => {
                if self.library.editing_row.is_some() {
                    self.library_confirm_rename(cx);
                    cx.stop_propagation();
                } else if collection_editing {
                    self.library_collection_confirm(cx);
                    cx.stop_propagation();
                }
            }
            "backspace" => {
                if self.library.editing_row.is_some() {
                    self.library.rename_draft.pop();
                    cx.notify();
                } else if collection_editing {
                    self.library.collection_draft.pop();
                    cx.notify();
                } else {
                    self.library.search.pop();
                    let next = self.library.search.clone();
                    self.library_set_search(&next, cx);
                }
                cx.stop_propagation();
            }
            _ => {
                if let Some(ch) = ks.key_char.as_deref() {
                    if !ch.is_empty() {
                        if self.library.editing_row.is_some() {
                            self.library.rename_draft.push_str(ch);
                        } else if collection_editing {
                            self.library.collection_draft.push_str(ch);
                        } else {
                            let next = format!("{}{}", self.library.search, ch);
                            self.library_set_search(&next, cx);
                        }
                        cx.notify();
                        cx.stop_propagation();
                    }
                }
            }
        }
    }
}
