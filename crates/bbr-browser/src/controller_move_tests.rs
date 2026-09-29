//! Blueprint D07: moving a tab by identity from a space that is not active
//! (the Undo of a cross-profile move).

use bbr_web::{ProfileId, WebViewId};

use crate::fake::EngineOp;
use crate::harness::{open, plain};
use crate::ControllerEvent;

#[test]
fn a_tab_moves_back_by_identity_and_is_recreated_in_the_source_store() {
    let (mut c, h) = plain();
    let source = c.snapshot().active_space;
    let profile2 = c.create_profile("Work");
    let work = c.create_space(profile2, "Work").unwrap();
    open(&mut c, "https://simd.example");
    let tab = match c.move_tab(0, work).unwrap() {
        ControllerEvent::TabMoved { tab, recreated, .. } => {
            assert!(recreated);
            tab
        }
        other => panic!("unexpected event: {other:?}"),
    };
    // The source space stays active; the moved tab lives in Work.
    assert_eq!(c.snapshot().active_space, source);
    h.reset_ops();
    let event = c.move_tab_id(tab, source).unwrap();
    assert_eq!(
        event,
        ControllerEvent::TabMoved {
            tab,
            to_space: source,
            recreated: true,
        }
    );
    // Recreated once, in the source profile's store (profile 1).
    assert!(h.ops().iter().any(|op| matches!(
        op,
        EngineOp::Create { profile: ProfileId(1), url, .. } if url == "https://simd.example"
    )));
    assert!(h
        .ops()
        .iter()
        .any(|op| matches!(op, EngineOp::Destroy { view: WebViewId(2) })));
    assert!(c
        .snapshot()
        .tabs
        .iter()
        .any(|t| t.url == "https://simd.example"));
    // An unknown tab is rejected without mutation.
    assert!(c.move_tab_id(bbr_domain::TabId(999), work).is_err());
}
