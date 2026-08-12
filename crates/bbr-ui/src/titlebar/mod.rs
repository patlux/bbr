//! The §11.14 unified titlebar navigation controls (packet
//! `titlebar-navigation-controls.md`): the plain [`TitlebarModel`] DTO and
//! the one [`TitlebarCallbacks`] set, the [`render::titlebar_surface`] row
//! composition, and the concrete forwarding closures. The row lives at the
//! top of the expanded full-height Sidebar rail inside the actual macOS
//! safe-titlebar layout guide; AppKit details (the native window query)
//! stay in the macOS composition edge.
//!
//! No browser/session/navigation authority lives here: the model carries
//! the genuine enablement of the focused pane and the callbacks route
//! through the existing app authorities (Cmd-B / Cmd-[ / Cmd-] / Cmd-R).

pub mod callbacks;
pub mod model;
pub mod render;

pub use model::{SafeArea, TitlebarCallbacks, TitlebarControl, TitlebarModel, TITLEBAR_ID};
pub use render::{
    titlebar_drag_spacer, titlebar_surface, NAV_GAP, TITLEBAR_GLYPH, TITLEBAR_HIT, TRAFFIC_GAP,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_ids_are_the_stable_15_contract() {
        assert_eq!(TitlebarControl::Toggle.id(), "bbr-titlebar-sidebar-toggle");
        assert_eq!(TitlebarControl::Back.id(), "bbr-nav-back");
        assert_eq!(TitlebarControl::Forward.id(), "bbr-nav-forward");
        assert_eq!(TitlebarControl::Reload.id(), "bbr-nav-reload");
        assert_eq!(TITLEBAR_ID, "bbr-titlebar");
    }

    #[test]
    fn safe_area_control_left_clears_the_traffic_lights() {
        // A traffic-light region at [20, 11, 14, 14] puts the controls at
        // the region's right edge — never a hardcoded x offset.
        let area = SafeArea {
            height: 28.0,
            traffic_lights: [20.0, 11.0, 14.0, 14.0],
        };
        assert_eq!(area.control_left(), 34.0);
        // The portable fallback starts at 0 (no traffic lights).
        assert_eq!(SafeArea::none().control_left(), 0.0);
        assert_eq!(SafeArea::none().height, 0.0);
    }

    #[test]
    fn hit_targets_meet_the_minimum_density() {
        assert!(TITLEBAR_HIT >= 32.0, "hit areas are at least 32 pt");
        assert_eq!(TITLEBAR_GLYPH, 18.0, "the §7 18 pt geometric family");
    }
}
