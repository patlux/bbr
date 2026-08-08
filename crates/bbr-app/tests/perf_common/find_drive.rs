//! Find-in-page scenario driving (task packet `find-in-page-portal-bar`)
//! for the warm interaction matrix.
//!
//! Owns the find matrix block: the production `Cmd-F` open
//! (`find_open`/`find_query_result`/`find_step`), measured on a
//! deterministic multi-match fixture (`find-repeated.html`, "alice" = 3
//! visible matches with hidden text skipped) through the typed benchmark
//! requests. Every sample starts at the production seam and commits only
//! from a render of the portal bar that carried the armed open/report
//! identity (fast-but-wrong guard); fixture/network time is excluded by
//! construction (the page is loaded before any sample). The block restores
//! the matrix state exactly — find closed, the transient find tab closed —
//! before the dense seeding that follows.
//!
//! Split from `drive.rs` so the find additions live in a focused module
//! under the ~500-line budget while the pre-existing matrix content stays
//! where it was (same pattern as `command_drive.rs`).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::{wait_browser_change, wait_title};
use super::find::{
    benchmark_find_open, benchmark_find_query_result, benchmark_find_step, FIND_PRESENTATION_ID,
};
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// The deterministic multi-match fixture title (find-repeated.html).
pub(super) const FIND_REPEATED_TITLE: &str = "BBR Find Repeated";
/// Visible "alice" matches on the fixture (hidden text excluded).
const ALICE_COUNT: usize = 3;
/// Visible "bob" matches on the fixture.
const BOB_COUNT: usize = 1;

/// Drive the find-in-page matrix block and return the raw samples per
/// scenario. Runs after the U2 sidebar block and before the dense seeding;
/// state before it: the default space's tabs with the dense block's
/// pre-conditions (whatever U2 left), no find presentation. The block opens
/// a transient find tab, measures the three warm find scenarios, then
/// restores the matrix state (find closed, transient tab closed).
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_find_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    page_find: &str,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    // Unmeasured setup: a transient tab on the deterministic multi-match
    // fixture, active in web focus. The fixture is loaded before any sample,
    // so fixture/network time never enters the measured durations.
    client
        .send_ok(&Request::OpenTab {
            url: page_find.to_string(),
        })
        .expect("open find fixture tab");
    let opened = client.snapshot().expect("find fixture snapshot");
    let find_view = opened.active_web_view.expect("find fixture view");
    if opened.title.as_deref() != Some(FIND_REPEATED_TITLE) {
        *since = wait_title(client, find_view, FIND_REPEATED_TITLE, *since);
    }
    // `open_tab` preserves the invoking focus domain (U2 leaves the matrix
    // chrome-focused through its pointer paths), so the origin here may be
    // chrome or web. The find benchmarks are origin-agnostic: opening
    // focuses the find field (chrome domain released) and Escape restores
    // whatever origin `find_open` captured.
    let sessions_before = opened.engine_sessions;
    let view_before = find_view;

    // find_open × N: each sample is a fresh production Cmd-F open from web
    // focus; the sample commits when the portal render drew the bar. Escape
    // (unmeasured) closes the bar between samples so every sample measures
    // the open transition, never a no-op re-focus.
    let mut find_open_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured = benchmark_find_open(client, since, *operation, find_view);
        find_open_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("find_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: Escape closes the bar (the origin was web, so
        // focus returns to the page) and clears highlights.
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape after find open sample");
        wait_find_closed(client);
    }

    // find_query_result × N: the bar is opened (unmeasured), then each
    // sample enters a fresh query through the production path and commits
    // only when WebKit reported the armed count/ordinal. Alternating
    // "alice" (3 matches) / "bob" (1 match) exercises both report shapes;
    // each query restarts the search, so no inter-sample reset is needed.
    client
        .send_ok(&Request::FindOpen)
        .expect("open find before query benchmarks");
    wait_find_open(client, find_view);
    let mut find_query_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let (query, count) = if sample % 2 == 0 {
            ("alice", ALICE_COUNT)
        } else {
            ("bob", BOB_COUNT)
        };
        let measured = benchmark_find_query_result(
            client,
            since,
            *operation,
            query,
            count,
            Some(1),
            find_view,
        );
        find_query_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("find_query_result".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // find_step × N: restart the search on the 3-match query (unmeasured),
    // then alternate next/previous from ordinal 1: even samples step next
    // (1 → 2), odd samples step previous (2 → 1). Every sample measures a
    // full WebKit step round trip and the re-rendered report.
    client
        .send_ok(&Request::FindSetQuery {
            query: "alice".into(),
        })
        .expect("restart alice for step benchmarks");
    wait_find_report(client, ALICE_COUNT, Some(1));
    let mut find_step_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let next = sample % 2 == 0;
        let ordinal = if next { Some(2) } else { Some(1) };
        let measured = benchmark_find_step(
            client,
            since,
            *operation,
            next,
            ALICE_COUNT,
            ordinal,
            find_view,
        );
        find_step_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("find_step".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // Restoration: Escape closes the find bar (web origin → focus back to
    // the page), then Cmd-W closes the transient fixture tab so the matrix
    // state (tab population, sessions) is exactly what the dense seeding
    // expected before the find block.
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "escape".into(),
        })
        .expect("escape after find block");
    wait_find_closed(client);
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "cmd-w".into(),
        })
        .expect("close transient find tab");
    *since = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::TabClosed { .. }),
        *since,
    )
    .0;
    let restored = client.snapshot().expect("find block restored snapshot");
    assert!(!restored.find.open, "find presentation closed");
    assert!(
        restored.engine_sessions <= sessions_before,
        "closing the transient tab never grows the session population"
    );
    assert!(
        restored.active_web_view != Some(view_before),
        "the transient fixture tab is gone"
    );

    let mut raw = BTreeMap::new();
    raw.insert("find_open".to_string(), find_open_samples);
    raw.insert("find_query_result".to_string(), find_query_samples);
    raw.insert("find_step".to_string(), find_step_samples);
    raw
}

/// Poll until the find presentation is fully closed (bounded by the harness
/// deadline).
fn wait_find_closed(client: &mut Client) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("find closed snapshot");
        if !snapshot.find.open {
            return;
        }
        assert!(
            Instant::now() < until,
            "find bar did not close: {:?}",
            snapshot.find
        );
        std::thread::yield_now();
    }
}

/// Poll until the find presentation is open with the bar over `view`.
fn wait_find_open(client: &mut Client, view: WebViewId) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("find open snapshot");
        if snapshot.find.open
            && snapshot.portal.presentation == Some(FIND_PRESENTATION_ID)
            && snapshot.active_web_view == Some(view)
        {
            return;
        }
        assert!(
            Instant::now() < until,
            "find bar did not open: {:?}",
            snapshot.find
        );
        std::thread::yield_now();
    }
}

/// Poll until WebKit reported the expected count/ordinal (bounded by the
/// harness deadline).
fn wait_find_report(client: &mut Client, count: usize, ordinal: Option<usize>) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("find report snapshot");
        let f = &snapshot.find;
        if f.match_count == Some(count) && f.current_match == ordinal && !f.pending {
            return;
        }
        assert!(
            Instant::now() < until,
            "find report did not land: expected count {count} ordinal {ordinal:?}; last {:?}",
            f
        );
        std::thread::yield_now();
    }
}
