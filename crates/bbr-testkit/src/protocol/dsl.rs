//! Context-free request helpers, so harness code reads as a script.

use super::*;
use bbr_web::{ProfileId, WebAction, WebViewId};

pub fn hello(token: &str, capabilities: &[&str]) -> Request {
    Request::Hello {
        protocol_version: PROTOCOL_VERSION,
        token: token.to_string(),
        capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
    }
}

pub fn navigate(url: impl Into<String>) -> Request {
    Request::Navigate { url: url.into() }
}

pub fn create_profile(profile_title: impl Into<String>, space_title: impl Into<String>) -> Request {
    Request::CreateProfile {
        profile_title: profile_title.into(),
        space_title: space_title.into(),
    }
}

pub fn create_space(profile: ProfileId, title: impl Into<String>) -> Request {
    Request::CreateSpace {
        profile,
        title: title.into(),
    }
}

pub fn activate_space(space: TestSpaceId) -> Request {
    Request::ActivateSpace { space }
}

pub fn open_tab(url: impl Into<String>) -> Request {
    Request::OpenTab { url: url.into() }
}

pub fn select_tab(index: usize) -> Request {
    Request::SelectTab { index }
}

pub fn close_tab(index: usize) -> Request {
    Request::CloseTab { index }
}

pub fn reopen_tab() -> Request {
    Request::ReopenTab
}

pub fn set_split(primary: usize, secondary: usize) -> Request {
    Request::SetSplit { primary, secondary }
}

pub fn close_split() -> Request {
    Request::CloseSplit
}

pub fn pin_tab(index: usize) -> Request {
    Request::PinTab { index }
}

pub fn unpin_tab(index: usize) -> Request {
    Request::UnpinTab { index }
}

pub fn reorder_tab(from: usize, to: usize) -> Request {
    Request::ReorderTab { from, to }
}

pub fn click_tab_pin(index: usize) -> Request {
    Request::ClickTabPin { index }
}

/// §11.14 titlebar toggle click (the `Cmd-B` pointer entry through the
/// production toggle path).
pub fn click_titlebar_toggle() -> Request {
    Request::ClickTitlebarToggle
}

/// §11.14 titlebar Back click (the same navigation authority as Cmd-[).
pub fn click_titlebar_back() -> Request {
    Request::ClickTitlebarBack
}

/// §11.14 titlebar Forward click (the same navigation authority as Cmd-]).
pub fn click_titlebar_forward() -> Request {
    Request::ClickTitlebarForward
}

/// §11.14 titlebar Reload click (the same navigation authority as Cmd-R).
pub fn click_titlebar_reload() -> Request {
    Request::ClickTitlebarReload
}

/// §11.14 pointer hover over a titlebar control (the production `on_hover`
/// listener path).
pub fn hover_titlebar_control(control: &str) -> Request {
    Request::HoverTitlebarControl {
        control: control.into(),
    }
}

/// §11.14 pointer leave from a titlebar control (the production `on_hover`
/// listener path).
pub fn unhover_titlebar_control() -> Request {
    Request::UnhoverTitlebarControl
}

/// §11.14 primary-button down on a titlebar control (the production press
/// slot; paints the §10.3 pressed wash).
pub fn press_titlebar_control(control: &str) -> Request {
    Request::PressTitlebarControl {
        control: control.into(),
    }
}

/// §11.14 primary-button up on a titlebar control (the production release
/// slot; clears the pressed wash).
pub fn release_titlebar_control() -> Request {
    Request::ReleaseTitlebarControl
}

pub fn click_footer_downloads() -> Request {
    Request::ClickFooterDownloads
}

pub fn click_footer_plus() -> Request {
    Request::ClickFooterPlus
}

pub fn click_plus_menu_item(index: usize) -> Request {
    Request::ClickPlusMenuItem { index }
}

pub fn open_space_config(space: TestSpaceId) -> Request {
    Request::OpenSpaceConfig { space }
}

pub fn space_config_select_theme(theme: impl Into<String>) -> Request {
    Request::SpaceConfigSelectTheme {
        theme: theme.into(),
    }
}

pub fn space_config_select_identity(identity: impl Into<String>) -> Request {
    Request::SpaceConfigSelectIdentity {
        identity: identity.into(),
    }
}

pub fn space_config_select_intensity(intensity: impl Into<String>) -> Request {
    Request::SpaceConfigSelectIntensity {
        intensity: intensity.into(),
    }
}

pub fn space_config_toggle_grain() -> Request {
    Request::SpaceConfigToggleGrain
}

pub fn space_config_done() -> Request {
    Request::SpaceConfigDone
}

pub fn space_config_cancel() -> Request {
    Request::SpaceConfigCancel
}

pub fn space_config_press_outside() -> Request {
    Request::SpaceConfigPressOutside
}

pub fn chrome_background_press() -> Request {
    Request::ChromeBackgroundPress
}

pub fn toggle_favorite_tab(index: usize) -> Request {
    Request::ToggleFavoriteTab { index }
}

pub fn click_favorite_tile(index: usize) -> Request {
    Request::ClickFavoriteTile { index }
}

pub fn click_folder_row(id: u64) -> Request {
    Request::ClickFolderRow { id }
}

pub fn click_folder_toggle(id: u64) -> Request {
    Request::ClickFolderToggle { id }
}

pub fn click_folder_member(folder: u64, member: usize) -> Request {
    Request::ClickFolderMember { folder, member }
}

pub fn click_folder_member_close(folder: u64, member: usize) -> Request {
    Request::ClickFolderMemberClose { folder, member }
}

pub fn move_tab_into_folder(index: usize, folder: u64) -> Request {
    Request::MoveTabIntoFolder { index, folder }
}

pub fn remove_tab_from_folder(index: usize, folder: u64) -> Request {
    Request::RemoveTabFromFolder { index, folder }
}

pub fn download_start(url: impl Into<String>, destination: impl Into<String>) -> Request {
    Request::DownloadStart {
        url: url.into(),
        destination: destination.into(),
    }
}

pub fn download_cancel(id: u64) -> Request {
    Request::DownloadCancel { id }
}

pub fn download_retry(id: u64) -> Request {
    Request::DownloadRetry { id }
}

pub fn click_download_row_action(row: usize) -> Request {
    Request::ClickDownloadRowAction { row }
}

pub fn click_download_row_delete(row: usize) -> Request {
    Request::ClickDownloadRowDelete { row }
}

pub fn click_downloads_clear() -> Request {
    Request::ClickDownloadsClear
}

pub fn click_downloads_outside() -> Request {
    Request::ClickDownloadsOutside
}

pub fn scroll_downloads(delta_px: f32) -> Request {
    Request::ScrollDownloads { delta_px }
}

pub fn benchmark_downloads_panel_open(operation_id: BenchmarkOperationId) -> Request {
    Request::BenchmarkDownloadsPanelOpen { operation_id }
}

pub fn benchmark_downloads_progress(
    operation_id: BenchmarkOperationId,
    id: u64,
    received: u64,
    total: u64,
) -> Request {
    Request::BenchmarkDownloadsProgress {
        operation_id,
        id,
        received,
        total,
    }
}

pub fn benchmark_downloads_row_retry(operation_id: BenchmarkOperationId, row: usize) -> Request {
    Request::BenchmarkDownloadsRowRetry { operation_id, row }
}

pub fn benchmark_downloads_row_restart(operation_id: BenchmarkOperationId, row: usize) -> Request {
    Request::BenchmarkDownloadsRowRestart { operation_id, row }
}

pub fn benchmark_downloads_row_delete(operation_id: BenchmarkOperationId, row: usize) -> Request {
    Request::BenchmarkDownloadsRowDelete { operation_id, row }
}

pub fn benchmark_downloads_clear(operation_id: BenchmarkOperationId) -> Request {
    Request::BenchmarkDownloadsClear { operation_id }
}

pub fn seed_downloads_attempt(
    url: impl Into<String>,
    destination: impl Into<String>,
    total: u64,
) -> Request {
    Request::SeedDownloadsAttempt {
        url: url.into(),
        destination: destination.into(),
        total,
    }
}

pub fn seed_downloads_cancelled(
    url: impl Into<String>,
    destination: impl Into<String>,
    total: u64,
) -> Request {
    Request::SeedDownloadsCancelled {
        url: url.into(),
        destination: destination.into(),
        total,
    }
}

pub fn begin_tab_drag(index: usize, x: f32, y: f32, press_in_row: f32) -> Request {
    Request::BeginTabDrag {
        index,
        x,
        y,
        press_in_row,
    }
}

pub fn move_tab_drag(dx: f32, dy: f32) -> Request {
    Request::MoveTabDrag { dx, dy }
}

pub fn end_tab_drag() -> Request {
    Request::EndTabDrag
}

pub fn cancel_tab_drag() -> Request {
    Request::CancelTabDrag
}

pub fn click_tab(index: usize) -> Request {
    Request::ClickTab { index }
}

pub fn click_tab_close(index: usize) -> Request {
    Request::ClickTabClose { index }
}

pub fn hover_tab_row(index: usize) -> Request {
    Request::HoverTabRow { index }
}

pub fn unhover_tab_row() -> Request {
    Request::UnhoverTabRow
}

pub fn hover_folder_row(folder_id: u64) -> Request {
    Request::HoverFolderRow { folder_id }
}

pub fn unhover_folder_row() -> Request {
    Request::UnhoverFolderRow
}

pub fn press_tab_row(index: usize) -> Request {
    Request::PressTabRow { index }
}

pub fn release_tab_row() -> Request {
    Request::ReleaseTabRow
}

/// Open the native portal probe through the general Portal API.
pub fn portal_open_probe() -> Request {
    Request::PortalOpenProbe
}

/// Close the portal probe through the production dismissal method.
pub fn portal_close_probe() -> Request {
    Request::PortalCloseProbe
}

/// Drive the production outside-dismiss method (the backdrop press path).
pub fn portal_outside_dismiss() -> Request {
    Request::PortalOutsideDismiss
}

/// Deterministically tear down the one native portal host (terminal; the
/// snapshot afterwards reports the empty/closed host).
pub fn portal_teardown() -> Request {
    Request::PortalTeardown
}

/// Drive the parent's real AppKit fullscreen transition
/// (`toggleFullScreen:`); the E2E polls `parent_fullscreen` + frame sync.
pub fn portal_set_fullscreen(fullscreen: bool) -> Request {
    Request::PortalSetFullscreen { fullscreen }
}

/// Drive the parent's real AppKit minimize/restore (`miniaturize:` /
/// `deminiaturize:`).
pub fn portal_set_minimized(minimized: bool) -> Request {
    Request::PortalSetMinimized { minimized }
}

/// Drive a real Window Server space-assignment transition (set/clear the
/// `CanJoinAllSpaces` collection behavior).
pub fn portal_set_all_spaces(all_spaces: bool) -> Request {
    Request::PortalSetAllSpaces { all_spaces }
}

/// Close the parent browser window through the real AppKit close path
/// (`performClose:`) while a portal presentation is open.
pub fn portal_close_window() -> Request {
    Request::PortalCloseWindow
}

/// Open the find-in-page presentation over the active web view (the
/// production `Cmd-F` path; task packet `find-in-page-portal-bar`).
pub fn find_open() -> Request {
    Request::FindOpen
}

/// Close the find presentation through the production close path (Escape /
/// close-button semantics): restores the origin focus domain and clears
/// highlights on the bound view.
pub fn find_close() -> Request {
    Request::FindClose
}

/// Set the find query and start a fresh search (the typing/backspace path).
pub fn find_set_query(query: impl Into<String>) -> Request {
    Request::FindSetQuery {
        query: query.into(),
    }
}

/// Navigate to the next match (Enter / next-button path).
pub fn find_next() -> Request {
    Request::FindNext
}

/// Navigate to the previous match (Shift-Enter / previous-button path).
pub fn find_previous() -> Request {
    Request::FindPrevious
}

/// Zoom the active view's site one contract step up (queue item 11, part A;
/// the production `Cmd-Plus` seam).
pub fn zoom_in() -> Request {
    Request::ZoomIn
}

/// Zoom the active view's site one contract step down (the `Cmd-Minus`
/// seam).
pub fn zoom_out() -> Request {
    Request::ZoomOut
}

/// Reset the active view's site to the default zoom level (the `Cmd-0`
/// seam).
pub fn zoom_reset() -> Request {
    Request::ZoomReset
}

/// Bring the app to the foreground (testkit-only; the shown-harness
/// frontmost-evidence premise).
pub fn activate_app() -> Request {
    Request::ActivateApp
}

/// Dispatch one keystroke through GPUI's real input path (the production
/// key seam the command/find/auth suites drive).
pub fn keystroke(keystroke: impl Into<String>) -> Request {
    Request::DispatchKeystroke {
        keystroke: keystroke.into(),
    }
}

/// Present a deterministic authentication challenge through the production
/// challenge-arrival seam (`BbrApp::auth_on_challenge`, queue item 10): the
/// sheet opens over the active window. The fixture is an HTTPS origin Basic
/// challenge for `intranet.example` with realm `Finance VPN`, bound to the
/// owning `web_view`; it carries only authority metadata, never credentials.
pub fn auth_present_challenge(profile: ProfileId, web_view: WebViewId) -> Request {
    use bbr_web::auth_challenges::{AuthAuthority, AuthScheme, HttpScheme};
    Request::AuthPresentChallenge {
        profile,
        web_view,
        http_scheme: HttpScheme::Https,
        scheme: AuthScheme::Basic,
        host: "intranet.example".to_string(),
        port: 0,
        realm: Some("Finance VPN".to_string()),
        authority: AuthAuthority::Origin,
        user_initiated: true,
    }
}

/// Record a deterministic authentication challenge WITHOUT presenting the
/// sheet (the awaiting challenge the measured open and the staged
/// presentations use). Same fixture as [`auth_present_challenge`]; never
/// carries credentials.
pub fn auth_record_challenge(profile: ProfileId, web_view: WebViewId) -> Request {
    use bbr_web::auth_challenges::{AuthAuthority, AuthScheme, HttpScheme};
    Request::AuthRecordChallenge {
        profile,
        web_view,
        http_scheme: HttpScheme::Https,
        scheme: AuthScheme::Basic,
        host: "intranet.example".to_string(),
        port: 0,
        realm: Some("Finance VPN".to_string()),
        authority: AuthAuthority::Origin,
        user_initiated: true,
    }
}

pub fn click_space(space: TestSpaceId) -> Request {
    Request::ClickSpace { space }
}

/// Set the curated theme token of a space through the production controller
/// appearance authority (the explicit record becomes authoritative over the
/// derived profile-hue default).
pub fn set_space_theme(space: TestSpaceId, theme: impl Into<String>) -> Request {
    Request::SetSpaceTheme {
        space,
        theme: theme.into(),
    }
}

/// Force the effective production `ThemeMode` regardless of the host
/// appearance (task packet `system-appearance-theme`); `None` clears the
/// override and returns to the observed macOS system appearance.
pub fn set_appearance_override(mode: Option<ThemeModeOverride>) -> Request {
    Request::SetAppearanceOverride { mode }
}

pub fn set_accessibility_override(
    increase_contrast: Option<bool>,
    reduce_motion: Option<bool>,
) -> Request {
    Request::SetAccessibilityOverride {
        increase_contrast,
        reduce_motion,
    }
}

pub fn set_full_screen_override(full_screen: Option<bool>) -> Request {
    Request::SetFullScreenOverride { full_screen }
}

pub fn click_address_zoom() -> Request {
    Request::ClickAddressZoom
}

pub fn click_space_new() -> Request {
    Request::ClickSpaceNew
}

/// Drive one synthetic horizontal scroll-wheel event through the production
/// sidebar swipe handler (DESIGN.md §11.3): `delta` is the raw wheel delta
/// `[x, y]` in natural pixels, `phase` the touch phase. `started` begins a
/// tracking gesture when the delta is horizontal-dominant, `moved` feeds
/// progress, `ended` commits (beyond the threshold) or cancels (below it).
pub fn swipe_scroll(delta: [f32; 2], phase: super::SwipePhaseWire) -> Request {
    Request::SwipeScroll { delta, phase }
}

/// Explicitly cancel a tracking sidebar swipe through the production
/// cancel seam: no commit, the chrome tint returns to the exact source
/// presentation.
pub fn swipe_cancel() -> Request {
    Request::SwipeCancel
}

pub fn benchmark_activate_space(operation_id: BenchmarkOperationId, space: TestSpaceId) -> Request {
    Request::BenchmarkActivateSpace {
        operation_id,
        space,
    }
}

/// Create a space through the production create→activate→configure path and
/// measure until the configuration surface renders open bound to the exact
/// new SpaceId (packet `space-creation-appearance-ui.md`).
pub fn benchmark_create_space(
    operation_id: BenchmarkOperationId,
    name: impl Into<String>,
) -> Request {
    Request::BenchmarkCreateSpace {
        operation_id,
        name: name.into(),
    }
}

/// Open the configuration surface for the exact `space` and commit the
/// curated `identity` through the production `space_config_done` authority,
/// measuring until the surface is drawn closed with the bound space still
/// active (packet `space-creation-appearance-ui.md`).
pub fn benchmark_space_config_identity_update(
    operation_id: BenchmarkOperationId,
    space: TestSpaceId,
    identity: impl Into<String>,
) -> Request {
    Request::BenchmarkSpaceConfigIdentityUpdate {
        operation_id,
        space,
        identity: identity.into(),
    }
}

pub fn inject_web_event(event: bbr_web::WebEvent) -> Request {
    Request::InjectWebEvent { event }
}

/// Deterministic classified-incident injection (testkit-only): drives the
/// exact production `on_incident` seam the native delegate's incident sink
/// uses, for a closed contract subreason (task packet
/// `error-surface-contract`).
pub fn inject_error_incident(
    profile: ProfileId,
    view: WebViewId,
    reason: bbr_web::web_errors::IncidentSubReason,
    url: Option<&str>,
) -> Request {
    Request::InjectErrorIncident {
        profile,
        view,
        reason,
        url: url.map(str::to_owned),
    }
}

/// Measure the production error-surface presentation: the classified
/// incident enters through the same seam the native incident sink drives
/// and the sample commits from the render that drew the surface (warm
/// presentation class).
pub fn benchmark_error_surface_present(operation_id: BenchmarkOperationId) -> Request {
    Request::BenchmarkErrorSurfacePresent { operation_id }
}

/// Press the error surface's primary recovery action through the exact
/// production mouse-down seam (`BbrApp::error_recover`).
pub fn click_error_action() -> Request {
    Request::ClickErrorAction
}

/// Press the error surface's Dismiss button through the exact production
/// mouse-down seam (`BbrApp::error_dismiss`).
pub fn click_error_dismiss() -> Request {
    Request::ClickErrorDismiss
}

pub fn move_tab(index: usize, to_space: TestSpaceId) -> Request {
    Request::MoveTab { index, to_space }
}

pub fn suspend_tab(index: usize) -> Request {
    Request::SuspendTab { index }
}

pub fn back() -> Request {
    Request::WebAction(WebAction::Back)
}

pub fn forward() -> Request {
    Request::WebAction(WebAction::Forward)
}

pub fn reload() -> Request {
    Request::WebAction(WebAction::Reload)
}

pub fn stop() -> Request {
    Request::WebAction(WebAction::Stop)
}

/// The canonical ids used by the Phase-1 spike (one profile, one view).
pub fn test_ids() -> (ProfileId, WebViewId) {
    (ProfileId(1), WebViewId(1))
}

pub fn drag_sidebar_divider(dx: f32) -> Request {
    Request::DragSidebarDivider { dx }
}

pub fn focus_sidebar_divider() -> Request {
    Request::FocusSidebarDivider
}

pub fn benchmark_resize_sidebar(operation_id: BenchmarkOperationId, width: f32) -> Request {
    Request::BenchmarkResizeSidebar {
        operation_id,
        width,
    }
}

pub fn benchmark_restore_sidebar(operation_id: BenchmarkOperationId) -> Request {
    Request::BenchmarkRestoreSidebar { operation_id }
}

/// §11.14 titlebar navigation-control commit benchmark (one Back/Forward/
/// Reload dispatch through the production authority).
pub fn benchmark_navigation_control(operation_id: BenchmarkOperationId, control: &str) -> Request {
    Request::BenchmarkNavigationControlCommit {
        operation_id,
        control: control.into(),
    }
}

pub fn benchmark_hover_tab(operation_id: BenchmarkOperationId, index: usize) -> Request {
    Request::BenchmarkHoverTab {
        operation_id,
        index,
    }
}

pub fn benchmark_press_tab(operation_id: BenchmarkOperationId, index: usize) -> Request {
    Request::BenchmarkPressTab {
        operation_id,
        index,
    }
}
