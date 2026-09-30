//! Blueprint D19 (several windows): window contexts on the controller —
//! each window has its own active space and selection, a tab shows in one
//! window at a time, and what another window selects stays live and shown
//! in that window's root.

use bbr_domain::TabLifecycle;
use bbr_web::WebViewId;

use crate::fake::{EngineOp, FakeHarness};
use crate::harness::{open, plain};
use crate::BrowserController;

fn shown(h: &FakeHarness, view: WebViewId) -> bool {
    h.ops()
        .iter()
        .rev()
        .find_map(|op| match op {
            EngineOp::Show { view: v, visible } if *v == view => Some(*visible),
            _ => None,
        })
        .unwrap_or(false)
}

fn window_of(h: &FakeHarness, view: WebViewId) -> u64 {
    h.ops()
        .iter()
        .rev()
        .find_map(|op| match op {
            EngineOp::Place { view: v, window } if *v == view => Some(*window),
            _ => None,
        })
        .unwrap_or(0)
}

fn view(c: &BrowserController, index: usize) -> WebViewId {
    c.snapshot().tabs[index].web_view.expect("a live session")
}

fn lifecycle(c: &BrowserController, index: usize) -> TabLifecycle {
    c.snapshot().tabs[index].lifecycle
}

/// Three tabs in the first window, the third selected.
fn three() -> (BrowserController, FakeHarness) {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    open(&mut c, "u3");
    (c, h)
}

#[test]
fn a_new_window_opens_on_the_space_with_nothing_selected() {
    let (mut c, h) = three();
    let first = c.current_window();
    let space = c.snapshot().active_space;
    let second = c.open_window();
    assert_ne!(first, second);
    assert_eq!(c.window_keys(), vec![first, second]);
    assert_eq!(c.window_space(second), Some(space));
    c.enter_window(second).unwrap();
    let snapshot = c.snapshot();
    assert_eq!(snapshot.active_space, space);
    assert_eq!(
        snapshot.active_index, None,
        "the empty state, New Tab ready"
    );
    // The first window's page stays shown in its own root.
    let third = snapshot.tabs[2].web_view.unwrap();
    assert!(shown(&h, third));
    assert_eq!(window_of(&h, third), first.0);
    assert_eq!(lifecycle(&c, 2), TabLifecycle::Active);
}

#[test]
fn each_window_selects_its_own_tab() {
    let (mut c, h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.enter_window(second).unwrap();
    c.select_tab(0).unwrap();
    let one = view(&c, 0);
    let three = view(&c, 2);
    assert!(
        shown(&h, one) && shown(&h, three),
        "both windows show a page"
    );
    assert_eq!(window_of(&h, one), second.0, "in the second window's root");
    assert_eq!(window_of(&h, three), first.0);
    assert_eq!(lifecycle(&c, 2), TabLifecycle::Active);
    c.enter_window(first).unwrap();
    assert_eq!(c.snapshot().active_index, Some(2));
    c.enter_window(second).unwrap();
    assert_eq!(c.snapshot().active_index, Some(0));
    c.check_invariants();
}

#[test]
fn selecting_another_windows_tab_moves_it() {
    let (mut c, h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.enter_window(second).unwrap();
    c.select_tab(2).unwrap();
    let three = view(&c, 2);
    assert_eq!(window_of(&h, three), second.0, "re-parented, not recreated");
    assert!(shown(&h, three));
    c.enter_window(first).unwrap();
    assert_eq!(
        c.snapshot().active_index,
        None,
        "the first window falls back to its empty state"
    );
    assert_eq!(view(&c, 2), three, "the same session");
}

#[test]
fn windows_keep_their_own_spaces() {
    let (mut c, _h) = three();
    let first = c.current_window();
    let home = c.snapshot().active_space;
    let profile = c.snapshot().active_profile;
    let work = c.create_space(profile, "Work").unwrap();
    let second = c.open_window();
    c.enter_window(second).unwrap();
    c.activate_space(work).unwrap();
    c.open_tab("w1").unwrap();
    c.enter_window(first).unwrap();
    assert_eq!(c.snapshot().active_space, home);
    assert_eq!(c.snapshot().active_index, Some(2));
    assert_eq!(c.window_space(second), Some(work));
    c.enter_window(second).unwrap();
    assert_eq!(c.snapshot().active_space, work);
    assert_eq!(c.snapshot().tabs.len(), 1);
}

#[test]
fn a_closed_tab_empties_the_other_windows_selection() {
    let (mut c, _h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.enter_window(second).unwrap();
    c.select_tab(0).unwrap();
    c.enter_window(first).unwrap();
    // Close the second window's tab from the first window.
    c.close_tab(0).unwrap();
    c.enter_window(second).unwrap();
    assert_eq!(c.snapshot().active_index, None);
    assert_eq!(c.snapshot().tabs.len(), 2);
    c.check_invariants();
}

#[test]
fn closing_a_window_keeps_its_tabs_warm() {
    let (mut c, h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.enter_window(second).unwrap();
    c.select_tab(0).unwrap();
    let one = view(&c, 0);
    c.close_window(second).unwrap();
    assert_eq!(c.current_window(), first);
    assert_eq!(c.window_keys(), vec![first]);
    assert_eq!(c.snapshot().active_index, Some(2));
    assert_eq!(c.snapshot().tabs.len(), 3, "no tab closed");
    assert!(!shown(&h, one), "hidden with its window");
    assert_eq!(lifecycle(&c, 0), TabLifecycle::Warm);
    assert!(c.enter_window(second).is_err());
    // The last window's context outlives it for the next window.
    c.close_window(first).unwrap();
    assert_eq!(c.current_window(), first);
}

#[test]
fn viewing_a_window_neither_moves_nor_shows_sessions() {
    let (mut c, h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.enter_window(second).unwrap();
    c.select_tab(0).unwrap();
    let one = view(&c, 0);
    c.enter_window(first).unwrap();
    let before = h.ops().len();
    // A render of the second window reads its context...
    c.view_window(second).unwrap();
    assert_eq!(c.current_window(), second);
    assert_eq!(c.snapshot().active_index, Some(0));
    assert_eq!(c.window_selection(first), c.tab_at(2).ok());
    // ...and gives it back without touching a session.
    c.view_window(first).unwrap();
    assert_eq!(c.snapshot().active_index, Some(2));
    assert_eq!(h.ops().len(), before, "no engine operation");
    assert!(shown(&h, one));
    assert_eq!(window_of(&h, one), second.0);
    c.check_invariants();
}

#[test]
fn an_operation_while_viewing_runs_in_the_viewed_window() {
    let (mut c, h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.view_window(second).unwrap();
    c.select_tab(1).unwrap();
    let two = view(&c, 1);
    assert_eq!(window_of(&h, two), second.0, "placed in the second root");
    c.view_window(first).unwrap();
    assert_eq!(c.snapshot().active_index, Some(2));
    assert_eq!(c.window_selection(second), c.tab_at(1).ok());
    assert!(shown(&h, two));
}

fn open_preview(c: &mut BrowserController, source: WebViewId, url: &str) -> WebViewId {
    use bbr_web::{
        decide, NewWindowDecision, NewWindowFeatures, NewWindowInput, NewWindowOutcome,
        NewWindowRequest, NewWindowTarget, ProfileId,
    };
    let request = NewWindowRequest {
        source,
        source_profile: ProfileId(1),
        url: url.into(),
        user_initiated: true,
        target: NewWindowTarget::NewWindow,
        input: NewWindowInput::Primary,
        features: NewWindowFeatures::default(),
    };
    let session = match decide(&request, NewWindowDecision::Preview) {
        NewWindowOutcome::CreateSession { session } => session,
        other => panic!("expected preview session, got {other:?}"),
    };
    c.open_preview_from_new_window(source, &session)
        .expect("open preview succeeds");
    c.preview().expect("preview open").view
}

#[test]
fn a_preview_belongs_to_the_window_that_opened_it() {
    let (mut c, h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.view_window(second).unwrap();
    c.select_tab(0).unwrap();
    let source = view(&c, 0);
    let preview = open_preview(&mut c, source, "https://preview.example");
    assert_eq!(window_of(&h, preview), second.0, "in the opener's root");
    // The first window has no preview; acting there keeps the other's.
    c.view_window(first).unwrap();
    assert!(c.preview().is_none());
    assert!(c.snapshot().preview.is_none());
    c.select_tab(1).unwrap();
    assert!(!h.ops().contains(&EngineOp::Destroy { view: preview }));
    c.view_window(second).unwrap();
    assert_eq!(c.preview().map(|p| p.view), Some(preview));
    c.check_invariants();
}

#[test]
fn a_waiting_windows_preview_closes_with_its_origin_or_window() {
    let (mut c, h) = three();
    let first = c.current_window();
    let second = c.open_window();
    c.view_window(second).unwrap();
    c.select_tab(0).unwrap();
    let source = view(&c, 0);
    let preview = open_preview(&mut c, source, "https://preview.example");
    // The first window takes the origin tab: the second loses it and its
    // preview closes.
    c.view_window(first).unwrap();
    c.select_tab(0).unwrap();
    assert!(h.ops().contains(&EngineOp::Destroy { view: preview }));
    c.view_window(second).unwrap();
    assert!(c.preview().is_none());
    // A preview goes with its window.
    c.select_tab(1).unwrap();
    let source = view(&c, 1);
    let preview = open_preview(&mut c, source, "https://again.example");
    c.view_window(first).unwrap();
    c.close_window(second).unwrap();
    assert!(h.ops().contains(&EngineOp::Destroy { view: preview }));
    c.check_invariants();
}

#[test]
fn a_views_window_follows_its_session() {
    let (mut c, _h) = three();
    let first = c.current_window();
    let one = view(&c, 0);
    assert_eq!(c.window_of_view(one), Some(first));
    let second = c.open_window();
    c.view_window(second).unwrap();
    c.select_tab(0).unwrap();
    assert_eq!(c.window_of_view(one), Some(second));
    assert_eq!(c.window_of_view(WebViewId(9999)), None);
}

#[test]
fn a_repeated_engine_report_leaves_the_drawn_state_alone() {
    use bbr_web::WebEvent;
    let (mut c, _h) = three();
    let one = view(&c, 0);
    let _ = c.apply_event(WebEvent::TitleChanged(one, "One".into()));
    let before = c.view_fingerprint(one);
    let _ = c.apply_event(WebEvent::TitleChanged(one, "One".into()));
    let _ = c.apply_event(WebEvent::CanGoBackChanged(one, false));
    assert_eq!(
        c.view_fingerprint(one),
        before,
        "a repeat changes nothing drawn"
    );
    let _ = c.apply_event(WebEvent::TitleChanged(one, "Two".into()));
    assert_ne!(c.view_fingerprint(one), before, "a new title is drawn");
}

#[test]
fn a_saved_window_comes_back_with_its_space_and_selections() {
    let (mut c, h) = three();
    let first = c.current_window();
    let home = c.snapshot().active_space;
    let profile = c.snapshot().active_profile;
    let work = c.create_space(profile, "Work").unwrap();
    let one = c.tab_at(0).unwrap();
    let before = h.ops().len();
    let second = c.restore_window(work, [(home, Some(one)), (work, None)]);
    assert_eq!(h.ops().len(), before, "no session moves before the refresh");
    c.refresh_sessions();
    assert_eq!(c.window_space(second), Some(work));
    let layouts = c.window_layouts();
    assert_eq!(layouts.len(), 2);
    assert_eq!(layouts[0].0, first);
    assert_eq!(layouts[0].1, home);
    assert_eq!(
        layouts[1],
        (second, work, vec![(home, Some(one)), (work, None)])
    );
    // The saved selection of home is kept for when the window shows home.
    c.view_window(second).unwrap();
    c.activate_space(home).unwrap();
    assert_eq!(c.snapshot().active_index, Some(0));
    assert_eq!(window_of(&h, view(&c, 0)), second.0);
    c.check_invariants();
}

#[test]
fn a_saved_window_drops_what_is_gone() {
    let (mut c, _h) = three();
    let home = c.snapshot().active_space;
    let key = c.restore_window(
        bbr_domain::SpaceId(9_999),
        [
            (home, Some(bbr_domain::TabId(9_998))),
            (bbr_domain::SpaceId(9_997), None),
        ],
    );
    c.refresh_sessions();
    assert_eq!(
        c.window_space(key),
        Some(home),
        "an unknown space falls back"
    );
    assert_eq!(c.window_selection(key), None, "an unknown tab is dropped");
    c.check_invariants();
}

#[test]
fn closing_a_tab_moves_the_closed_count_and_reopening_does_not() {
    let (mut c, _h) = three();
    assert_eq!(c.closed_tab_count(), 0);
    c.close_tab(0).unwrap();
    assert_eq!(c.closed_tab_count(), 1);
    c.reopen_tab().unwrap();
    assert_eq!(c.closed_tab_count(), 1, "a reopen closes nothing");
}

#[test]
fn a_tab_dropped_from_another_window_lands_at_its_slot_and_shows_here() {
    let (mut c, h) = three();
    let first = c.current_window();
    let home = c.snapshot().active_space;
    let profile = c.snapshot().active_profile;
    let work = c.create_space(profile, "Work").unwrap();
    let second = c.open_window();
    c.view_window(second).unwrap();
    c.activate_space(work).unwrap();
    c.open_tab("w1").unwrap();
    c.open_tab("w2").unwrap();
    // The first window's page (index 2 of home) dropped between w1 and w2.
    c.view_window(first).unwrap();
    let dragged = c.tab_at(2).unwrap();
    let view_before = view(&c, 2);
    c.view_window(second).unwrap();
    c.drop_tab_here(dragged, Some(1)).unwrap();
    let snapshot = c.snapshot();
    assert_eq!(snapshot.tabs.len(), 3);
    assert_eq!(snapshot.tabs[1].url, "u3");
    assert_eq!(snapshot.active_index, Some(1), "shown here");
    assert_eq!(view(&c, 1), view_before, "same profile: the same session");
    assert_eq!(window_of(&h, view_before), second.0);
    assert_eq!(c.space_of_tab(dragged), Some(work));
    // The first window lost it.
    assert_eq!(c.window_space(first), Some(home));
    assert_ne!(c.window_selection(first), Some(dragged));
    c.check_invariants();
}

#[test]
fn a_drop_within_the_same_space_reorders_by_splice() {
    let (mut c, _h) = three();
    let second = c.open_window();
    c.view_window(second).unwrap();
    let first_tab = c.tab_at(0).unwrap();
    // Slot 2 (between u2 and u3) for u1: splice → index 1.
    c.drop_tab_here(first_tab, Some(2)).unwrap();
    let urls: Vec<_> = c.snapshot().tabs.iter().map(|t| t.url.clone()).collect();
    assert_eq!(urls, ["u2", "u1", "u3"]);
    assert_eq!(c.snapshot().active_index, Some(1));
    c.check_invariants();
}

#[test]
fn a_drop_into_another_profile_recreates_the_session_there() {
    let (mut c, h) = three();
    let first = c.current_window();
    let event = c.create_profile_space("Private", "Home").unwrap();
    let crate::ControllerEvent::ProfileCreated { space, .. } = event else {
        panic!("profile created: {event:?}");
    };
    let second = c.open_window();
    c.view_window(second).unwrap();
    c.activate_space(space).unwrap();
    c.view_window(first).unwrap();
    let dragged = c.tab_at(0).unwrap();
    let old = view(&c, 0);
    c.view_window(second).unwrap();
    let outcome = c.drop_tab_here(dragged, Some(0)).unwrap();
    assert!(
        matches!(
            outcome,
            crate::ControllerEvent::TabMoved {
                recreated: true,
                ..
            }
        ),
        "{outcome:?}"
    );
    assert!(h.ops().contains(&EngineOp::Destroy { view: old }));
    let new = view(&c, 0);
    assert_ne!(new, old, "a fresh session in the destination store");
    assert_eq!(window_of(&h, new), second.0);
    c.check_invariants();
}
