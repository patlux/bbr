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

use gpui::{
    canvas, hsla, point, prelude::*, px, quad, size, AnyElement, BorderStyle, Bounds, BoxShadow,
    ContentMask, Corners, Edges, Hsla, PaintQuad, Pixels, Rgba,
};

use crate::theme::{Theme, ThemeMode};

/// The stable semantic id of the content card (DESIGN §15).
pub const CONTENT_CARD_ID: &str = "bbr-content-card";
/// The stable semantic id of the draggable chassis band above the card.
pub const CHASSIS_BAND_ID: &str = "bbr-chassis-band";
/// `sp:content-inset` (DESIGN §5): the frame thickness around the card —
/// the only value off the 4 pt grid, equal to the card radius so the corner
/// arc and the distance to the window edge are the same number.
pub const CONTENT_INSET: f32 = 10.0;
/// The card corner radius (DESIGN §2, §18.3): stays 10 = `sp:content-inset`
/// although v2 moved `radius:lg` to 12.
pub const CARD_RADIUS: f32 = CONTENT_INSET;

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

/// Blueprint D09 (DESIGN §11.17): the Peek frame's horizontal inset inside
/// the card (`sp6`).
pub const PEEK_INSET_X: f32 = 24.0;
/// The Peek frame's top and bottom inset inside the card (`sp4`).
pub const PEEK_INSET_Y: f32 = 16.0;
/// The gap between the Peek bar and the preview below it (`sp2`).
pub const PEEK_GAP: f32 = 8.0;
/// The preview's corner radius (`radius:md`); the native layer clips to it.
pub const PEEK_PREVIEW_RADIUS: f32 = 10.0;

/// The Peek presentation inside the content card (blueprint D09): the bar
/// sits inset at the card top and the preview WKWebView sits in an inset
/// frame under it. The warm origin keeps the whole card underneath; the
/// portal washes exactly the card minus the preview. All rects are
/// `[x, y, width, height]` in window points and never go negative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PeekFrame {
    /// The card's inner box (the origin's native slot).
    pub card: [f32; 4],
    /// The Peek bar.
    pub bar: [f32; 4],
    /// The preview's native slot.
    pub preview: [f32; 4],
    /// The card's corner radius (0 in full screen): the wash follows it so
    /// it never paints onto the chassis.
    pub card_radius: f32,
}

impl PeekFrame {
    /// Resolve the frame for a card inner box and the bar height
    /// (`density:row`).
    pub fn resolve(card: [f32; 4], card_radius: f32, bar_height: f32) -> Self {
        let [x, y, width, height] = card;
        let inner_width = (width - 2.0 * PEEK_INSET_X).max(0.0);
        let bar = [x + PEEK_INSET_X, y + PEEK_INSET_Y, inner_width, bar_height];
        let preview_top = y + PEEK_INSET_Y + bar_height + PEEK_GAP;
        let preview_height = (y + height - PEEK_INSET_Y - preview_top).max(0.0);
        let preview = [x + PEEK_INSET_X, preview_top, inner_width, preview_height];
        Self {
            card,
            bar,
            preview,
            card_radius,
        }
    }

    /// The Portal capture regions while the Peek shows (the washed origin
    /// around the preview, bar included): the card minus the preview frame
    /// as four strips. The preview and the chrome outside the card take
    /// the pointer themselves.
    pub fn capture_rects(&self) -> [[f32; 4]; 4] {
        let [cx, cy, cw, ch] = self.card;
        let [px, py, pw, ph] = self.preview;
        [
            [cx, cy, cw, (py - cy).max(0.0)],
            [cx, py + ph, cw, (cy + ch - py - ph).max(0.0)],
            [cx, py, (px - cx).max(0.0), ph],
            [px + pw, py, (cx + cw - px - pw).max(0.0), ph],
        ]
    }

    /// The preview slot's insets inside the card `(top, right, bottom,
    /// left)` — what the app's layout applies around the preview slot.
    pub fn preview_insets(bar_height: f32) -> [f32; 4] {
        [
            PEEK_INSET_Y + bar_height + PEEK_GAP,
            PEEK_INSET_X,
            PEEK_INSET_Y,
            PEEK_INSET_X,
        ]
    }
}

/// The card's 1 pt rim: the inner ring of the seated edge, directly
/// outside the native slot (DESIGN §2).
pub const CARD_RIM: f32 = 1.0;
/// The outer extent of the seated edge: the 1 pt seam sits outside the rim,
/// so the edge reaches `CARD_SEAM` beyond the card's rounded rect.
pub const CARD_SEAM: f32 = CARD_RIM + 1.0;

/// The `elev:1.content.shadow` token for the shell's mode: the seat under
/// the card (DESIGN §2) — two short, low-alpha, downward layers. Static —
/// painted once per frame with the card, never animated. The edge rings
/// are not shadows; see [`content_card_edge`].
///
/// ```text
/// Light: 0 1 2 rgba(23,26,29,0.06), 0 3 10 rgba(23,26,29,0.10)
/// Dark:  0 1 2 rgba(0,0,0,0.45), 0 4 12 rgba(0,0,0,0.38)
/// ```
///
/// GPUI paints the vector back to front: far seat first, near seat last.
pub fn content_card_shadow(theme: &Theme) -> Vec<BoxShadow> {
    let shadow = |color, y: f32, blur: f32| BoxShadow {
        color,
        offset: point(px(0.0), px(y)),
        blur_radius: px(blur),
        spread_radius: px(0.0),
        inset: false,
    };
    match theme.mode {
        ThemeMode::Light => vec![
            shadow(hsla(0.58, 0.12, 0.10, 0.10), 3.0, 10.0),
            shadow(hsla(0.58, 0.12, 0.10, 0.06), 1.0, 2.0),
        ],
        ThemeMode::Dark => vec![
            shadow(hsla(0.0, 0.0, 0.0, 0.38), 4.0, 12.0),
            shadow(hsla(0.0, 0.0, 0.0, 0.45), 1.0, 2.0),
        ],
    }
}

/// The `elev:1.content.edge` token: the two crisp 1 pt rings around the
/// card's rounded rect — a light rim directly outside the slot and a dark
/// seam outside the rim — one bevel, never two light lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardEdge {
    pub rim: Hsla,
    pub seam: Hsla,
}

/// The edge colours for the shell's mode.
///
/// ```text
/// Light: rim white, seam rgba(23,26,29,0.12)
/// Dark:  rim rgba(255,255,255,0.08), seam rgba(0,0,0,0.55)
/// ```
pub fn content_card_edge(theme: &Theme) -> CardEdge {
    match theme.mode {
        ThemeMode::Light => CardEdge {
            rim: hsla(0.0, 0.0, 1.0, 1.0),
            seam: hsla(0.58, 0.12, 0.10, 0.12),
        },
        ThemeMode::Dark => CardEdge {
            rim: hsla(0.0, 0.0, 1.0, 0.08),
            seam: hsla(0.0, 0.0, 0.0, 0.55),
        },
    }
}

/// A ring quad: the band between a rounded rect and the same rect deflated
/// by `width` (concentric corners). The fill is the ring colour at alpha 0
/// so the quad pipeline's straight-alpha edge blend cannot darken the
/// anti-aliased inner edge.
fn ring(bounds: Bounds<Pixels>, radius: Pixels, width: Pixels, color: Hsla) -> PaintQuad {
    let mut fill = color;
    fill.a = 0.0;
    quad(
        bounds,
        Corners::all(radius),
        fill,
        Edges::all(width),
        color,
        BorderStyle::Solid,
    )
}

fn corner_cover(bounds: Bounds<Pixels>, radius: Pixels, chassis: Rgba) -> PaintQuad {
    ring(dilate(bounds, radius), radius * 2.0, radius, chassis.into())
}

fn dilate(bounds: Bounds<Pixels>, amount: Pixels) -> Bounds<Pixels> {
    Bounds::new(
        point(bounds.left() - amount, bounds.top() - amount),
        size(
            bounds.size.width + amount * 2.0,
            bounds.size.height + amount * 2.0,
        ),
    )
}

/// The card's corner covers (DESIGN §2): GPUI clips children to rectangles
/// only, so a GPUI surface hosted in the card — Library, History, the
/// error/crash/recovery/welcome/empty panes — paints its square corners over
/// the card's arc. The covers paint the chassis back over exactly those four
/// wedges: one ring whose inner edge *is* the card's rounded rect (outer
/// radius `2r`, width `r`), masked to the four `r × r` corner squares; the
/// quad pipeline anti-aliases that edge the way it draws the card's own
/// background. Painted as the card's last child: above every GPUI child,
/// beneath the native web view (which clips itself through its layer).
/// `chassis` is the chassis paint at this moment; the wedges carry the tint
/// but not the D6 grain (4 × ~21 pt²). A `radius` of `0` (full screen)
/// paints nothing.
pub fn corner_covers(radius: f32, chassis: Rgba) -> AnyElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            if radius <= 0.0 {
                return;
            }
            let r = px(radius);
            let corners = [
                point(bounds.left(), bounds.top()),
                point(bounds.right() - r, bounds.top()),
                point(bounds.left(), bounds.bottom() - r),
                point(bounds.right() - r, bounds.bottom() - r),
            ];
            for origin in corners {
                let mask = ContentMask {
                    bounds: Bounds::new(origin, size(r, r)),
                };
                window.with_content_mask(Some(mask), |window| {
                    window.paint_quad(corner_cover(bounds, r, chassis));
                });
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

/// The edge layer (DESIGN §2): the rim and seam as exact concentric 1 pt
/// rings around the card's rounded rect. Spread box shadows keep the
/// element radius and run ~0.4 pt thicker on the corner diagonal; rings
/// painted as quads are concentric to the pixel. The layer is a chassis
/// child laid out as the card dilated by [`CARD_SEAM`] and painted after the
/// card, so it sits above the card's children and the corner covers,
/// beneath the self-clipping native view, and on the flush leading edge on
/// the rail's last 2 pt. A `radius` of `0` (full screen) paints nothing.
pub fn edge_layer(radius: f32, edge: CardEdge) -> AnyElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            if radius <= 0.0 {
                return;
            }
            let r = px(radius);
            let card = dilate(bounds, px(-CARD_SEAM));
            window.paint_quad(ring(
                dilate(card, px(CARD_SEAM)),
                r + px(CARD_SEAM),
                px(CARD_SEAM - CARD_RIM),
                edge.seam,
            ));
            window.paint_quad(ring(
                dilate(card, px(CARD_RIM)),
                r + px(CARD_RIM),
                px(CARD_RIM),
                edge.rim,
            ));
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density};

    #[test]
    fn peek_frame_insets_the_bar_and_the_preview_inside_the_card() {
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            let bar_height = f32::from(density.row());
            let frame = PeekFrame::resolve([260.0, 10.0, 900.0, 700.0], CARD_RADIUS, bar_height);
            assert_eq!(frame.bar, [284.0, 26.0, 852.0, bar_height]);
            assert_eq!(frame.preview[0], frame.bar[0]);
            assert_eq!(frame.preview[2], frame.bar[2]);
            assert_eq!(frame.preview[1], frame.bar[1] + bar_height + PEEK_GAP);
            // The preview ends one inset above the card bottom.
            assert_eq!(
                frame.preview[1] + frame.preview[3],
                10.0 + 700.0 - PEEK_INSET_Y
            );
            // The layout insets describe the same preview rect.
            let [top, right, bottom, left] = PeekFrame::preview_insets(bar_height);
            assert_eq!(frame.preview[0], 260.0 + left);
            assert_eq!(frame.preview[1], 10.0 + top);
            assert_eq!(frame.preview[0] + frame.preview[2], 260.0 + 900.0 - right);
            assert_eq!(frame.preview[1] + frame.preview[3], 10.0 + 700.0 - bottom);
        }
        // The capture strips tile the card minus the preview exactly.
        let frame = PeekFrame::resolve([260.0, 10.0, 900.0, 700.0], CARD_RADIUS, 36.0);
        let area: f32 = frame.capture_rects().iter().map(|r| r[2] * r[3]).sum();
        assert_eq!(area, 900.0 * 700.0 - frame.preview[2] * frame.preview[3]);
        // A tiny card never yields negative geometry.
        let tiny = PeekFrame::resolve([0.0, 0.0, 20.0, 30.0], 0.0, 36.0);
        assert!(tiny.bar[2] >= 0.0 && tiny.preview[2] >= 0.0 && tiny.preview[3] >= 0.0);
    }

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
    fn the_seat_and_the_edge_follow_the_documented_tokens_per_mode() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
            let stack = content_card_shadow(&theme);
            let [far, near] = stack.as_slice() else {
                panic!("{mode:?}: seat (far, near) — got {}", stack.len());
            };
            // The seat: two short downward layers, the far one still short,
            // no spread (the rings are not shadows), nothing inset.
            assert_eq!(near.offset, point(px(0.0), px(1.0)));
            assert_eq!(near.blur_radius, px(2.0));
            assert!(far.offset.y >= px(3.0) && far.offset.y <= px(4.0));
            assert!(far.blur_radius <= px(12.0), "the seat does not glow");
            assert!(stack
                .iter()
                .all(|shadow| !shadow.inset && shadow.spread_radius == px(0.0)));
            // The edge: the seam is darker than the rim in both modes (one
            // bevel, never two light lines); the light rim is opaque white.
            let edge = content_card_edge(&theme);
            assert!(edge.seam.l < edge.rim.l);
            match mode {
                ThemeMode::Light => assert_eq!(edge.rim, hsla(0.0, 0.0, 1.0, 1.0)),
                ThemeMode::Dark => assert!(edge.rim.a < 0.2 && edge.seam.a > edge.rim.a),
            }
            assert_eq!(CARD_SEAM, CARD_RIM + 1.0);
        }
    }

    #[test]
    fn corner_cover_inner_edge_matches_the_card_not_the_edge_rings() {
        let bounds = Bounds::new(point(px(100.0), px(50.0)), size(px(300.0), px(200.0)));
        let chassis = gpui::rgb(0xe5e5e2);
        let cover = corner_cover(bounds, px(CARD_RADIUS), chassis);
        assert_eq!(dilate(cover.bounds, px(-CARD_RADIUS)), bounds);
        assert_eq!(
            cover.corner_radii.top_left - cover.border_widths.left,
            px(CARD_RADIUS)
        );
        assert_eq!(cover.border_widths, Edges::all(px(CARD_RADIUS)));
        assert_eq!(cover.border_color, Hsla::from(chassis));
        let fill = cover.background.as_solid().expect("solid fill");
        assert_eq!(
            (fill.h, fill.s, fill.l),
            (
                cover.border_color.h,
                cover.border_color.s,
                cover.border_color.l
            )
        );
        assert_eq!(fill.a, 0.0, "no dark fringe along the card arc");
    }

    #[test]
    fn rings_are_concentric_bands_with_a_non_darkening_fill() {
        let bounds = Bounds::new(point(px(100.0), px(50.0)), size(px(300.0), px(200.0)));
        let color = hsla(0.58, 0.12, 0.10, 0.12);
        let band = ring(dilate(bounds, px(2.0)), px(12.0), px(1.0), color);
        assert_eq!(
            band.bounds,
            Bounds::new(point(px(98.0), px(48.0)), size(px(304.0), px(204.0)))
        );
        assert_eq!(band.corner_radii, Corners::all(px(12.0)));
        assert_eq!(band.border_widths, Edges::all(px(1.0)));
        assert_eq!(band.border_color, color);
        // The fill is the ring colour at alpha 0, never transparent black.
        let fill = band.background.as_solid().expect("solid fill");
        assert_eq!(
            (fill.h, fill.s, fill.l, fill.a),
            (color.h, color.s, color.l, 0.0)
        );
    }
}
