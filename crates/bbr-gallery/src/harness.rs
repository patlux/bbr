//! The in-process gallery test harness (packet `component-gallery-mvp.md`,
//! live-review finding 1/2): GPUI's established main-thread test support
//! (`TestAppContext` + `VisualTestContext` on `TestPlatform`) opens the
//! gallery window on the test thread — no real window server, no real main
//! thread — with the **production gpui-component integration pattern**
//! (`gpui_component::init` + a `gpui_component::Root` window wrapper, the
//! exact shape the Core Controls require). The harness focuses the tree,
//! dispatches real keyboard events and real pointer clicks through the
//! production paths, forces draws (proving every story renders without
//! panic), and reads the semantic snapshot. No network, no listener, no
//! fixture server, no bbr-testkit socket.

use gpui::{
    AppContext, Bounds, Entity, Focusable, Modifiers, Pixels, Point, TestAppContext,
    VisualTestContext,
};

use crate::app::{GalleryApp, GallerySnapshot};

/// Run a gallery scenario in-process through GPUI's test platform with the
/// production gpui-component integration: initialize gpui_component, open
/// the gallery window with a `gpui_component::Root` wrapper (the same
/// pattern the production app and the controls require), focus the tree,
/// and run the scenario with the window context and the app entity.
pub fn run_scenario(scenario: impl FnOnce(&mut VisualTestContext, &Entity<GalleryApp>)) {
    let mut cx = TestAppContext::single();
    cx.update(gpui_component::init);
    let mut app_entity: Option<Entity<GalleryApp>> = None;
    let (_root, window_cx) = cx.add_window_view(|window, cx| {
        let app = cx.new(GalleryApp::new);
        app_entity = Some(app.clone());
        gpui_component::Root::new(app, window, cx)
    });
    // The window title carries the honest scope copy — the same title the
    // production `open_window` sets (finding 3).
    window_cx.update(|window, _cx| {
        window.set_window_title(crate::GALLERY_WINDOW_TITLE);
    });
    // Focus the tree (the app's one focus handle) so dispatched keys reach
    // the production key handler.
    window_cx.update(|window, cx| {
        let handle = app_entity
            .as_ref()
            .expect("app built with the window root")
            .read(cx)
            .focus_handle(cx);
        window.focus(&handle, cx);
    });
    window_cx.run_until_parked();
    let app = app_entity.as_ref().expect("app built with the window root");
    scenario(window_cx, app);
}

/// Read the app's semantic snapshot through the window context.
pub fn snapshot(window: &mut VisualTestContext, app: &Entity<GalleryApp>) -> GallerySnapshot {
    window.read(|cx| app.read(cx).snapshot(cx)).clone()
}

/// Dispatch a keystroke through the production key-dispatch path (real
/// keyboard event; printable keys go through the simulated-IME input
/// handler, exactly like real typing).
pub fn dispatch(window: &mut VisualTestContext, keystroke: &str) {
    window.simulate_keystrokes(keystroke);
}

/// Type plain text through the production simulated-IME input path:
/// `simulate_input` dispatches every character — including spaces — with
/// `key_char` set, so the focused input handler receives the exact text
/// through `dispatch_keystroke` → `input_handler.dispatch_input`, the same
/// path real typing takes. (`simulate_keystrokes` splits on whitespace and
/// would silently drop spaces.)
pub fn type_text(window: &mut VisualTestContext, text: &str) {
    window.simulate_input(text);
}

/// Force a draw of the window: proves the current story renders without
/// panicking (the paint path runs the render) and populates the debug
/// bounds map for the real-pointer scenarios.
pub fn flush(window: &mut VisualTestContext) {
    window.update(|window, cx| {
        window.draw(cx).clear();
    });
}

/// The on-screen bounds of a `debug_selector` target after a draw — the
/// real-pointer scenarios click these.
pub fn bounds_of(window: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    window
        .debug_bounds(selector)
        .expect("debug selector was rendered")
}

/// A real pointer click at the center of the given `debug_selector`
/// target (mouse down + mouse up through the production event path).
pub fn click_selector(window: &mut VisualTestContext, selector: &'static str) {
    let bounds = bounds_of(window, selector);
    let center: Point<Pixels> = bounds.center();
    window.simulate_click(center, Modifiers::none());
}

/// Navigate the tree to the story with the given id using **only real
/// keyboard events** (Home, then ↓ per flattened node, then Enter). All
/// groups start expanded, so the visible order equals the declaration
/// order.
pub fn move_to_story(window: &mut VisualTestContext, app: &Entity<GalleryApp>, story_id: u64) {
    let index = snapshot(window, app)
        .tree_nodes
        .iter()
        .position(|node| node.story_id == Some(story_id))
        .expect("story node registered");
    dispatch(window, "home");
    for _ in 0..index {
        dispatch(window, "down");
    }
    dispatch(window, "enter");
}
