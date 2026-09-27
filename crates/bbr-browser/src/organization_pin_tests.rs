//! Folder-membership invariant tests at the controller level (task packet
//! `folder-pinned-invariant`): grouping pins a regular tab in the same
//! transition, ungrouping keeps the pin, any controller unpin ends the
//! membership, the session budget spares members as pinned tabs, and the
//! id-keyed grouping pins in the owning space without switching.

use bbr_domain::{FolderError, FolderId, ProfileId};

use crate::organization_tests::populated;
use crate::{ControllerEvent, OrganizationController, OrganizationError, OrganizationEvent};

/// Create one folder in the active space and return its id.
fn folder(org: &mut OrganizationController, c: &crate::BrowserController) -> FolderId {
    match org.new_folder(c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn move_tab_into_folder_pins_atomically_and_keeps_identity() {
    let (mut c, space, tabs) = populated();
    let views: Vec<_> = c.snapshot().tabs.iter().map(|t| t.web_view).collect();
    let mut org = OrganizationController::new();
    let folder = folder(&mut org, &c);
    assert_eq!(
        org.move_tab_into_folder(&mut c, 0, folder).unwrap(),
        OrganizationEvent::TabMovedIntoFolder {
            tab: tabs[0],
            folder,
            pinned_index: Some(0),
        }
    );
    // Tab `c` at index 2 is pinned by the same transition: it moves to the
    // end of the pinned region (index 1).
    assert_eq!(
        org.move_tab_into_folder(&mut c, 2, folder).unwrap(),
        OrganizationEvent::TabMovedIntoFolder {
            tab: tabs[2],
            folder,
            pinned_index: Some(1),
        }
    );
    let snap = org.organization_snapshot(&c);
    assert_eq!(snap.folders[0].members, vec![tabs[0], tabs[2]]);
    let c_snap = c.snapshot();
    assert_eq!(
        c_snap
            .tabs
            .iter()
            .map(|t| (t.pinned, t.web_view))
            .collect::<Vec<_>>(),
        vec![(true, views[0]), (true, views[2]), (false, views[1])],
        "members are pinned; every tab keeps its exact session"
    );
    assert!(c_snap.all_tabs.iter().all(|t| t.space == space));
    assert_eq!(c_snap.engine_sessions, 3, "no session was recreated");
    // Re-grouping an already pinned member reports no new pin.
    assert_eq!(
        org.move_tab_into_folder(&mut c, 0, folder).unwrap(),
        OrganizationEvent::TabMovedIntoFolder {
            tab: tabs[0],
            folder,
            pinned_index: None,
        }
    );
    org.check_invariants(&c).unwrap();
}

#[test]
fn ungrouping_leaves_a_loose_pinned_tab() {
    let (mut c, _space, tabs) = populated();
    let mut org = OrganizationController::new();
    let folder = folder(&mut org, &c);
    org.move_tab_into_folder(&mut c, 1, folder).unwrap();
    // `b` is now pinned at index 0.
    assert_eq!(
        org.remove_tab_from_folder(&c, 0, folder).unwrap(),
        OrganizationEvent::TabRemovedFromFolder {
            tab: tabs[1],
            folder
        }
    );
    assert!(org.organization_snapshot(&c).folders[0].members.is_empty());
    let snap = c.snapshot();
    assert_eq!(snap.tabs[0].url, "https://b.example");
    assert!(snap.tabs[0].pinned, "ungroup keeps the pin");
    assert!(matches!(
        org.remove_tab_from_folder(&c, 0, FolderId(99)),
        Err(OrganizationError::Folder(FolderError::UnknownFolder(_)))
    ));
    org.check_invariants(&c).unwrap();
}

#[test]
fn unpinning_a_member_through_the_controller_ends_its_membership() {
    let (mut c, _space, tabs) = populated();
    let mut org = OrganizationController::new();
    let folder = folder(&mut org, &c);
    org.move_tab_into_folder(&mut c, 0, folder).unwrap();
    org.move_tab_into_folder(&mut c, 1, folder).unwrap();
    // Any controller unpin path (keyboard, pin control, tab menu, drag).
    c.unpin_tab(0).unwrap();
    let snap = org.organization_snapshot(&c);
    assert_eq!(
        snap.folders[0].members,
        vec![tabs[1]],
        "the unpinned tab left its folder"
    );
    let unpinned = c.snapshot();
    let a = unpinned.all_tabs.iter().find(|t| t.id == tabs[0]).unwrap();
    assert!(!a.pinned, "the unpin stands; grouping never re-pins it");
    org.check_invariants(&c).unwrap();
}

#[test]
fn folder_members_survive_the_session_budget_as_pinned_tabs() {
    let (mut c, _space, tabs) = populated();
    let mut org = OrganizationController::new();
    let folder = folder(&mut org, &c);
    // `a` is the least recently used hidden tab: without the pin the budget
    // would suspend it first.
    org.move_tab_into_folder(&mut c, 0, folder).unwrap();
    c.enforce_session_budget(2).unwrap();
    let snap = c.snapshot();
    let lifecycle = |id| snap.all_tabs.iter().find(|t| t.id == id).unwrap().lifecycle;
    assert!(
        matches!(
            lifecycle(tabs[0]),
            crate::TabLifecycle::Active | crate::TabLifecycle::Warm
        ),
        "the folder member stays live through its canonical pinned state"
    );
    assert_eq!(lifecycle(tabs[1]), crate::TabLifecycle::Suspended);
}

#[test]
fn move_tab_into_folder_id_pins_in_the_owning_space_without_switching() {
    let (mut c, default_space, tabs) = populated();
    let mut org = OrganizationController::new();
    let folder = folder(&mut org, &c);
    let second = match c.create_space_event(ProfileId(1), "Second").unwrap() {
        ControllerEvent::SpaceCreated { space, .. } => space,
        other => panic!("unexpected event: {other:?}"),
    };
    c.activate_space(second).unwrap();
    assert_eq!(
        org.move_tab_into_folder_id(&mut c, tabs[2], folder)
            .unwrap(),
        OrganizationEvent::TabMovedIntoFolder {
            tab: tabs[2],
            folder,
            pinned_index: Some(0),
        }
    );
    let snap = c.snapshot();
    assert_eq!(snap.active_space, second, "the active space is unchanged");
    let moved = snap.all_tabs.iter().find(|t| t.id == tabs[2]).unwrap();
    assert!(moved.pinned && moved.space == default_space);
    org.check_invariants(&c).unwrap();
}
