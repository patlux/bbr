//! Renderer-responsiveness performance scenario drivers.
//!
//! Registered in the optimized interaction matrix but intentionally not run
//! while Patrick's performance-execution override is active. Each scenario
//! uses at least `TrialConfig::interaction_samples` (the canonical config is
//! 30) and rejects fast-but-wrong app states before accepting a sample.

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId, Request};
use bbr_testkit::Client;
use bbr_web::{WebEvent, WebViewId};

use super::benchmarks::wait_measurement;
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

#[allow(clippy::too_many_arguments)]
pub fn drive_responsiveness_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    page_one: &str,
    first: WebViewId,
    second: WebViewId,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    assert!(
        config.interaction_samples >= 30,
        "renderer performance scenarios require at least 30 configured samples"
    );
    let mut out = BTreeMap::new();

    // The source state is derived through the real engine-event transport.
    client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
    client
        .send_ok(&Request::InjectWebEvent {
            event: WebEvent::UnresponsiveChanged(first, true),
        })
        .unwrap();
    let source = client.snapshot().expect("unresponsive performance source");
    assert!(source.tabs[0].unresponsive);
    assert_eq!(source.active_web_view, Some(first));
    assert_eq!(source.engine_sessions, 2);

    let mut chrome = Vec::with_capacity(config.interaction_samples * 2);
    // Warm-neighbor selection: 30 exact render-inclusive samples.
    for sample in 0..config.interaction_samples {
        let (source_index, source_view, neighbor_index, neighbor_view) = if sample % 2 == 0 {
            (0, first, 1, second)
        } else {
            (1, second, 0, first)
        };
        client
            .send_ok(&Request::SelectTab {
                index: source_index,
            })
            .unwrap();
        client
            .send_ok(&Request::InjectWebEvent {
                event: WebEvent::UnresponsiveChanged(source_view, true),
            })
            .unwrap();
        let ns = run_chrome(
            client,
            since,
            operation,
            source_view,
            neighbor_index,
            neighbor_view,
            2,
            false,
        );
        chrome.push(ns);
        harness
            .raw_log
            .interactions
            .entry("unresponsive_chrome".into())
            .or_default()
            .push(ns);
    }

    // Cmd-W close → exact warm neighbor: 30 samples. Fixture/navigation setup
    // is outside the timed region; each sample opens a disposable third tab,
    // derives unresponsive through the real event transport, then measures
    // only the normal close/presentation path back to the same warm neighbor.
    client.send_ok(&Request::SelectTab { index: 1 }).unwrap();
    for _ in 0..config.interaction_samples {
        client
            .send_ok(&Request::OpenTab {
                url: page_one.into(),
            })
            .unwrap();
        let opened = client.snapshot().expect("unresponsive close setup");
        let source = opened.active_web_view.expect("disposable source");
        assert_eq!(opened.tabs.len(), 3);
        assert_eq!(opened.engine_sessions, 3);
        client
            .send_ok(&Request::InjectWebEvent {
                event: WebEvent::UnresponsiveChanged(source, true),
            })
            .unwrap();
        let ns = run_chrome(client, since, operation, source, 1, second, 2, true);
        chrome.push(ns);
        harness
            .raw_log
            .interactions
            .entry("unresponsive_chrome".into())
            .or_default()
            .push(ns);
    }
    out.insert("unresponsive_chrome".into(), chrome);

    // Probe tick: first force a completing probe, then immediately run the
    // cadence tick inside the 1s re-arm interval. The app guard proves no
    // state/session/focus mutation; an idle tick emits no probe traffic.
    client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
    client
        .send_ok(&Request::InjectWebEvent {
            event: WebEvent::UnresponsiveChanged(first, false),
        })
        .unwrap();
    let mut ticks = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let operation_id = BenchmarkOperationId(*operation);
        *operation += 1;
        client
            .send_ok(&Request::BenchmarkProbeTick {
                operation_id,
                view: first,
                expected_sessions: 2,
            })
            .expect("benchmark idle probe tick");
        let (seq, measurement) =
            wait_measurement(client, BenchmarkKind::ProbeTick, operation_id, *since);
        *since = seq;
        ticks.push(measurement.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("probe_tick".into())
            .or_default()
            .push(measurement.duration_ns);
    }
    out.insert("probe_tick".into(), ticks);
    out
}

#[allow(clippy::too_many_arguments)]
fn run_chrome(
    client: &mut Client,
    since: &mut u64,
    operation: &mut u64,
    source: WebViewId,
    neighbor_index: usize,
    expected_neighbor: WebViewId,
    expected_sessions: usize,
    close: bool,
) -> u64 {
    let operation_id = BenchmarkOperationId(*operation);
    *operation += 1;
    client
        .send_ok(&Request::BenchmarkUnresponsiveChrome {
            operation_id,
            source,
            neighbor_index,
            expected_neighbor,
            expected_sessions,
            close,
        })
        .expect("benchmark unresponsive chrome");
    let (seq, measurement) = wait_measurement(
        client,
        BenchmarkKind::UnresponsiveChrome,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measurement.active_web_view, Some(expected_neighbor));
    measurement.duration_ns
}
