//! The warm tab/sidebar timings must exercise glass, not a guest's opaque
//! accessibility fallback. Setup and observations stay outside timed regions.
use bbr_testkit::protocol::{dsl, Request, Snapshot};
use bbr_testkit::Client;

pub(super) fn prepare(client: &mut Client) {
    client
        .send_ok(&dsl::set_display_accessibility(
            Some(false),
            None,
            Some(false),
        ))
        .expect("enable glass independently of machine display preferences");
    client
        .send_ok(&Request::FlushFrame)
        .expect("draw glass setup");
    assert_present(&client.snapshot().expect("glass setup snapshot"));
}

pub(super) fn assert_present(snapshot: &Snapshot) {
    let material = snapshot
        .content_card
        .chrome_material
        .expect("native chrome material");
    assert!(material.attached && material.enabled && material.card_mask_valid);
    assert!(material.behind_window && material.follows_activation);
    assert!(!material.window_opaque && !material.reduce_transparency);
    assert!(material.tint_alpha > 0.0 && material.tint_alpha < 1.0);
    assert_eq!(material.web_view_opaque, Some(true), "website stays opaque");
}
