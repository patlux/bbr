//! The testkit semantic snapshot and the §15 visible-id derivation.
//!
//! `TestSnapshot` is the app-local typed state the E2E protocol converts to
//! the wire `Snapshot`; `visible_ids` mirrors the current render so the id
//! assertions cannot drift from the shell. Both are testkit-only.

#[cfg(feature = "testkit")]
use bbr_browser::ControllerSnapshot;

#[cfg(feature = "testkit")]
#[derive(Debug, Clone, PartialEq)]
pub struct TestSnapshot {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub web_view_hosted: bool,
    /// Native dark cover over WebKit's provisional-navigation plane.
    pub navigation_cover_visible: bool,
    pub active_profile: bbr_web::ProfileId,
    pub active_profile_title: Option<String>,
    /// The active profile's identity hue name (DESIGN.md §12).
    pub active_profile_hue: Option<String>,
    pub active_space: bbr_testkit::protocol::TestSpaceId,
    pub tabs: Vec<bbr_browser::TabSnapshot>,
    /// Complete persistent tab identity across spaces (the command corpus
    /// source): space order, then tab order.
    pub all_tabs: Vec<bbr_browser::GlobalTabSnapshot>,
    /// Complete profile/space collections in creation order.
    pub profiles: Vec<bbr_browser::ProfileSnapshot>,
    pub spaces: Vec<bbr_browser::SpaceSnapshot>,
    pub active_tab_index: Option<usize>,
    pub active_web_view: Option<bbr_web::WebViewId>,
    pub engine_sessions: usize,
    pub engine_kind: &'static str,
    pub chrome_focused: bool,
    pub shell_modes: bbr_testkit::protocol::ShellModes,
    pub tab_states: Vec<bbr_testkit::protocol::TabVisualState>,
    pub row_titles: Vec<String>,
    /// D2 lift evidence: the ids of the rail surfaces painted with
    /// `elev:lift` (selected tab row, selected folder member).
    pub lifted_ids: Vec<String>,
    /// The pinned/regular boundary (DESIGN.md §11.2): leading `pinned_count`
    /// tabs of the active space are pinned.
    pub pinned_count: usize,
    /// A command-bar destination hint (DESIGN.md §11.3), e.g. the
    /// destination profile of a staged move.
    pub command_bar_hint: Option<String>,
    /// The §11.3 cross-context transient: after a cross-profile space
    /// activation the destination profile appears at the top of the command
    /// field for one interaction ("in Profile · <title>"). `None` at rest;
    /// same-profile switching never sets it; the next chrome context
    /// mutation clears it.
    pub command_profile_hint: Option<String>,
    /// The dragged tab row's active-space index (§10.9). `None` when no drag
    /// is in flight; the dragged row renders `dragging`.
    pub drag_source: Option<usize>,
    /// The current drag target: the tab insertion slot (within the source
    /// region). `None` when the drag targets the favorites grid, a chip, or
    /// nothing (an invalid no-target zone).
    pub drag_slot: Option<usize>,
    /// §10.9/§11.10: whether the in-flight drag currently targets the
    /// favorites grid (a drop toggles the dragged tab in/out of the
    /// window-global favorites). Typed alongside `drag_slot`/`drag_chip`
    /// so a favorites-targeted drag is asserted by state, not by geometry
    /// guessing.
    pub drag_favorites: bool,
    /// The §11.10 favorites region's rendered window-relative rect
    /// `[x, y, width, height]`, recorded by the production band anchor
    /// (the exact drop-band span the resolver hits). `None` while no band
    /// renders (no tiles and no in-flight drag).
    pub favorites_rect: Option<[f32; 4]>,
    /// The current drag target's persistent space chip (DESIGN.md §11.3):
    /// the exact `bbr-space-chip-<persistent-space-id>` the pointer hovers
    /// during a drag-to-chip. `None` outside the chips (tab slot, favorites
    /// grid, or an invalid no-target zone).
    pub drag_chip: Option<bbr_testkit::protocol::TestSpaceId>,
    /// Blueprint D07: the destination-profile label text.
    pub drag_destination: Option<String>,
    pub drag_valid: bool,
    /// §11.3 permanent space strip: one chip per space in canonical display
    /// order (the vector order is display order; the chip/identity ids
    /// derive from the persistent `space` id). The strip is always part of
    /// the rail; `selected` mirrors the active space exactly.
    pub space_chips: Vec<bbr_testkit::protocol::SpaceChipSnapshot>,
    /// §11.3 resolved appearance evidence of the active space: effective
    /// theme/identity names plus the resolved chrome/chip tint hex the
    /// render paints (the one central presentation model; never native/web
    /// pixels). `None` only in the invariant-forbidden empty browser.
    pub active_appearance: Option<bbr_testkit::protocol::SpaceAppearanceSnapshot>,
    /// §6/§18.5 transient-motion evidence and the idle render counter.
    pub motion: bbr_testkit::protocol::MotionSnapshot,
    /// §11.3/§11.4 strip scroll evidence (read-only from the production
    /// `space_strip_scroll` handle): the viewport, the signed offset, the
    /// max offset, and each chip's raw layout bounds in canonical order.
    pub space_strip_scroll: bbr_testkit::protocol::SpaceStripScrollSnapshot,
    /// §11.3 Space-configuration surface state while open: the bound exact
    /// persistent SpaceId, the header title, the rename draft, the selected
    /// curated theme/identity names, and the surface's chrome-focus state.
    /// `None` while closed (the surface is not rendered).
    pub space_config: Option<bbr_testkit::protocol::SpaceConfigSnapshot>,
    /// Blueprint D05: the open Delete Space sheet (`None` while closed).
    pub space_delete: Option<bbr_testkit::protocol::SpaceDeleteSnapshot>,
    /// Blueprint D14: the open Back/Forward history menu.
    pub nav_history: Option<bbr_testkit::protocol::NavHistorySnapshot>,
    /// DESIGN.md §11.3 sidebar-swipe evidence: the truthful active space at
    /// every phase plus the gesture's source/destination persistent space
    /// ids, display-order direction, signed progress, and the interpolated
    /// chrome/chip tints while tracking (chrome-only presentation).
    pub swipe: bbr_testkit::protocol::SwipeSnapshot,
    pub visible_ids: Vec<String>,
    /// The typed semantic tree of the current render (DESIGN.md §15 /
    /// accessibility track): roles, names, values, enabled/selected/focused
    /// state, and hierarchy. Internal model — GPUI 0.2.2 has no platform
    /// accessibility layer, so these are not macOS Accessibility nodes; the
    /// platform bridge is Phase 9.
    pub semantics: Vec<bbr_testkit::protocol::SemanticNode>,
    pub command_bar_mode: bbr_testkit::protocol::CommandBarMode,
    pub command_bar_text: String,
    /// The §11.5 address pill's derived domain (display-only — the pill
    /// never holds the editable text, a cursor or a selection).
    pub address_domain: String,
    /// Blueprint D03: the address identity kind (`secure`, `not-secure`,
    /// `internal`, `file`, `other`, `empty`).
    pub address_identity: String,
    /// The §11.5 address-pill hover state (the fill steps one
    /// brighter/darker via the token pair).
    pub address_hovered: bool,
    /// The §11.5 address pill's measured window-relative rect (the §11.4
    /// overlay anchor: the overlay covers this rect).
    pub address_rect: Option<[f32; 4]>,
    /// The open command-surface presentation (DESIGN.md §11.4): the centered
    /// Cmd-T palette, the field-anchored Cmd-L/click overlay, or none. At
    /// most one is open at a time (mutual exclusion by construction).
    pub command_surface: bbr_testkit::protocol::CommandSurface,
    /// The typed state of the command result surface (DESIGN.md §11.5).
    pub command_result_state: bbr_testkit::protocol::CommandResultState,
    /// The ranked results of the latest published generation (limit 16).
    pub command_results: Vec<bbr_testkit::protocol::CommandResultSnapshot>,
    /// The truthful total match count before the limit-16 window truncation
    /// (the full-search presentation evidence).
    pub command_result_total: usize,
    /// Byte offset of the command-text cursor (UTF-8 safe).
    pub command_cursor: usize,
    /// Selected byte range `[start, end)` when non-empty.
    pub command_selection: Option<[usize; 2]>,
    /// Lifecycle population counts of the active space (dense-list evidence).
    pub lifecycle_counts: bbr_testkit::protocol::LifecycleCounts,
    /// The dense-list render contract (§11.6): bounded rendered window,
    /// deterministic scroll anchor, active-row visibility.
    pub dense: bbr_testkit::protocol::DenseListSnapshot,
    pub sidebar_visible: bool,
    /// §11.14 titlebar presentation evidence (task packet
    /// `titlebar-navigation-controls`): the native safe-area guide the row
    /// renders from, the sidebar collapse state, and the genuine
    /// Back/Forward/Reload availability of the focused pane.
    pub titlebar: bbr_testkit::protocol::TitlebarSnapshot,
    /// DESIGN §2 seated content card frame and native clip.
    pub content_card: bbr_testkit::protocol::ContentCardSnapshot,
    /// §17.14 address zoom chip percent (`None` at 100 %).
    pub address_zoom: Option<u16>,
    /// §11.4 heading identity (the active chip's value).
    pub active_space_identity: Option<String>,
    /// §11.10 grid columns and derived tile size.
    pub favorites_columns: usize,
    pub favorites_tile_size: [f32; 2],
    pub favorites_region_height: f32,
    /// §10.9/§11.2/§11.10 drag offers of the in-flight tab drag.
    pub drag_offers: bbr_testkit::protocol::DragOffersSnapshot,
    /// The effective sidebar rail width: the DESIGN §5 adjustable width
    /// clamped to the window (0 when collapsed — the rail is gone).
    pub sidebar_width: f32,
    /// The DESIGN §5 adjustable-range contract, exposed so E2E asserts the
    /// same constants the render enforces.
    pub sidebar_min: f32,
    pub sidebar_default: f32,
    pub sidebar_max: f32,
    /// The native-content floor (DESIGN §5/§13): the WKWebView slot is never
    /// laid out narrower, so the rail yields below the requested width in
    /// compact windows.
    pub content_min: f32,
    /// The resize divider drag state (idle vs dragging).
    pub divider_dragging: bool,
    /// The stable divider label (future Accessibility; §15 shared identity).
    pub divider_label: String,
    /// The last native slot rect handed to the engine (WKWebView frame).
    pub content_rect: [f32; 4],
    /// The authoritative pane layout (primary/secondary/focused ids), or
    /// `None` when no live pane is rendered.
    pub pane_layout: Option<bbr_browser::PaneLayout>,
    /// Keyed per-pane native geometry in pane-layout order (the split shared
    /// contract evidence: pane ids and their real WKWebView frames).
    pub pane_rects: Vec<bbr_testkit::protocol::PaneRectSnapshot>,
    pub window_size: [f32; 2],
    /// The native window content-area size `[width, height]` in points
    /// (excluding the title bar): the truthful restored geometry — GPUI's
    /// window options apply the persisted bounds as the content rect, so
    /// this is the size a persisted window row reproduces. Measured via
    /// the live session's window `contentLayoutRect`; `[0.0, 0.0]` before
    /// the first native session exists.
    pub window_content_size: [f32; 2],
    /// Typed diagnostics of the one native portal host (task packet
    /// `native-portal-host`; DESIGN.md §15): the active presentation and its
    /// native attachment/frame/visibility evidence.
    pub portal: bbr_testkit::protocol::PortalSnapshot,
    /// Typed find-in-page presentation state (task packet
    /// `find-in-page-portal-bar`): open presentation, field focus, query,
    /// WebKit-reported count/ordinal, pending request, and staleness.
    pub find: bbr_testkit::protocol::FindSnapshot,
    /// Typed per-site zoom state (queue item 11, part A; task packet
    /// `site-settings-zoom-contract`): effective level of the active view's
    /// site, the normalized origin, the applied level, and pending write.
    pub zoom: bbr_testkit::protocol::ZoomSnapshot,
    /// Typed BBR context-menu presentation state (queue item 12, part B;
    /// task packet `bbr-context-menus-over-web`): open presentation, focus,
    /// selection, the deterministic row model with genuine enablement, the
    /// pointer anchor, and the native bridge's ownership diagnostics.
    pub context_menu: bbr_testkit::protocol::ContextMenuSnapshot,
    /// Typed Tab/Favorite context-menu presentation state (task packet
    /// `tab-favorite-context-menus`; DESIGN.md §11.13): the exact bound
    /// `TabId`, the keyboard/pointer cursor, the open submenu, the
    /// deterministic row model, and the resolved clamped placement.
    /// Ordinary GPUI chrome — never a portal surface.
    pub tab_menu: bbr_testkit::protocol::TabMenuSnapshot,
    /// Typed credential-sheet presentation state (queue item 10): the
    /// presented challenge's identity/metadata, lifecycle state, redacted
    /// outcome, and masked edit flags. By construction no credential can
    /// appear in any wire shape.
    pub auth: bbr_testkit::protocol::AuthSnapshot,
    /// Whether the §11.4 plus menu is open and its selected item.
    pub plus_menu_open: bool,
    pub plus_menu_selection: usize,
    /// §11.10 favorite tiles and §11.11 folder rows.
    pub favorites: Vec<bbr_testkit::protocol::FavoriteTileSnapshot>,
    pub folders: Vec<bbr_testkit::protocol::FolderRowSnapshot>,
    /// §11.12 download rows in deterministic panel order (Active by
    /// ascending `DownloadId`, then History most-recent-terminal first);
    /// row index `<n>` is stable for E2E.
    pub download_rows: Vec<bbr_testkit::protocol::DownloadRowSnapshot>,
    /// §11.12 downloads panel render contract (open, focus, bounded window).
    pub downloads: bbr_testkit::protocol::DownloadsPanelSnapshot,
    /// The last completed real WebKit content probe (testkit-only; macOS):
    /// the committed-document + native url/title observation the E2E gateway
    /// uses as real content evidence. `None` until a probe completion
    /// landed.
    #[cfg(all(target_os = "macos", feature = "testkit"))]
    pub web_content_probe: Option<bbr_testkit::protocol::WebContentProbe>,
    /// §11.8-style error-surface presentation (task packet
    /// `error-surface-contract`): the classified incident the content area
    /// renders, or the empty default when no error surface is shown.
    pub error_surface: bbr_testkit::protocol::ErrorSurfaceSnapshot,
    /// Phase-7 app-owned persistence composition (task packet
    /// `persistence-core` + the command-history provider slice): the
    /// truthful startup store disposition, the effective global search
    /// engine, and the in-memory history provider size.
    pub store: bbr_testkit::protocol::StoreSnapshot,
    /// Phase-7 startup restore evidence (warm-restore slice): whether a
    /// persisted browser model was applied chrome-first and how many tab
    /// identities stayed session-less (deferred) after the restore.
    pub restore: bbr_testkit::protocol::RestoreDisposition,
    /// Productive new-window request/open/block counters.
    pub new_window: bbr_testkit::protocol::NewWindowSnapshot,
    /// Web-content navigation gesture diagnostics (N1): commit
    /// classification counters and live opener-provenance pairs.
    pub content_gesture: bbr_testkit::protocol::ContentGestureSnapshot,
    /// The transient Peek preview (Link Preview / Peek packet): `None` at
    /// rest. Mirrors the controller snapshot's preview state.
    pub preview: Option<bbr_testkit::protocol::PreviewSnapshot>,
    /// Whether the Peek bar is currently presented in the one native portal
    /// window.
    pub peek_presented: bool,
    /// Theme mode the portal render last drew the Peek bar in.
    pub peek_theme: Option<String>,
    /// Blueprint D09: the Peek frame the portal drew (while presented).
    pub peek_frame: Option<bbr_testkit::protocol::PeekFrameSnapshot>,
    /// Blueprint D19 menus.
    pub menu_bar: Vec<bbr_testkit::protocol::MenuSnapshot>,
    pub dock_menu: Vec<bbr_testkit::protocol::MenuItemSnapshot>,
    /// Blueprint D17 recovery pane.
    pub recovery: Option<String>,
    pub restore_notice: Option<String>,
    pub about: Option<String>,
    /// Blueprint D08 overlays.
    pub toast: Option<bbr_testkit::protocol::ToastSnapshot>,
    /// Blueprint D02: the command field as a text-input client.
    pub command_input: Option<bbr_testkit::protocol::CommandInputSnapshot>,
    pub text_input: Option<bbr_testkit::protocol::TextInputReadback>,
    pub rail_reveal: String,
    pub floating_rail: Option<bbr_testkit::protocol::FloatingRailSnapshot>,
    pub floating_rail_leaving: bool,
    pub floating_rail_exits: u64,
    pub settings: Option<bbr_testkit::protocol::SettingsSnapshot>,
    /// Blueprint D10: the History page while it is open.
    pub history: Option<bbr_testkit::protocol::HistorySnapshot>,
    /// Blueprint D11: the permission prompt, site settings and marks.
    pub permissions: bbr_testkit::protocol::PermissionsSnapshot,
    /// Blueprint D24: the media card and sessions.
    pub media: bbr_testkit::protocol::MediaSnapshot,
    /// Blueprint D19: every open browser window.
    pub windows: Vec<bbr_testkit::protocol::WindowEntry>,
    pub window_refreshes: u64,
    pub split: Option<bbr_testkit::protocol::SplitSnapshot>,
    pub split_drop: Option<bbr_testkit::protocol::SplitDropSnapshot>,
    pub split_rows: Vec<usize>,
    /// Blueprint D12: the client-certificate sheet and answers.
    pub client_certificates: bbr_testkit::protocol::ClientCertificatesSnapshot,
    /// Blueprint D13: links to other apps.
    pub open_app: bbr_testkit::protocol::OpenAppSnapshot,
    /// Blueprint D15: the first run while it shows.
    pub onboarding: Option<bbr_testkit::protocol::OnboardingSnapshot>,
    pub zoom_readout: Option<u32>,
    /// Native z-order evidence: while a preview is open the preview
    /// WKWebView's subview index must exceed its covered origin's (the
    /// preview is raised above the warm origin). `None` at rest.
    pub preview_z_order: Option<bbr_testkit::protocol::PreviewZOrder>,
    /// DESIGN.md §11.16 Saved Pages Library presentation state (B3 packet
    /// `saved-pages-library`): open flag, search/sort, the bounded row
    /// window, the flat collections, the multi-select set, the destructive
    /// remove confirmation, and the native-view hide evidence. The empty
    /// default means the Library is closed.
    pub library: bbr_testkit::protocol::SavedPagesLibrarySnapshot,
}

#[cfg(feature = "testkit")]
mod u2;
#[cfg(feature = "testkit")]
pub use u2::U2IdsInput;

/// The stable `bbr-*` ids present in the current shell render (DESIGN.md
/// §15). Derived from the same structures the renderer consumes — rows via
/// the row-visual indices, the content-pane branch mirroring `Render::render`
/// (native slot / crash panel / empty pane), the command cursor mirroring the
/// command-bar component — so the E2E id assertion cannot drift from the
/// shell: if a spec row's ids are not present here, the E2E fails.
#[cfg(all(test, feature = "testkit"))]
#[allow(clippy::too_many_arguments)]
pub fn visible_ids(
    browser: &ControllerSnapshot,
    chrome_focused: bool,
    command_bar_mode: crate::command_bar::CommandBarMode,
    sidebar_visible: bool,
    command_bar_hint: Option<&str>,
    command_profile_hint: Option<&str>,
    open_surface: crate::app::palette::CommandSurface,
    command_state: crate::app::command_rank::CommandResultState,
    command_results: &[crate::app::command_rank::CommandResultRow],
    command_surface: &[String],
    drag_slot: Option<usize>,
    drag_chip: Option<bbr_browser::SpaceId>,
    dense: crate::dense::DenseRenderState,
    pane_layout: Option<bbr_browser::PaneLayout>,
) -> Vec<String> {
    visible_ids_u2(
        browser,
        chrome_focused,
        command_bar_mode,
        sidebar_visible,
        command_bar_hint,
        command_profile_hint,
        open_surface,
        command_state,
        command_results,
        command_surface,
        drag_slot,
        drag_chip,
        dense,
        pane_layout,
        &U2IdsInput::default(),
    )
}

#[cfg(feature = "testkit")]
#[allow(clippy::too_many_arguments)]
pub fn visible_ids_u2(
    browser: &ControllerSnapshot,
    chrome_focused: bool,
    command_bar_mode: crate::command_bar::CommandBarMode,
    sidebar_visible: bool,
    command_bar_hint: Option<&str>,
    command_profile_hint: Option<&str>,
    open_surface: crate::app::palette::CommandSurface,
    command_state: crate::app::command_rank::CommandResultState,
    command_results: &[crate::app::command_rank::CommandResultRow],
    command_surface: &[String],
    drag_slot: Option<usize>,
    drag_chip: Option<bbr_browser::SpaceId>,
    dense: crate::dense::DenseRenderState,
    pane_layout: Option<bbr_browser::PaneLayout>,
    u2: &u2::U2IdsInput<'_>,
) -> Vec<String> {
    let mut ids: Vec<String> = vec!["bbr-root".into()];
    if sidebar_visible {
        for id in [
            "bbr-sidebar",
            // §11.14: the titlebar row is the rail's top strip (the
            // sidebar toggle + the Back/Forward/Reload cluster) — it
            // disappears with the rail on collapse.
            "bbr-titlebar",
            "bbr-titlebar-sidebar-toggle",
            "bbr-nav-back",
            "bbr-nav-forward",
            "bbr-nav-reload",
            "bbr-active-space-title",
            "bbr-active-space-identity",
            "bbr-new-tab",
            "bbr-tab-list",
            "bbr-tab-scroll",
            "bbr-sidebar-divider",
        ] {
            ids.push(id.into());
        }
        // §11.4 polish: the organized-regions hairline renders exactly
        // while the organized area has content (pinned rows or a folders
        // list) — truthful absence when both are empty (mirrors the
        // render's gating in `sidebar::tabs`).
        if crate::shell::pinned_count(&browser.tabs) > 0 || !u2.folders.is_empty() {
            ids.push(bbr_ui::sidebar::tabs::ORGANIZED_HAIRLINE_ID.into());
        }
        // §11.5: the sidebar command slot renders the display-only address
        // pill (`bbr-address` + domain + accessory) unless a non-address
        // staging mode is active without a floating presentation (New
        // Space / Move Tab / the empty New Tab surface) — only then is the
        // `bbr-command-bar` staging input visible. The address pill is
        // never a field: no `bbr-command-text`/`bbr-command-cursor` at rest
        // or behind an open palette/overlay.
        if open_surface == crate::app::palette::CommandSurface::None && command_bar_mode.is_active()
        {
            ids.push("bbr-command-bar".into());
            ids.push("bbr-command-text".into());
        } else {
            ids.push("bbr-address".into());
            ids.push("bbr-address-domain".into());
            ids.push("bbr-address-accessory".into());
        }
        // The §11.5 result palette ids (rows or typed state rows) mirror the
        // render exactly (shared derivation in `command_surface`).
        ids.extend(command_surface.iter().cloned());
        if command_bar_hint.is_some() {
            ids.push("bbr-command-hint".into());
        }
        if command_profile_hint.is_some() {
            ids.push("bbr-command-profile-hint".into());
        }
        if crate::shell::status_visible(browser) {
            ids.push(bbr_ui::components::STATUS_ID.into());
        }
        // §11.4 spaces region plus U2 footer/favorites/folders/menu use the
        // exact same gating and identities as the render.
        u2::extend_visible_ids(&mut ids, browser, u2);
    }
    // The §11.4 palette/overlay presentation ids mirror the open surface
    // (shared derivation in `app::palette`): the container/field ids plus
    // the published result rows (or the typed state row). The overlay floats
    // above the window content (elev:2, §13), so it appears independent of
    // the sidebar rail.
    ids.extend(crate::app::palette::surface_ids(
        open_surface,
        command_state,
        command_results,
    ));
    if chrome_focused
        && command_bar_mode.is_active()
        && open_surface == crate::app::palette::CommandSurface::None
    {
        ids.push("bbr-command-cursor".into());
    }
    // The native content slots render whenever a pane layout exists (the
    // focused/active view hosts a live native session). Every rendered slot
    // id is keyed by its `WebViewId`; `visible_ids` must not advertise an
    // unkeyed element that GPUI did not render.
    // The §11.8-style error surface (task packet `error-surface-contract`)
    // replaces the active view's native slot in the content area: the
    // render draws `bbr-error-*` ids and never the slot it replaced.
    // The §11.16 Saved Pages Library (B3 packet `saved-pages-library`)
    // wins over the error surface: while open the internal chrome surface
    // covers the content area (the active view's native slot is hidden,
    // never destroyed), so its ids render and neither the slot nor the
    // error surface ids are advertised.
    if u2.library_open {
        ids.extend(u2.library_ids.iter().cloned());
    } else if !u2.history_ids.is_empty() {
        ids.extend(u2.history_ids.iter().cloned());
    } else if u2.error_surface_visible {
        ids.push("bbr-error-panel".into());
        ids.push("bbr-error-title".into());
        ids.push("bbr-error-copy".into());
        if u2.error_surface_origin {
            ids.push("bbr-error-url".into());
        }
        ids.push("bbr-error-action".into());
        ids.push("bbr-error-dismiss".into());
    } else if let Some(layout) = pane_layout {
        // The §11.8a unresponsive pane (renderer-isolation lane) replaces
        // the exact active view's native slot while its renderer is
        // reported unresponsive (a sibling of the crash authority: the
        // session and tab stay alive under the Wait / Reload / Close
        // surface). Mirrors `render.rs` per slot: the pane is drawn only
        // over the active view's slot (never a Peek preview slot — the
        // preview view is not the active tab's view — and never the
        // non-active split member), so the slot it replaced is not
        // advertised.
        let active_view = browser.active_view;
        let active_unresponsive = browser
            .active_index
            .and_then(|index| browser.tabs.get(index))
            .is_some_and(|tab| tab.unresponsive);
        let pane_over = |id: bbr_web::WebViewId| active_view == Some(id) && active_unresponsive;
        if pane_over(layout.primary) {
            ids.extend(
                [
                    "bbr-unresponsive-panel",
                    "bbr-unresponsive-wait",
                    "bbr-unresponsive-reload",
                    "bbr-unresponsive-close",
                ]
                .map(String::from),
            );
        } else {
            ids.push(format!("bbr-native-web-slot-{}", layout.primary.0));
        }
        if let Some(secondary) = layout.secondary {
            if pane_over(secondary) {
                ids.extend(
                    [
                        "bbr-unresponsive-panel",
                        "bbr-unresponsive-wait",
                        "bbr-unresponsive-reload",
                        "bbr-unresponsive-close",
                    ]
                    .map(String::from),
                );
            } else {
                ids.push(format!("bbr-native-web-slot-{}", secondary.0));
            }
        }
    }
    // Rendered rows only: every pinned row (the fixed block) plus the
    // bounded regular window the shell actually built (§11.6) — a dense
    // collection never exposes ids for rows it did not render. The window is
    // the last frame's render state, which can lag the model by a frame
    // (snapshots are taken on demand, right after a mutation, before GPUI
    // draws the next frame), so the exposed rows are intersected with the
    // current model: a row the mutation just removed (e.g. the final close
    // into the empty pane) is never exposed by a stale window.
    let total = browser.tabs.len();
    let pinned = crate::shell::pinned_count(&browser.tabs);
    // §11.11: grouped pinned tabs render inside their folder only.
    let grouped = crate::shell::organized::grouped_indices(u2.folders, browser);
    let mut rendered = crate::shell::organized::loose_pinned(pinned.min(total), &grouped);
    rendered.extend(
        (dense.first_rendered..dense.first_rendered + dense.rendered_rows)
            .filter(|&index| index < total),
    );
    for index in rendered {
        ids.push(format!("bbr-tab-row-{index}"));
        ids.push(format!("bbr-tab-title-{index}"));
        ids.push(format!("bbr-tab-favicon-{index}"));
        ids.push(format!("bbr-tab-state-{index}"));
        ids.push(format!("bbr-tab-close-{index}"));
        ids.push(format!("bbr-tab-pin-{index}"));
    }
    if let Some(slot) = drag_slot {
        ids.push(format!("bbr-drag-tick-{slot}"));
    }
    // §10.9/§11.3: while a drag targets a strip chip, the targeted chip
    // renders its §12 destination-profile companion (`bbr-profile-chip-<n>`
    // — the destination space's profile index), so the companion id is part
    // of the rendered surface exactly while it is visible.
    if let Some(drag_chip) = drag_chip {
        if let Some(space) = browser.spaces.iter().find(|s| s.id == drag_chip) {
            if let Some(profile) = browser.profiles.iter().find(|p| p.id == space.profile) {
                ids.push(format!("bbr-profile-chip-{}", profile.index));
            }
        }
    }
    let active_tab = browser
        .active_index
        .and_then(|index| browser.tabs.get(index));
    if browser.active_view.is_none() {
        if active_tab.is_some_and(|tab| tab.crashed) {
            ids.extend(
                ["bbr-crash-panel", "bbr-crash-reload", "bbr-crash-close"].map(String::from),
            );
        } else {
            ids.extend(["bbr-empty-new-tab", "bbr-empty-new-tab-button"].map(String::from));
        }
    }
    ids
}

#[cfg(all(test, feature = "testkit"))]
mod tests;
