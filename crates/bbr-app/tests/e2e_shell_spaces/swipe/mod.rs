//! Real-app DESIGN.md §11.3 sidebar-swipe evidence (F2a swipe product
//! kernel): the production horizontal trackpad wheel path drives the pure
//! gesture machine, chrome-only interpolation, and the existing
//! `activate_space` authority over real WKWebViews.
//!
//! The testkit cannot synthesize GPUI scroll-wheel events publicly, so every
//! step drives the **exact production handler** the sidebar's
//! `on_scroll_wheel` listener calls (`BbrApp::on_sidebar_scroll_wheel`)
//! through the typed `SwipeScroll` request — same dominance filter, phase
//! gating, threshold, and activation authority as a real trackpad gesture.
//! The release phase is always driven with a **zero delta** (`[0.0, 0.0]`),
//! exactly what a real macOS `Ended` event carries; the handler must resolve
//! it independent of horizontal dominance. No sleeps: every step
//! synchronizes on semantic events or snapshot state under a deadline.
//!
//! The suite proves:
//!
//! 1. **Below-threshold release and explicit cancel** emit no switch, change
//!    no controller state, and return the chrome to the exact source
//!    presentation ([`cancel`]).
//! 2. **Above-threshold release commits exactly one switch** in both
//!    display directions with warm WebView/session identity, unchanged
//!    content/pane geometry, and chrome focus preserved ([`commit`]).
//! 3. **Boundary (no adjacent space) and vertical-dominant deltas are
//!    no-ops**: the gesture never begins and the machine stays Idle
//!    ([`noops`]).
//! 4. **Tracking is typed and truthful**: the snapshot carries
//!    active/source/destination/direction/progress plus the interpolated
//!    chrome/chip tints, the semantic selected chip stays on the source
//!    until commit (recursive lookup under `bbr-root`), and the tint clears
//!    on cancel ([`tracking`]).
//! 5. **Cross-profile adjacent swipe keeps the existing authority**: no
//!    session recreation and the one-interaction command-field profile hint
//!    ([`cross_profile`]).

use super::e2e_semantics_common::find;
use super::*;

mod cancel;
mod commit;
mod cross_profile;
mod noops;
mod tint_fade;
mod tracking;

/// The persistent chip id of a space (`bbr-space-chip-<persistent-id>`).
fn chip_id(space: bbr_testkit::protocol::TestSpaceId) -> String {
    format!("bbr-space-chip-{}", space.0)
}

/// The semantic selected state of one space's chip — the recursive §15
/// lookup under `bbr-root` (the sole top-level semantic node; every chip
/// node is nested in the sidebar/footer/strip branch), the same established
/// helper the semantics suites use.
fn chip_selected(
    snapshot: &bbr_testkit::protocol::Snapshot,
    space: bbr_testkit::protocol::TestSpaceId,
) -> bool {
    find(&snapshot.semantics, &chip_id(space))
        .map(|node| node.selected)
        .unwrap_or(false)
}

/// The one selected chip's space in the snapshot (exactly one is selected).
fn selected_space(
    snapshot: &bbr_testkit::protocol::Snapshot,
) -> bbr_testkit::protocol::TestSpaceId {
    let selected: Vec<_> = snapshot
        .space_chips
        .iter()
        .filter(|chip| chip.selected)
        .collect();
    assert_eq!(selected.len(), 1, "exactly one chip renders selected");
    selected[0].space
}

/// Create `title` in `profile`, return its persistent space id.
fn create_space_id(
    client: &mut Client,
    since: &mut u64,
    profile: bbr_web::ProfileId,
    title: &str,
) -> bbr_testkit::protocol::TestSpaceId {
    client
        .send_ok(&dsl::create_space(profile, title))
        .expect("create space");
    let (seq, created) = wait_browser_change_matching(
        client,
        |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
        *since,
    );
    *since = seq;
    match created {
        BrowserChange::SpaceCreated { space, .. } => space,
        other => panic!("unexpected change: {other:?}"),
    }
}

/// Activate `space` through the production chip listener and wait for the
/// typed activation event.
fn activate_space_id(
    client: &mut Client,
    since: &mut u64,
    space: bbr_testkit::protocol::TestSpaceId,
) {
    client
        .send_ok(&dsl::click_space(space))
        .expect("activate space");
    let seq = wait_browser_change(client, BrowserChange::SpaceActivated { space }, *since);
    *since = seq;
}
