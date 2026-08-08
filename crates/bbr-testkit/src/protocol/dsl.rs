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

pub fn click_footer_downloads() -> Request {
    Request::ClickFooterDownloads
}

pub fn click_footer_spaces() -> Request {
    Request::ClickFooterSpaces
}

pub fn click_footer_plus() -> Request {
    Request::ClickFooterPlus
}

pub fn click_plus_menu_item(index: usize) -> Request {
    Request::ClickPlusMenuItem { index }
}

pub fn toggle_favorite_tab(index: usize) -> Request {
    Request::ToggleFavoriteTab { index }
}

pub fn click_favorite_tile(index: usize) -> Request {
    Request::ClickFavoriteTile { index }
}

pub fn click_folder_row(index: usize) -> Request {
    Request::ClickFolderRow { index }
}

pub fn click_folder_toggle(index: usize) -> Request {
    Request::ClickFolderToggle { index }
}

pub fn click_folder_member(folder: usize, member: usize) -> Request {
    Request::ClickFolderMember { folder, member }
}

pub fn click_folder_member_close(folder: usize, member: usize) -> Request {
    Request::ClickFolderMemberClose { folder, member }
}

pub fn move_tab_into_folder(index: usize, folder: usize) -> Request {
    Request::MoveTabIntoFolder { index, folder }
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

pub fn begin_tab_drag(index: usize, y: f32, press_in_row: f32) -> Request {
    Request::BeginTabDrag {
        index,
        y,
        press_in_row,
    }
}

pub fn move_tab_drag(dy: f32) -> Request {
    Request::MoveTabDrag { dy }
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

pub fn click_space(space: TestSpaceId) -> Request {
    Request::ClickSpace { space }
}

pub fn click_space_new() -> Request {
    Request::ClickSpaceNew
}

pub fn benchmark_activate_space(operation_id: BenchmarkOperationId, space: TestSpaceId) -> Request {
    Request::BenchmarkActivateSpace {
        operation_id,
        space,
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
