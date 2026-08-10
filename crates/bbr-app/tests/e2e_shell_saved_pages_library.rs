//! Real-app Saved Pages Library E2E (B3 packet `saved-pages-library`).
//!
//! Launches the real `bbr` process and drives the integrated Library
//! surface through the production seams: the Cmd-Shift-L entry and Escape
//! dismissal, the deterministic search/sort, multi-select, the inline
//! pointer rename draft with Confirm/Cancel, the destructive remove
//! confirmation, the flat collection lifecycle, the selection toolbar and
//! collection action surface through their pointer paths, the production
//! scroll-wheel path, restart persistence, the tab/record independence
//! contract, and the 1k bounded-render window with
//! scroll/selection/long/missing fallbacks.
//!
//! Every mutation routes through the one app-owned Library composition and
//! the one AppStore/SavedPageCommand authority the typed snapshot exposes —
//! never a second persistence or identity policy. Opening a Library row
//! goes through the B2 command authority: exactly one fresh regular tab in
//! the currently active Space/Profile, never a silent selection.
//!
//! The tests live in split modules under `e2e_shell_saved_pages_library/`
//! so no file outgrows the ~500-line budget; this root holds the shared
//! harness helpers and the module wiring.
//!
//! No sleeps: every step synchronizes on typed snapshot state or semantic
//! events under an explicit deadline. GUI suites must not run in parallel
//! (AppKit window geometry), so every test takes the serial lock.

#![cfg(target_os = "macos")]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

#[path = "e2e_shell_saved_pages_library/support.rs"]
mod support;

#[path = "e2e_shell_saved_pages_library/activate.rs"]
mod activate;
#[path = "e2e_shell_saved_pages_library/collections.rs"]
mod collections;
#[path = "e2e_shell_saved_pages_library/entry.rs"]
mod entry;
#[path = "e2e_shell_saved_pages_library/one_k.rs"]
mod one_k;
#[path = "e2e_shell_saved_pages_library/pointer_collections.rs"]
mod pointer_collections;
#[path = "e2e_shell_saved_pages_library/pointer_toolbar.rs"]
mod pointer_toolbar;
#[path = "e2e_shell_saved_pages_library/pointer_wheel.rs"]
mod pointer_wheel;
#[path = "e2e_shell_saved_pages_library/rename.rs"]
mod rename;
#[path = "e2e_shell_saved_pages_library/restart.rs"]
mod restart;
#[path = "e2e_shell_saved_pages_library/search_sort.rs"]
mod search_sort;
#[path = "e2e_shell_saved_pages_library/select_remove.rs"]
mod select_remove;

pub use bbr_testkit::protocol::{CommandBarMode, SavedPagesLibrarySnapshot};
pub use e2e_common::*;

/// Dispatch one keystroke through the production GPUI input path.
pub(crate) fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

/// Save one canonical record through the one B2 save authority (the typed
/// seam Cmd-D shares; no navigation needed).
pub(crate) fn save(client: &mut Client, url: &str, title: &str) {
    client
        .send_ok(&bbr_testkit::Request::SaveSavedPage {
            url: url.into(),
            title: title.into(),
        })
        .expect("save saved page");
}

/// Open the Library through its production Cmd-Shift-L entry and wait for
/// the search field to own chrome focus.
pub(crate) fn open_library(client: &mut Client, since: &mut u64) {
    key(client, "cmd-shift-l");
    *since = wait_chrome_focus(client, true, *since);
    let snapshot = client.snapshot().expect("library open snapshot");
    assert!(snapshot.library.open, "Cmd-Shift-L opens the Library");
}

/// Open the Library through the typed production request (the same app
/// entry; used after the open contract is proven by the keystroke path).
pub(crate) fn open_library_typed(client: &mut Client, since: &mut u64) {
    client
        .send_ok(&bbr_testkit::Request::OpenSavedPagesLibrary)
        .expect("open library request");
    *since = wait_chrome_focus(client, true, *since);
    let snapshot = client.snapshot().expect("typed library open snapshot");
    assert!(snapshot.library.open, "the typed request opens the Library");
}

/// Close the Library through the production Escape path and wait for web
/// focus to return (the exact native view the Library hid is restored).
pub(crate) fn close_library_escape(client: &mut Client, since: &mut u64) {
    key(client, "escape");
    *since = wait_chrome_focus(client, false, *since);
    let snapshot = client.snapshot().expect("library close snapshot");
    assert!(!snapshot.library.open, "Escape closes the Library");
}

/// The active-space tab count plus the global tab count plus the live-
/// session count: the untouched tab/record evidence before and after
/// Library mutations.
pub(crate) fn tab_evidence(snapshot: &bbr_testkit::protocol::Snapshot) -> (usize, usize, usize) {
    (
        snapshot.tabs.len(),
        snapshot.all_tabs.len(),
        snapshot.engine_sessions,
    )
}

/// Wait for the initial fixture page title after spawn (the shared
/// preamble every suite test uses), returning the advanced event cursor.
pub(crate) fn wait_initial_title(client: &mut Client, since: &mut u64) {
    let first = client.snapshot().expect("first snapshot");
    if let Some(view) = first.active_web_view {
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            *since = wait_title(client, view, PAGE_ONE_TITLE, *since);
        }
    }
}
