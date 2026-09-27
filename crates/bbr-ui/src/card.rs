//! The seated content card (DESIGN.md §2, §5, §11.4, §13) — the pure
//! geometry authority for the frame the native web view sits in.
//!
//! The card is the content, not a card over content: its inner box is the
//! native slot. It is inset by `sp:content-inset` on the top, trailing and
//! bottom edges and flush against the rail on the leading edge. Collapsing
//! the sidebar adds the leading inset and grows the top inset to the native
//! titlebar band (`win:titlebar`), so the traffic lights never sit on web
//! content. Native full screen is full-bleed: no inset, no radius, no
//! shadow. GPUI paints the chassis and the static `elev:1.content` shadow
//! outside the slot; the platform adapter clips the web view to the radius.

use gpui::{hsla, point, px, BoxShadow};

use crate::theme::{Theme, ThemeMode};

/// The stable semantic id of the content card (DESIGN §15).
pub const CONTENT_CARD_ID: &str = "bbr-content-card";
/// The stable semantic id of the draggable chassis band above the card.
pub const CHASSIS_BAND_ID: &str = "bbr-chassis-band";
/// `sp:content-inset` (DESIGN §5): the frame thickness around the card —
/// the only value off the 4 pt grid, equal to `radius:lg` so the corner arc
/// and the distance to the window edge are the same number.
pub const CONTENT_INSET: f32 = 10.0;
/// The card corner radius: `radius:lg`.
pub const CARD_RADIUS: f32 = 10.0;

/// The resolved card frame for one window state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardInsets {
    pub top: f32,
    pub trailing: f32,
    pub bottom: f32,
    pub leading: f32,
    pub radius: f32,
    /// Whether the static `elev:1.content` shadow paints.
    pub shadow: bool,
}

impl CardInsets {
    /// The card frame for a window state. `titlebar_band` is the native
    /// safe titlebar strip height (`win:titlebar`); a collapsed sidebar
    /// keeps that band as empty, draggable chassis so the traffic lights
    /// never sit on web content (never less than the regular inset).
    pub fn resolve(sidebar_visible: bool, full_screen: bool, titlebar_band: f32) -> Self {
        if full_screen {
            return Self {
                top: 0.0,
                trailing: 0.0,
                bottom: 0.0,
                leading: 0.0,
                radius: 0.0,
                shadow: false,
            };
        }
        Self {
            top: if sidebar_visible {
                CONTENT_INSET
            } else {
                titlebar_band.max(CONTENT_INSET)
            },
            trailing: CONTENT_INSET,
            bottom: CONTENT_INSET,
            leading: if sidebar_visible { 0.0 } else { CONTENT_INSET },
            radius: CARD_RADIUS,
            shadow: true,
        }
    }

    /// The card's inner box `[x, y, width, height]` (window points) for a
    /// window of `size` with the rail rendering at `rail_width` (0 when
    /// collapsed). This is the native slot rect; it never goes negative.
    pub fn inner_rect(&self, size: [f32; 2], rail_width: f32) -> [f32; 4] {
        let x = rail_width + self.leading;
        let y = self.top;
        let width = (size[0] - x - self.trailing).max(0.0);
        let height = (size[1] - y - self.bottom).max(0.0);
        [x, y, width, height]
    }
}

/// The `elev:1.content.shadow` token for the shell's mode: two short, dark,
/// low-alpha layers that seat the card; on dark a 1 px white ring because a
/// pure shadow is invisible on a dark chassis. Static — painted once per
/// frame with the card, never animated.
///
/// ```text
/// Light: 0 1 2 rgba(23,26,29,0.06), 0 8 20 rgba(23,26,29,0.10)
/// Dark:  0 0 0 1px rgba(255,255,255,0.05), 0 1 2 rgba(0,0,0,0.45),
///        0 10 24 rgba(0,0,0,0.38)
/// ```
pub fn content_card_shadow(theme: &Theme) -> Vec<BoxShadow> {
    let shadow = |color, y: f32, blur: f32, spread: f32| BoxShadow {
        color,
        offset: point(px(0.0), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    };
    match theme.mode {
        ThemeMode::Light => vec![
            shadow(hsla(0.58, 0.12, 0.10, 0.06), 1.0, 2.0, 0.0),
            shadow(hsla(0.58, 0.12, 0.10, 0.10), 8.0, 20.0, 0.0),
        ],
        ThemeMode::Dark => vec![
            shadow(hsla(0.0, 0.0, 1.0, 0.05), 0.0, 0.0, 1.0),
            shadow(hsla(0.0, 0.0, 0.0, 0.45), 1.0, 2.0, 0.0),
            shadow(hsla(0.0, 0.0, 0.0, 0.38), 10.0, 24.0, 0.0),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density};

    #[test]
    fn expanded_card_is_flush_with_the_rail_and_inset_elsewhere() {
        let card = CardInsets::resolve(true, false, 52.0);
        assert_eq!(
            (card.top, card.trailing, card.bottom, card.leading),
            (10.0, 10.0, 10.0, 0.0)
        );
        assert_eq!((card.radius, card.shadow), (10.0, true));
        // The shipped 1080 × 700 window with the 300 pt rail: a 770 pt card.
        assert_eq!(
            card.inner_rect([1080.0, 700.0], 300.0),
            [300.0, 10.0, 770.0, 680.0]
        );
    }

    #[test]
    fn collapsed_card_keeps_the_titlebar_band_and_gains_the_leading_inset() {
        let card = CardInsets::resolve(false, false, 52.0);
        assert_eq!(
            (card.top, card.trailing, card.bottom, card.leading),
            (52.0, 10.0, 10.0, 10.0)
        );
        assert_eq!(
            card.inner_rect([1080.0, 700.0], 0.0),
            [10.0, 52.0, 1060.0, 638.0]
        );
        // Without a native titlebar strip the band never undercuts the inset.
        assert_eq!(CardInsets::resolve(false, false, 0.0).top, 10.0);
    }

    #[test]
    fn full_screen_is_full_bleed_without_radius_or_shadow() {
        for visible in [true, false] {
            let card = CardInsets::resolve(visible, true, 52.0);
            assert_eq!(
                (
                    card.top,
                    card.trailing,
                    card.bottom,
                    card.leading,
                    card.radius
                ),
                (0.0, 0.0, 0.0, 0.0, 0.0)
            );
            assert!(!card.shadow);
        }
        assert_eq!(
            CardInsets::resolve(true, true, 52.0).inner_rect([1440.0, 900.0], 300.0),
            [300.0, 0.0, 1140.0, 900.0]
        );
    }

    #[test]
    fn the_inner_box_never_goes_negative() {
        let card = CardInsets::resolve(false, false, 52.0);
        assert_eq!(card.inner_rect([15.0, 40.0], 0.0), [10.0, 52.0, 0.0, 0.0]);
    }

    #[test]
    fn the_shadow_follows_the_documented_tokens_per_mode() {
        let light = Theme::new(ThemeMode::Light, ContrastMode::Standard, Density::Normal);
        let light = content_card_shadow(&light);
        assert_eq!(light.len(), 2);
        assert_eq!(light[1].offset, point(px(0.0), px(8.0)));
        assert_eq!(light[1].blur_radius, px(20.0));
        let dark = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
        let dark = content_card_shadow(&dark);
        assert_eq!(dark.len(), 3, "dark adds the 1 px ring");
        assert_eq!(dark[0].spread_radius, px(1.0));
        assert_eq!(dark[2].blur_radius, px(24.0));
        assert!(dark.iter().chain(light.iter()).all(|shadow| !shadow.inset));
    }
}
