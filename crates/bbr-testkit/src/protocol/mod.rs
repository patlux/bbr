//! Protocol types shared by the gateway server and the E2E client.
//!
//! All types are plain data with `serde` derives so they round-trip through
//! JSON without leaking any framework type onto the wire. `protocol_version`
//! is part of the handshake; bump it when a type here changes in a breaking
//! way (see `docs/e2e-protocol.md` in E2E_ARCHITECTURE.md).
//!
//! The wire types are split by concern so no module outgrows the ~500-line
//! budget:
//!
//! - [`snapshot`] — the typed `Snapshot` and its row/space/profile parts
//!   plus the §15 visual-state vocabulary.
//! - [`benchmark`] — the benchmark kinds and presentation measurements.
//! - [`request`] — the gateway request enum.
//! - [`event`] — the semantic event stream.
//! - [`dsl`] — context-free request helpers for harness code.
//!
//! Everything callers reference keeps the single `bbr_testkit::protocol::*`
//! path through the re-exports below (additive protocol growth stays on the
//! same wire version).

use serde::{Deserialize, Serialize};

mod benchmark;
pub mod dsl;
mod event;
mod history;
mod media;
mod permissions;
mod request;
mod snapshot;
mod split;

pub use benchmark::{BenchmarkKind, PresentationMeasurement};
pub use event::{BrowserChange, Event};
pub use history::{
    ClearDataSnapshot, ClearOptionSnapshot, HistoryGroupSnapshot, HistoryInput, HistoryRowSnapshot,
    HistorySnapshot,
};
pub use media::{MediaCardSnapshot, MediaInput, MediaSessionEntry, MediaSnapshot};
pub use permissions::{
    ClientCertificateInput, ClientCertificateSnapshot, ClientCertificatesSnapshot,
    OnboardingSnapshot, OpenAppInput, OpenAppSheetSnapshot, OpenAppSnapshot, PermissionInput,
    PermissionPromptSnapshot, PermissionsSnapshot, SiteSettingsSnapshot, WelcomeInput,
};
pub use request::{Request, SavedPagesLibraryCollectionOp, SettingsInput, TextInputOp};
pub use snapshot::{
    AddressRestFill, AuthSnapshot, ChipCompanionSnapshot, CommandInputSnapshot,
    CommandResultSnapshot, CommandResultState, CommandSurface, ContentCardSnapshot,
    ContentGestureSnapshot, ContextMenuItemSnapshot, ContextMenuNativeSnapshot,
    ContextMenuSnapshot, DenseListSnapshot, DownloadGroup, DownloadMachineState, DownloadRowAction,
    DownloadRowSnapshot, DownloadsPanelSnapshot, DragOffersSnapshot, ErrorSurfaceCategory,
    ErrorSurfaceSnapshot, FavoriteTileSnapshot, FindSnapshot, FloatingRailSnapshot,
    FolderRowSnapshot, GlobalTabSnapshot, IllustrationSnapshot, LifecycleCounts, MenuItemSnapshot,
    MenuSnapshot, MotionMomentSnapshot, MotionSnapshot, NavHistorySnapshot, NewWindowSnapshot,
    PaneLayoutSnapshot, PaneRectSnapshot, PeekFrameSnapshot, PortalSnapshot, PreviewSnapshot,
    PreviewZOrder, ProfileSnapshot, ReloadRestSurface, RestoreDisposition,
    SavedPagesCollectionSnapshot, SavedPagesLibraryRowSnapshot, SavedPagesLibrarySnapshot,
    SemanticNode, SemanticRole, SettingsRowSnapshot, SettingsSheetSnapshot, SettingsSnapshot,
    Snapshot, SpaceAppearanceSnapshot, SpaceChipSnapshot, SpaceConfigSnapshot, SpaceDeleteSnapshot,
    SpaceSnapshot, SpaceStripScrollSnapshot, StoreDiscardReasonSnapshot, StoreLoadSnapshot,
    StoreSnapshot, SwipeSnapshot, TabMenuRowSnapshot, TabMenuSnapshot, TabSnapshot, TabVisualState,
    TextInputReadback, TitlebarSnapshot, ToastSnapshot, WebContentProbe, ZoomSnapshot,
};
pub use split::{SplitInput, SplitPaneSnapshot, SplitSnapshot};

/// Current protocol version understood by this crate.
pub const PROTOCOL_VERSION: u32 = 7;

/// The GPUI `TouchPhase` of one synthetic swipe event on the wire
/// (DESIGN.md §11.3 swipe): `started` begins a tracking gesture, `moved`
/// feeds progress, `ended` commits or cancels. The protocol carries its own
/// vocabulary so the wire never leaks the GPUI type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwipePhaseWire {
    Started,
    Moved,
    Ended,
}

/// Observable command-bar intent, independent from native focus ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandBarMode {
    Inactive,
    Location,
    NewTab,
    /// `Cmd-Shift-N`: stage a new space name in the active profile.
    NewSpace,
    /// `Cmd-Shift-M`: stage a move of the active tab into a destination
    /// space (typed by name); Enter commits, Escape cancels.
    MoveTab,
}

/// The deterministic appearance override on the wire (task packet
/// `system-appearance-theme`): light/dark, mapped to the production
/// `bbr_ui::theme::ThemeMode` in `bbr-app`. The wire keeps its own
/// vocabulary so no GPUI or bbr-ui type leaks into the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeModeOverride {
    Light,
    Dark,
}

/// The shell's resolved §15 mode fields (theme/contrast/density/motion).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShellModes {
    pub theme: String,
    pub contrast: String,
    pub density: String,
    /// §15 `reduced_motion`: the effective macOS Reduce Motion setting
    /// (task packet `v2-system-prefs`). An older payload without the field
    /// deserializes to `false`.
    #[serde(default)]
    pub reduced_motion: bool,
}

/// Stable space identity on the portable test wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSpaceId(pub u64);

/// Harness-assigned identity for one measured production-path mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkOperationId(pub u64);

// ---------------------------------------------------------------------------
// Indirection layer: bbr-web already carries WebViewId/ProfileId/WebAction
// with serde support; re-export the ids used by harnesses so they don't
// need to reach into bbr-web themselves for constructing requests.
// ---------------------------------------------------------------------------

pub use bbr_web::{ProfileId as TestProfileId, WebViewId as TestWebViewId};

/// Structured error payload. Kept as data (not `anyhow`) so the client can
/// match on it without importing the app's error types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Error {
    pub code: String,
    pub message: String,
}

impl Error {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

// The `Snapshot` payload grew with the §15 mode fields and per-row states;
// boxing it would churn the wire format for a lint that only measures enum
// size on the harness side.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// Reply to [`Request::Hello`].
    Welcome {
        protocol_version: u32,
        app_version: String,
        pid: u32,
        capabilities: Vec<String>,
        next_seq: u64,
    },
    /// Plain success.
    Ok,
    /// Reply to [`Request::Attach`].
    Replay {
        events: Vec<(u64, Event)>,
        next_seq: u64,
    },
    /// Reply to [`Request::GetSnapshot`].
    Snapshot(Snapshot),
    /// Reply to [`Request::SavedPageAvailability`] (B2 packet
    /// `saved-pages-actions-command`): the availability of `Bookmark Page`
    /// and `Remove Bookmark` for the probed URL through the one app-owned
    /// command authority, without mutating source state.
    SavedPageAvailability { can_save: bool, can_remove: bool },
    /// Failure carrying a human-readable reason.
    Error(Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_web::WebViewId;
    use serde_json::json;

    /// The split requests keep their exact externally-tagged JSON shape on
    /// the wire (no framework types, no renamed fields), and the fixed
    /// literals deserialize back into the same typed values.
    #[test]
    fn split_requests_have_exact_wire_shape() {
        let set = Request::SetSplit {
            primary: 0,
            secondary: 1,
        };
        assert_eq!(
            serde_json::to_value(&set).unwrap(),
            json!({ "SetSplit": { "primary": 0, "secondary": 1 } })
        );
        assert_eq!(
            serde_json::from_str::<Request>(r#"{"SetSplit":{"primary":0,"secondary":1}}"#).unwrap(),
            set
        );
        // A unit variant serializes as the bare variant name.
        assert_eq!(
            serde_json::to_value(Request::CloseSplit).unwrap(),
            json!("CloseSplit")
        );
        assert_eq!(
            serde_json::from_str::<Request>("\"CloseSplit\"").unwrap(),
            Request::CloseSplit
        );
    }

    /// The Saved Pages Library pointer-rename requests keep their exact
    /// externally-tagged JSON shape on the wire (the B3 pointer rename
    /// flow: begin → draft → confirm/cancel), and the fixed literals
    /// deserialize back into the same typed values.
    #[test]
    fn library_rename_requests_have_exact_wire_shape() {
        let begin = Request::BeginSavedPagesLibraryRowRename { id: 7 };
        assert_eq!(
            serde_json::to_value(&begin).unwrap(),
            json!({ "BeginSavedPagesLibraryRowRename": { "id": 7 } })
        );
        assert_eq!(
            serde_json::from_str::<Request>(r#"{"BeginSavedPagesLibraryRowRename":{"id":7}}"#)
                .unwrap(),
            begin
        );
        let draft = Request::SetSavedPagesLibraryRenameDraft {
            text: "Work".into(),
        };
        assert_eq!(
            serde_json::to_value(&draft).unwrap(),
            json!({ "SetSavedPagesLibraryRenameDraft": { "text": "Work" } })
        );
        // The unit variants serialize as the bare variant name.
        assert_eq!(
            serde_json::to_value(Request::ConfirmSavedPagesLibraryRename).unwrap(),
            json!("ConfirmSavedPagesLibraryRename")
        );
        assert_eq!(
            serde_json::from_str::<Request>("\"ConfirmSavedPagesLibraryRename\"").unwrap(),
            Request::ConfirmSavedPagesLibraryRename
        );
        assert_eq!(
            serde_json::to_value(Request::CancelSavedPagesLibraryRename).unwrap(),
            json!("CancelSavedPagesLibraryRename")
        );
        assert_eq!(
            serde_json::from_str::<Request>("\"CancelSavedPagesLibraryRename\"").unwrap(),
            Request::CancelSavedPagesLibraryRename
        );
    }

    /// The split outcomes keep their exact externally-tagged JSON shape on
    /// the semantic event stream, and the fixed literals deserialize back
    /// into the same typed values.
    #[test]
    fn split_events_have_exact_wire_shape() {
        let opened = BrowserChange::SplitOpened {
            primary: 0,
            secondary: 1,
        };
        assert_eq!(
            serde_json::to_value(&opened).unwrap(),
            json!({ "SplitOpened": { "primary": 0, "secondary": 1 } })
        );
        assert_eq!(
            serde_json::from_str::<BrowserChange>(r#"{"SplitOpened":{"primary":0,"secondary":1}}"#)
                .unwrap(),
            opened
        );
        assert_eq!(
            serde_json::to_value(BrowserChange::SplitClosed).unwrap(),
            json!("SplitClosed")
        );
        assert_eq!(
            serde_json::from_str::<BrowserChange>("\"SplitClosed\"").unwrap(),
            BrowserChange::SplitClosed
        );
    }

    /// The keyed pane evidence types keep their exact plain-value JSON shape
    /// (numeric `WebViewId`s, `null` secondary, plain rect arrays) and the
    /// fixed literals deserialize back into the same typed values.
    #[test]
    fn pane_geometry_snapshots_have_exact_wire_shape() {
        let split_layout = PaneLayoutSnapshot {
            primary: WebViewId(1),
            secondary: Some(WebViewId(2)),
            focused: WebViewId(2),
        };
        assert_eq!(
            serde_json::to_value(split_layout).unwrap(),
            json!({ "primary": 1, "secondary": 2, "focused": 2 })
        );
        assert_eq!(
            serde_json::from_str::<PaneLayoutSnapshot>(
                r#"{"primary":1,"secondary":2,"focused":2}"#
            )
            .unwrap(),
            split_layout
        );
        let single_layout = PaneLayoutSnapshot {
            primary: WebViewId(7),
            secondary: None,
            focused: WebViewId(7),
        };
        assert_eq!(
            serde_json::to_value(single_layout).unwrap(),
            json!({ "primary": 7, "secondary": null, "focused": 7 })
        );
        assert_eq!(
            serde_json::from_str::<PaneLayoutSnapshot>(
                r#"{"primary":7,"secondary":null,"focused":7}"#
            )
            .unwrap(),
            single_layout
        );
        let pane = PaneRectSnapshot {
            view: WebViewId(1),
            rect: [260.0, 0.0, 400.0, 700.0],
        };
        assert_eq!(
            serde_json::to_value(pane).unwrap(),
            json!({ "view": 1, "rect": [260.0, 0.0, 400.0, 700.0] })
        );
        assert_eq!(
            serde_json::from_str::<PaneRectSnapshot>(
                r#"{"view":1,"rect":[260.0,0.0,400.0,700.0]}"#
            )
            .unwrap(),
            pane
        );
    }

    /// The split shared contract rides the existing wire version (additive
    /// growth): a full snapshot carrying keyed pane evidence still
    /// round-trips through the real JSON framing. Exact per-type shapes are
    /// asserted above; this is the whole-envelope evidence.
    #[test]
    fn split_snapshot_round_trips_with_keyed_pane_evidence() {
        let rects = vec![
            PaneRectSnapshot {
                view: WebViewId(1),
                rect: [260.0, 0.0, 400.0, 700.0],
            },
            PaneRectSnapshot {
                view: WebViewId(2),
                rect: [660.0, 0.0, 400.0, 700.0],
            },
        ];
        let layout = PaneLayoutSnapshot {
            primary: WebViewId(1),
            secondary: Some(WebViewId(2)),
            focused: WebViewId(2),
        };
        let snapshot = Snapshot {
            pane_layout: Some(layout),
            pane_rects: rects.clone(),
            content_rect: [660.0, 0.0, 400.0, 700.0],
            ..serde_json::from_str(
                r#"{"url":null,"title":null,"loading":false,"can_go_back":false,
                "can_go_forward":false,"web_view_hosted":true,"active_profile":1,
                "active_profile_title":"Default","active_profile_hue":"cobalt",
                "active_space":1,"tabs":[],"profiles":[],"spaces":[],
                "active_tab_index":null,"active_web_view":1,"engine_sessions":2,
                "lifecycle_counts":{"active":0,"warm":0,"suspended":0,"discarded":0},
                "dense":{"item_count":0,"pinned_rows":0,"first_rendered":0,
                "rendered_rows":0,"window_rows":0,"scroll_anchor":0,
                "active_visible":false,"active_row":null},"engine_kind":"native-wkwebview",
                "chrome_focused":false,"shell_modes":{"theme":"dark","contrast":"standard",
                "density":"normal"},"tab_states":[],"row_titles":[],"pinned_count":0,
                "command_bar_hint":null,"command_profile_hint":null,"drag_source":null,
                "drag_slot":null,"drag_valid":false,"space_chips":[],"visible_ids":["bbr-root"],"semantics":[],
                "command_bar_mode":"Inactive","command_bar_text":"",
                "sidebar_visible":true,"sidebar_width":260.0,"sidebar_min":200.0,
                "sidebar_default":260.0,"sidebar_max":360.0,"content_min":64.0,
                "divider_dragging":false,"divider_label":"Sidebar divider",
                "content_rect":[660.0,0.0,400.0,700.0],"pane_layout":null,
                "pane_rects":[],"window_size":[1080.0,700.0]}"#,
            )
            .unwrap()
        };
        let decoded_snapshot: Snapshot =
            serde_json::from_str(&serde_json::to_string(&snapshot).unwrap()).unwrap();
        assert_eq!(decoded_snapshot, snapshot);
        assert_eq!(decoded_snapshot.pane_layout.unwrap().focused, WebViewId(2));
        assert_eq!(decoded_snapshot.pane_rects, rects);
    }

    /// The new pane geometry types keep their plain-value shape (no
    /// framework types leak onto the wire).
    #[test]
    fn pane_geometry_is_a_plain_value() {
        let layout = PaneLayoutSnapshot {
            primary: WebViewId(7),
            secondary: None,
            focused: WebViewId(7),
        };
        assert_eq!(layout.clone(), layout);
        let pane = PaneRectSnapshot {
            view: WebViewId(7),
            rect: [0.0, 0.0, 1.0, 1.0],
        };
        assert_eq!(pane.clone(), pane);
    }

    /// The §11.4 command-surface wire enum keeps the snake_case shape, and
    /// a snapshot carrying the new field round-trips through the full wire
    /// shape. Legacy omission (a snapshot without the field deserializes to
    /// the closed state) is proven by the pane-geometry test above, whose
    /// literal predates the field and relies on `#[serde(default)]`.
    #[test]
    fn command_surface_wire_shape_and_snapshot_round_trip() {
        assert_eq!(
            serde_json::to_value(CommandSurface::None).unwrap(),
            json!("none")
        );
        assert_eq!(
            serde_json::to_value(CommandSurface::Palette).unwrap(),
            json!("palette")
        );
        assert_eq!(
            serde_json::to_value(CommandSurface::Overlay).unwrap(),
            json!("overlay")
        );
        assert_eq!(
            serde_json::from_str::<CommandSurface>("\"palette\"").unwrap(),
            CommandSurface::Palette
        );
        // A snapshot carrying the new field round-trips through the full
        // wire shape (the pane-geometry literal above predates the field;
        // its omission relies on `#[serde(default)]`).
        let with_surface: Snapshot = serde_json::from_str(
            r#"{"url":null,"title":null,"loading":false,"can_go_back":false,
                "can_go_forward":false,"web_view_hosted":true,"active_profile":1,
                "active_profile_title":"Default","active_profile_hue":"cobalt",
                "active_space":1,"tabs":[],"profiles":[],"spaces":[],
                "active_tab_index":null,"active_web_view":1,"engine_sessions":2,
                "lifecycle_counts":{"active":0,"warm":0,"suspended":0,"discarded":0},
                "dense":{"item_count":0,"pinned_rows":0,"first_rendered":0,
                "rendered_rows":0,"window_rows":0,"scroll_anchor":0,
                "active_visible":false,"active_row":null},"engine_kind":"native-wkwebview",
                "chrome_focused":false,"shell_modes":{"theme":"dark","contrast":"standard",
                "density":"normal"},"tab_states":[],"row_titles":[],"pinned_count":0,
                "command_bar_hint":null,"command_profile_hint":null,"drag_source":null,
                "drag_slot":null,"drag_valid":false,"space_chips":[],"visible_ids":["bbr-root"],"semantics":[],
                "command_bar_mode":"Inactive","command_bar_text":"",
                "command_surface":"overlay",
                "sidebar_visible":true,"sidebar_width":260.0,"sidebar_min":200.0,
                "sidebar_default":260.0,"sidebar_max":360.0,"content_min":64.0,
                "divider_dragging":false,"divider_label":"Sidebar divider",
                "content_rect":[660.0,0.0,400.0,700.0],"pane_layout":null,
                "pane_rects":[],"window_size":[1080.0,700.0]}"#,
        )
        .unwrap();
        assert_eq!(with_surface.command_surface, CommandSurface::Overlay);
    }

    /// The testkit-only direct-renderer capture keeps its externally tagged
    /// path payload exact. The app separately constrains the authenticated
    /// request to one direct `/tmp/*.png` destination.
    #[test]
    fn capture_chrome_request_has_exact_wire_shape() {
        let request = Request::CaptureChrome {
            path: "/tmp/bbr-visual.png".into(),
        };
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "CaptureChrome": { "path": "/tmp/bbr-visual.png" } })
        );
        assert_eq!(
            serde_json::from_str::<Request>(r#"{"CaptureChrome":{"path":"/tmp/bbr-visual.png"}}"#)
                .unwrap(),
            request
        );
    }

    /// The portal probe requests keep their exact externally-tagged JSON
    /// shape on the wire (unit variants serialize as the bare name) and the
    /// fixed literals deserialize back into the same typed values.
    #[test]
    fn portal_requests_have_exact_wire_shape() {
        for (request, literal) in [
            (Request::PortalOpenProbe, "PortalOpenProbe"),
            (Request::PortalCloseProbe, "PortalCloseProbe"),
            (Request::PortalOutsideDismiss, "PortalOutsideDismiss"),
            (Request::PortalTeardown, "PortalTeardown"),
            (Request::ActivateApp, "ActivateApp"),
        ] {
            assert_eq!(serde_json::to_value(&request).unwrap(), json!(literal));
            assert_eq!(
                serde_json::from_str::<Request>(&format!("\"{literal}\"")).unwrap(),
                request
            );
        }
    }

    /// The portal snapshot keeps its exact plain-value JSON shape (numeric
    /// ids and window numbers, plain rect array, optional frontmost window)
    /// and round-trips through the real framing. The default state is the
    /// closed host (logical visibility false, no presentation, no rect).
    #[test]
    fn portal_snapshot_round_trips_with_exact_wire_shape() {
        let active = PortalSnapshot {
            presentation: Some(7),
            host: Some("native-above-web".into()),
            visible: true,
            rect: Some([300.0, 170.0, 480.0, 360.0]),
            ids: vec![
                "bbr-portal-probe".into(),
                "bbr-portal-backdrop".into(),
                "bbr-portal-card".into(),
            ],
            child_attached: true,
            native_visible: true,
            frame_synced: true,
            nonactivating_panel: true,
            parent_window_number: 11,
            portal_window_number: 12,
            frontmost_at_probe_point: Some(12),
            frontmost_at_probe_point_is_this_app: true,
            below_portal_at_probe_point: Some(11),
            portal_owns_probe_point: true,
            portal_is_key: false,
            portal_is_main: false,
            parent_fullscreen: false,
            parent_minimized: false,
            parent_on_active_space: true,
            portal_on_active_space: true,
            portal_scale_factor: 2.0,
            hit_regions: None,
            ignores_mouse_events: false,
        };
        assert_eq!(
            serde_json::to_value(&active).unwrap(),
            json!({
                "presentation": 7,
                "host": "native-above-web",
                "visible": true,
                "rect": [300.0, 170.0, 480.0, 360.0],
                "ids": ["bbr-portal-probe", "bbr-portal-backdrop", "bbr-portal-card"],
                "child_attached": true,
                "native_visible": true,
                "frame_synced": true,
                "nonactivating_panel": true,
                "parent_window_number": 11,
                "portal_window_number": 12,
                "frontmost_at_probe_point": 12,
                "frontmost_at_probe_point_is_this_app": true,
                "below_portal_at_probe_point": 11,
                "portal_owns_probe_point": true,
                "portal_is_key": false,
                "portal_is_main": false,
                "parent_fullscreen": false,
                "parent_minimized": false,
                "parent_on_active_space": true,
                "portal_on_active_space": true,
                "portal_scale_factor": 2.0,
                "hit_regions": null,
                "ignores_mouse_events": false
            })
        );
        let decoded: PortalSnapshot =
            serde_json::from_str(&serde_json::to_string(&active).unwrap()).unwrap();
        assert_eq!(decoded, active);

        let closed = PortalSnapshot::default();
        assert!(!closed.visible);
        assert_eq!(closed.presentation, None);
        assert_eq!(closed.rect, None);
        assert!(closed.ids.is_empty());
        assert!(!closed.child_attached);
        assert!(!closed.native_visible);
        assert!(!closed.frame_synced);
        assert!(!closed.nonactivating_panel);
        assert_eq!(closed.parent_window_number, 0);
        assert_eq!(closed.portal_window_number, 0);
        assert_eq!(closed.frontmost_at_probe_point, None);
        assert!(
            !closed.frontmost_at_probe_point_is_this_app,
            "no frontmost window means nothing is this app's"
        );
    }

    /// A snapshot that omits the portal field (an older harness client)
    /// still deserializes: the portal diagnostics default to the closed
    /// host.
    #[test]
    fn snapshot_without_portal_field_defaults_to_closed_host() {
        let snapshot: Snapshot = serde_json::from_str(
            r#"{"url":null,"title":null,"loading":false,"can_go_back":false,
            "can_go_forward":false,"web_view_hosted":false,"active_profile":1,
            "active_profile_title":null,"active_profile_hue":null,
            "active_space":1,"tabs":[],"profiles":[],"spaces":[],
            "active_tab_index":null,"active_web_view":null,"engine_sessions":0,
            "lifecycle_counts":{"active":0,"warm":0,"suspended":0,"discarded":0},
            "dense":{"item_count":0,"pinned_rows":0,"first_rendered":0,
            "rendered_rows":0,"window_rows":0,"scroll_anchor":0,
            "active_visible":false,"active_row":null},"engine_kind":"native-wkwebview",
            "chrome_focused":false,"shell_modes":{"theme":"dark","contrast":"standard",
            "density":"normal"},"tab_states":[],"row_titles":[],"pinned_count":0,
            "command_bar_hint":null,"command_profile_hint":null,"drag_source":null,
            "drag_slot":null,"drag_valid":false,"space_chips":[],"visible_ids":["bbr-root"],"semantics":[],
            "command_bar_mode":"Inactive","command_bar_text":"",
            "sidebar_visible":true,"sidebar_width":260.0,"sidebar_min":200.0,
            "sidebar_default":260.0,"sidebar_max":360.0,"content_min":64.0,
            "divider_dragging":false,"divider_label":"Sidebar divider",
            "content_rect":[260.0,0.0,400.0,700.0],"pane_layout":null,
            "pane_rects":[],"window_size":[1080.0,700.0]}"#,
        )
        .unwrap();
        assert_eq!(snapshot.portal, PortalSnapshot::default());
        // An older harness client omitting the appearance evidence defaults
        // to no active appearance (the field is additive, never required).
        assert_eq!(snapshot.active_appearance, None);
    }

    /// The §11.3 strip chip wire shape stays plain and round-trips: the
    /// persistent space id is the chip identity, `selected` is the exact
    /// active-space evidence, `theme` is the effective curated token name,
    /// and display order is the vector order (the ids derive from `space`,
    /// never the position). An older client omitting `theme` still
    /// deserializes with the empty default.
    #[test]
    fn space_chip_snapshot_round_trips_with_exact_wire_shape() {
        use crate::protocol::TestSpaceId;
        let chips = vec![
            SpaceChipSnapshot {
                space: TestSpaceId(2),
                title: "Default".into(),
                identity: "code".into(),
                identity_kind: "glyph".into(),
                theme: "cobalt".into(),
                selected: true,
                drag_target: false,
                companion: None,
            },
            SpaceChipSnapshot {
                space: TestSpaceId(5),
                title: "Reading".into(),
                identity: "book".into(),
                identity_kind: "glyph".into(),
                theme: "pine".into(),
                selected: false,
                drag_target: false,
                companion: None,
            },
        ];
        let json = serde_json::to_value(&chips).unwrap();
        assert_eq!(
            json,
            json!([
                {"space":2,"title":"Default","identity":"code","identity_kind":"glyph","theme":"cobalt","selected":true,"drag_target":false,"companion":null},
                {"space":5,"title":"Reading","identity":"book","identity_kind":"glyph","theme":"pine","selected":false,"drag_target":false,"companion":null}
            ])
        );
        let decoded: Vec<SpaceChipSnapshot> = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, chips);
        // Exactly one chip carries the active-space selection.
        assert_eq!(chips.iter().filter(|c| c.selected).count(), 1);
        // An older client omitting `theme`/`identity_kind` still
        // deserializes (empty names).
        let legacy = serde_json::from_value::<SpaceChipSnapshot>(json!(
            {"space":2,"title":"Default","identity":"code","selected":true}
        ))
        .unwrap();
        assert_eq!(legacy.theme, "");
        assert_eq!(legacy.identity_kind, "");
    }

    /// The resolved appearance evidence round-trips as plain data: the
    /// active space, the effective theme/identity names, and the resolved
    /// chrome/chip tint hex from the one central presentation model. A
    /// snapshot that omits the field (an older harness client) still
    /// deserializes to `None`.
    #[test]
    fn active_appearance_snapshot_round_trips_and_defaults_to_none() {
        use crate::protocol::SpaceAppearanceSnapshot;
        let appearance = SpaceAppearanceSnapshot {
            space: crate::protocol::TestSpaceId(2),
            theme: "cobalt".into(),
            identity: "code".into(),
            chrome_tint: "#1C2028".into(),
            chip_tint: "#222837".into(),
            intensity: "rich".into(),
            grain: true,
        };
        let json = serde_json::to_value(&appearance).unwrap();
        assert_eq!(
            json,
            json!({
                "space":2,
                "theme":"cobalt",
                "identity":"code",
                "chrome_tint":"#1C2028",
                "chip_tint":"#222837",
                "intensity":"rich",
                "grain":true
            })
        );
        let decoded: SpaceAppearanceSnapshot = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, appearance);
        // v2 D6/D7: an older client omitting the paper decodes the
        // deterministic defaults.
        let legacy: SpaceAppearanceSnapshot = serde_json::from_value(json!({
            "space":2,
            "theme":"cobalt",
            "identity":"code",
            "chrome_tint":"#1C2028",
            "chip_tint":"#222837"
        }))
        .unwrap();
        assert_eq!(
            (legacy.intensity.as_str(), legacy.grain),
            ("standard", false)
        );
    }

    /// The §11.14/§11.5 rest-state evidence round-trips as plain data: the
    /// native safe-area guide, navigation availability, the typed address
    /// rest fill (translucent borderless token pair by chrome polarity)
    /// and Reload rest surface (transparent), plus the real
    /// hovered/pressed presentation state. An older client omitting the
    /// new fields still deserializes (serde defaults).
    #[test]
    fn titlebar_snapshot_round_trips_rest_state_evidence() {
        use crate::protocol::{AddressRestFill, ReloadRestSurface, TitlebarSnapshot};
        let titlebar = TitlebarSnapshot {
            safe_area_height: 28.0,
            traffic_lights: [12.0, 8.0, 72.0, 16.0],
            sidebar_collapsed: false,
            can_go_back: true,
            can_go_forward: false,
            can_reload: true,
            address_rest_fill: AddressRestFill::BorderedFieldOnDark,
            reload_rest: ReloadRestSurface::Transparent,
            reload_hovered: true,
            reload_pressed: false,
        };
        let json = serde_json::to_value(titlebar).unwrap();
        assert_eq!(
            json,
            json!({
                "safe_area_height":28.0,
                "traffic_lights":[12.0,8.0,72.0,16.0],
                "sidebar_collapsed":false,
                "can_go_back":true,
                "can_go_forward":false,
                "can_reload":true,
                "address_rest_fill":"bordered_field_on_dark",
                "reload_rest":"transparent",
                "reload_hovered":true,
                "reload_pressed":false
            })
        );
        let decoded: TitlebarSnapshot = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, titlebar);
        // An older client omitting the rest-state fields still
        // deserializes to the documented defaults: translucent-on-dark
        // fill, transparent Reload rest, and no hover/press.
        let legacy = serde_json::from_value::<TitlebarSnapshot>(json!(
            {"safe_area_height":28.0,"sidebar_collapsed":false,"can_reload":true}
        ))
        .unwrap();
        assert_eq!(
            legacy.address_rest_fill,
            AddressRestFill::BorderedFieldOnDark
        );
        assert_eq!(legacy.reload_rest, ReloadRestSurface::Transparent);
        assert!(!legacy.reload_hovered);
        assert!(!legacy.reload_pressed);
    }

    /// The §11.3 strip scroll evidence round-trips as plain data: the raw
    /// viewport/offset/max offset plus one bounds rect per chip in display
    /// order. A snapshot that omits the field (an older harness client)
    /// still deserializes: the scroll evidence defaults to the idle strip.
    #[test]
    fn space_strip_scroll_evidence_round_trips_with_an_idle_default() {
        use crate::protocol::SpaceStripScrollSnapshot;
        let scroll = SpaceStripScrollSnapshot {
            viewport: [260.0, 664.0, 168.0, 36.0],
            offset: [-312.0, 0.0],
            max_offset: [312.0, 0.0],
            chip_bounds: vec![[264.0, 668.0, 96.0, 28.0], [368.0, 668.0, 104.0, 28.0]],
        };
        let json = serde_json::to_value(&scroll).unwrap();
        assert_eq!(
            json,
            json!({
                "viewport":[260.0,664.0,168.0,36.0],
                "offset":[-312.0,0.0],
                "max_offset":[312.0,0.0],
                "chip_bounds":[[264.0,668.0,96.0,28.0],[368.0,668.0,104.0,28.0]]
            })
        );
        let decoded: SpaceStripScrollSnapshot = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, scroll);
        // The default is the all-zero idle strip (an older harness client
        // omitting the field still deserializes): no viewport, no offset,
        // no chip rects.
        let idle = SpaceStripScrollSnapshot::default();
        assert_eq!(idle.viewport, [0.0; 4]);
        assert_eq!(idle.offset, [0.0, 0.0]);
        assert_eq!(idle.max_offset, [0.0, 0.0]);
        assert!(idle.chip_bounds.is_empty());
    }
}
