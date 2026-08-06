//! Vertical tab row presentation (DESIGN.md §4, §10, §11.2).
//!
//! The one-line title budget and separator-aware truncation input live here
//! (the pixel-exact cut stays in the component), the §10 visual row states,
//! and the pinned/regular region boundary. All derive from the same snapshot
//! source the sidebar renders.

use crate::theme::{Spacing, Theme};

/// DESIGN.md §4 one-line row budget: the title area is the sidebar width
/// minus the fixed row anatomy (2× row padding, the favicon tile, 3
/// inter-item gaps, the 8 pt state slot, and the close control). Per-char
/// width is a conservative 0.45 × the `type:title` size (SF Pro averages
/// ≈0.5 em), so [`crate::shell::presentation::truncate_title`] never fires
/// for a title that would fit: GPUI's `text_ellipsis` stays the pixel-exact
/// backstop for the visual cut.
pub fn title_budget_chars(theme: &Theme) -> usize {
    let density = theme.density;
    let padding = f32::from(Spacing::sp2()) * 2.0;
    let favicon = f32::from(density.icon());
    let gaps = f32::from(Spacing::sp2()) * 3.0;
    let state_slot = 8.0;
    let close = f32::from(density.icon());
    let fixed = padding + favicon + gaps + state_slot + close;
    let area = f32::from(density.sidebar_width()) - fixed;
    let per_char = f32::from(theme.type_.title.size) * 0.45;
    (area / per_char).floor().max(1.0) as usize
}

/// DESIGN.md §10 row state for the E2E snapshot's `tab_states` field:
/// error (crash) wins over loading; loading wins over selected; the active
/// row renders `selected`; everything else is `default`.
pub fn row_state(active: bool, loading: bool, crashed: bool) -> crate::components::TabVisualState {
    if crashed {
        return crate::components::TabVisualState::Error;
    }
    if loading {
        return crate::components::TabVisualState::Loading;
    }
    if active {
        return crate::components::TabVisualState::Selected;
    }
    crate::components::TabVisualState::Default
}

/// Index of the first non-pinned tab — the pinned/regular region boundary
/// (DESIGN.md §11.2). The pinned region renders only when it has rows.
pub fn pinned_count(tabs: &[bbr_browser::TabSnapshot]) -> usize {
    tabs.iter().position(|t| !t.pinned).unwrap_or(tabs.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_state_ordering_is_error_then_loading_then_selected() {
        assert_eq!(
            row_state(true, false, true),
            crate::components::TabVisualState::Error
        );
        assert_eq!(
            row_state(true, true, false),
            crate::components::TabVisualState::Loading
        );
        assert_eq!(
            row_state(true, false, false),
            crate::components::TabVisualState::Selected
        );
        assert_eq!(
            row_state(false, false, false),
            crate::components::TabVisualState::Default
        );
        assert_eq!(
            row_state(false, true, true),
            crate::components::TabVisualState::Error
        );
    }

    #[test]
    fn pinned_count_separates_regions() {
        let snap = |index: usize, pinned: bool| bbr_browser::TabSnapshot {
            index,
            url: "https://example.com".into(),
            title: format!("tab {index}"),
            pinned,
            loading: false,
            crashed: false,
            active: index == 0,
            web_view: None,
            lifecycle: bbr_browser::TabLifecycle::Warm,
            last_used: 0,
        };
        let tabs = vec![snap(0, true), snap(1, true), snap(2, false)];
        assert_eq!(pinned_count(&tabs), 2);
        // No pinned rows → boundary at zero.
        let tabs = vec![snap(0, false)];
        assert_eq!(pinned_count(&tabs), 0);
    }

    #[test]
    fn title_budget_tracks_density_geometry() {
        let normal = Theme::dark();
        let small = Theme::new(
            crate::theme::ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::Small,
        );
        let xl = Theme::new(
            crate::theme::ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::ExtraLarge,
        );
        // Normal density ships the 260 pt sidebar: ~30 chars at 13 pt title.
        assert!(title_budget_chars(&normal) >= 25);
        assert!(title_budget_chars(&normal) <= 40);
        // Wider sidebars get a larger budget than narrow ones.
        assert!(title_budget_chars(&xl) > title_budget_chars(&small));
    }
}
