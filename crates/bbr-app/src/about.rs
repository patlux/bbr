//! Blueprint D18 (board L-12, DESIGN §1): the About BBR window — the brand
//! mark, the name and the version. BBR has no update channel yet (DESIGN
//! §19 question on D18), so the window states no update status rather than
//! claiming one; the update half of the board stays open.

use gpui::{
    div, point, prelude::*, px, size, App, Bounds, Context, TitlebarOptions, Window, WindowBounds,
    WindowHandle, WindowKind, WindowOptions,
};

use bbr_ui::icons::Icon;
use bbr_ui::theme::Theme;

/// The §15 ids of the About window.
pub const ABOUT_ID: &str = "bbr-about";
pub const ABOUT_VERSION_ID: &str = "bbr-about-version";

/// The About line: "Version 0.1.0".
pub fn version_line() -> String {
    format!("Version {}", env!("CARGO_PKG_VERSION"))
}

/// The About window's root view.
pub struct AboutView {
    theme: Theme,
}

impl Render for AboutView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let p = theme.palette;
        div()
            .id(ABOUT_ID)
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(56.0))
            .gap(px(10.0))
            .bg(p.surface_canvas)
            .child(
                div()
                    .size(px(88.0))
                    .rounded(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(p.accent)
                    .child(Icon::ChassisMark.canvas_with(&theme, px(56.0), p.on_accent)),
            )
            .child(
                div()
                    .text_size(theme.type_.display.size)
                    .font_weight(theme.type_.display.weight)
                    .text_color(p.ink)
                    .child("BBR"),
            )
            .child(
                div()
                    .id(ABOUT_VERSION_ID)
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child(version_line()),
            )
    }
}

/// Open the About window (or bring the open one forward). `visible` is
/// false for the hidden test harness.
pub fn open(
    existing: Option<WindowHandle<AboutView>>,
    theme: Theme,
    visible: bool,
    cx: &mut App,
) -> Option<WindowHandle<AboutView>> {
    if let Some(handle) = existing {
        if handle
            .update(cx, |_view, window, _cx| window.activate_window())
            .is_ok()
        {
            return Some(handle);
        }
    }
    let bounds = Bounds::centered(None, size(px(340.0), px(380.0)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("About BBR".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(12.0), px(12.0))),
        }),
        focus: visible,
        show: visible,
        kind: WindowKind::Normal,
        is_resizable: false,
        is_minimizable: false,
        ..WindowOptions::default()
    };
    cx.open_window(options, |_window, cx| cx.new(|_cx| AboutView { theme }))
        .map_err(|error| log::error!("could not open About BBR: {error}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_line_names_the_workspace_version() {
        assert_eq!(
            version_line(),
            format!("Version {}", env!("CARGO_PKG_VERSION"))
        );
    }
}
