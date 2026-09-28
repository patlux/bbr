//! Controller-level organization contract tests (lane L5): the
//! `NewSpace`, `NewFolder`, `FavoriteToggle`, and `MoveTabIntoFolder`
//! commands plus the read model, driven against the deterministic fake
//! engine through the exact `Rc<RefCell<dyn WebEngine>>` seam. No macOS
//! frameworks, filesystem, or wall clock.

use bbr_domain::{FolderError, FolderId};
use bbr_domain::{ProfileId, SpaceId, TabId};

use crate::harness::plain;
use crate::{
    ControllerError, ControllerEvent, OrganizationController, OrganizationError, OrganizationEvent,
};

/// A controller with the default space and three open tabs.
pub(crate) fn populated() -> (crate::BrowserController, SpaceId, Vec<TabId>) {
    let (mut c, _h) = plain();
    let space = c.snapshot().active_space;
    let mut tabs = Vec::new();
    for u in [
        "https://a.example",
        "https://b.example",
        "https://c.example",
    ] {
        c.open_tab(u).unwrap();
        tabs.push(
            c.snapshot()
                .all_tabs
                .iter()
                .find(|t| t.url == u)
                .unwrap()
                .id,
        );
    }
    (c, space, tabs)
}

// ---- NewSpace -------------------------------------------------------------

#[test]
fn new_space_creates_in_the_active_profile_without_switching() {
    let (mut c, _h) = plain();
    let active = c.snapshot().active_space;
    let mut org = OrganizationController::new();
    let event = org.new_space(&mut c, "Work").unwrap();
    let (profile, space) = match event {
        OrganizationEvent::SpaceCreated { profile, space } => (profile, space),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(profile, ProfileId(1), "created in the active profile");
    let snap = c.snapshot();
    assert!(snap.spaces.iter().any(|s| s.id == space));
    assert_eq!(snap.spaces.len(), 2);
    assert_eq!(
        snap.active_space, active,
        "NewSpace stages in the command bar; it never switches the space"
    );
    assert!(
        c.snapshot()
            .spaces
            .iter()
            .find(|s| s.id == space)
            .unwrap()
            .active_tab
            .is_none(),
        "a new space starts empty"
    );
}

#[test]
fn new_space_delegate_contract_is_pinned() {
    let (mut c, _h) = plain();
    // `new_space` reads the profile of the active space, which always exists,
    // so the delegate error path is unreachable through the public API today.
    // Pin the two contracts the command relies on instead: a facade error
    // wraps into `OrganizationError::Controller`, and `create_space_event`
    // emits exactly the `SpaceCreated` variant the `unreachable!` branch
    // depends on.
    let err = c.create_space_event(ProfileId(999), "Ghost").unwrap_err();
    assert!(matches!(err, ControllerError::Domain(_)));
    assert!(matches!(
        OrganizationError::Controller(err.clone()),
        OrganizationError::Controller(_)
    ));
    assert!(err.to_string().contains("unknown profile"));
    match c.create_space_event(ProfileId(1), "Pinned contract") {
        Ok(ControllerEvent::SpaceCreated { .. }) => {}
        other => panic!("create_space_event must only emit SpaceCreated, got {other:?}"),
    }
}

// ---- NewFolder ------------------------------------------------------------

#[test]
fn new_folder_creates_in_the_active_space_with_distinct_ids() {
    let (c, space, _) = populated();
    let mut org = OrganizationController::new();
    let event = org.new_folder(&c, "Work").unwrap();
    let (id, space_again, title) = match event {
        OrganizationEvent::FolderCreated { id, space, title } => (id, space, title),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(title, "Work");
    assert_eq!(space_again, space);
    let snap = org.organization_snapshot(&c);
    assert_eq!(snap.folders.len(), 1);
    assert_eq!(snap.folders[0].id, id);
    assert!(snap.folders[0].members.is_empty(), "folders start empty");
    let second = org.new_folder(&c, "Docs").unwrap();
    assert_ne!(
        id,
        match second {
            OrganizationEvent::FolderCreated { id, .. } => id,
            other => panic!("unexpected event: {other:?}"),
        }
    );
    assert_eq!(org.organization_snapshot(&c).folders.len(), 2);
}

#[test]
fn rename_folder_commits_a_trimmed_title_and_rejects_empty_ones_whole() {
    let (c, _, _) = populated();
    let mut org = OrganizationController::new();
    let id = match org.new_folder(&c, "New Folder").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(
        org.rename_folder(id, " Reading ").unwrap(),
        OrganizationEvent::FolderRenamed {
            id,
            title: "Reading".into()
        }
    );
    assert_eq!(org.organization_snapshot(&c).folders[0].title, "Reading");
    assert!(org.rename_folder(id, "").is_err());
    assert_eq!(org.organization_snapshot(&c).folders[0].title, "Reading");
    assert!(org.rename_folder(FolderId(77), "Ghost").is_err());
}

#[test]
fn new_folder_uses_the_active_space_across_switches() {
    let (mut c, default_space, _) = populated();
    let other = c.create_space_event(ProfileId(1), "Second").unwrap();
    let other_space = match other {
        ControllerEvent::SpaceCreated { space, .. } => space,
        other => panic!("unexpected event: {other:?}"),
    };
    let mut org = OrganizationController::new();
    let f1 = org.new_folder(&c, "Default Folder").unwrap();
    let (id1, space1, _) = match f1 {
        OrganizationEvent::FolderCreated { id, space, .. } => (id, space, ()),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(space1, default_space);
    c.activate_space(other_space).unwrap();
    let f2 = org.new_folder(&c, "Second Folder").unwrap();
    let (id2, space2, _) = match f2 {
        OrganizationEvent::FolderCreated { id, space, .. } => (id, space, ()),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(space2, other_space);
    assert_ne!(id1, id2);
    let snap = org.organization_snapshot(&c);
    assert_eq!(snap.folders.len(), 2, "folders accumulate across spaces");
}

// ---- FavoriteToggle -------------------------------------------------------

#[test]
fn favorite_toggle_adds_then_removes_the_concrete_tab() {
    let (c, space, tabs) = populated();
    let mut org = OrganizationController::new();
    let on = org.favorite_toggle(&c, 0).unwrap();
    assert_eq!(
        on,
        OrganizationEvent::FavoriteToggled {
            added: true,
            tab: tabs[0],
            space,
            profile: ProfileId(1),
        }
    );
    let snap = org.organization_snapshot(&c);
    assert_eq!(snap.favorites[0].tab, tabs[0]);
    assert_eq!(snap.favorites[0].space, space);
    assert_eq!(snap.favorites[0].profile, ProfileId(1));
    assert_eq!(snap.favorites[0].url, "https://a.example");
    let off = org.favorite_toggle(&c, 0).unwrap();
    assert_eq!(
        off,
        OrganizationEvent::FavoriteToggled {
            added: false,
            tab: tabs[0],
            space,
            profile: ProfileId(1),
        }
    );
    assert!(org.organization_snapshot(&c).favorites.is_empty());
}

#[test]
fn favorite_toggle_dedups_by_tab_and_keeps_insertion_order() {
    let (c, _space, tabs) = populated();
    let mut org = OrganizationController::new();
    org.favorite_toggle(&c, 0).unwrap();
    org.favorite_toggle(&c, 1).unwrap();
    org.favorite_toggle(&c, 2).unwrap();
    org.favorite_toggle(&c, 0).unwrap();
    let snap = org.organization_snapshot(&c);
    assert_eq!(
        snap.favorites.iter().map(|f| f.tab).collect::<Vec<_>>(),
        vec![tabs[1], tabs[2]]
    );
    org.favorite_toggle(&c, 0).unwrap();
    let snap = org.organization_snapshot(&c);
    assert_eq!(
        snap.favorites.iter().map(|f| f.tab).collect::<Vec<_>>(),
        vec![tabs[1], tabs[2], tabs[0]]
    );
}

#[test]
fn favorite_toggle_rejects_out_of_bounds_indices() {
    let (c, _space, _) = populated();
    let mut org = OrganizationController::new();
    assert_eq!(
        org.favorite_toggle(&c, 3),
        Err(OrganizationError::IndexOutOfBounds { index: 3, len: 3 })
    );
    assert!(org.organization_snapshot(&c).favorites.is_empty());
}

#[test]
fn closing_the_favorited_tab_removes_its_tile() {
    let (mut c, _space, tabs) = populated();
    let mut org = OrganizationController::new();
    org.favorite_toggle(&c, 0).unwrap();
    assert_eq!(org.organization_snapshot(&c).favorites[0].tab, tabs[0]);
    c.close_tab(0).unwrap();
    assert!(org.organization_snapshot(&c).favorites.is_empty());
    org.check_invariants(&c).unwrap();
}

#[test]
fn favorite_follows_the_same_tab_when_it_moves_across_profiles() {
    let (mut c, original_space, tabs) = populated();
    let mut org = OrganizationController::new();
    org.favorite_toggle(&c, 0).unwrap();
    let profile = c.create_profile("Personal");
    let destination = c.create_space(profile, "Personal").unwrap();
    c.move_tab(0, destination).unwrap();
    let snap = org.organization_snapshot(&c);
    assert_eq!(snap.favorites.len(), 1);
    assert_eq!(snap.favorites[0].tab, tabs[0]);
    assert_eq!(snap.favorites[0].space, destination);
    assert_eq!(snap.favorites[0].profile, profile);
    assert_ne!(snap.favorites[0].space, original_space);
}

#[test]
fn activating_a_cross_profile_favorite_switches_inside_the_same_window() {
    let (mut c, work_space, tabs) = populated();
    let work_view = c.snapshot().tabs[0].web_view.expect("favorite has session");
    let mut org = OrganizationController::new();
    org.favorite_toggle(&c, 0).unwrap();

    let personal = c.create_profile("Personal");
    let personal_space = c.create_space(personal, "Private").unwrap();
    c.activate_space(personal_space).unwrap();
    c.open_tab("https://private.example").unwrap();
    assert_eq!(c.snapshot().active_profile, personal);

    assert_eq!(
        org.activate_favorite(&mut c, 0).unwrap(),
        OrganizationEvent::FavoriteActivated {
            tab: tabs[0],
            space: work_space,
            profile: ProfileId(1),
            index: 0,
        }
    );
    let snap = c.snapshot();
    assert_eq!(snap.active_space, work_space);
    assert_eq!(snap.active_profile, ProfileId(1));
    assert_eq!(snap.tabs[0].web_view, Some(work_view));
    assert_eq!(snap.all_tabs.iter().filter(|t| t.id == tabs[0]).count(), 1);
}

#[test]
fn favorite_toggle_never_moves_or_copies_the_tab() {
    let (c, _space, _) = populated();
    let mut org = OrganizationController::new();
    let before = c.snapshot();
    org.favorite_toggle(&c, 0).unwrap();
    let after = c.snapshot();
    assert_eq!(before, after, "favorite toggle only changes presentation");
}

// ---- MoveTabIntoFolder ----------------------------------------------------

#[test]
fn move_tab_into_folder_moves_between_folders_and_noops_inside_one() {
    let (mut c, _space, tabs) = populated();
    let mut org = OrganizationController::new();
    let work = match org.new_folder(&c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    let docs = match org.new_folder(&c, "Docs").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    org.move_tab_into_folder(&mut c, 0, work).unwrap();
    org.move_tab_into_folder(&mut c, 0, docs).unwrap();
    let snap = org.organization_snapshot(&c);
    assert!(snap
        .folders
        .iter()
        .find(|f| f.id == work)
        .unwrap()
        .members
        .is_empty());
    assert_eq!(
        snap.folders.iter().find(|f| f.id == docs).unwrap().members,
        vec![tabs[0]]
    );
    // Moving into the folder the tab already belongs to is a no-op.
    let before = org.organization_snapshot(&c);
    org.move_tab_into_folder(&mut c, 0, docs).unwrap();
    assert_eq!(org.organization_snapshot(&c), before);
}

#[test]
fn move_tab_into_folder_rejects_unknown_folder_whole() {
    let (mut c, _space, _) = populated();
    let mut org = OrganizationController::new();
    let before = org.organization_snapshot(&c);
    let browser_before = c.snapshot();
    assert!(matches!(
        org.move_tab_into_folder(&mut c, 0, FolderId(99)),
        Err(OrganizationError::Folder(FolderError::UnknownFolder(
            FolderId(99)
        )))
    ));
    assert_eq!(org.organization_snapshot(&c), before, "no partial mutation");
    assert_eq!(c.snapshot(), browser_before, "the tab was not pinned");
}

#[test]
fn move_tab_into_folder_rejects_out_of_bounds_indices() {
    let (mut c, _space, _) = populated();
    let mut org = OrganizationController::new();
    let folder = match org.new_folder(&c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(
        org.move_tab_into_folder(&mut c, 3, folder),
        Err(OrganizationError::IndexOutOfBounds { index: 3, len: 3 })
    );
    assert!(org.organization_snapshot(&c).folders[0].members.is_empty());
}

#[test]
fn move_tab_into_folder_rejects_cross_space_tabs() {
    let (mut c, _default_space, tabs) = populated();
    let second_space = match c.create_space_event(ProfileId(1), "Second").unwrap() {
        ControllerEvent::SpaceCreated { space, .. } => space,
        other => panic!("unexpected event: {other:?}"),
    };
    let mut org = OrganizationController::new();
    let folder = match org.new_folder(&c, "Default Folder").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    // Move the first tab into the second space, then activate it: the tab
    // now belongs to a different space than the folder.
    c.move_tab(0, second_space).unwrap();
    c.activate_space(second_space).unwrap();
    assert_eq!(
        org.move_tab_into_folder(&mut c, 0, folder),
        Err(OrganizationError::Folder(
            FolderError::TabNotInFolderSpace {
                tab: tabs[0],
                folder
            }
        )),
        "a tab of another space cannot join an active-space folder"
    );
    // Rejected whole: the folder stays empty and the tab stays regular.
    assert!(org.organization_snapshot(&c).folders[0].members.is_empty());
    assert!(
        !c.snapshot().tabs[0].pinned,
        "a rejected grouping never pins"
    );
}

// ---- Reconciliation -------------------------------------------------------

#[test]
fn commands_reconcile_memberships_after_close_and_move() {
    let (mut c, _space, tabs) = populated();
    let mut org = OrganizationController::new();
    let folder = match org.new_folder(&c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    org.move_tab_into_folder(&mut c, 0, folder).unwrap();
    org.move_tab_into_folder(&mut c, 1, folder).unwrap();
    // Close the first tab (a controller mutation the store cannot observe).
    c.close_tab(0).unwrap();
    // Another command must not observe or keep the stale membership.
    org.favorite_toggle(&c, 0).unwrap();
    let snap = org.organization_snapshot(&c);
    assert_eq!(
        snap.folders
            .iter()
            .find(|f| f.id == folder)
            .unwrap()
            .members,
        vec![tabs[1]],
        "closed tab membership was reconciled away"
    );
}

#[test]
fn reconcile_drops_memberships_after_a_tab_moves_spaces() {
    let (mut c, _space, tabs) = populated();
    let second_space = match c.create_space_event(ProfileId(1), "Second").unwrap() {
        ControllerEvent::SpaceCreated { space, .. } => space,
        other => panic!("unexpected event: {other:?}"),
    };
    let mut org = OrganizationController::new();
    let folder = match org.new_folder(&c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    org.move_tab_into_folder(&mut c, 0, folder).unwrap();
    org.move_tab_into_folder(&mut c, 1, folder).unwrap();
    // Move the first member into the second space (a controller mutation the
    // store cannot observe): reconciliation must drop that membership.
    c.move_tab(0, second_space).unwrap();
    assert_eq!(org.reconcile(&c), 1);
    let snap = org.organization_snapshot(&c);
    assert_eq!(
        snap.folders
            .iter()
            .find(|f| f.id == folder)
            .unwrap()
            .members,
        vec![tabs[1]],
        "moved-tab membership was reconciled away"
    );
    org.check_invariants(&c).unwrap();
    // The moved tab is still alive in its new space; only the membership
    // changed. Re-activating the source space keeps the folder intact.
    c.activate_space(second_space).unwrap();
    assert_eq!(c.snapshot().tabs.len(), 1);
    org.check_invariants(&c).unwrap();
}

#[test]
fn reconcile_returns_the_number_of_removed_memberships() {
    let (mut c, _space, _tabs) = populated();
    let mut org = OrganizationController::new();
    let folder = match org.new_folder(&c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    org.move_tab_into_folder(&mut c, 0, folder).unwrap();
    org.move_tab_into_folder(&mut c, 1, folder).unwrap();
    c.close_tab(0).unwrap();
    assert_eq!(org.reconcile(&c), 1);
    assert_eq!(org.reconcile(&c), 0, "second reconcile is a no-op");
}

// ---- Invariants and read model -------------------------------------------

#[test]
fn invariants_hold_across_command_sequences() {
    let (mut c, _space, _) = populated();
    let mut org = OrganizationController::new();
    org.new_folder(&c, "Work").unwrap();
    let folder = org.organization_snapshot(&c).folders.first().unwrap().id;
    org.move_tab_into_folder(&mut c, 0, folder).unwrap();
    org.move_tab_into_folder(&mut c, 1, folder).unwrap();
    org.favorite_toggle(&c, 2).unwrap();
    org.check_invariants(&c).unwrap();
    // Closing a tab makes membership stale; a command reconciles, so the
    // checker passes again afterwards.
    c.close_tab(0).unwrap();
    org.favorite_toggle(&c, 0).unwrap();
    org.check_invariants(&c).unwrap();
}

#[test]
fn snapshot_exposes_stable_index_identities() {
    let (mut c, _space, _) = populated();
    let mut org = OrganizationController::new();
    org.new_folder(&c, "Work").unwrap();
    org.new_folder(&c, "Docs").unwrap();
    org.favorite_toggle(&c, 1).unwrap();
    org.favorite_toggle(&c, 0).unwrap();
    let snap = org.organization_snapshot(&c);
    assert_eq!(
        snap.folders
            .iter()
            .map(|f| (f.index, f.title.as_str()))
            .collect::<Vec<_>>(),
        vec![(0, "Work"), (1, "Docs")]
    );
    assert_eq!(
        snap.favorites
            .iter()
            .map(|f| (f.index, f.url.as_str()))
            .collect::<Vec<_>>(),
        vec![(0, "https://b.example"), (1, "https://a.example")]
    );
    // Moving tabs around never changes folder/favorite identities.
    c.pin_tab(2).unwrap();
    let snap = org.organization_snapshot(&c);
    assert_eq!(snap.folders[0].index, 0);
    assert_eq!(snap.favorites[0].index, 0);
}

#[test]
fn organization_controller_is_a_plain_value() {
    let (c, _space, _) = populated();
    let mut org = OrganizationController::new();
    org.favorite_toggle(&c, 0).unwrap();
    assert_eq!(org.clone(), org);
}
