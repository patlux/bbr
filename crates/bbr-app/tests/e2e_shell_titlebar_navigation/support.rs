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
/// path (the same bindings the shell installs: `cmd-s`, `cmd-[`, `cmd-]`,
/// `cmd-r`).
pub(super) fn keystroke(client: &mut Client, key: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: key.into(),
        })
        .expect("keystroke");
}

/// A reload can repeat the old title before commit, then clear it until the
/// document supplies its final title. Neither a queued title nor loading=false
/// alone proves this action completed. Require a fresh native load/commit/end
/// sequence and the final snapshot, all within one existing suite deadline.
pub(super) fn reload_and_wait(
    client: &mut Client,
    view: bbr_web::WebViewId,
    title: &str,
    request: bbr_testkit::Request,
) -> Snapshot {
    use bbr_testkit::protocol::{Event, Request, Response};

    let until = deadline();
    let Response::Replay { next_seq, .. } = client
        .request(&Request::Attach { since: None })
        .expect("pre-reload event cursor")
    else {
        panic!("expected event replay before reload");
    };
    let mut since = next_seq.saturating_sub(1);
    client.send_ok(&request).expect("reload action");
    for expected in [
        WebEvent::LoadingChanged(view, true),
        WebEvent::DidCommitNavigation(view),
        WebEvent::LoadingChanged(view, false),
    ] {
        since = client
            .wait_for_event_since(
                |event| matches!(event, Event::Web(web) if *web == expected),
                since,
                until,
            )
            .expect("fresh reload lifecycle event")
            .0;
    }
    loop {
        assert!(Instant::now() < until, "reload title did not settle");
        flush_frame(client);
        let snapshot = client.snapshot().expect("settled reload snapshot");
        if !snapshot.loading && snapshot.title.as_deref() == Some(title) {
            return snapshot;
        }
        std::thread::yield_now();
    }
}

/// The expected native content rect for a visible sidebar of `width` in a
/// window of `window_width` (the rail on the left, the WKWebView slot
/// filling the remainder at its own stable height — the macOS titlebar
/// makes the window-bounds height larger than the content NSView).
pub(super) fn content_rect_for(width: f32, window_width: f32, height: f32) -> [f32; 4] {
    [
        width,
        CONTENT_INSET,
        window_width - width - CONTENT_INSET,
        height,
    ]
}

/// DESIGN §2 `sp:content-inset` of the seated content card.
pub(super) const CONTENT_INSET: f32 = 10.0;

/// The collapsed card inner box: the leading inset plus the native
/// titlebar band above the card (§2 rule 5).
pub(super) fn collapsed_rect_for(window_width: f32, height: f32, band: f32) -> [f32; 4] {
    let band = band.max(CONTENT_INSET);
    [
        CONTENT_INSET,
        band,
        window_width - 2.0 * CONTENT_INSET,
        height - (band - CONTENT_INSET),
    ]
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
