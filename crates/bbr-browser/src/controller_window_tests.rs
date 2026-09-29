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
