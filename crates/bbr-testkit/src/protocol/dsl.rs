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
