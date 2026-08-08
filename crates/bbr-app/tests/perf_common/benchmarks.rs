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
    PresentationMeasurement, Request, TabVisualState, TestSpaceId,
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
/// (activation routes through the space-strip chip pointer path), and expose
/// the destination space's chip as `selected` with its stable
/// `bbr-space-chip-<persistent-space-id>` id.
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
    assert_eq!(
        snapshot.command_profile_hint, None,
        "same-profile switching must not announce a profile transition"
    );
    // Rendered space identity (DESIGN.md §11.3/§12/§15): the destination
    // space's chip renders `selected` and its persistent-space chip id is
    // present. The strip's display order is the canonical grouped order; the
    // chip id derives from the persistent `SpaceId`, never render position.
    let active_chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.space == space)
        .expect("destination chip in the strip");
    assert!(
        active_chip.selected,
        "destination space chip renders selected"
    );
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-space-chip-{}", space.0)),
        "destination space chip id present"
    );
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-space-identity-{}", space.0)),
        "destination space identity id present"
    );
    measured
}

/// Cross-profile view-only space activation through the production activation
/// path (DESIGN.md §11.3 cross-context rule): the destination profile appears
/// at the top of the command field for one interaction. The switch stays a
/// warm view-only presentation — the live-session count never changes — so
/// the sample shares the 50/100 ms warm budget, and the driver asserts the
/// hint copy in the accepted snapshot while the in-app probe rejects any
/// presentation that misses or invents it.
pub(crate) fn benchmark_space_switch_cross_profile(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    space: TestSpaceId,
    expected_view: WebViewId,
    expected_sessions: usize,
    expected_hint: Option<&str>,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkActivateSpace {
            operation_id,
            space,
        })
        .expect("benchmark cross-profile space activation");
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
    let snapshot = client.snapshot().expect("cross space switch snapshot");
    assert_eq!(snapshot.active_space, space);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "cross-profile view-only switching never recreates sessions"
    );
    assert!(snapshot.chrome_focused);
    assert_eq!(
        snapshot.command_profile_hint.as_deref(),
        expected_hint,
        "cross-profile activation shows the §11.3 destination hint"
    );
    if expected_hint.is_some() {
        assert!(
            snapshot
                .visible_ids
                .iter()
                .any(|id| id == "bbr-command-profile-hint"),
            "the command-field hint id renders for the cross-profile switch"
        );
    }
    // Rendered space identity (DESIGN.md §11.3/§12/§15): the destination
    // space's chip renders `selected` with its stable persistent-space chip
    // and identity ids.
    let active_chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.space == space)
        .expect("destination chip in the strip");
    assert!(
        active_chip.selected,
        "destination space chip renders selected"
    );
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-space-chip-{}", space.0)),
        "destination space chip id present"
    );
    measured
}

/// The production pin toggle of the active tab (`Cmd-Shift-P` path): the
/// pinned/regular boundary moves while the selected tab and its live session
/// stay. The sample must show the same active native view and the expected
/// pinned region boundary, with chrome focus preserved.
pub(crate) fn benchmark_pin_tab(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
    expected_pinned_after: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("pin before snapshot");
    assert_eq!(
        before.pinned_count,
        1 - expected_pinned_after,
        "toggle alternates"
    );
    client
        .send_ok(&Request::BenchmarkPinTab { operation_id })
        .expect("benchmark pin toggle");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::TabPin, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "the pin command keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("pin snapshot");
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(snapshot.pinned_count, expected_pinned_after);
    assert!(snapshot.chrome_focused);
    measured
}

/// The production region-bounded reorder of the active tab by `delta` rows
/// (the `Cmd-Shift-↑/↓` path): the row order changes while the selected tab,
/// its session, and the region boundary stay. The sample must show the same
/// active native view and the boundary untouched.
pub(crate) fn benchmark_reorder_tab(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    delta: isize,
    expected_view: WebViewId,
    expected_pinned: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkReorderTab {
            operation_id,
            delta,
        })
        .expect("benchmark reorder");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::TabReorder, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "the reorder command keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("reorder snapshot");
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.pinned_count, expected_pinned,
        "region boundary intact"
    );
    assert!(snapshot.chrome_focused);
    measured
}

/// The production move of the active tab into `to_space` through the staged
/// command path. The sample must present the source space's repaired
/// selection (the active space stays the source) and keep the chrome focus
/// domain. `expected_recreated` asserts the wire event's recreation flag:
/// same-profile moves keep the live session, cross-profile moves destroy and
/// recreate it once in the destination store.
pub(crate) fn benchmark_move_tab(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    to_space: TestSpaceId,
    source_repair_view: WebViewId,
    expected_recreated: bool,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let cursor = *since;
    client
        .send_ok(&Request::BenchmarkMoveTab {
            operation_id,
            to_space,
        })
        .expect("benchmark move");
    let kind = if expected_recreated {
        BenchmarkKind::TabMoveCrossProfile
    } else {
        BenchmarkKind::TabMoveSameProfile
    };
    // The `TabMoved` wire event is emitted synchronously with the commit, so
    // both it and the presentation sample are awaited from the pre-request
    // cursor; the recreation flag is part of the contract assertion.
    let (seq_change, change) = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::TabMoved { to_space: space, .. } if *space == to_space),
        cursor,
    );
    let (seq_meas, measured) = wait_measurement(client, kind, operation_id, cursor);
    *since = seq_meas.max(seq_change);
    assert_eq!(measured.active_web_view, Some(source_repair_view));
    assert!(
        measured.chrome_focused,
        "the move command keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    match change {
        BrowserChange::TabMoved { recreated, .. } => {
            assert_eq!(recreated, expected_recreated);
        }
        other => panic!("unexpected change: {other:?}"),
    }
    let snapshot = client.snapshot().expect("move snapshot");
    assert_eq!(snapshot.active_web_view, Some(source_repair_view));
    assert!(snapshot.chrome_focused);
    measured
}

/// Pointer hover entry on a visible tab row through the production `on_hover`
/// listener path (`BenchmarkHoverTab` → `set_row_hover`, the exact app
/// method the row's element handler calls): the §10.2 hover wash + close
/// reveal must be present in the accepted sample's render state. The hover
/// never touches selection or sessions (transient indication).
pub(crate) fn benchmark_hover(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("hover before snapshot");
    let sessions_before = before.engine_sessions;
    client
        .send_ok(&Request::BenchmarkHoverTab {
            operation_id,
            index,
        })
        .expect("benchmark hover");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::TabHover, operation_id, *since);
    *since = seq;
    let snapshot = client.snapshot().expect("hover snapshot");
    assert_eq!(
        snapshot.tab_states.get(index),
        Some(&TabVisualState::Hover),
        "hover renders the §10.2 wash on the hovered row"
    );
    assert_eq!(
        snapshot.engine_sessions, sessions_before,
        "hovering never touches the session population"
    );
    measured
}

/// Pointer press on a visible tab row through the production mouse-down
/// listener path (`BenchmarkPressTab` → `set_row_pressed`): the §10.3
/// active wash must be present in the accepted sample's render state.
/// Pressing is a chrome-pointer state, not a selection mutation — selection
/// and sessions stay untouched.
pub(crate) fn benchmark_press(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("press before snapshot");
    let sessions_before = before.engine_sessions;
    let active_before = before.active_tab_index;
    client
        .send_ok(&Request::BenchmarkPressTab {
            operation_id,
            index,
        })
        .expect("benchmark press");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::TabPress, operation_id, *since);
    *since = seq;
    let snapshot = client.snapshot().expect("press snapshot");
    assert_eq!(
        snapshot.tab_states.get(index),
        Some(&TabVisualState::Active),
        "press renders the §10.3 active wash on the pressed row"
    );
    assert_eq!(
        snapshot.active_tab_index, active_before,
        "the press state alone never selects"
    );
    assert_eq!(
        snapshot.engine_sessions, sessions_before,
        "pressing never touches the session population"
    );
    measured
}

/// A dense-list scroll through the production scroll state
/// (`BenchmarkDenseScroll`): the tracked offset the
/// native wheel listener mutates is moved by `delta_px`, the resulting frame
/// is drawn through the production path, and the sample commits only when
/// the render actually moved the deterministic scroll anchor (a no-op at the
/// collection edges cancels instead of committing fast-but-wrong). The
/// rendered window must stay bounded (`rendered_rows ≤ 64`) and the pinned
/// block must survive; scrolling never touches the session population.
pub(crate) fn benchmark_dense_scroll(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    delta_px: f32,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("dense scroll before snapshot");
    let anchor_before = before.dense.scroll_anchor;
    let sessions_before = before.engine_sessions;
    client
        .send_ok(&Request::BenchmarkDenseScroll {
            operation_id,
            delta_px,
        })
        .expect("benchmark dense scroll");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::DenseScroll, operation_id, *since);
    *since = seq;
    let snapshot = client.snapshot().expect("dense scroll snapshot");
    assert_ne!(
        snapshot.dense.scroll_anchor, anchor_before,
        "the measured scroll moved the deterministic anchor"
    );
    assert!(
        snapshot.dense.rendered_rows <= 64,
        "dense render stays bounded after scroll ({} rows)",
        snapshot.dense.rendered_rows
    );
    assert_eq!(
        snapshot.engine_sessions, sessions_before,
        "scrolling the dense list never touches sessions"
    );
    assert!(
        snapshot.visible_ids.iter().any(|id| id == "bbr-tab-row-0"),
        "the pinned block stays rendered above the scroll view"
    );
    measured
}

/// A production selection inside the dense collection
/// (`BenchmarkDenseSelect`): the pointer row path selects `index`, the frame
/// that renders the selection is drawn through the production path, and the
/// sample commits only when the selected row lies inside the bounded window
/// the render actually built (pinned rows are always on screen; a regular
/// row outside the window cancels — the instant scroll-into-view must have
/// run). `expected_view`/`expected_sessions` assert the warm-selection
/// contract: the existing session is reused and the live population is
/// untouched.
pub(crate) fn benchmark_dense_select(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
    expected_view: WebViewId,
    expected_sessions: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDenseSelect {
            operation_id,
            index,
        })
        .expect("benchmark dense select");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::DenseSelect, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "pointer selection keeps the chrome focus domain"
    );
    let snapshot = client.snapshot().expect("dense select snapshot");
    assert_eq!(snapshot.active_tab_index, Some(index));
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert!(
        snapshot.dense.active_visible,
        "the selected row is inside the rendered window"
    );
    assert!(
        snapshot.dense.rendered_rows <= 64,
        "dense render stays bounded after selection"
    );
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "warm dense selection never recreates sessions"
    );
    measured
}
