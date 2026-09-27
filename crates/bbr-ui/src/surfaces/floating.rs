//! The shared v2 floating-surface anatomy (DESIGN.md §18.4): the 1 pt card
//! edge, the 4 pt menu padding, the inset `tint.accent` cursor that
//! replaced the 2 pt accent rail, and the 1 pt/2 pt field edge. Every menu,
//! panel and field consumes these instead of authoring its own edge, so the
//! family cannot drift apart again.

use gpui::{Pixels, Rgba, Styled};

use crate::theme::{Radius, Spacing, Theme};

/// §18.4: the floating card's 1 pt edge colour — `hairline`, or `accent`
/// while the surface owns keyboard focus (the §9 cue for menus).
pub fn edge(theme: &Theme, focused: bool) -> Rgba {
    if focused {
        theme.palette.accent
    } else {
        theme.palette.hairline
    }
}

/// §18.4: menus pad their rows by 4 pt on every side, so the inset cursor
/// never touches the card edge.
pub fn menu_padding() -> Pixels {
    Spacing::sp1()
}

/// §18.4: the menu/list cursor — an inset row surface (`radius:sm`) filled
/// with `tint.accent`. No accent rail: the accent marks one thing at a time
/// (§3.2) and the fill already carries the cursor.
pub fn menu_cursor<E: Styled>(el: E, theme: &Theme) -> E {
    el.bg(theme.palette.tint_accent).rounded(Radius.small())
}

/// §9/§18.4: a text field inside a floating surface — 1 pt `hairline` at
/// rest, the 2 pt `accent` ring while focused.
pub fn field_edge<E: Styled>(el: E, theme: &Theme, focused: bool) -> E {
    if focused {
        el.border_2().border_color(theme.palette.accent)
    } else {
        el.border_1().border_color(theme.palette.hairline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, ThemeMode};

    #[test]
    fn edges_follow_focus_and_padding_is_four_points() {
        let theme = Theme::new(ThemeMode::Light, ContrastMode::Standard, Density::Normal);
        assert_eq!(edge(&theme, false), theme.palette.hairline);
        assert_eq!(edge(&theme, true), theme.palette.accent);
        assert_eq!(menu_padding(), gpui::px(4.0));
    }

    #[test]
    fn the_cursor_is_the_inset_tint_accent_surface() {
        // The cursor pair holds the §18.2 `tint.accent` text bars in every
        // palette (the palette gate proves ink ≥ 9:1, accent.ink ≥ 4.5:1).
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for contrast in [ContrastMode::Standard, ContrastMode::Increased] {
                let theme = Theme::new(mode, contrast, Density::Normal);
                let p = theme.palette;
                assert!(crate::palette::contrast(p.ink, p.tint_accent) >= 9.0);
                assert_ne!(p.tint_accent, p.surface_overlay);
            }
        }
    }
}
