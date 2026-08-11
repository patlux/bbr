//! `bbr-gallery` — the dev-only BBR Gallery (packet
//! `component-gallery-mvp.md`): a separate native GPUI app, keyboard-
//! operable component tree on the left, real `bbr-ui` story on the right
//! canvas with mode/state controls, plus the Core Controls stories that
//! render the exact vendored `gpui-component` 0.5.1 production controls.
//! Never packaged into the `.app`, never part of `just e2e`, no
//! WebKit/network/socket/portal/persistence.
//!
//! The gallery is **macOS-gated for v1** (parked decision 3 in
//! `ui-library-anchor.md`): the whole binary implementation, harness, and
//! E2E compile only on macOS. On every other target the crate evaluates to
//! an empty `main`, so the workspace check/clippy stays green without any
//! `allow(dead_code)`. `bbr-ui` itself remains portable.

#[cfg(target_os = "macos")]
mod address_app;
#[cfg(target_os = "macos")]
mod address_matrix;
#[cfg(target_os = "macos")]
mod address_recorder;
#[cfg(target_os = "macos")]
mod app;
#[cfg(target_os = "macos")]
mod auth_app;
#[cfg(target_os = "macos")]
mod auth_matrix;
#[cfg(target_os = "macos")]
mod auth_recorder;
#[cfg(target_os = "macos")]
mod canvas;
#[cfg(target_os = "macos")]
mod command_app;
#[cfg(target_os = "macos")]
mod command_fixture;
#[cfg(target_os = "macos")]
mod command_matrix;
#[cfg(target_os = "macos")]
mod command_recorder;
#[cfg(target_os = "macos")]
mod context;
#[cfg(target_os = "macos")]
mod context_menu_app;
#[cfg(target_os = "macos")]
mod context_menu_matrix;
#[cfg(target_os = "macos")]
mod context_menu_recorder;
#[cfg(target_os = "macos")]
mod control_observer;
#[cfg(target_os = "macos")]
mod controls;
#[cfg(target_os = "macos")]
mod downloads_app;
#[cfg(target_os = "macos")]
mod downloads_matrix;
#[cfg(target_os = "macos")]
mod downloads_panels;
#[cfg(target_os = "macos")]
mod downloads_recorder;
#[cfg(target_os = "macos")]
mod error_app;
#[cfg(target_os = "macos")]
mod error_matrix;
#[cfg(target_os = "macos")]
mod error_panel_matrix;
#[cfg(target_os = "macos")]
mod error_recorder;
#[cfg(target_os = "macos")]
mod favorites_matrix;
#[cfg(target_os = "macos")]
mod find_app;
#[cfg(target_os = "macos")]
mod find_matrix;
#[cfg(target_os = "macos")]
mod find_recorder;
#[cfg(target_os = "macos")]
mod fixtures;
#[cfg(target_os = "macos")]
mod folders_matrix;
#[cfg(target_os = "macos")]
mod footer_matrix;
#[cfg(target_os = "macos")]
mod group4_app;
#[cfg(target_os = "macos")]
mod group4_recorder;
#[cfg(target_os = "macos")]
mod library_fixtures;
#[cfg(target_os = "macos")]
mod library_observer;
#[cfg(target_os = "macos")]
mod library_recorder;
#[cfg(target_os = "macos")]
mod menu_matrix;
#[cfg(target_os = "macos")]
mod profile_chip_matrix;
#[cfg(target_os = "macos")]
mod recorder;
#[cfg(target_os = "macos")]
mod registry;
#[cfg(target_os = "macos")]
mod sidebar_app;
#[cfg(target_os = "macos")]
mod sidebar_fixture;
#[cfg(target_os = "macos")]
mod sidebar_matrix;
#[cfg(target_os = "macos")]
mod sidebar_observer;
#[cfg(target_os = "macos")]
mod sidebar_recorder;
#[cfg(target_os = "macos")]
mod snapshot;
#[cfg(target_os = "macos")]
mod stories;
#[cfg(target_os = "macos")]
mod tab_row_app;
#[cfg(target_os = "macos")]
mod tab_row_matrix;
#[cfg(target_os = "macos")]
mod tree;
#[cfg(target_os = "macos")]
mod tree_surface;

#[cfg(test)]
mod dependency_contract;
#[cfg(all(test, target_os = "macos"))]
mod e2e;
#[cfg(all(test, target_os = "macos"))]
mod harness;

#[cfg(target_os = "macos")]
use gpui::{px, size, App, AppContext, Bounds, Entity, WindowBounds, WindowOptions};

/// The gallery window title — the visible app copy (finding 3).
#[cfg(target_os = "macos")]
pub const GALLERY_WINDOW_TITLE: &str =
    "BBR Gallery — Foundations / Primitives / Browser Components / Compositions";

/// The gallery window options — shared by the production app and the
/// in-process test harness so both open the same window (bounds, focus,
/// show).
#[cfg(target_os = "macos")]
pub fn window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1200.0), px(800.0)),
            cx,
        ))),
        focus: true,
        show: true,
        ..Default::default()
    }
}

/// Open the gallery window the production way: `gpui_component::init` has
/// run, the window root is `gpui_component::Root` wrapping the app entity
/// (the exact integration pattern the Core Controls require), and the
/// window title carries the honest scope copy. Returns the window handle
/// and the app entity (the harness drives both).
#[cfg(target_os = "macos")]
pub fn open_window(
    cx: &mut App,
) -> (
    gpui::WindowHandle<gpui_component::Root>,
    Entity<app::GalleryApp>,
) {
    let options = window_options(cx);
    let mut entity: Option<Entity<app::GalleryApp>> = None;
    let window = cx
        .open_window(options, |window, cx| {
            let app_entity = cx.new(app::GalleryApp::new);
            let handle = app_entity.clone();
            entity = Some(handle);
            cx.new(|cx| gpui_component::Root::new(app_entity, window, cx))
        })
        .expect("open gallery window");
    let _ = window.update(cx, |_, window, _| {
        window.set_window_title(GALLERY_WINDOW_TITLE)
    });
    (window, entity.expect("gallery app entity"))
}

#[cfg(target_os = "macos")]
fn main() {
    let application = gpui_platform::application();
    application.run(move |cx: &mut App| {
        // The production gpui-component integration pattern the Core
        // Controls stories need: the theme/global state and the Root
        // window wrapper.
        gpui_component::init(cx);
        let (_window, _entity) = open_window(cx);
        cx.activate(true);
    });
}

/// Non-macOS targets: the gallery is a macOS-only dev tool for v1 (parked
/// decision 3). The crate evaluates to an empty binary so the workspace
/// stays green without lint allows.
#[cfg(not(target_os = "macos"))]
fn main() {}
