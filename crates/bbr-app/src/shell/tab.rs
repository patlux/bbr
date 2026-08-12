//! Vertical tab row presentation (DESIGN.md §4, §10, §11.2).
//!
//! The one-line title budget and separator-aware truncation input live here
//! (the pixel-exact cut stays in the component), the §10 visual row states,
//! and the pinned/regular region boundary. All derive from the same snapshot
//! source the sidebar renders.

use bbr_ui::theme::{Spacing, Theme};

/// DESIGN.md §4 one-line row budget: the title area is the sidebar width
/// minus the fixed row anatomy (2× `density:row-padding-x`, the favicon
/// tile, 3 inter-item gaps, the `density:state-slot`, and the
/// `density:hit-min.close` close hit square). Per-char width is a
/// conservative 0.45 × the `type:title` size (SF Pro averages ≈0.5 em), so
/// [`bbr_ui::text::truncate_title`] never fires for a title
/// that would fit: GPUI's `text_ellipsis` stays the pixel-exact backstop
/// for the visual cut.
pub fn title_budget_chars(theme: &Theme) -> usize {
    let density = theme.density;
    let padding = f32::from(density.row_padding_x()) * 2.0;
    let favicon = f32::from(density.icon());
    let gaps = f32::from(Spacing::sp2()) * 3.0;
    let state_slot = f32::from(density.state_slot());
    let close = f32::from(density.hit_min_close());
    let fixed = padding + favicon + gaps + state_slot + close;
    let area = f32::from(density.sidebar_width()) - fixed;
    let per_char = f32::from(theme.type_.title.size) * 0.45;
    (area / per_char).floor().max(1.0) as usize
}

/// DESIGN.md §10 row state for the E2E snapshot's `tab_states` field.
/// Composition (the more specific state wins, §10) is: error (crash), then
/// unresponsive (§10.11 — a sibling of the crash authority; the session
/// stays alive), then loading, then pressed (`active`), then selected,
/// then hover, then default. The `active` press wash wins over `selected`
/// so the pointer-down wash is visible on the row being pressed, and
/// `hover` never overrides an already-selected or pressed row.
pub fn row_state(
    active: bool,
    loading: bool,
    crashed: bool,
    unresponsive: bool,
    pressed: bool,
    hovered: bool,
) -> bbr_ui::state::TabVisualState {
    if crashed {
        return bbr_ui::state::TabVisualState::Error;
    }
    if unresponsive {
        return bbr_ui::state::TabVisualState::Unresponsive;
    }
    if loading {
        return bbr_ui::state::TabVisualState::Loading;
    }
    if pressed {
        return bbr_ui::state::TabVisualState::Active;
    }
    if active {
        return bbr_ui::state::TabVisualState::Selected;
    }
    if hovered {
        return bbr_ui::state::TabVisualState::Hover;
    }
    bbr_ui::state::TabVisualState::Default
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
    fn row_state_ordering_is_error_then_unresponsive_then_loading_then_pressed() {
        assert_eq!(
            row_state(true, false, true, false, false, false),
            bbr_ui::state::TabVisualState::Error
        );
        // Unresponsive is a sibling of the crash authority: a crashed tab
        // never renders unresponsive (the crash destroyed the session), and
        // an unresponsive tab beats loading (a hung renderer shows the
        // lifering + clock, not the spinner).
        assert_eq!(
            row_state(true, true, false, true, false, false),
            bbr_ui::state::TabVisualState::Unresponsive
        );
        assert_eq!(
            row_state(true, true, false, false, false, false),
            bbr_ui::state::TabVisualState::Loading
        );
        // A pressed row shows the §10.3 active wash even when it is (or
        // just became) selected; hover never overrides press or selection.
        assert_eq!(
            row_state(true, false, false, false, true, true),
            bbr_ui::state::TabVisualState::Active
        );
        assert_eq!(
            row_state(true, false, false, false, false, true),
            bbr_ui::state::TabVisualState::Selected
        );
        assert_eq!(
            row_state(false, false, false, false, true, false),
            bbr_ui::state::TabVisualState::Active
        );
        assert_eq!(
            row_state(false, false, false, false, false, true),
            bbr_ui::state::TabVisualState::Hover
        );
        assert_eq!(
            row_state(false, false, false, false, false, false),
            bbr_ui::state::TabVisualState::Default
        );
        assert_eq!(
            row_state(false, true, true, false, false, false),
            bbr_ui::state::TabVisualState::Error
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
            unresponsive: false,
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
            bbr_ui::theme::ThemeMode::Dark,
            bbr_ui::theme::ContrastMode::Standard,
            bbr_ui::theme::Density::Small,
        );
        let xl = Theme::new(
            bbr_ui::theme::ThemeMode::Dark,
            bbr_ui::theme::ContrastMode::Standard,
            bbr_ui::theme::Density::ExtraLarge,
        );
        // Normal density ships the 300 pt sidebar: ~29 chars at 14 pt title.
        assert!(title_budget_chars(&normal) >= 25);
        assert!(title_budget_chars(&normal) <= 40);
        // Wider sidebars get a larger budget than narrow ones.
        assert!(title_budget_chars(&xl) > title_budget_chars(&small));
    }
}
