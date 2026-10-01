//! Closed pinned tab scenario driving (task packet
//! `closed-pins-and-favorites`, DESIGN.md §10.12) for the interaction
//! matrix.
//!
//! Owns three scenarios on a dedicated space with one pinned tab and one
//! regular neighbor, both on deterministic fixture pages loaded before any
//! sample:
//!
//! - `pinned_tab_close`: the production ⌘W path on the active pinned tab
//!   closes it in place — the row stays, its session is destroyed and the
//!   warm neighbor is presented (the `active_tab_close` warm class,
//!   50/100 ms).
//! - `closed_tab_reopen`: the production ⇧⌘T path reopens the closed row in
//!   place with a fresh session at the page it closed on (the session
//!   recreation class of `tab_reopen`, 100/150 ms).
//! - `closed_row_select`: the production pointer path on the closed row
//!   reopens it with a fresh session at its pinned page (the same session
//!   recreation class, 100/150 ms).
//!
//! Every sample commits from the in-app probe's authoritative presentation
//! (the active view must be the expected one); the driver re-verifies the
//! typed snapshot (row count, closed lifecycle, session count) so a
//! fast-but-wrong close or reopen is rejected. The block re-activates the
//! space that was active before it, so the blocks after it see their state.

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId, BrowserChange, Request};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::{benchmark_keystroke, wait_browser_change, wait_measurement, wait_title};
use super::harness::AppHarness;
use super::scenarios::{TrialConfig, PAGE_ONE_TITLE, PAGE_TWO_TITLE};

/// Drive the closed-pin matrix block and return the raw samples.
pub(super) fn drive_closed_pin_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    operation: &mut u64,
    page_one: &str,
    page_two: &str,
) -> BTreeMap<String, Vec<u64>> {
    let before = client.snapshot().expect("pre-block snapshot");
    let neighbor = setup(client, since, page_one, page_two);
    let mut close_samples = Vec::with_capacity(2 * config.interaction_samples);
    let mut reopen_samples = Vec::with_capacity(config.interaction_samples);
    let mut select_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..2 * config.interaction_samples {
        // The pinned row is open and selected before every close.
        let closed = close_pinned(client, since, *operation, neighbor);
        record(harness, "pinned_tab_close", &mut close_samples, closed);
        *operation += 1;
        let (key, reopened, samples) = if sample % 2 == 0 {
            let ns = reopen_by_shortcut(client, since, *operation, neighbor);
            ("closed_tab_reopen", ns, &mut reopen_samples)
        } else {
            let ns = reopen_by_pointer(client, since, *operation, neighbor);
            ("closed_row_select", ns, &mut select_samples)
        };
        record(harness, key, samples, reopened);
        *operation += 1;
    }
    // Unmeasured teardown: close the neighbor and the pinned row (in place)
    // so the block leaves no live session behind, then re-activate the
    // previous space.
    for index in [1, 0] {
        client
            .send_ok(&Request::CloseTab { index })
            .expect("close a kept-places tab");
    }
    client
        .send_ok(&Request::ActivateSpace {
            space: before.active_space,
        })
        .expect("re-activate the previous space");
    let (seq, _) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == before.active_space),
        *since,
    );
    *since = seq;
    assert_eq!(
        client
            .snapshot()
            .expect("post-block snapshot")
            .engine_sessions,
        before.engine_sessions,
        "the block leaves no live session behind"
    );
    let mut raw = BTreeMap::new();
    raw.insert("pinned_tab_close".to_string(), close_samples);
    raw.insert("closed_tab_reopen".to_string(), reopen_samples);
    raw.insert("closed_row_select".to_string(), select_samples);
    raw
}

/// Keep one accepted sample for the report and the raw log.
fn record(harness: &mut AppHarness, key: &str, samples: &mut Vec<u64>, ns: u64) {
    samples.push(ns);
    harness
        .raw_log
        .interactions
        .entry(key.into())
        .or_default()
        .push(ns);
}

/// Unmeasured setup: a dedicated space with [pinned page one | page two],
/// both loaded, the pinned row selected. Returns the neighbor's view.
fn setup(client: &mut Client, since: &mut u64, page_one: &str, page_two: &str) -> WebViewId {
    let profile = client
        .snapshot()
        .expect("pre-block snapshot")
        .active_profile;
    client
        .send_ok(&Request::CreateSpace {
            profile,
            title: "Kept Places".into(),
        })
        .expect("create kept-places space");
    let (seq, created) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
        *since,
    );
    *since = seq;
    let BrowserChange::SpaceCreated { space, .. } = created else {
        panic!("unexpected change: {created:?}");
    };
    client
        .send_ok(&Request::ActivateSpace { space })
        .expect("activate kept-places space");
    let (seq, _) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::SpaceActivated { space: s } if *s == space),
        *since,
    );
    *since = seq;
    for (url, title) in [(page_one, PAGE_ONE_TITLE), (page_two, PAGE_TWO_TITLE)] {
        client
            .send_ok(&Request::OpenTab {
                url: url.to_string(),
            })
            .expect("open kept-places tab");
        let opened = client.snapshot().expect("opened snapshot");
        let view = opened.active_web_view.expect("opened view");
        if opened.title.as_deref() != Some(title) {
            *since = wait_title(client, view, title, *since);
        }
    }
    let neighbor = client.snapshot().expect("neighbor").tabs[1]
        .web_view
        .expect("neighbor view");
    client
        .send_ok(&Request::ClickTab { index: 0 })
        .expect("select the first row");
    let (seq, _) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::TabSelected { index } if *index == 0),
        *since,
    );
    *since = seq;
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "cmd-shift-p".into(),
        })
        .expect("pin the first row");
    let (seq, _) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::TabPinned { index } if *index == 0),
        *since,
    );
    *since = seq;
    let pinned = client.snapshot().expect("pinned snapshot");
    assert_eq!(pinned.pinned_count, 1);
    assert_eq!(pinned.active_tab_index, Some(0));
    neighbor
}

/// `pinned_tab_close`: ⌘W on the selected pinned row.
fn close_pinned(client: &mut Client, since: &mut u64, operation: u64, neighbor: WebViewId) -> u64 {
    let measured = benchmark_keystroke(
        client,
        since,
        operation,
        BenchmarkKind::ActiveTabClose,
        "cmd-w",
    );
    assert_eq!(
        measured.active_web_view,
        Some(neighbor),
        "the warm neighbor is presented"
    );
    let snapshot = client.snapshot().expect("closed snapshot");
    assert_eq!(snapshot.tabs.len(), 2, "the pinned row stays");
    assert_eq!(snapshot.tabs[0].lifecycle, "closed");
    assert_eq!(snapshot.tabs[0].web_view, None, "its session is gone");
    assert_eq!(snapshot.active_tab_index, Some(1));
    measured.duration_ns
}

/// `closed_tab_reopen`: ⇧⌘T reopens the closed row in place.
fn reopen_by_shortcut(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    neighbor: WebViewId,
) -> u64 {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkReopenTab { operation_id })
        .expect("benchmark reopen");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::TabReopen, operation_id, *since);
    *since = seq;
    assert_reopened(client, measured.active_web_view, neighbor);
    measured.duration_ns
}

/// `closed_row_select`: a pointer press on the closed row reopens it.
fn reopen_by_pointer(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    neighbor: WebViewId,
) -> u64 {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkClickTab {
            operation_id,
            index: 0,
        })
        .expect("benchmark closed-row click");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::TabSelectPointer,
        operation_id,
        *since,
    );
    *since = seq;
    assert_reopened(client, measured.active_web_view, neighbor);
    measured.duration_ns
}

/// The reopened row presents a fresh session in the same place.
fn assert_reopened(client: &mut Client, presented: Option<WebViewId>, neighbor: WebViewId) {
    let fresh = presented.expect("a fresh session is presented");
    assert_ne!(fresh, neighbor);
    let snapshot = client.snapshot().expect("reopened snapshot");
    assert_eq!(snapshot.tabs.len(), 2, "no row was added");
    assert_eq!(snapshot.active_tab_index, Some(0));
    assert_eq!(snapshot.tabs[0].web_view, Some(fresh));
    assert_eq!(snapshot.tabs[0].lifecycle, "active");
    assert_eq!(
        snapshot.tabs[1].web_view,
        Some(neighbor),
        "the neighbor stays warm"
    );
}
