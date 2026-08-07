//! U2 behavior contract tests (DESIGN.md §11.4/§11.10/§11.11): the
//! window-global favorites invariants, favorite activation through the
//! corrected L5 command, the spaces-region toggle, and the folder
//! presentation state — driven against the L5 `OrganizationController` over
//! the deterministic `NullEngine`, exactly the seam the E2E gateway and the
//! production composition root use. No macOS frameworks, filesystem, or
//! wall clock.

use std::cell::RefCell;
use std::rc::Rc;

use bbr_browser::{BrowserController, ControllerEvent, OrganizationController, OrganizationEvent};
use bbr_web::ProfileId;

use crate::engine::{EngineRef, NullEngine};

/// A fresh controller over the null engine with the default space, plus the
/// companion organization service.
fn harness() -> (BrowserController, OrganizationController) {
    let engine: EngineRef = Rc::new(RefCell::new(NullEngine::default()));
    let mut next: u64 = 0;
    let controller = BrowserController::new(
        engine,
        Box::new(move || {
            next += 1;
            next
        }),
    );
    (controller, OrganizationController::new())
}

/// The id of the profile backing `space` (domain ids share one monotonic
/// counter, so they are never small stable constants).
fn profile_of(c: &BrowserController, space: bbr_browser::SpaceId) -> ProfileId {
    c.snapshot()
        .spaces
        .iter()
        .find(|s| s.id == space)
        .expect("space exists")
        .profile
}

/// Open `count` tabs in the active space, returning their concrete `TabId`s
/// in space order.
fn open_tabs(c: &mut BrowserController, count: usize) -> Vec<bbr_browser::GlobalTabSnapshot> {
    for i in 0..count {
        c.open_tab(format!("https://tab{i}.example")).unwrap();
    }
    c.snapshot()
        .all_tabs
        .iter()
        .filter(|tab| tab.space == c.snapshot().active_space)
        .cloned()
        .collect()
}

/// Create and activate another space of the given profile, returning its id.
fn activate_new_space(
    c: &mut BrowserController,
    profile: ProfileId,
    title: &str,
) -> bbr_browser::SpaceId {
    let space = match c.create_space_event(profile, title).unwrap() {
        ControllerEvent::SpaceCreated { space, .. } => space,
        other => panic!("expected SpaceCreated, got {other:?}"),
    };
    c.activate_space(space).unwrap();
    space
}

// ---- Favorites: window-global invariants ---------------------------------

#[test]
fn favorites_stay_identical_across_same_profile_space_switches() {
    let (mut c, mut org) = harness();
    let default_space = c.snapshot().active_space;
    let default_profile = c.snapshot().active_profile;
    let tabs = open_tabs(&mut c, 3);
    org.favorite_toggle(&c, 1).unwrap(); // favorite the second tab
    let before = org.organization_snapshot(&c);

    // Switch to another space of the same profile: the grid keeps the exact
    // same tile order and owner.
    activate_new_space(&mut c, default_profile, "Work");
    let after = org.organization_snapshot(&c);
    assert_eq!(before.favorites, after.favorites, "tiles never reorder");
    let tile = &after.favorites[0];
    assert_eq!(tile.tab, tabs[1].id, "the exact concrete TabId");
    assert_eq!(tile.space, default_space, "the canonical owning space");
    assert_eq!(
        tile.profile, default_profile,
        "derived from the owning space"
    );
}

#[test]
fn favorites_stay_identical_across_cross_profile_space_switches() {
    let (mut c, mut org) = harness();
    let default_space = c.snapshot().active_space;
    open_tabs(&mut c, 2);
    org.favorite_toggle(&c, 0).unwrap();
    let before = org.organization_snapshot(&c);

    // A second profile: activating one of its spaces must never filter or
    // replace the grid (a profile is a container boundary, not a window).
    let personal = match c.create_profile_space("Personal", "Personal").unwrap() {
        ControllerEvent::ProfileCreated { space, .. } => space,
        other => panic!("expected ProfileCreated, got {other:?}"),
    };
    let default_profile = c.snapshot().active_profile;
    c.activate_space(personal).unwrap();
    c.open_tab("https://personal.example").unwrap();
    let after = org.organization_snapshot(&c);
    assert_eq!(before.favorites, after.favorites);
    let tile = &after.favorites[0];
    assert_eq!(tile.tab, before.favorites[0].tab);
    assert_eq!(tile.space, default_space);
    assert_eq!(tile.profile, default_profile);
}

#[test]
fn favorite_activation_switches_space_and_selects_the_exact_tab_without_copy_or_move() {
    let (mut c, mut org) = harness();
    let default_space = c.snapshot().active_space;
    open_tabs(&mut c, 3);
    org.favorite_toggle(&c, 2).unwrap(); // favorite tab 2 (the last tab)
    let favorite = org.organization_snapshot(&c).favorites[0].clone();

    // Leave the owning space and make it non-trivial to return to.
    activate_new_space(&mut c, ProfileId(1), "Work");
    c.open_tab("https://work.example").unwrap();
    let before = c.snapshot();
    assert_ne!(
        before.active_space, default_space,
        "we left the owning space"
    );
    let total_before = before.all_tabs.len();

    let default_profile = c.snapshot().active_profile;
    let event = org.activate_favorite(&mut c, 0).unwrap();
    assert_eq!(
        event,
        OrganizationEvent::FavoriteActivated {
            tab: favorite.tab,
            space: default_space,
            profile: default_profile,
            index: 2,
        }
    );
    let after = c.snapshot();
    assert_eq!(after.active_space, default_space, "same-window switch");
    assert_eq!(after.active_profile, default_profile);
    assert_eq!(
        after.all_tabs.len(),
        total_before,
        "activation never copies, moves, or recreates a tab"
    );
    // The selected tab is exactly the favorite's concrete tab, unchanged.
    assert_eq!(after.active_index, Some(2));
    let selected = after
        .all_tabs
        .iter()
        .find(|tab| tab.id == favorite.tab)
        .expect("favorited tab still exists");
    assert_eq!(selected.id, favorite.tab);
    assert_eq!(selected.space, favorite.space);
    assert_eq!(selected.profile, favorite.profile);
}

#[test]
fn closing_the_concrete_tab_removes_its_favorite_tile() {
    let (mut c, mut org) = harness();
    open_tabs(&mut c, 2);
    org.favorite_toggle(&c, 0).unwrap();
    assert_eq!(org.organization_snapshot(&c).favorites.len(), 1);

    c.close_tab(0).unwrap();
    let after = org.organization_snapshot(&c);
    assert!(
        after.favorites.is_empty(),
        "closing the concrete tab drops the tile (reconciled on read)"
    );
}

#[test]
fn moving_the_tab_makes_the_favorite_follow_its_stable_tab_id() {
    let (mut c, mut org) = harness();
    let default_space = c.snapshot().active_space;
    open_tabs(&mut c, 2);
    org.favorite_toggle(&c, 0).unwrap();
    let tab = org.organization_snapshot(&c).favorites[0].tab;

    // Move the favorited tab into another space of a second profile: the
    // favorite follows the same `TabId` and resolves the new owner.
    let personal = match c.create_profile_space("Personal", "Personal").unwrap() {
        ControllerEvent::ProfileCreated { space, .. } => space,
        other => panic!("expected ProfileCreated, got {other:?}"),
    };
    let personal_profile = profile_of(&c, personal);
    c.move_tab(0, personal).unwrap();
    let after = org.organization_snapshot(&c);
    assert_eq!(after.favorites.len(), 1, "the tile follows the moved tab");
    let tile = &after.favorites[0];
    assert_eq!(tile.tab, tab, "the same stable TabId");
    assert_eq!(tile.space, personal, "resolves the new space");
    assert_eq!(tile.profile, personal_profile, "resolves the new profile");
    assert_ne!(tile.space, default_space);
}

#[test]
fn favorite_toggle_adds_then_removes_without_touching_the_tab() {
    let (mut c, mut org) = harness();
    let tabs = open_tabs(&mut c, 2);
    let add = org.favorite_toggle(&c, 0).unwrap();
    assert!(matches!(
        add,
        OrganizationEvent::FavoriteToggled { added: true, .. }
    ));
    assert_eq!(org.organization_snapshot(&c).favorites.len(), 1);

    // Toggling again removes; the tab's identity and membership are intact.
    let remove = org.favorite_toggle(&c, 0).unwrap();
    assert!(matches!(
        remove,
        OrganizationEvent::FavoriteToggled { added: false, .. }
    ));
    assert!(org.organization_snapshot(&c).favorites.is_empty());
    let after = c.snapshot();
    assert_eq!(
        after.all_tabs.len(),
        tabs.len(),
        "no tab was created/closed"
    );
    assert_eq!(after.active_index, Some(1), "selection untouched");
}

// ---- Plus menu: New Space / New Folder dispatch --------------------------

#[test]
fn plus_menu_new_space_creates_a_real_space_in_the_active_profile() {
    let (mut c, mut org) = harness();
    let before = c.snapshot().active_space;
    let event = org.new_space(&mut c, "Menu Space").unwrap();
    let (profile, space) = match event {
        OrganizationEvent::SpaceCreated { profile, space } => (profile, space),
        other => panic!("expected SpaceCreated, got {other:?}"),
    };
    let active_profile = c.snapshot().active_profile;
    assert_eq!(profile, active_profile, "created in the active profile");
    assert!(c.snapshot().spaces.iter().any(|s| s.id == space));
    assert_eq!(
        c.snapshot().active_space,
        before,
        "the menu action creates without switching (staging owns switching)"
    );
}

#[test]
fn plus_menu_new_folder_creates_a_session_local_folder_in_the_active_space() {
    let (c, mut org) = harness();
    let space = c.snapshot().active_space;
    let event = org.new_folder(&c, "Menu Folder").unwrap();
    let created = match event {
        OrganizationEvent::FolderCreated { space, title, .. } => (space, title),
        other => panic!("expected FolderCreated, got {other:?}"),
    };
    assert_eq!(created.0, space, "session-local in the active space");
    assert_eq!(created.1, "Menu Folder");
    let snapshot = org.organization_snapshot(&c);
    assert_eq!(snapshot.folders.len(), 1);
    assert_eq!(snapshot.folders[0].members.len(), 0, "starts empty");
}

// ---- Folder presentation -------------------------------------------------

#[test]
fn folder_presentation_grows_deterministically_and_toggles_per_folder() {
    use crate::components::folders::FolderPresentation;
    let mut p = FolderPresentation::default();
    assert!(!p.is_expanded(0));
    p.toggle(0);
    assert!(p.is_expanded(0));
    p.toggle(0);
    assert!(!p.is_expanded(0), "snap back on the second toggle");
    p.toggle(3);
    assert!(p.is_expanded(3));
    assert!(!p.is_expanded(2), "toggling 3 leaves earlier folders alone");
}
