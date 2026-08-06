//! Per-interaction benchmark drivers for the real-app performance suites.
//!
//! Each driver sends a typed `Benchmark*` gateway request that starts the
//! measurement inside the app process immediately before the production
//! mutation, then waits for the in-process `PresentationCommitted` sample and
//! asserts the semantic state the sample claims (active view, space, sidebar,
//! focus, mode). Socket transport is never part of the measured duration; a
//! fast-but-wrong sample is rejected by the state assertions.

use std::time::{Duration, Instant};

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, BrowserChange, CommandBarMode, Event,
    PresentationMeasurement, Request, SpaceVisualState, TestSpaceId,
};
use bbr_testkit::Client;
use bbr_web::{WebEvent, WebViewId};

pub(crate) fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

pub(crate) fn connect(socket: &std::path::Path) -> Client {
    let until = Instant::now() + Duration::from_secs(60);
    loop {
        match Client::connect(socket) {
            Ok(client) => return client,
            Err(_) if Instant::now() < until => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => panic!("could not connect to gateway: {error}"),
        }
    }
}

pub(crate) fn measurement(
    event: &Event,
    kind: BenchmarkKind,
    operation_id: BenchmarkOperationId,
) -> Option<PresentationMeasurement> {
    match event {
        Event::PresentationCommitted(measured)
            if measured.kind == kind && measured.operation_id == operation_id =>
        {
            Some(measured.clone())
        }
        _ => None,
    }
}

pub(crate) fn wait_measurement(
    client: &mut Client,
    kind: BenchmarkKind,
    operation_id: BenchmarkOperationId,
    since: u64,
) -> (u64, PresentationMeasurement) {
    let (seq, event) = client
        .wait_for_event_since(
            |event| measurement(event, kind, operation_id).is_some(),
            since,
            deadline(),
        )
        .expect("presentation measurement");
    (seq, measurement(&event, kind, operation_id).unwrap())
}

/// Wait for the next [`Event::BrowserChanged`] matching `matches` and return
/// its sequence cursor plus the concrete [`BrowserChange`].
pub(crate) fn wait_browser_change(
    client: &mut Client,
    matches: impl Fn(&BrowserChange) -> bool,
    since: u64,
) -> (u64, BrowserChange) {
    let (seq, event) = client
        .wait_for_event_since(
            |event| matches!(event, Event::BrowserChanged { kind } if matches(kind)),
            since,
            deadline(),
        )
        .expect("browser change");
    let Event::BrowserChanged { kind } = event else {
        unreachable!()
    };
    (seq, kind)
}

pub(crate) fn wait_title(client: &mut Client, view: WebViewId, title: &str, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::Web(WebEvent::TitleChanged(id, value)) if *id == view && value == title),
            since,
            deadline(),
        )
        .expect("title event")
        .0
}

pub(crate) fn benchmark_select(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkSelectTab {
            operation_id,
            index,
        })
        .expect("benchmark tab selection");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::WarmTabSwitch, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    let snapshot = client.snapshot().expect("selected snapshot");
    assert_eq!(snapshot.active_tab_index, Some(index));
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    measured
}

pub(crate) fn benchmark_keystroke(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    kind: BenchmarkKind,
    keystroke: &str,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDispatchKeystroke {
            operation_id,
            kind,
            keystroke: keystroke.into(),
        })
        .expect("benchmark keystroke");
    let (seq, measured) = wait_measurement(client, kind, operation_id, *since);
    *since = seq;
    measured
}

pub(crate) fn benchmark_sidebar(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_visible: bool,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDispatchKeystroke {
            operation_id,
            kind: BenchmarkKind::SidebarToggle,
            keystroke: "cmd-b".into(),
        })
        .expect("benchmark sidebar toggle");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::SidebarToggle, operation_id, *since);
    *since = seq;
    assert_eq!(measured.sidebar_visible, expected_visible);
    let snapshot = client.snapshot().expect("sidebar snapshot");
    assert_eq!(snapshot.sidebar_visible, expected_visible);
    measured
}

/// Resize the sidebar to an absolute width through the production resize
/// path (the mutation the pointer and keyboard listeners share). The sample
/// commits only at the exact clamped geometry the render enforces, so a
/// fast-but-wrong sample (wrong width or visibility) is rejected both by the
/// in-app pending check and by these assertions; socket transport is never
/// part of the measured duration.
pub(crate) fn benchmark_resize_sidebar(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    width: f32,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let geometry = client.snapshot().expect("resize geometry snapshot");
    let expected = width
        .clamp(geometry.sidebar_min, geometry.sidebar_max)
        .min((geometry.window_size[0] - geometry.content_min).max(0.0));
    client
        .send_ok(&Request::BenchmarkResizeSidebar {
            operation_id,
            width,
        })
        .expect("benchmark sidebar resize");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::SidebarResize, operation_id, *since);
    *since = seq;
    assert!(measured.sidebar_visible, "resize keeps the sidebar visible");
    assert!(
        (measured.content_rect[0] - expected).abs() <= 0.5,
        "resize sample committed at {:.1} pt, expected the {expected:.1} pt clamp",
        measured.content_rect[0]
    );
    let snapshot = client.snapshot().expect("resize snapshot");
    assert!(snapshot.sidebar_visible);
    assert!(
        (snapshot.sidebar_width - expected).abs() <= 0.5,
        "snapshot shows {:.1} pt, expected the {expected:.1} pt clamp",
        snapshot.sidebar_width
    );
    assert_eq!(
        snapshot.active_web_view, measured.active_web_view,
        "resize never changes the selected native view"
    );
    measured
}

/// Sidebar restoration: collapse through the production Cmd-B path
/// (unmeasured setup inside the app), then measure the expand that restores
/// the last expanded width. The restored width must equal the pre-collapse
/// rail width (DESIGN §11.4), and the measured sample must show it.
pub(crate) fn benchmark_restore_sidebar(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("pre-collapse snapshot");
    let collapsed_width = before.sidebar_width;
    assert!(before.sidebar_visible, "restore needs a visible sidebar");
    client
        .send_ok(&Request::BenchmarkRestoreSidebar { operation_id })
        .expect("benchmark sidebar restore");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::SidebarRestore, operation_id, *since);
    *since = seq;
    assert!(measured.sidebar_visible, "restore ends expanded");
    assert!(
        (measured.content_rect[0] - collapsed_width).abs() <= 0.5,
        "restored rail at {:.1} pt, expected the collapsed {collapsed_width:.1} pt",
        measured.content_rect[0]
    );
    let snapshot = client.snapshot().expect("restore snapshot");
    assert!(snapshot.sidebar_visible);
    assert!(
        (snapshot.sidebar_width - collapsed_width).abs() <= 0.5,
        "restored width {:.1} pt != collapsed {collapsed_width:.1} pt",
        snapshot.sidebar_width
    );
    assert_eq!(
        snapshot.active_web_view, measured.active_web_view,
        "restore never changes the selected native view"
    );
    measured
}

/// Visual pointer selection of a visible tab row through the production
/// listener path. Chrome focus must be preserved (the interaction contract's
/// pointer-select domain) and the measured presentation must show the
/// expected native view without recreating sessions.
pub(crate) fn benchmark_click(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkClickTab {
            operation_id,
            index,
        })
        .expect("benchmark pointer tab selection");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::TabSelectPointer,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "pointer selection keeps the chrome focus domain"
    );
    let snapshot = client.snapshot().expect("pointer selection snapshot");
    assert_eq!(snapshot.active_tab_index, Some(index));
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert!(snapshot.chrome_focused);
    assert_eq!(
        snapshot.engine_sessions, 2,
        "warm pointer select keeps sessions"
    );
    measured
}

/// Reopen of the most recently closed tab: fresh session presented with the
/// preserved URL while the invoking focus domain is untouched (chrome stays
/// chrome, web stays web — the interaction contract's reopen rule).
pub(crate) fn benchmark_reopen(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    neighbor_view: WebViewId,
    expected_chrome_focused: bool,
) -> (PresentationMeasurement, WebViewId) {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkReopenTab { operation_id })
        .expect("benchmark reopen");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::TabReopen, operation_id, *since);
    *since = seq;
    let fresh = measured
        .active_web_view
        .expect("reopen presents a fresh native session");
    assert_ne!(fresh, neighbor_view, "reopen creates a fresh incarnation");
    assert_eq!(
        measured.chrome_focused, expected_chrome_focused,
        "reopen preserves the invoking focus domain"
    );
    let snapshot = client.snapshot().expect("reopened snapshot");
    assert_eq!(snapshot.tabs.len(), 2);
    assert_eq!(snapshot.engine_sessions, 2);
    assert_eq!(snapshot.active_web_view, Some(fresh));
    assert_eq!(snapshot.chrome_focused, expected_chrome_focused);
    (measured, fresh)
}

/// Activation of a space whose selected tab is already warm: the production
/// activation path must present the destination space's own active tab with
/// the same native session (no recreation), keep the chrome focus domain
/// (activation routes through the space-list pointer path), and expose the
/// destination row as `selected` with its stable `bbr-space-row-<n>` id.
///
/// `expected_sessions` is the live-session count both spaces' tabs hold
/// together; the activation must never change it (warm switching only
/// show/hides).
pub(crate) fn benchmark_space_switch(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    space: TestSpaceId,
    expected_view: WebViewId,
    expected_sessions: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkActivateSpace {
            operation_id,
            space,
        })
        .expect("benchmark space activation");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::WarmSpaceSwitch, operation_id, *since);
    *since = seq;
    assert_eq!(
        measured.active_space, space,
        "measured presentation shows the destination space"
    );
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "space activation keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("space switch snapshot");
    assert_eq!(snapshot.active_space, space);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "warm space switching never recreates sessions"
    );
    assert!(snapshot.chrome_focused);
    // Rendered space identity (DESIGN.md §11.3/§12/§15): the destination
    // space row renders `selected` and its row/chip ids are present. The
    // snapshot's space rows are grouped by profile in render order; with one
    // profile the render order equals creation order, so the row position
    // equals the space's creation-order index.
    let active_index = snapshot
        .spaces
        .iter()
        .find(|s| s.space == space)
        .map(|s| s.index)
        .expect("destination space in collection");
    assert_eq!(
        snapshot.space_states[active_index],
        SpaceVisualState::Selected,
        "destination space row renders selected"
    );
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-space-row-{active_index}")),
        "destination space row id present"
    );
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == "bbr-profile-chip-0"),
        "profile chip id present"
    );
    measured
}
