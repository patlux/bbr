//! Blueprint D16 (boards L-3…L-9): the Settings window — a separate,
//! fixed-size macOS window (⌘, / BBR › Settings…) that renders the app's
//! Settings model and hands every interaction back to `BbrApp`, the one
//! authority. It follows the app (observes the entity), so a change made
//! anywhere shows at once; closing it only forgets the window.

use gpui::{
    point, prelude::*, px, size, App, Bounds, Context, Entity, Subscription, TitlebarOptions,
    Window, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};

use bbr_ui::surfaces::settings::{settings_surface, SettingsCallback};

use crate::app::BbrApp;

/// The window size (boards: 780 × 620).
pub const SETTINGS_SIZE: [f32; 2] = [780.0, 620.0];

/// The Settings window's root view.
pub struct SettingsView {
    app: Entity<BbrApp>,
    focus: gpui::FocusHandle,
    _observe: Subscription,
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.app.read(cx).settings_model();
        window.set_window_title(model.pane.title());
        let app = self.app.clone();
        let events: SettingsCallback = std::rc::Rc::new(move |event, window, cx| {
            app.update(cx, |view, cx| {
                view.on_settings_event(event, Some(window), cx)
            });
        });
        settings_surface(&model, &self.focus, events)
    }
}

/// Blueprint D19: the Settings window follows another browser window's
/// shell once the one it was opened from closed (the Settings state
/// itself is shared).
pub fn rebind(handle: WindowHandle<SettingsView>, app: Entity<BbrApp>, cx: &mut App) {
    let _ = handle.update(cx, |view, _window, cx| {
        view._observe = cx.observe(&app, |_view, _app, cx| cx.notify());
        view.app = app;
        cx.notify();
    });
}

/// Open the Settings window (or bring the open one forward). `visible` is
/// false for the hidden test harness.
pub fn open(
    existing: Option<WindowHandle<SettingsView>>,
    app: Entity<BbrApp>,
    visible: bool,
    cx: &mut App,
) -> Option<WindowHandle<SettingsView>> {
    if let Some(handle) = existing {
        if handle
            .update(cx, |_view, window, _cx| window.activate_window())
            .is_ok()
        {
            return Some(handle);
        }
    }
    let bounds = Bounds::centered(None, size(px(SETTINGS_SIZE[0]), px(SETTINGS_SIZE[1])), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Settings".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(12.0), px(12.0))),
        }),
        focus: visible,
        show: visible,
        kind: WindowKind::Normal,
        is_resizable: false,
        ..WindowOptions::default()
    };
    let handle = cx
        .open_window(options, |window, cx| {
            cx.new(|cx| {
                let observe = cx.observe(&app, |_view, _app, cx| cx.notify());
                let focus = cx.focus_handle();
                window.focus(&focus, cx);
                SettingsView {
                    app: app.clone(),
                    focus,
                    _observe: observe,
                }
            })
        })
        .map_err(|error| log::error!("could not open Settings: {error}"))
        .ok()?;
    // Closing the window (its close button or the testkit) forgets it.
    let settings_id = handle.window_id();
    cx.on_window_closed(move |cx, closed| {
        if closed == settings_id {
            app.update(cx, |view, cx| {
                view.settings_closed();
                cx.notify();
            });
        }
    })
    .detach();
    Some(handle)
}
