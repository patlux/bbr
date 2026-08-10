//! The B3 Saved Pages Library collection action surface and selection
//! toolbar operations (packet `saved-pages-library`): the production
//! pointer paths that stage and commit the flat-collection create/rename/
//! remove cycle, move the multi-select into/out of a collection, clear the
//! selection, stage the destructive remove for the selected records, and
//! toggle the move-to sheet. Split from `mod` so no composition file
//! outgrows the ~500-line budget.
//!
//! Every mutation routes through the one existing AppStore authority
//! (`create_saved_collection` / `rename_saved_collection` /
//! `remove_saved_collection` / `move_saved_pages_to_collection`) — no
//! second persistence or identity policy exists.

use gpui::Context;

use bbr_ui::components::LibraryCollectionAction;

use super::{BbrApp, LibraryState};

impl LibraryState {
    /// True while a collection rename or create input is staged (the
    /// keyboard owns the collection draft then).
    pub(crate) fn collection_input_staged(&self) -> bool {
        self.collection_create_staged
            || matches!(
                self.collection_action,
                Some((_, LibraryCollectionAction::Rename))
            )
    }

    /// Stage the collection create input: clear the draft, drop any staged
    /// rename/remove and any row rename, and present the create form.
    pub(crate) fn begin_collection_create(&mut self) {
        self.collection_create_staged = true;
        self.collection_draft.clear();
        self.collection_action = None;
        self.editing_row = None;
    }

    /// Leave the staged collection create/rename/remove without committing.
    pub(crate) fn cancel_collection_surface(&mut self) {
        self.collection_create_staged = false;
        self.collection_action = None;
        self.collection_draft.clear();
    }

    /// Take the staged collection action for the app to dispatch: clears
    /// the staging and returns exactly what must commit (create with the
    /// draft, rename with id + draft, remove with id, or nothing).
    pub(crate) fn take_collection_commit(&mut self) -> CollectionCommit {
        if self.collection_create_staged {
            let title = std::mem::take(&mut self.collection_draft);
            self.collection_create_staged = false;
            self.collection_action = None;
            return CollectionCommit::Create(title);
        }
        let Some((id, action)) = self.collection_action.take() else {
            self.collection_draft.clear();
            return CollectionCommit::None;
        };
        let draft = std::mem::take(&mut self.collection_draft);
        match action {
            LibraryCollectionAction::Rename => CollectionCommit::Rename(id, draft),
            LibraryCollectionAction::Remove => CollectionCommit::Remove(id),
        }
    }
}

/// The staged collection action the app commits through the one AppStore
/// authority (the pure `take_collection_commit` result).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CollectionCommit {
    Create(String),
    Rename(u64, String),
    Remove(u64),
    None,
}

impl BbrApp {
    /// The selection toolbar Clear control: clear the whole multi-select
    /// set (the pointer path the `bbr-saved-pages-selection-clear` control
    /// calls).
    pub fn library_clear_selection(&mut self, cx: &mut Context<Self>) {
        self.library.selected.clear();
        self.library.move_sheet_open = false;
        cx.notify();
    }

    /// The selection toolbar Remove control: stage the destructive remove
    /// confirmation for exactly the current multi-select set (the pointer
    /// path the `bbr-saved-pages-selection-remove` control calls). No
    /// selection stages nothing.
    pub fn library_remove_selected(&mut self, cx: &mut Context<Self>) {
        let selected = self.library.selected.clone();
        if selected.is_empty() {
            return;
        }
        self.library_begin_remove(&selected, cx);
    }

    /// The selection toolbar Move control: toggle the move-to sheet.
    pub fn library_toggle_move_sheet(&mut self, cx: &mut Context<Self>) {
        self.library.move_sheet_open = !self.library.move_sheet_open;
        cx.notify();
    }

    /// Move the current multi-select into (or out of) one flat collection
    /// through the one AppStore authority (`None` clears the membership).
    /// Consumes the selection and closes the move sheet.
    pub fn library_move_selection(&mut self, collection: Option<u64>, cx: &mut Context<Self>) {
        let selected = std::mem::take(&mut self.library.selected);
        self.library.move_sheet_open = false;
        if selected.is_empty() {
            cx.notify();
            return;
        }
        self.store
            .move_saved_pages_to_collection(&selected, collection);
        cx.notify();
    }

    /// Create one flat collection through the one AppStore authority.
    /// Returns the new stable [`CollectionId`] or `None` for an invalid
    /// title (a no-op at the store).
    pub fn library_create_collection(
        &mut self,
        title: &str,
        cx: &mut Context<Self>,
    ) -> Option<u64> {
        let created = self.store.create_saved_collection(title);
        cx.notify();
        created
    }

    /// Rename one flat collection through the one AppStore authority (an
    /// unknown identity or an invalid title is a no-op at the store).
    pub fn library_rename_collection(&mut self, id: u64, title: &str, cx: &mut Context<Self>) {
        self.store.rename_saved_collection(id, title);
        cx.notify();
    }

    /// Remove one flat collection through the one AppStore authority:
    /// member records are moved out first, then the collection row is
    /// removed (the strict foreign key stays satisfied). An unknown
    /// identity is a no-op at the store.
    pub fn library_remove_collection(&mut self, id: u64, cx: &mut Context<Self>) {
        self.store.remove_saved_collection(id);
        cx.notify();
    }

    /// The collection create entry (`+ New collection`): stage the inline
    /// create input.
    pub fn library_begin_collection_create(&mut self, cx: &mut Context<Self>) {
        self.library.begin_collection_create();
        cx.notify();
    }

    /// Commit whatever collection action is staged (create / rename /
    /// remove) through the one AppStore authority. Returns the new stable
    /// [`CollectionId`] when a create committed, `None` otherwise.
    pub fn library_collection_confirm(&mut self, cx: &mut Context<Self>) -> Option<u64> {
        let created = match self.library.take_collection_commit() {
            CollectionCommit::Create(title) => {
                if title.trim().is_empty() {
                    None
                } else {
                    self.library_create_collection(&title, cx)
                }
            }
            CollectionCommit::Rename(id, draft) => {
                self.library_rename_collection(id, &draft, cx);
                None
            }
            CollectionCommit::Remove(id) => {
                self.library_remove_collection(id, cx);
                None
            }
            CollectionCommit::None => None,
        };
        cx.notify();
        created
    }

    /// Leave the staged collection action without committing.
    pub fn library_collection_cancel(&mut self, cx: &mut Context<Self>) {
        if !self.library.collection_input_staged() && self.library.collection_action.is_none() {
            return;
        }
        self.library.cancel_collection_surface();
        cx.notify();
    }

    /// The pointer collection-row action: stage the deterministic
    /// rename → remove → none cycle for the clicked collection. Staging
    /// rename seeds the draft with the collection's current title; any
    /// other staged input (create / row rename) yields to the collection
    /// action surface.
    pub fn library_collection_click(&mut self, id: u64, cx: &mut Context<Self>) {
        self.library.collection_create_staged = false;
        self.library.editing_row = None;
        let next = match self.library.collection_action.take() {
            Some((staged, action)) if staged == id => action.next().map(|next| (id, next)),
            _ => Some((id, LibraryCollectionAction::Rename)),
        };
        self.library.collection_action = next;
        if matches!(
            self.library.collection_action,
            Some((_, LibraryCollectionAction::Rename))
        ) {
            // Seed the staged rename draft with the current title so the
            // inline input starts from the visible label.
            let title = self.store.saved_collection_title(id).unwrap_or_default();
            self.library.collection_draft = title;
        } else if self.library.collection_action.is_none() {
            self.library.collection_draft.clear();
        }
        cx.notify();
    }

    /// The direct collection-draft setter (the testkit seam the staged
    /// inline input's typed requests share; the production keyboard routes
    /// printable input into the draft through the same app state).
    #[cfg(feature = "testkit")]
    pub fn library_set_collection_draft(&mut self, text: &str, cx: &mut Context<Self>) {
        if !self.library.collection_input_staged() {
            return;
        }
        self.library.collection_draft = text.to_string();
        cx.notify();
    }
}
