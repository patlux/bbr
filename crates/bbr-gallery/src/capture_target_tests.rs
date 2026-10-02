//! Observation and guest-plan contracts. No GPUI context, draw, window or native input.
use super::*;
use gpui::{point, px, size};

fn bounds(x: f32, y: f32, width: f32, height: f32) -> Bounds<Pixels> {
    Bounds::new(point(px(x), px(y)), size(px(width), px(height)))
}
fn control() -> Bounds<Pixels> {
    bounds(48.0, 144.0, 180.0, 20.0)
}
fn clip() -> Bounds<Pixels> {
    bounds(0.0, 0.0, 1200.0, 1000.0)
}

#[test]
fn exactly_one_current_child_is_required_and_no_observation_is_inherited() {
    let mut target = Target::default();
    assert!(target.token().is_none());
    let first = target.begin_draw();
    assert!(target.resolve(first, true).is_err());
    assert_eq!(target.resolve(first, false).unwrap(), None);
    target.observe(Some(first), vec![control()], clip());
    assert_eq!(target.resolve(first, true).unwrap(), Some(control()));
    assert!(target.resolve(first, false).is_err());
    let next = target.begin_draw();
    assert_ne!(first, next);
    assert!(target.resolve(first, true).is_err());
    assert!(target.resolve(next, true).is_err());
    target.observe(Some(next), vec![control()], clip());
    assert_eq!(target.resolve(next, true).unwrap(), Some(control()));
}

#[test]
fn stale_unarmed_duplicate_and_failed_observations_poison_the_draw() {
    for fault in [
        "stale",
        "unarmed",
        "duplicate",
        "multiple",
        "missing",
        "invalid",
    ] {
        let mut target = Target::default();
        let old = target.begin_draw();
        target.invalidate();
        assert!(target.token().is_none());
        let current = target.begin_draw();
        match fault {
            "stale" => target.observe(Some(old), vec![control()], clip()),
            "unarmed" => target.observe(None, vec![control()], clip()),
            "duplicate" => target.observe(Some(current), vec![control()], clip()),
            "multiple" => target.observe(Some(current), vec![control(), control()], clip()),
            "missing" => target.observe(Some(current), vec![], clip()),
            "invalid" => target.observe(Some(current), vec![bounds(0.0, 0.0, 0.0, 0.0)], clip()),
            _ => unreachable!(),
        }
        // A good later callback must not erase the earlier bad observation.
        target.observe(Some(current), vec![control()], clip());
        assert!(target.resolve(current, true).is_err(), "{fault}");
        assert!(target.resolve(current, false).is_err(), "{fault}");
        let fresh = target.begin_draw();
        target.observe(Some(fresh), vec![control()], clip());
        target.resolve(fresh, true).unwrap();
    }
}

#[test]
fn nonfinite_nonpositive_clipped_and_off_frame_targets_fail_without_clamping() {
    for (child, mask) in [
        (bounds(f32::NAN, 144.0, 180.0, 20.0), clip()),
        (bounds(48.0, 144.0, f32::INFINITY, 20.0), clip()),
        (bounds(48.0, 144.0, -1.0, 20.0), clip()),
        (bounds(48.0, 144.0, 180.0, 0.0), clip()),
        (bounds(47.99, 144.0, 180.0, 20.0), clip()),
        (bounds(48.0, 95.99, 180.0, 20.0), clip()),
        (bounds(48.0, 144.0, 1104.01, 20.0), clip()),
        (bounds(48.0, 144.0, 180.0, 808.01), clip()),
        (control(), bounds(49.0, 144.0, 180.0, 20.0)),
        (control(), bounds(48.0, 144.0, 179.99, 20.0)),
        (control(), bounds(48.0, 144.0, 180.0, 19.99)),
        (control(), bounds(f32::INFINITY, 0.0, 1200.0, 1000.0)),
        (control(), bounds(0.0, 0.0, 0.0, 1000.0)),
    ] {
        let mut target = Target::default();
        let token = target.begin_draw();
        target.observe(Some(token), vec![child], mask);
        assert!(target.resolve(token, true).is_err(), "{child:?} / {mask:?}");
    }
    let mut target = Target::default();
    let token = target.begin_draw();
    target.observe(Some(token), vec![control()], control());
    assert_eq!(target.resolve(token, true).unwrap(), Some(control()));
}

#[test]
fn guest_gate_explicitly_enables_capture_and_selects_ignored_scenarios() {
    // Parsing only: do not launch the VM or execute a GUI scenario on the host.
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let result = std::process::Command::new("python3")
        .args(["-I", "-c", r#"
import pathlib, sys
sys.path.insert(0, str(pathlib.Path(sys.argv[1]).resolve() / 'scripts'))
import e2e_vm
plan = e2e_vm.plan_for(['e2e-gallery-capture'])
assert plan == [('e2e-gallery-capture', '-p bbr-gallery --features visual-capture --bin bbr-gallery', '--ignored --nocapture --test-threads=1')], plan
"#])
        .arg(repo)
        .output()
        .expect("read the existing guest plan");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn page_reset_does_not_reuse_a_prior_draw_token() {
    let page = crate::capture_page::PageCapture::default();
    let before = page.0.borrow_mut().checkbox.begin_draw();
    page.0
        .borrow_mut()
        .checkbox
        .observe(Some(before), vec![control()], clip());
    page.reset(Some(2..5));
    let after = page.0.borrow_mut().checkbox.begin_draw();
    assert_ne!(before, after);
    page.0
        .borrow_mut()
        .checkbox
        .observe(Some(before), vec![control()], clip());
    assert!(page.0.borrow().checkbox.resolve(after, false).is_err());
}
