//! Regression coverage for preview teardown after controller mutations.

use bbr_web::{
    decide, NewWindowDecision, NewWindowFeatures, NewWindowInput, NewWindowOutcome,
    NewWindowRequest, NewWindowTarget, ProfileId, WebViewId,
};

use crate::fake::{EngineOp, FakeHarness};
use crate::harness::{open, plain};

fn open_preview(c: &mut crate::BrowserController, source: WebViewId, url: &str) {
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
}

fn assert_preview_destroyed(c: &crate::BrowserController, h: &FakeHarness, view: WebViewId) {
    assert!(c.snapshot().preview.is_none());
    assert!(h.ops().contains(&EngineOp::Destroy { view }));
}

#[test]
fn opening_or_reopening_a_tab_ends_the_preview() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, "https://preview.example");
    let preview_view = c.snapshot().preview.unwrap().view;
    h.reset_ops();

    open(&mut c, "https://new.example");
    assert_preview_destroyed(&c, &h, preview_view);

    c.close_tab(1).unwrap();
    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, "https://preview.example/again");
    let preview_view = c.snapshot().preview.unwrap().view;
    h.reset_ops();

    c.reopen_tab().unwrap();
    assert_preview_destroyed(&c, &h, preview_view);
}

#[test]
fn closing_or_moving_the_origin_tab_ends_the_preview() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, "https://preview.example");
    let preview_view = c.snapshot().preview.unwrap().view;
    h.reset_ops();

    c.close_tab(0).unwrap();
    assert_preview_destroyed(&c, &h, preview_view);

    c.reopen_tab().unwrap();
    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, "https://preview.example/again");
    let preview_view = c.snapshot().preview.unwrap().view;
    let other = c.create_space(ProfileId(1), "Other").unwrap();
    h.reset_ops();

    c.move_tab(0, other).unwrap();
    assert_preview_destroyed(&c, &h, preview_view);
}

#[test]
fn split_composition_changes_end_the_preview() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, "https://preview.example");
    let preview_view = c.snapshot().preview.unwrap().view;
    h.reset_ops();

    c.open_split(0, 1).unwrap();
    assert_preview_destroyed(&c, &h, preview_view);

    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, "https://preview.example/again");
    let preview_view = c.snapshot().preview.unwrap().view;
    h.reset_ops();

    c.close_split().unwrap();
    assert_preview_destroyed(&c, &h, preview_view);
}
