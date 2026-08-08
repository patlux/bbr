//! Shared harness support and typed assertions for the BBR context-menu E2E
//! (task packet `bbr-context-menus-over-web`).
//!
//! The menu renders inside the one `NativeAboveWeb` portal window, so the
//! merged native-portal evidence contract applies verbatim: the `0xC4`
//! presentation id, the stable §15 ids (backdrop, card, five rows), child
//! attachment, parent-content frame sync, and the non-key/non-main policy.
//! The menu's open path is driven through the exact engine event transport
//! the native bridge uses (`Request::InjectWebEvent` with
//! `WebEvent::ContextMenuRequested`), so the suites exercise the production
//! event-driven open seam.
//!
//! All geometry mirrors the settled tokens the app resolves against
//! (`CONTEXT_MENU_WIDTH` = the §11.4 compact clamp 220 pt, `CONTEXT_MENU_MARGIN`
//! = `sp:2` 8 pt). No invented sizes.

use std::time::Instant;

use crate::e2e_common::*;
use bbr_testkit::protocol::Snapshot;
use bbr_web::{WebEvent, WebViewId};

/// The menu's presentation id (`PresentationId(0xC4)` in
/// `crate::app::context_menu`), exposed through the snapshot's typed
/// `portal.presentation` field.
pub(super) const MENU_PRESENTATION_ID: u64 = 0xC4;
/// The §11.4 command-surface compact clamp width (settled token).
pub(super) const MENU_WIDTH: f32 = 220.0;
/// The portal placement margin (`sp:2`, settled token).
pub(super) const MENU_MARGIN: f32 = 8.0;
/// The five deterministic row keys of the menu (render order).
pub(super) const MENU_ROW_KEYS: [&str; 5] =
    ["back", "forward", "reload", "copy-link", "open-new-tab"];
/// The stable §15 ids the menu renders (DESIGN.md §15; registered in
/// `crate::portal::probe::PortalProbeState::ids`).
pub(super) const MENU_IDS: [&str; 7] = [
    "bbr-context-menu-backdrop",
    "bbr-context-menu",
    "bbr-context-menu-row-0",
    "bbr-context-menu-row-1",
    "bbr-context-menu-row-2",
    "bbr-context-menu-row-3",
    "bbr-context-menu-row-4",
];

/// Dispatch one keystroke through GPUI's real input path (the same seam the
/// command/find suites use): `up`/`down`/`enter` reach the menu's key
/// handler while a menu presentation owns keyboard focus; `escape` routes
/// through the global `FocusWeb` binding that closes an open menu.
pub(super) fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

pub(super) fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// Open the menu through the exact engine event transport the native bridge
/// uses: inject `WebEvent::ContextMenuRequested(view, point)` and wait for
/// the typed open snapshot. `point` is in window content coordinates
/// (points, top-left origin) — the same space the portal placement math
/// uses.
pub(super) fn open_menu(client: &mut Client, view: WebViewId, point: [f32; 2]) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::InjectWebEvent {
            event: WebEvent::ContextMenuRequested(
                view,
                bbr_web::WebRect::new(point[0], point[1], 0.0, 0.0),
            ),
        })
        .expect("inject context menu request");
    wait_menu_open(client, view, point, deadline())
}

/// Poll until the menu presentation is open over `view` at `point` (the
/// injected event is applied by the event pump, so the snapshot can observe
/// the pre-open frame). Bounded by the deadline.
pub(super) fn wait_menu_open(
    client: &mut Client,
    view: WebViewId,
    point: [f32; 2],
    deadline: Instant,
) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("menu open snapshot");
        let m = &snapshot.context_menu;
        if m.open && m.bound_view == Some(view) && m.point == point {
            assert_menu_open(&snapshot, view, point);
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "context menu did not open over {view:?} at {point:?}; last open={} bound={:?} point={:?}",
            m.open,
            m.bound_view,
            m.point
        );
        std::thread::yield_now();
    }
}

/// Poll until the menu presentation is fully dismissed ([`assert_menu_closed`]
/// holds).
pub(super) fn wait_menu_closed(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("menu closed snapshot");
        if !snapshot.context_menu.open {
            assert_menu_closed(&snapshot);
            return snapshot;
        }
        assert!(Instant::now() < deadline, "context menu did not close");
        std::thread::yield_now();
    }
}

/// Assert the typed open menu (task packet evidence section): the `0xC4`
/// `NativeAboveWeb` presentation, the stable §15 ids, menu focus, the
/// deterministic row model bound to `view`, the armed pointer anchor, child
/// attachment, native visibility, parent-content frame sync, and the
/// non-key/non-main policy.
pub(super) fn assert_menu_open(snapshot: &Snapshot, view: WebViewId, point: [f32; 2]) {
    let m = &snapshot.context_menu;
    let p = &snapshot.portal;
    assert!(m.open, "menu presentation open");
    assert!(m.focused, "menu owns keyboard focus while open");
    assert_eq!(m.bound_view, Some(view), "menu bound to the active view");
    assert_eq!(
        m.point, point,
        "menu anchored at the injected pointer point"
    );
    assert_eq!(
        m.items.iter().map(|i| i.key.as_str()).collect::<Vec<_>>(),
        MENU_ROW_KEYS.to_vec(),
        "deterministic row model in render order"
    );
    assert_eq!(p.presentation, Some(MENU_PRESENTATION_ID));
    assert_eq!(p.host.as_deref(), Some("native-above-web"));
    assert!(p.visible);
    assert_eq!(
        p.ids,
        MENU_IDS.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );
    assert!(p.child_attached);
    assert!(p.native_visible, "AppKit window is ordered in");
    assert!(
        p.frame_synced,
        "portal frame pinned to the parent content rect"
    );
    assert!(
        p.nonactivating_panel,
        "the non-activating panel policy is applied natively"
    );
    assert!(
        p.portal_window_number > 0 && p.parent_window_number > 0,
        "both windows have AppKit numbers"
    );
    assert_ne!(
        p.portal_window_number, p.parent_window_number,
        "the menu is its own window, not the browser window"
    );
    assert!(!p.portal_is_key, "showing never makes the portal key");
    assert!(!p.portal_is_main, "showing never makes the portal main");
}

/// Assert the menu presentation is fully dismissed: `context_menu.open`
/// false and the one portal host back to its typed closed default (nothing
/// visible, no ids, still attached, never key/main) — the "no orphan"
/// contract after any dismissal.
pub(super) fn assert_menu_closed(snapshot: &Snapshot) {
    assert!(!snapshot.context_menu.open, "menu presentation closed");
    assert!(!snapshot.context_menu.focused, "menu focus released");
    assert_eq!(snapshot.context_menu.selected, None, "selection cleared");
    let p = &snapshot.portal;
    assert_eq!(p.presentation, None, "no active presentation");
    assert_eq!(p.host, None, "no active host");
    assert!(!p.visible, "logical visibility false while closed");
    assert_eq!(p.rect, None, "no resolved rect while closed");
    assert!(
        p.ids.is_empty(),
        "no bbr-context-menu-* ids while closed: {:?}",
        p.ids
    );
    assert!(
        p.child_attached,
        "host stays attached to the browser window"
    );
    assert!(
        !p.native_visible,
        "AppKit window is never shown while closed"
    );
    assert!(
        p.frame_synced,
        "frame stays pinned to the parent content rect"
    );
    assert!(!p.portal_is_key, "non-key policy holds");
    assert!(!p.portal_is_main, "non-main policy holds");
}

/// Assert that browser ownership and the active session identity are
/// structurally untouched by menu mutations (the merged portal suite's
/// invariant, minus the chrome-focus equality: the menu's focus
/// choreography deliberately restores the captured origin focus domain, so
/// a chrome-origin session legitimately ends chrome-focused; the origin
/// restoration is asserted separately per scenario).
pub(super) fn assert_identity_preserved(snapshot: &Snapshot, baseline: &Snapshot) {
    assert_eq!(
        snapshot.active_web_view, baseline.active_web_view,
        "menu mutations must never change the selected WebViewId"
    );
    assert_eq!(
        snapshot.engine_sessions, baseline.engine_sessions,
        "menu mutations must never create, destroy, or recreate sessions"
    );
    assert_eq!(
        snapshot.active_profile, baseline.active_profile,
        "menu mutations must never change the active profile"
    );
    assert_eq!(
        snapshot.active_space, baseline.active_space,
        "menu mutations must never change the active space"
    );
}

/// Wait until the fixture page finished loading (its title is known), the
/// same load-completion synchronization the merged suites use: the menu's
/// enablement must never race the page's late history events.
pub(super) fn wait_fixture_loaded(
    client: &mut Client,
    view: WebViewId,
    title: &str,
    since: u64,
) -> u64 {
    let snapshot = client.snapshot().expect("fixture title snapshot");
    if snapshot.title.as_deref() == Some(title) {
        return since;
    }
    wait_title(client, view, title, since)
}

/// Wait until the real WKWebView reported `can_go_back`/`can_go_forward`
/// for `view` through the real `CanGoBackChanged`/`CanGoForwardChanged`
/// events (the genuine enablement feed of the menu's Back/Forward rows).
pub(super) fn wait_history_state(
    client: &mut Client,
    view: WebViewId,
    can_back: bool,
    can_forward: bool,
    since: u64,
) -> u64 {
    let mut cursor = since;
    if can_back {
        cursor = wait_web_event(
            client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, true) if *id == view),
            cursor,
        );
    }
    if can_forward {
        cursor = wait_web_event(
            client,
            |e| matches!(e, WebEvent::CanGoForwardChanged(id, true) if *id == view),
            cursor,
        );
    }
    cursor
}

/// The menu row snapshot with `key` (panics when the row is missing).
pub(super) fn row<'a>(
    snapshot: &'a Snapshot,
    key: &str,
) -> &'a bbr_testkit::protocol::ContextMenuItemSnapshot {
    snapshot
        .context_menu
        .items
        .iter()
        .find(|item| item.key == key)
        .unwrap_or_else(|| panic!("menu row {key} missing: {:?}", snapshot.context_menu.items))
}
