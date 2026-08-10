//! Library state and collection-action unit tests (split from `mod` so no
//! file outgrows the ~500-line budget).

use super::*;
use bbr_ui::components::LibraryCollectionAction;

use super::collections::CollectionCommit;

#[test]
fn state_transitions_are_deterministic() {
    let mut state = LibraryState::default();
    assert!(!state.open);
    state.open = true;
    assert!(state.open);
    // Multi-select toggles in and out.
    state.selected.push(1);
    state.selected.push(2);
    assert_eq!(state.selected, vec![1, 2]);
    // Remove confirmation stages targets and clears on cancel.
    state.remove_targets = vec![1, 2];
    state.remove_confirm = true;
    state.remove_confirm = false;
    state.remove_targets.clear();
    assert!(state.remove_targets.is_empty());
    // The collection pointer cycle is deterministic.
    state.collection_action = None;
    state.collection_action = Some((7, LibraryCollectionAction::Rename));
    assert_eq!(
        state.collection_action,
        Some((7, LibraryCollectionAction::Rename))
    );
    state.collection_action = state
        .collection_action
        .and_then(|(id, action)| action.next().map(|next| (id, next)));
    assert_eq!(
        state.collection_action,
        Some((7, LibraryCollectionAction::Remove))
    );
    state.collection_action = state
        .collection_action
        .and_then(|(id, action)| action.next().map(|next| (id, next)));
    assert_eq!(state.collection_action, None, "the cycle closes");
}

#[test]
fn hidden_view_bookkeeping_restores_exactly() {
    let mut state = LibraryState::default();
    let view = WebViewId(42);
    assert!(!state.is_hidden(view));
    state.mark_hidden(view);
    assert!(state.is_hidden(view));
    // A different view is not restored by mistake.
    assert!(!state.take_hidden(WebViewId(43)));
    assert!(
        state.is_hidden(view),
        "the wrong id never clears the right one"
    );
    assert!(state.take_hidden(view));
    assert!(!state.is_hidden(view));
    // Forget drops bookkeeping for a closed view.
    state.mark_hidden(view);
    state.forget_view(view);
    assert!(!state.is_hidden(view));
}

#[test]
fn collection_action_machine_names_are_stable() {
    assert_eq!(LibraryCollectionAction::Rename.as_str(), "rename");
    assert_eq!(LibraryCollectionAction::Remove.as_str(), "remove");
}

#[test]
fn collection_surface_staging_is_deterministic_and_mutually_exclusive() {
    let mut state = LibraryState {
        collection_action: Some((7, LibraryCollectionAction::Rename)),
        collection_draft: "Work".into(),
        ..LibraryState::default()
    };
    // Staging create clears any prior action/draft and presents the form.
    state.begin_collection_create();
    assert!(state.collection_create_staged);
    assert!(state.collection_input_staged());
    assert_eq!(state.collection_action, None);
    assert!(state.collection_draft.is_empty());
    assert_eq!(state.editing_row, None);
    // A staged rename owns the input; a staged remove never does.
    state.cancel_collection_surface();
    state.collection_action = Some((7, LibraryCollectionAction::Rename));
    state.collection_draft = "Inbox".into();
    assert!(state.collection_input_staged());
    state.collection_action = Some((7, LibraryCollectionAction::Remove));
    assert!(!state.collection_input_staged(), "remove has no input");
    // Cancel clears the whole surface.
    state.cancel_collection_surface();
    assert_eq!(state.collection_action, None);
    assert!(!state.collection_create_staged);
    assert!(state.collection_draft.is_empty());
}

#[test]
fn collection_commit_takes_the_staged_action_exactly_once() {
    let mut state = LibraryState::default();
    // A create with an empty draft commits a Create with the (empty) draft
    // — the store rejects it; the staging clears either way.
    state.begin_collection_create();
    assert_eq!(
        state.take_collection_commit(),
        CollectionCommit::Create(String::new())
    );
    assert!(!state.collection_create_staged);
    // A rename commits the id + draft and clears the staging.
    state.collection_action = Some((7, LibraryCollectionAction::Rename));
    state.collection_draft = "Inbox".into();
    assert_eq!(
        state.take_collection_commit(),
        CollectionCommit::Rename(7, "Inbox".into())
    );
    assert_eq!(state.collection_action, None);
    assert!(state.collection_draft.is_empty());
    // A remove commits the id and clears the staging.
    state.collection_action = Some((7, LibraryCollectionAction::Remove));
    assert_eq!(state.take_collection_commit(), CollectionCommit::Remove(7));
    assert_eq!(state.collection_action, None);
    // Nothing staged commits nothing.
    assert_eq!(state.take_collection_commit(), CollectionCommit::None);
}
