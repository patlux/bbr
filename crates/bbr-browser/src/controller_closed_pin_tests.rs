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
