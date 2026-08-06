//! Shared dense-list shell E2E helpers (Phase-4 dense slice, DESIGN.md
//! §11.6).
//!
//! Seeding and contract assertions shared by the dense suite binaries
//! (`e2e_shell_dense` for the 500/230-identity lifecycle and keyboard
//! scenarios, `e2e_shell_dense_edge` for the edge/drag scenarios). Keeping
//! them here keeps every suite file under the ~500-line budget while the
//! scenarios stay in one place per test. The dense contract mirrors
//! DESIGN.md §11.6: exactly `live_cap` live sessions, `total - live_cap`
//! suspended, the keep set (pinned + recent) owning views, and the active
//! identity pinned. Seeding runs through the production controller open path
//! in bounded batches; between batches the concurrent live-session
//! population is asserted against `live_cap + 1` — the bounded-transient
//! evidence that preparing dense state never holds `total` WKWebViews.
//! Scroll is driven through the production scroll state (the tracked
//! `ScrollHandle` the native wheel listener, keyboard traversal, and render
//! share).

#![allow(dead_code)] // shared helper module: each suite uses a subset

use bbr_testkit::protocol::{LifecycleCounts, Request, Snapshot};
use bbr_testkit::Client;

/// The bounded render window the shell lays out per frame (DESIGN.md §11.6
/// visible range + overscan; mirrors `crate::dense::DENSE_WINDOW_ROWS`).
pub const WINDOW_ROWS: usize = 64;

pub fn flush(client: &mut Client) {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
}

pub fn scroll_dense(client: &mut Client, delta_px: f32) {
    client
        .send_ok(&Request::ScrollDenseTabs { delta_px })
        .expect("dense scroll");
}

pub fn keystroke(client: &mut Client, key: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: key.into(),
        })
        .expect("keystroke");
}

/// Seed `total` dense identities through the production controller open
/// path in `batch`-sized chunks. Between batches the concurrent live-session
/// population is asserted against `live_cap + 1` — the bounded-transient
/// evidence that preparing dense state never holds `total` WKWebViews.
pub fn seed_dense(
    client: &mut Client,
    base_url: &str,
    total: usize,
    pinned: usize,
    live_cap: usize,
    batch: usize,
) {
    let existing = client.snapshot().expect("pre-seed snapshot").tabs.len();
    let recent_start = total.saturating_sub(live_cap.saturating_sub(pinned));
    let to_open = total.saturating_sub(existing);
    let mut seeded = 0;
    while seeded < to_open {
        let chunk = batch.min(to_open - seeded);
        client
            .send_ok(&Request::SeedDenseState {
                tab_count: chunk,
                pinned_count: pinned,
                recent_start,
                url: base_url.to_string(),
            })
            .expect("seed dense batch");
        seeded += chunk;
        let snap = client.snapshot().expect("post-batch snapshot");
        assert!(
            snap.engine_sessions <= live_cap + 1,
            "preparing dense state must never hold more than live_cap + 1 \
             concurrent sessions; observed {} (cap {live_cap})",
            snap.engine_sessions
        );
    }
}

/// Converge the live-session population to exactly `live_cap` through the
/// production budget policy and assert the dense contract: `total`
/// identities, exactly `live_cap` sessions, `total - live_cap` suspended,
/// the keep set (pinned + recent) owning views, pinned/regular boundary
/// intact, and the active identity pinned.
pub fn assert_dense_contract(snap: &Snapshot, total: usize, pinned: usize, live_cap: usize) {
    assert_eq!(snap.tabs.len(), total, "all dense identities present");
    assert_eq!(
        snap.engine_sessions, live_cap,
        "exactly the session cap owns a live native view"
    );
    let counts = snap.lifecycle_counts;
    assert_eq!(
        counts,
        LifecycleCounts {
            active: 1,
            warm: live_cap - 1,
            suspended: total - live_cap,
            discarded: 0,
        },
        "one active + (cap − 1) warm live, the rest suspended"
    );
    assert_eq!(snap.pinned_count, pinned, "pinned region boundary intact");
    assert_eq!(snap.active_tab_index, Some(0), "active identity is tab 0");
    assert!(
        snap.tabs[0].pinned,
        "the active identity stays inside the pinned region"
    );
    let recent_start = total - (live_cap - pinned);
    for (index, tab) in snap.tabs.iter().enumerate() {
        let kept = index < pinned || index >= recent_start;
        assert_eq!(
            tab.web_view.is_some(),
            kept,
            "tab {index} session presence must follow the keep set"
        );
        assert_eq!(
            tab.lifecycle,
            if kept {
                if index == 0 {
                    "active"
                } else {
                    "warm"
                }
            } else {
                "suspended"
            }
            .to_string(),
            "tab {index} lifecycle must follow the keep set"
        );
    }
}

/// The fixture URL for a relative path under the running fixture server.
pub fn fixtures_url_long(base: &str, path: &str) -> String {
    let idx = base.find("/page1.html").expect("fixture base");
    format!("{}{path}", &base[..idx])
}
