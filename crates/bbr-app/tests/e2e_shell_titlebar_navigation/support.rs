//! Titlebar-navigation E2E helpers (task packet
//! `titlebar-navigation-controls.md`): the shared harness and the §5/§11.14
//! geometry constants, mirroring the suite conventions of the sidebar and
//! split-geometry suites.

use super::*;
use bbr_testkit::protocol::Snapshot;

/// The DESIGN §5 default rail width (mirrors
/// `bbr_ui::sidebar::geometry::SIDEBAR_DEFAULT`).
pub(super) const SIDEBAR_DEFAULT: f32 = 300.0;
/// Draw one real GPUI frame synchronously so the paint-driven native
/// geometry callbacks fire (the hidden harness has no display link).
pub(super) fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// Drive one production keyboard shortcut through the real GPUI action
/// path (the same bindings the shell installs: `cmd-b`, `cmd-[`, `cmd-]`,
/// `cmd-r`).
pub(super) fn keystroke(client: &mut Client, key: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: key.into(),
        })
        .expect("keystroke");
}

/// The expected native content rect for a visible sidebar of `width` in a
/// window of `window_width` (the rail on the left, the WKWebView slot
/// filling the remainder at its own stable height — the macOS titlebar
/// makes the window-bounds height larger than the content NSView).
pub(super) fn content_rect_for(width: f32, window_width: f32, height: f32) -> [f32; 4] {
    [width, 0.0, window_width - width, height]
}

/// Titlebar interactions must never change the selected WebViewId or the
/// live session count (they route through the same authorities as the
/// shortcuts, which are session-preserving by contract).
pub(super) fn assert_view_and_sessions_preserved(
    snapshot: &Snapshot,
    view: bbr_web::WebViewId,
    sessions: usize,
) {
    assert_eq!(
        snapshot.active_web_view,
        Some(view),
        "titlebar mutations must never change the selected WebViewId"
    );
    assert_eq!(
        snapshot.engine_sessions, sessions,
        "titlebar mutations must never recreate or destroy sessions"
    );
}
