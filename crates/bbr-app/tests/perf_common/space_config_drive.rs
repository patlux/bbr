//! §11.3 Space-configuration scenario driving (packet
//! `space-creation-appearance-ui.md`) for the interaction matrix.
//!
//! Owns the matrix block for the create→activate→configure lifecycle and
//! the exact-SpaceId identity commit: `space_create_present` (the
//! production `commit_new_space` path — the new Space activates immediately
//! and its optional name/icon configuration surface renders open bound to
//! the exact created persistent SpaceId) and `space_config_identity_update`
//! (the production `space_config_done` path — the surface closes with the
//! bound Space still active and the same warm session). Both commit only
//! from render-gated presentations (in-app fast-but-wrong guards) and the
//! drivers re-verify the typed snapshots.
//!
//! Execution stays unexecuted/unpassed under Patrick's active override —
//! this block compiles into the canonical runner so a future un-deferred
//! run exercises the exact production paths. Split out of `drive.rs` so
//! the additions live in a focused module under the ~500-line budget,
//! mirroring the B3 `library_drive` module.

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_testkit::Client;

use super::benchmarks::{
    benchmark_space_config_identity_update, benchmark_space_create, wait_browser_change, wait_title,
};
use super::harness::AppHarness;
use super::scenarios::{TrialConfig, PAGE_TWO_TITLE};

/// The deterministic curated identities the identity-update loop alternates
/// between (both validated members of the closed emoji allowlist, so every
/// sample actually changes the bound chip's identity).
const IDENTITIES: [(&str, &str); 2] = [("🌊", "🌊"), ("🔥", "🔥")];

/// Drive the §11.3 Space-configuration matrix block and return the raw
/// samples per scenario.
pub(super) fn drive_space_config_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    operation: &mut u64,
    page_two: &str,
) -> BTreeMap<String, Vec<u64>> {
    let before = client.snapshot().expect("pre-block snapshot");
    let default_profile = before.active_profile;
    let expected_sessions = before.engine_sessions;

    // Unmeasured setup: one existing "Config" space with its own warm tab
    // (the same production controller path the space-switch block uses) is
    // the identity-update target. The create samples each create another
    // fresh space through the production user path.
    client
        .send_ok(&Request::CreateSpace {
            profile: default_profile,
            title: "Config".into(),
        })
        .expect("create config space");
    let (seq, created) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
        *since,
    );
    *since = seq;
    let config_space = match created {
        BrowserChange::SpaceCreated { space, .. } => space,
        other => panic!("unexpected change: {other:?}"),
    };
    client
        .send_ok(&Request::ActivateSpace {
            space: config_space,
        })
        .expect("activate config space");
    let (seq, _) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == config_space),
        *since,
    );
    *since = seq;
    client
        .send_ok(&Request::OpenTab {
            url: page_two.to_string(),
        })
        .expect("warm config space tab");
    let warm = client.snapshot().expect("config warm snapshot");
    let config_view = warm.active_web_view.expect("config space warm view");
    if warm.title.as_deref() != Some(PAGE_TWO_TITLE) {
        *since = wait_title(client, config_view, PAGE_TWO_TITLE, *since);
    }

    // space_create_present × N: each sample creates a fresh Space through
    // the production `commit_new_space` path (creation commits before
    // configuration, the new Space activates immediately, its config
    // surface renders open bound to the exact created SpaceId). The
    // created-space ids grow per sample; the live-session count never
    // changes (a fresh Space is empty).
    let mut create_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let name = format!("Created {sample}");
        let (measured, created) =
            benchmark_space_create(client, since, *operation, &name, expected_sessions);
        create_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("space_create_present".into())
            .or_default()
            .push(measured.duration_ns);
        // The created space is the new active space; keep the strip chrome
        // domain stable for the next sample (no session was created).
        assert_eq!(created, measured.active_space);
        *operation += 1;
    }

    // space_config_identity_update × N: each sample opens the configuration
    // surface for the exact existing Config space, commits one curated
    // identity through the production `space_config_done` authority, and
    // commits only when the surface is drawn closed with the bound space
    // still active and the same warm session. The two deterministic
    // identities alternate so every sample actually changes the chip.
    client
        .send_ok(&Request::ActivateSpace {
            space: config_space,
        })
        .expect("activate config space before identity samples");
    let (seq, _) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == config_space),
        *since,
    );
    *since = seq;
    let mut identity_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let (identity, expected) = IDENTITIES[sample % IDENTITIES.len()];
        let measured = benchmark_space_config_identity_update(
            client,
            since,
            *operation,
            config_space,
            identity,
            expected,
            expected_sessions,
        );
        identity_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("space_config_identity_update".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    let mut raw = BTreeMap::new();
    raw.insert("space_create_present".to_string(), create_samples);
    raw.insert("space_config_identity_update".to_string(), identity_samples);
    raw
}
