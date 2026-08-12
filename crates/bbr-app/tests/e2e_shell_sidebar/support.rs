use super::*;
use bbr_testkit::protocol::Snapshot;

/// DESIGN §5 adjustable-range contract (mirrors
/// `bbr_ui::sidebar::geometry` constants the snapshot exposes; the E2E
/// asserts the same numbers the render enforces).
pub(super) const SIDEBAR_MIN: f32 = 200.0;
pub(super) const SIDEBAR_DEFAULT: f32 = 300.0;
pub(super) const SIDEBAR_MAX: f32 = 400.0;
/// The native-content floor: the WKWebView slot is never laid out narrower.
pub(super) const CONTENT_MIN: f32 = 64.0;
/// Keyboard resize step for the focused divider (DESIGN §9).
pub(super) const RESIZE_STEP: f32 = 16.0;

pub(super) fn keystroke(client: &mut Client, key: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: key.into(),
        })
        .expect("keystroke");
}

/// Draw one real GPUI frame synchronously so the paint-driven native
/// geometry callbacks fire (the hidden harness has no display link).
pub(super) fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// The expected native content rect for a visible sidebar of `width` in a
/// window of `window_width`: the rail on the left, the WKWebView slot filling
/// the remainder at the slot's own stable height (the macOS titlebar makes
/// the window-bounds height larger than the content NSView, so the slot
/// height is captured once and asserted for drift, never against bounds).
pub(super) fn content_rect_for(width: f32, window_width: f32, height: f32) -> [f32; 4] {
    [width, 0.0, window_width - width, height]
}

pub(super) fn assert_view_and_sessions_preserved(
    snapshot: &Snapshot,
    view: bbr_web::WebViewId,
    sessions: usize,
) {
    assert_eq!(
        snapshot.active_web_view,
        Some(view),
        "sidebar mutations must never change the selected WebViewId"
    );
    assert_eq!(
        snapshot.engine_sessions, sessions,
        "sidebar mutations must never recreate or destroy sessions"
    );
}
