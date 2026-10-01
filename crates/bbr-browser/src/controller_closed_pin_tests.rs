//! Closed pinned tabs at the controller level (DESIGN.md §10.12, task packet
//! `closed-pins-and-favorites`): closing a pinned tab destroys its session
//! but keeps the row at its pinned page; selecting the row recreates one
//! session at the pinned URL.

use bbr_domain::TabLifecycle;
use bbr_web::{ProfileId, WebEvent, WebViewId};

use crate::fake::EngineOp;
use crate::harness::{open, wired};
use crate::ControllerEvent;

#[test]
fn a_closed_pinned_tab_owns_no_session_and_reopens_its_pinned_url() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "https://mail.example");
        open(&mut c, "https://news.example");
        c.pin_tab(0).unwrap();
    }
    // The pinned tab navigates on; its pinned page stays the pinned URL.
    h.emit(WebEvent::UrlChanged(
        WebViewId(1),
        "https://mail.example/inbox/42".into(),
    ));
    let mut c = c.borrow_mut();
    c.select_tab(0).unwrap();
    assert_eq!(
        c.close_tab(0).unwrap(),
        ControllerEvent::TabClosed { index: 0 }
    );
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 2, "the pinned row stays");
    assert_eq!(snap.tabs[0].lifecycle, TabLifecycle::Closed);
    assert_eq!(snap.tabs[0].web_view, None);
    assert_eq!(snap.tabs[0].url, "https://mail.example");
    assert_eq!(snap.active_index, Some(1), "the next open tab is selected");
    assert_eq!(snap.engine_sessions, 1);
    assert_eq!(snap.lifecycle_counts.closed, 1);

    h.reset_ops();
    assert_eq!(
        c.select_tab(0).unwrap(),
        ControllerEvent::TabSelected { index: 0 }
    );
    assert!(h.ops().contains(&EngineOp::Create {
        view: WebViewId(3),
        profile: ProfileId(1),
        url: "https://mail.example".into(),
    }));
    let snap = c.snapshot();
    assert_eq!(snap.tabs[0].lifecycle, TabLifecycle::Active);
    assert_eq!(snap.tabs[0].web_view, Some(WebViewId(3)));
    assert_eq!(snap.engine_sessions, 2);
}

#[test]
fn reopen_closed_tab_recreates_the_session_at_the_page_it_closed_on() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "https://mail.example");
        open(&mut c, "https://news.example");
        c.pin_tab(0).unwrap();
    }
    h.emit(WebEvent::UrlChanged(
        WebViewId(1),
        "https://mail.example/inbox/42".into(),
    ));
    let mut c = c.borrow_mut();
    c.close_tab(0).unwrap();
    assert!(c.can_reopen_tab());
    assert_eq!(c.recently_closed()[0].url, "https://mail.example/inbox/42");
    h.reset_ops();
    assert_eq!(
        c.reopen_tab().unwrap(),
        ControllerEvent::TabReopened {
            index: 0,
            url: "https://mail.example/inbox/42".into()
        }
    );
    assert!(h.ops().contains(&EngineOp::Create {
        view: WebViewId(3),
        profile: ProfileId(1),
        url: "https://mail.example/inbox/42".into(),
    }));
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 2, "the same row reopened");
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.tabs[0].lifecycle, TabLifecycle::Active);
    assert!(!c.can_reopen_tab());
}

#[test]
fn the_organization_persists_reconciled_and_restores_with_its_ids() {
    let (mut c, _space, tabs) = crate::organization_tests::populated();
    let mut org = crate::OrganizationController::new();
    org.new_folder(&c, "Work").unwrap();
    let folder = org.folders(&c)[0].id;
    org.move_tab_into_folder_id(&mut c, tabs[0], folder)
        .unwrap();
    org.favorite_toggle_id(&mut c, tabs[1]).unwrap();
    let (folders, favorites) = org.persisted(&c);
    assert_eq!(folders[0].tabs, vec![tabs[0]]);
    assert_eq!(favorites, vec![tabs[1]]);
    // A tab unpinned since the last command leaves the export, not the
    // live store (it reconciles on its next command).
    let index = c
        .snapshot()
        .all_tabs
        .iter()
        .find(|tab| tab.id == tabs[0])
        .unwrap()
        .index;
    c.unpin_tab(index).unwrap();
    let (stale_free, _) = org.persisted(&c);
    assert!(stale_free[0].tabs.is_empty());
    // Restore keeps ids and order; a broken organization is rejected.
    let restored =
        crate::OrganizationController::restored(&c, stale_free.clone(), favorites.clone()).unwrap();
    assert_eq!(restored.persisted(&c), (stale_free.clone(), favorites));
    let mut broken = stale_free;
    broken[0].tabs = vec![tabs[2]];
    assert!(crate::OrganizationController::restored(&c, broken, Vec::new()).is_err());
}
