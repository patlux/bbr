//! Space menu authorities (blueprint D05): delete a space with its tabs,
//! folders and sessions, and move a space within its profile group —
//! driven against the deterministic fake engine.

use bbr_domain::{ProfileId, SpaceId, SpaceMove, TabId};

use crate::harness::plain;
use crate::{BrowserController, ControllerEvent, OrganizationController, OrganizationEvent};

fn space(c: &mut BrowserController, title: &str) -> SpaceId {
    match c.create_space_event(ProfileId(1), title).unwrap() {
        ControllerEvent::SpaceCreated { space, .. } => space,
        other => panic!("unexpected event: {other:?}"),
    }
}

fn tabs_of(c: &BrowserController, space: SpaceId) -> Vec<TabId> {
    c.snapshot()
        .all_tabs
        .iter()
        .filter(|tab| tab.space == space)
        .map(|tab| tab.id)
        .collect()
}

#[test]
fn deleting_the_active_space_destroys_its_sessions_and_activates_the_fallback() {
    let (mut c, h) = plain();
    let first = c.snapshot().active_space;
    c.open_tab("https://keep.example").unwrap();
    let doomed = space(&mut c, "Doomed");
    c.activate_space(doomed).unwrap();
    c.open_tab("https://a.example").unwrap();
    c.open_tab("https://b.example").unwrap();
    let doomed_tabs = tabs_of(&c, doomed);
    let mut org = OrganizationController::new();
    let folder = match org.new_folder(&c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    org.move_tab_into_folder_id(&mut c, doomed_tabs[0], folder)
        .unwrap();
    org.favorite_toggle_id(&mut c, doomed_tabs[1]).unwrap();

    let summary = org.space_delete_summary(&c, doomed).unwrap();
    assert_eq!(summary.title, "Doomed");
    assert_eq!(
        (summary.tabs, summary.folders, summary.favorites),
        (2, 1, 1)
    );
    assert!(summary.deletable);

    let event = org.delete_space(&mut c, doomed).unwrap();
    assert_eq!(
        event,
        OrganizationEvent::SpaceDeleted {
            space: doomed,
            fallback: first,
            closed_tabs: 2,
            removed_folders: 1,
            activated: true,
        }
    );
    let snapshot = c.snapshot();
    assert_eq!(snapshot.active_space, first);
    assert!(snapshot.spaces.iter().all(|s| s.id != doomed));
    assert_eq!(
        snapshot.all_tabs.len(),
        1,
        "only the fallback's tab remains"
    );
    assert_eq!(
        h.engine.borrow().live_session_count(),
        1,
        "every session of the deleted space was destroyed"
    );
    let organization = org.organization_snapshot(&c);
    assert!(organization.folders.is_empty(), "the space's folder went");
    assert!(organization.favorites.is_empty(), "its favorite tile went");

    // Reopen Closed Tab brings the deleted space's first tab back into the
    // fallback with a fresh session — pinned, as its grouping left it.
    c.reopen_tab().unwrap();
    assert_eq!(tabs_of(&c, first)[0], doomed_tabs[0]);
    assert!(c.snapshot().tabs[0].pinned, "reopened pinned");
    assert_eq!(h.engine.borrow().live_session_count(), 2);
}

#[test]
fn deleting_an_inactive_space_keeps_the_active_space() {
    let (mut c, _h) = plain();
    let first = c.snapshot().active_space;
    let other = space(&mut c, "Other");
    let mut org = OrganizationController::new();
    match org.delete_space(&mut c, other).unwrap() {
        OrganizationEvent::SpaceDeleted {
            fallback,
            activated,
            ..
        } => {
            assert_eq!(fallback, first);
            assert!(!activated);
        }
        other => panic!("unexpected event: {other:?}"),
    }
    assert_eq!(c.snapshot().active_space, first);
}

#[test]
fn the_last_space_of_a_profile_is_never_deleted() {
    let (mut c, h) = plain();
    let only = c.snapshot().active_space;
    c.open_tab("https://a.example").unwrap();
    let mut org = OrganizationController::new();
    org.new_folder(&c, "Keep").unwrap();
    assert!(!org.space_delete_summary(&c, only).unwrap().deletable);
    assert!(org.delete_space(&mut c, only).is_err());
    assert_eq!(c.snapshot().spaces.len(), 1);
    assert_eq!(h.engine.borrow().live_session_count(), 1);
    assert_eq!(org.organization_snapshot(&c).folders.len(), 1);
    assert!(org.delete_space(&mut c, SpaceId(999)).is_err());
}

#[test]
fn moves_reorder_spaces_and_report_the_new_position() {
    let (mut c, _h) = plain();
    let first = c.snapshot().active_space;
    let second = space(&mut c, "Second");
    assert!(!c.can_move_space(first, SpaceMove::Earlier));
    assert!(c.can_move_space(first, SpaceMove::Later));
    assert_eq!(
        c.move_space(first, SpaceMove::Later).unwrap(),
        ControllerEvent::SpaceMoved {
            space: first,
            to: 1
        }
    );
    let order: Vec<SpaceId> = c.snapshot().spaces.iter().map(|s| s.id).collect();
    assert_eq!(order, vec![second, first]);
    assert_eq!(c.snapshot().active_space, first, "moves never activate");
    assert!(c.move_space(first, SpaceMove::Later).is_err());
}

#[test]
fn unpacking_a_folder_keeps_its_members_as_loose_pinned_tabs() {
    let (mut c, h) = plain();
    c.open_tab("https://a.example").unwrap();
    c.open_tab("https://b.example").unwrap();
    let space = c.snapshot().active_space;
    let tabs = tabs_of(&c, space);
    let mut org = OrganizationController::new();
    let folder = match org.new_folder(&c, "Work").unwrap() {
        OrganizationEvent::FolderCreated { id, .. } => id,
        other => panic!("unexpected event: {other:?}"),
    };
    org.move_tab_into_folder_id(&mut c, tabs[1], folder)
        .unwrap();
    org.move_tab_into_folder_id(&mut c, tabs[0], folder)
        .unwrap();
    let sessions = h.engine.borrow().live_session_count();
    assert_eq!(
        org.unpack_folder(&c, folder).unwrap(),
        OrganizationEvent::FolderUnpacked {
            id: folder,
            members: vec![tabs[1], tabs[0]],
        }
    );
    assert!(org.organization_snapshot(&c).folders.is_empty());
    let snapshot = c.snapshot();
    assert_eq!(snapshot.all_tabs.len(), 2, "no tab closed");
    assert!(
        snapshot.tabs.iter().all(|tab| tab.pinned),
        "members stay pinned"
    );
    assert_eq!(h.engine.borrow().live_session_count(), sessions);
    assert!(org.unpack_folder(&c, folder).is_err(), "unknown folder");
}
