//! Blueprint D06: profile rename, colour and deletion through the
//! controller.

use bbr_web::WebViewId;

use crate::harness::{open, plain};
use crate::ControllerEvent;

#[test]
fn deleting_the_active_profile_destroys_its_sessions_and_activates_the_fallback() {
    let (mut c, h) = plain();
    let default_profile = c.active_profile();
    let default_space = c.snapshot().active_space;
    open(&mut c, "https://default.test");
    let ControllerEvent::ProfileCreated { profile, space } =
        c.create_profile_space("Work", "Work").unwrap()
    else {
        panic!("expected ProfileCreated");
    };
    c.activate_space(space).unwrap();
    open(&mut c, "https://work.test");
    let work_view: WebViewId = c.snapshot().active_view.expect("work session");
    assert!(c.can_delete_profile(profile));
    let event = c.delete_profile(profile).unwrap();
    assert_eq!(
        event,
        ControllerEvent::ProfileDeleted {
            profile,
            fallback: default_space,
            spaces: 1,
            closed_tabs: 1,
            activated: true,
        }
    );
    let snapshot = c.snapshot();
    assert_eq!(snapshot.active_space, default_space);
    assert_eq!(snapshot.profiles.len(), 1);
    assert!(snapshot
        .all_tabs
        .iter()
        .all(|tab| tab.space == default_space));
    assert!(
        h.ops()
            .contains(&crate::fake::EngineOp::Destroy { view: work_view }),
        "the work session is gone"
    );
    assert!(
        !c.can_delete_profile(default_profile),
        "the last profile stays"
    );
}

#[test]
fn renaming_and_recolouring_show_in_the_snapshot() {
    let (mut c, _h) = plain();
    let profile = c.active_profile();
    c.rename_profile(profile, "Personal").unwrap();
    c.set_profile_hue(profile, bbr_domain::ProfileHue::ALL[3])
        .unwrap();
    let snapshot = c.snapshot();
    let entry = snapshot.profiles.iter().find(|p| p.id == profile).unwrap();
    assert_eq!(entry.title, "Personal");
    assert_eq!(entry.hue, bbr_domain::ProfileHue::ALL[3]);
}
