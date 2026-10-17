//! The warm tab/sidebar timings must exercise glass, not a guest's opaque
//! accessibility fallback. Setup and observations stay outside timed regions.
use std::collections::BTreeMap;

use bbr_testkit::protocol::{
    dsl, BenchmarkKind, BenchmarkOperationId, Request, SettingsInput, Snapshot,
};
use bbr_testkit::Client;

use super::benchmarks::wait_measurement;
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// The Settings transparency default the matrix leaves behind.
const DEFAULT_TRANSPARENCY: u8 = 50;
/// Alternating values: every sample changes the painted coverage, and the
/// cycle crosses the default, both ends and the glass-off value 0.
const TRANSPARENCY_CYCLE: [u8; 6] = [80, 20, 100, 0, 65, 35];

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

/// §11.4: switch full motion on (or back to the observed system value)
/// while keeping the glass setup of [`prepare`]: one override carries all
/// three display options, so motion never clears the glass overrides.
pub(super) fn full_motion(client: &mut Client, on: bool) {
    client
        .send_ok(&dsl::set_display_accessibility(
            Some(false),
            on.then_some(false),
            Some(false),
        ))
        .expect("motion override with the glass setup");
}

/// `chrome_transparency_change` × N: the Settings slider's production event
/// until the browser window paints the new tint. The in-app probe requires
/// the exact armed coverage; the driver checks the material state, the
/// ordering of coverages and the unchanged active view, then restores the
/// default (unmeasured) so the glass assertions of later blocks hold.
pub(super) fn drive_transparency_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    let before = client.snapshot().expect("transparency setup snapshot");
    assert_present(&before);
    let view = before.active_web_view.expect("active view");
    let mut previous = (
        DEFAULT_TRANSPARENCY,
        before.content_card.chrome_material.unwrap().tint_alpha,
    );
    let mut samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let percent = TRANSPARENCY_CYCLE[sample % TRANSPARENCY_CYCLE.len()];
        let operation_id = BenchmarkOperationId(*operation);
        client
            .send_ok(&Request::BenchmarkChromeTransparency {
                operation_id,
                percent,
            })
            .expect("benchmark chrome transparency");
        let (seq, measured) = wait_measurement(
            client,
            BenchmarkKind::ChromeTransparencyChange,
            operation_id,
            *since,
        );
        *since = seq;
        assert_eq!(measured.active_web_view, Some(view), "no session change");
        let snapshot = client.snapshot().expect("transparency snapshot");
        let material = snapshot.content_card.chrome_material.expect("material");
        assert_eq!(material.enabled, percent > 0, "0 % turns the glass off");
        assert_eq!(material.web_view_opaque, Some(true), "website stays opaque");
        if percent == 0 {
            assert_eq!(material.tint_alpha, 1.0);
        } else {
            assert!(material.tint_alpha < 1.0);
        }
        // More transparency always paints less tint, and the reverse.
        if percent > previous.0 {
            assert!(
                material.tint_alpha < previous.1,
                "{percent} vs {previous:?}"
            );
        } else {
            assert!(
                material.tint_alpha > previous.1,
                "{percent} vs {previous:?}"
            );
        }
        previous = (percent, material.tint_alpha);
        samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("chrome_transparency_change".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }
    client
        .send_ok(&Request::SettingsInput {
            input: SettingsInput::Slide {
                row: "appearance-transparency".into(),
                value: DEFAULT_TRANSPARENCY,
            },
        })
        .expect("restore the default transparency");
    client
        .send_ok(&Request::FlushFrame)
        .expect("draw the default transparency");
    assert_present(&client.snapshot().expect("restored transparency"));
    BTreeMap::from([("chrome_transparency_change".to_string(), samples)])
}

pub(super) fn assert_present(snapshot: &Snapshot) {
    let material = snapshot
        .content_card
        .chrome_material
        .expect("native chrome material");
    assert!(material.attached && material.enabled && material.card_mask_valid);
    assert!(material.behind_window && material.follows_activation);
    assert!(!material.window_opaque && !material.reduce_transparency);
    assert!(material.tint_alpha > 0.0 && material.tint_alpha <= 0.67);
    assert_eq!(material.web_view_opaque, Some(true), "website stays opaque");
}
