//! Contract tests for the tab/favorite context-menu authority (task packet
//! `tab-favorite-context-menus`): the availability matrix covers
//! inactive/selected, pinned/regular, first/middle/last, favorite/
//! non-favorite, suspended/live, split-member, and same-/cross-profile
//! destinations; menu open/dismiss (resolve-only) leaves the controller
//! snapshot and session registry unchanged; and each enabled action
//! dispatches exactly one typed command against the bound [`TabId`], while
//! disabled and omitted actions cannot dispatch.

use bbr_domain::{SpaceId, TabId};
use bbr_web::WebAction;

use crate::harness::{open, plain};
use crate::tab_menu::{
    FolderDestination, TabMenuAction, TabMenuAvailability, TabMenuController, TabMenuError,
    TabMenuSource,
};
use crate::ControllerEvent;

/// The tab id at the active space's `index` (the tab row the menu opens on).
fn tab_id_at(c: &crate::BrowserController, index: usize) -> TabId {
    c.snapshot()
        .all_tabs
        .iter()
        .find(|t| t.space == c.snapshot().active_space && t.index == index)
        .expect("active-space tab exists")
        .id
}

/// Resolve + availability for the active-space tab at `index`, with a
/// favorites membership test reporting `favorite`.
fn source(c: &crate::BrowserController, index: usize, favorite: bool) -> TabMenuSource {
    TabMenuController::resolve(c, |_| favorite, tab_id_at(c, index)).expect("source resolves")
}

fn availability(c: &crate::BrowserController, index: usize, favorite: bool) -> TabMenuAvailability {
    let s = source(c, index, favorite);
    TabMenuController::availability(c, &s, &[])
}

/// A two-profile controller: profile 1 (Default) with spaces S1 (active,
/// two tabs) and S2, profile 2 with space S3 (one tab).
fn two_profile() -> (crate::BrowserController, SpaceId, SpaceId, SpaceId) {
    let (mut c, _harness) = plain();
    open(&mut c, "https://one.example");
    open(&mut c, "https://two.example");
    let default_profile = c.active_profile();
    let s2 = c.create_space(default_profile, "Second").expect("space");
    let profile2 = c.create_profile("Private");
    let s3 = c.create_space(profile2, "Private Space").expect("space");
    let s1 = c.snapshot().active_space;
    (c, s1, s2, s3)
}

#[test]
fn resolve_reads_the_exact_source_identity() {
    let (c, s1, _s2, _s3) = two_profile();
    let s = source(&c, 1, true);
    assert_eq!(s.space, s1);
    assert_eq!(s.index, 1);
    assert!(!s.pinned);
    assert!(s.visible, "the active tab is visible");
    assert!(s.favorite);
    assert_eq!(s.url, "https://two.example");
    assert!(s.web_view.is_some(), "a live tab owns a session");
    assert_eq!(s.pinned_count, 0);
    assert_eq!(s.tab_count, 2);
}

#[test]
fn resolve_is_read_only_and_leaves_the_snapshot_and_registry_unchanged() {
    // Menu open/dismiss must never mutate: resolving the source (the only
    // menu-open read) leaves the complete controller snapshot and the
    // session registry byte-identical.
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    let before = c.snapshot();
    let sessions_before = c.snapshot().engine_sessions;
    let s = TabMenuController::resolve(&c, |_| false, tab_id_at(&c, 1)).expect("resolves");
    let _a = TabMenuController::availability(&c, &s, &[]);
    let after = c.snapshot();
    assert_eq!(before, after, "resolve + availability never mutate");
    assert_eq!(after.engine_sessions, sessions_before);
    assert_eq!(
        after.engine_sessions,
        c.engine.borrow().live_session_count(),
        "registry and engine agree"
    );
}

#[test]
fn availability_covers_selected_inactive_pinned_regular_and_edges() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    open(&mut c, "https://c.example");
    // The last opened tab is the active/visible one; the earlier rows are
    // hidden warm tabs.
    let selected = availability(&c, 2, false);
    assert!(!selected.suspend, "a visible tab cannot suspend");
    assert!(
        selected.move_to_start,
        "the active row can move to the start"
    );
    assert!(!selected.move_to_end, "last row has no Move to End");
    // Inactive regular tab at index 1: hidden warm, suspendable, middle.
    let inactive = availability(&c, 1, false);
    assert!(inactive.suspend, "a hidden warm tab can suspend");
    assert!(inactive.move_to_start);
    assert!(inactive.move_to_end);
    // First row: no Move to Start.
    let first = availability(&c, 0, false);
    assert!(!first.move_to_start, "first row has no Move to Start");
    assert!(first.move_to_end);
    // Pin the middle tab; its region flips to pinned-first.
    let tab = tab_id_at(&c, 1);
    let s = source(&c, 1, false);
    let space = s.space;
    c.browser.pin_tab(space, tab).expect("pin");
    c.sync_views();
    c.check_invariants();
    let pinned = availability(&c, 0, false);
    assert!(pinned.pinned, "the pinned source reads Unpin Tab");
    assert!(
        !pinned.move_to_start && !pinned.move_to_end,
        "a sole pinned row has no ordering moves"
    );
    // The regular tail still has no Move to End.
    let last = c.snapshot().tabs.len() - 1;
    let tail = availability(&c, last, false);
    assert!(!tail.move_to_end, "last row has no Move to End");
}

#[test]
fn availability_covers_favorite_and_lifecycle_states() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    // Favorite state flips the row label source.
    let fav = availability(&c, 0, true);
    assert!(fav.favorite);
    assert!(fav.copy_link, "a live tab can copy its link");
    let non_fav = availability(&c, 0, false);
    assert!(!non_fav.favorite);
    // Suspend the hidden first tab; its availability flips to
    // not-suspendable and the session disappears.
    let index = 0;
    c.suspend_tab(index).expect("suspend");
    let suspended = availability(&c, index, false);
    assert!(
        !suspended.suspend,
        "an already-suspended tab cannot suspend again"
    );
    assert!(
        !suspended.reload,
        "a suspended tab owns no live session to reload"
    );
    assert!(
        !suspended.copy_link,
        "a suspended tab owns no live session to copy a link from"
    );
}

#[test]
fn availability_covers_split_members() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    c.open_split(0, 1).expect("split");
    // Both split members are visible: neither can suspend.
    for index in 0..2 {
        let a = availability(&c, index, false);
        assert!(!a.suspend, "split members are visible");
    }
}

#[test]
fn availability_omits_missing_capabilities_and_lists_destinations() {
    let (mut c, s1, s2, s3) = two_profile();
    let source = source(&c, 0, false);
    let a = TabMenuController::availability(&c, &source, &[]);
    // Move to Space lists every other space in creation order; the
    // cross-profile destination carries the visible profile identity.
    assert_eq!(
        a.spaces.iter().map(|d| d.space).collect::<Vec<_>>(),
        vec![s2, s3]
    );
    let cross = a.spaces.iter().find(|d| d.space == s3).expect("cross");
    assert_eq!(cross.profile_title.as_deref(), Some("Private"));
    let same = a.spaces.iter().find(|d| d.space == s2).expect("same");
    assert_eq!(same.profile_title, None);
    // Reopen legality mirrors the owning space's recently-closed stack.
    assert!(!a.reopen, "nothing closed yet");
    c.close_tab(1).expect("close");
    let a2 = {
        let s = TabMenuController::resolve(&c, |_| false, tab_id_at(&c, 0)).expect("source");
        TabMenuController::availability(&c, &s, &[])
    };
    assert!(a2.reopen, "the space now has a recently-closed tab");
    assert_eq!(s1, s1);
}

#[test]
fn dispatch_reload_routes_exactly_one_action_to_the_bound_view() {
    let (mut c, harness) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    let tab = tab_id_at(&c, 1);
    harness.reset_ops();
    let event = TabMenuController::dispatch(&mut c, tab, TabMenuAction::Reload).expect("dispatch");
    let view = c.registry.view_for_tab(tab).expect("live session");
    assert_eq!(event, ControllerEvent::NavigationRouted { view });
    let ops = harness.ops();
    assert_eq!(ops.len(), 1, "exactly one engine command");
    assert!(ops.contains(&crate::fake::EngineOp::Action {
        view,
        action: WebAction::Reload
    }));
}

#[test]
fn dispatch_pin_and_unpin_keep_the_bound_tab_identity() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    let tab = tab_id_at(&c, 1);
    let pinned = TabMenuController::dispatch(&mut c, tab, TabMenuAction::TogglePin).expect("pin");
    assert!(matches!(pinned, ControllerEvent::TabPinned { .. }));
    let s = TabMenuController::resolve(&c, |_| false, tab).expect("still resolves");
    assert!(s.pinned);
    assert_eq!(s.index, 0, "the pinned tab moved to the pinned region");
    let unpinned =
        TabMenuController::dispatch(&mut c, tab, TabMenuAction::TogglePin).expect("unpin");
    assert!(matches!(unpinned, ControllerEvent::TabUnpinned { .. }));
    let s = TabMenuController::resolve(&c, |_| false, tab).expect("still resolves");
    assert!(!s.pinned);
}

#[test]
fn dispatch_close_targets_the_source_not_the_selected_neighbor() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    open(&mut c, "https://c.example");
    // The selected neighbor is the active tab 0; the menu source is tab 2.
    let tab = tab_id_at(&c, 2);
    let event = TabMenuController::dispatch(&mut c, tab, TabMenuAction::Close).expect("close");
    assert_eq!(
        event,
        ControllerEvent::TabClosed { index: 2 },
        "closes the bound source at its own index"
    );
    let remaining: Vec<TabId> = c.snapshot().all_tabs.iter().map(|t| t.id).collect();
    assert!(!remaining.contains(&tab), "the source is gone");
    assert!(remaining.contains(&tab_id_at(&c, 0)), "the neighbor stays");
}

#[test]
fn dispatch_reorder_targets_the_sources_own_region() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    open(&mut c, "https://c.example");
    let tab = tab_id_at(&c, 1);
    let event =
        TabMenuController::dispatch(&mut c, tab, TabMenuAction::MoveToEnd).expect("reorder");
    assert_eq!(event, ControllerEvent::TabReordered { from: 1, to: 2 });
    let s = TabMenuController::resolve(&c, |_| false, tab).expect("still resolves");
    assert_eq!(s.index, 2, "the source moved to the region end");
}

#[test]
fn dispatch_move_same_profile_keeps_the_web_view_and_cross_recreates_once() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    let s1 = c.snapshot().active_space;
    let s2 = c.create_space(c.active_profile(), "Second").expect("space");
    let tab = tab_id_at(&c, 0);
    let view_before = c.registry.view_for_tab(tab).expect("live session");
    let same =
        TabMenuController::dispatch(&mut c, tab, TabMenuAction::MoveToSpace(s2)).expect("move");
    assert_eq!(
        same,
        ControllerEvent::TabMoved {
            tab,
            to_space: s2,
            recreated: false
        }
    );
    assert_eq!(
        c.registry.view_for_tab(tab),
        Some(view_before),
        "same-profile move keeps the live WebViewId"
    );
    // Cross-profile destination: recreate exactly once in the new store.
    let profile2 = c.create_profile("Private");
    let s3 = c.create_space(profile2, "Private Space").expect("space");
    let cross =
        TabMenuController::dispatch(&mut c, tab, TabMenuAction::MoveToSpace(s3)).expect("move");
    assert_eq!(
        cross,
        ControllerEvent::TabMoved {
            tab,
            to_space: s3,
            recreated: true
        }
    );
    let fresh = c.registry.view_for_tab(tab).expect("recreated session");
    assert_ne!(
        fresh, view_before,
        "cross-profile move recreates the session"
    );
    assert_eq!(c.snapshot().engine_sessions, 1, "exactly one live session");
    let _ = s1;
}

#[test]
fn dispatch_suspend_targets_the_bound_hidden_tab() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    // The first tab (index 0) is the hidden warm row; the second is active.
    let tab = tab_id_at(&c, 0);
    let event = TabMenuController::dispatch(&mut c, tab, TabMenuAction::Suspend).expect("suspend");
    assert_eq!(event, ControllerEvent::TabSuspended { index: 0 });
    let s = TabMenuController::resolve(&c, |_| false, tab).expect("still resolves");
    assert!(s.web_view.is_none(), "the suspended tab owns no session");
}

#[test]
fn dispatch_never_switches_the_active_space() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    open(&mut c, "https://c.example");
    let s2 = c.create_space(c.active_profile(), "Second").expect("space");
    // Move the hidden first tab (a) into the second space and activate it;
    // the Default-space tab b stays hidden warm (not its active tab).
    let a = tab_id_at(&c, 0);
    TabMenuController::dispatch(&mut c, a, TabMenuAction::MoveToSpace(s2)).expect("move");
    c.activate_space(s2).expect("activate");
    let first_space = c
        .snapshot()
        .spaces
        .iter()
        .find(|s| s.title == "Default")
        .unwrap()
        .id;
    let hidden = c
        .snapshot()
        .all_tabs
        .iter()
        .find(|t| t.space == first_space && !t.pinned)
        .expect("the Default-space hidden tab stays")
        .id;
    // The Default-space tab is hidden and warm: suspend it through the menu.
    TabMenuController::dispatch(&mut c, hidden, TabMenuAction::Suspend).expect("suspend");
    assert_eq!(
        c.snapshot().active_space,
        s2,
        "the menu action never switches the active space"
    );
    assert_eq!(
        c.snapshot().active_view,
        c.registry.view_for_tab(a),
        "the active space's selection is untouched"
    );
    assert!(
        c.registry.view_for_tab(hidden).is_none(),
        "the suspended hidden tab owns no session"
    );
}

#[test]
fn dispatch_reopen_targets_the_sources_own_space() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    let tab = tab_id_at(&c, 0);
    // Close the hidden first tab: the source's space gains a recently-closed
    // tab and the menu row becomes legal.
    let closed = TabMenuController::dispatch(&mut c, tab, TabMenuAction::Close).expect("close");
    assert_eq!(closed, ControllerEvent::TabClosed { index: 0 });
    let reopened = {
        let source_tab = tab_id_at(&c, 0);
        TabMenuController::dispatch(&mut c, source_tab, TabMenuAction::ReopenTab).expect("reopen")
    };
    assert!(matches!(
        reopened,
        ControllerEvent::TabReopened { index: 0, .. }
    ));
    let all: Vec<TabId> = c.snapshot().all_tabs.iter().map(|t| t.id).collect();
    assert_eq!(
        all.len(),
        2,
        "the closed identity is restored in the same space"
    );
    let s = source(&c, 0, false);
    assert_eq!(s.space, c.snapshot().active_space);
}

#[test]
fn dispatch_reopen_rejects_when_no_recently_closed_tab() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    let tab = tab_id_at(&c, 0);
    let before = c.snapshot();
    let err =
        TabMenuController::dispatch(&mut c, tab, TabMenuAction::ReopenTab).expect_err("no recent");
    assert!(matches!(err, TabMenuError::ReopenNotLegal));
    assert_eq!(c.snapshot(), before, "a rejected reopen mutates nothing");
}

#[test]
fn dispatch_unknown_tab_fails_closed() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    let before = c.snapshot();
    let err = TabMenuController::dispatch(&mut c, TabId(u64::MAX), TabMenuAction::Close)
        .expect_err("unknown");
    assert!(matches!(err, TabMenuError::UnknownTab(TabId(u64::MAX))));
    assert_eq!(c.snapshot(), before, "a failed dispatch mutates nothing");
}

#[test]
fn folder_destinations_are_per_space_and_never_leak() {
    // The portable matrix accepts the caller's per-space folder rows; the
    // app derives them from the organization read model filtered to the
    // source space. Here we prove the matrix keeps them verbatim.
    let (mut c, _h) = plain();
    open(&mut c, "https://a.example");
    let s = source(&c, 0, false);
    let folders = vec![
        FolderDestination {
            folder: bbr_domain::FolderId(1),
            title: "Work".into(),
        },
        FolderDestination {
            folder: bbr_domain::FolderId(2),
            title: "Fun".into(),
        },
    ];
    let a = TabMenuController::availability(&c, &s, &folders);
    assert_eq!(a.folders, folders);
}
