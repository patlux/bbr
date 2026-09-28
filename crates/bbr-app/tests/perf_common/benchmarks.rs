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
        // The §2 content card's trailing `sp:content-inset` is charged to
        // the native slot, not to the rail.
        .min((geometry.window_size[0] - geometry.content_min - 10.0).max(0.0));
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
/// D4 fade path (v2 S7): the measured switch committed first (the caller
/// checked the destination presentation); when the chrome tint changed under
/// full motion, exactly one `space-tint-fade` decoration started after that
/// commit — the fade never gates the measured presentation.
fn assert_fade_path(
    before: &bbr_testkit::protocol::Snapshot,
    after: &bbr_testkit::protocol::Snapshot,
) {
    let tint = |snapshot: &bbr_testkit::protocol::Snapshot| {
        snapshot
            .active_appearance
            .as_ref()
            .map(|appearance| appearance.chrome_tint.clone())
    };
    let fades = |snapshot: &bbr_testkit::protocol::Snapshot| {
        snapshot
            .motion
            .started
            .iter()
            .filter(|start| start.moment == "space-tint-fade")
            .count()
    };
    if tint(before) != tint(after) && !after.shell_modes.reduced_motion {
        let fade = after
            .motion
            .started
            .iter()
            .rev()
            .find(|start| start.moment == "space-tint-fade")
            .expect("a tint change starts the D4 fade");
        assert_eq!(fade.variant, "animated");
        assert!(
            fades(after) > fades(before) || fades(before) >= 16,
            "one new fade per committed tint change"
        );
    }
}

pub(crate) fn benchmark_space_switch(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    space: TestSpaceId,
    expected_view: WebViewId,
    expected_sessions: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    // D4 fade path (v2 S7): read outside the timed region.
    let before = client.snapshot().expect("pre-switch snapshot");
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
    assert_fade_path(&before, &snapshot);
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

/// A §11.3 sidebar swipe commit measured through the real production gesture
/// path (task packet `space-footer-strip-appearance-ui`, `swipe_commit`
/// performance layer): the app drives `Started`/`Moved` wheel updates plus a
/// zero-delta `TouchPhase::Ended` through the exact handler the sidebar's
/// `on_scroll_wheel` listener calls, and the ended release resolves the
/// tracking gesture into exactly one existing `activate_space` commit of the
/// adjacent persistent `source` → `destination` pair (never a direct
/// activation, never a benchmark-only state machine). Same warm class as
/// `benchmark_space_switch`: the live-session count must never change, and
/// the sample commits only when the exact destination space, its warm
/// selected view, the Idle machine phase, and the source-deselected /
/// destination-selected chip states are presented (fast-but-wrong guard).
pub(crate) fn benchmark_swipe_commit(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    source: TestSpaceId,
    destination: TestSpaceId,
    expected_view: WebViewId,
    expected_sessions: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    assert_ne!(
        source, destination,
        "every sample is a real adjacent commit"
    );
    client
        .send_ok(&Request::BenchmarkSwipeCommit {
            operation_id,
            destination,
        })
        .expect("benchmark swipe commit");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::SwipeCommit, operation_id, *since);
    *since = seq;
    assert_eq!(
        measured.active_space, destination,
        "measured presentation shows the destination space"
    );
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "swipe commit keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    assert!(
        measured.content_rect[2] > 0.0 && measured.content_rect[3] > 0.0,
        "swipe commit keeps non-zero native content geometry"
    );
    let snapshot = client.snapshot().expect("swipe commit snapshot");
    assert_eq!(snapshot.active_space, destination);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "warm swipe switching never recreates sessions"
    );
    assert!(snapshot.chrome_focused);
    assert_eq!(
        snapshot.command_profile_hint, None,
        "same-profile swipes must not announce a profile transition"
    );
    // The machine returned Idle after the zero-delta Ended: a resolved
    // gesture leaves no lingering source/destination (a repeated Ended can
    // never double-commit, and a stale tracking machine is rejected here).
    assert_eq!(snapshot.swipe.phase, "idle", "gesture Idle after Ended");
    assert_eq!(snapshot.swipe.source, None);
    assert_eq!(snapshot.swipe.destination, None);
    assert_eq!(snapshot.swipe.progress, 0.0);
    // Exact persistent chip states: the destination chip renders selected,
    // the source chip is deselected, and the stable §11.3 chip/identity ids
    // of the destination are present in the render.
    let destination_chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.space == destination)
        .expect("destination chip in the strip");
    assert!(
        destination_chip.selected,
        "destination space chip renders selected"
    );
    let source_chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.space == source)
        .expect("source chip in the strip");
    assert!(
        !source_chip.selected,
        "source space chip is deselected after the commit"
    );
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-space-chip-{}", destination.0)),
        "destination space chip id present"
    );
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-space-identity-{}", destination.0)),
        "destination space identity id present"
    );
    measured
}

/// Wait until the bound view's real WebKit history availability matches
/// `(can_back, can_forward)` (the `CanGoBackChanged`/`CanGoForwardChanged`
/// engine events). The definitive "correct history state presented"
/// milestone for the N1 gesture scenarios: a gesture that never navigated
/// produces no state change and this wait fails loudly instead of accepting
/// a fast-but-wrong sample.
pub(crate) fn wait_history_events(
    client: &mut Client,
    view: WebViewId,
    can_back: bool,
    can_forward: bool,
    since: u64,
) -> u64 {
    let mut cursor = since;
    if can_back {
        cursor = client
            .wait_for_event_since(
                |event| {
                    matches!(event, Event::Web(WebEvent::CanGoBackChanged(id, true)) if *id == view)
                },
                cursor,
                deadline(),
            )
            .expect("can_go_back event")
            .0;
    }
    if can_forward {
        cursor = client
            .wait_for_event_since(
                |event| {
                    matches!(event, Event::Web(WebEvent::CanGoForwardChanged(id, true)) if *id == view)
                },
                cursor,
                deadline(),
            )
            .expect("can_go_forward event")
            .0;
    }
    if !can_back {
        cursor = client
            .wait_for_event_since(
                |event| {
                    matches!(event, Event::Web(WebEvent::CanGoBackChanged(id, false)) if *id == view)
                },
                cursor,
                deadline(),
            )
            .expect("can_go_back false event")
            .0;
    }
    if !can_forward {
        cursor = client
            .wait_for_event_since(
                |event| {
                    matches!(event, Event::Web(WebEvent::CanGoForwardChanged(id, false)) if *id == view)
                },
                cursor,
                deadline(),
            )
            .expect("can_go_forward false event")
            .0;
    }
    cursor
}

/// The §11.3 create→activate→configure lifecycle (packet
/// `space-creation-appearance-ui.md`, `space_create_present` scenario): the
/// production `commit_new_space` path creates and activates a fresh Space
/// and opens its optional name/icon configuration surface. The sample
/// commits only from a render that drew the surface open bound to the exact
/// new persistent SpaceId (the in-app render-gated guard); the driver then
/// re-verifies the typed snapshot: the created Space is active, its chip
/// renders `selected` with the stable persistent id, the configuration
/// surface is bound to the exact created id, and the live-session count is
/// unchanged (a fresh Space is empty — no session is created). Returns the
/// measurement plus the created SpaceId for the caller's bookkeeping.
pub(crate) fn benchmark_space_create(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    name: &str,
    expected_sessions: usize,
) -> (PresentationMeasurement, TestSpaceId) {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkCreateSpace {
            operation_id,
            name: name.into(),
        })
        .expect("benchmark space create");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::SpaceCreatePresent,
        operation_id,
        *since,
    );
    *since = seq;
    let snapshot = client.snapshot().expect("space create snapshot");
    let created = snapshot.active_space;
    assert_eq!(
        measured.active_space, created,
        "measured presentation shows the created space"
    );
    assert_eq!(
        measured.space_config_space,
        Some(created),
        "the measured surface is bound to the exact created SpaceId"
    );
    assert!(
        measured.chrome_focused,
        "the create presentation keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    // The typed snapshot agrees: the created space is active, its chip is
    // selected with the persistent id, the config surface is bound to the
    // exact created id, and no session was created.
    assert_eq!(snapshot.active_space, created);
    let surface = snapshot
        .space_config
        .as_ref()
        .expect("configuration surface open");
    assert_eq!(surface.space, created, "surface bound to the created id");
    let chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.space == created)
        .expect("created chip in the strip");
    assert!(chip.selected, "the created space's chip renders selected");
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-space-chip-{}", created.0)),
        "created chip id present"
    );
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "creating a space never creates a session"
    );
    (measured, created)
}

/// The §11.3 identity commit for the exact bound Space (packet
/// `space-creation-appearance-ui.md`, `space_config_identity_update`
/// scenario): the production `space_config_done` authority commits the
/// curated identity/theme/rename for the exact bound persistent SpaceId and
/// the surface closes. The sample commits only from a render that drew the
/// surface closed with the bound space still active (the in-app render-gated
/// guard); the driver then re-verifies the typed snapshot: the bound chip
/// carries the committed identity and stays selected, no sibling chip
/// changed, no session was created, and the surface is gone on the wire.
pub(crate) fn benchmark_space_config_identity_update(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    space: TestSpaceId,
    identity: &str,
    expected_identity: &str,
    expected_sessions: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkSpaceConfigIdentityUpdate {
            operation_id,
            space,
            identity: identity.into(),
        })
        .expect("benchmark space config identity update");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::SpaceConfigIdentityUpdate,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_space, space,
        "the bound space stays active after the commit"
    );
    assert_eq!(
        measured.space_config_space, None,
        "the measured surface is drawn closed"
    );
    assert!(
        measured.chrome_focused,
        "the commit returns to the strip chrome domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("space config commit snapshot");
    assert_eq!(
        snapshot.space_config, None,
        "the surface is gone on the wire"
    );
    let chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.space == space)
        .expect("bound chip in the strip");
    assert_eq!(
        chip.identity, expected_identity,
        "the bound chip carries the committed identity"
    );
    assert!(
        chip.selected,
        "the bound chip stays selected (active space)"
    );
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "the identity commit never touches sessions"
    );
    measured
}

/// A committed logical Back content gesture (N1 packet
/// `web-content-navigation-gestures`, `web_history_swipe_back` scenario):
/// the app dispatches one Back through the exact production gesture method
/// against the bound `view` of a deterministic already-loaded two-entry
/// history. The sample is accepted only after the correct URL/history state
/// is presented: the bound view's title returns to `previous_title` and the
/// real WebKit availability reports `can_go_back == false` /
/// `can_go_forward == true` (the two-entry history is at its root, forward
/// history intact). The same view stays presented; tabs and sessions never
/// change (fast-but-wrong guard).
pub(crate) fn benchmark_web_history_swipe_back(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    view: WebViewId,
    previous_title: &str,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkWebHistorySwipeBack { operation_id, view })
        .expect("benchmark web history swipe back");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::WebHistorySwipeBack,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(view),
        "history swipe keeps the exact bound view presented"
    );
    // The real WebKit state milestone: title of the previous entry, then the
    // history availability flip (the same event order the E2E suite waits).
    *since = wait_title(client, view, previous_title, *since);
    *since = wait_history_events(client, view, false, true, *since);
    let snapshot = client.snapshot().expect("swipe-back snapshot");
    assert_eq!(snapshot.active_web_view, Some(view));
    assert_eq!(snapshot.title.as_deref(), Some(previous_title));
    assert!(!snapshot.can_go_back, "two-entry history is at its root");
    assert!(snapshot.can_go_forward, "forward history remains");
    measured
}

/// A committed Back-at-settled-child-root opener return (N1 packet
/// `web-content-navigation-gestures`, `child_tab_swipe_return` scenario):
/// the app dispatches one Back through the exact production gesture method
/// against the settled child root. The sample is accepted only after the
/// correct state is presented: the exact warm opener's view is the active
/// presented view, the child session is gone (sessions drop by exactly
/// `sessions_after`), the child tab closed, and no opener relation remains.
pub(crate) fn benchmark_child_tab_swipe_return(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    child_view: WebViewId,
    opener_view: WebViewId,
    sessions_after: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let sessions_before = client
        .snapshot()
        .expect("child-tab pre-return snapshot")
        .engine_sessions;
    client
        .send_ok(&Request::BenchmarkChildTabSwipeReturn {
            operation_id,
            child_view,
        })
        .expect("benchmark child tab swipe return");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::ChildTabSwipeReturn,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(opener_view),
        "the exact warm opener view is presented after the return"
    );
    let snapshot = client.snapshot().expect("child-tab return snapshot");
    assert_eq!(
        snapshot.active_web_view,
        Some(opener_view),
        "opener presented by identity"
    );
    // The documented contract is a delta: exactly the child's session is
    // destroyed and every other session (the warm opener, other warm tabs)
    // survives. An absolute count would depend on unrelated warm tabs the
    // matrix opened earlier.
    assert_eq!(
        sessions_before - snapshot.engine_sessions,
        sessions_after,
        "child session destroyed, opener session reused warm"
    );
    assert!(
        snapshot.content_gesture.opener_relations.is_empty(),
        "the closed child's relation is invalidated"
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

/// The production drag-to-chip move of the tab at `from_index` into
/// `to_space`: the concrete row is pressed, the pointer is moved onto the
/// destination's rendered chip (the exact production chip-hit geometry), and
/// the release commits through the drag drop → the existing move authority
/// (`BenchmarkDragMoveTab`). The sample must present the source space's
/// repaired selection (the active space stays the source) and keep the
/// chrome focus domain. `expected_recreated` asserts the wire event's
/// recreation flag: same-profile moves keep the live session, cross-profile
/// moves destroy and recreate it once in the destination store. Same typed
/// kinds/report keys/budgets as the direct move; only the measured
/// production entry is the drag-to-chip path.
pub(crate) fn benchmark_move_tab(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    from_index: usize,
    to_space: TestSpaceId,
    source_repair_view: WebViewId,
    expected_recreated: bool,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let cursor = *since;
    client
        .send_ok(&Request::BenchmarkDragMoveTab {
            operation_id,
            from_index,
            to_space,
        })
        .expect("benchmark drag move");
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
