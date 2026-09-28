//! The drawing of the §11.8 / D10 pane illustrations (S12), after the Paper
//! posters `empty_space`, `empty_crash` and `empty_offline`: token colours,
//! GPUI quads and paths only — never an image, never web content (§13).
//! Coordinates are the posters' 400 × 240 artboard points, cropped to each
//! drawing; the frames come from the pure functions in `illustration`.

use gpui::{
    canvas, div, point, prelude::*, px, quad, size, transparent_black, BorderStyle, Bounds, Div,
    PathBuilder, Pixels, Point, Rgba, Window,
};

use bbr_domain::SpaceTheme;

use super::illustration::{OfflineFrame, SpaceFrame};
use crate::theme::{elevation, space_tint, Radius, Theme, TintIntensity};

/// The empty-space drawing: the whole poster artboard.
pub(super) const EMPTY_W: f32 = 400.0;
pub(super) const EMPTY_H: f32 = 240.0;
/// The heartbeat band: the artboard's full width, ±45 pt around the line.
pub(super) const BEAT_W: f32 = 400.0;
pub(super) const BEAT_H: f32 = 90.0;
/// The offline row: laptop to globe, cropped from (40, 92).
pub(super) const LINK_W: f32 = 310.0;
pub(super) const LINK_H: f32 = 58.0;

/// One floating tab card: artboard x/y, the favicon hue (`None`: `ink`),
/// the title width.
const CARDS: [(f32, f32, Option<SpaceTheme>, f32); 6] = [
    (28.0, 40.0, Some(SpaceTheme::Cobalt), 90.0),
    (236.0, 34.0, Some(SpaceTheme::Terracotta), 70.0),
    (258.0, 150.0, Some(SpaceTheme::Pine), 80.0),
    (30.0, 168.0, Some(SpaceTheme::Amber), 64.0),
    (150.0, 194.0, Some(SpaceTheme::Plum), 76.0),
    (282.0, 94.0, None, 60.0),
];
/// The ⌘ key cap: artboard origin and size.
const KEY: (f32, f32, f32, f32) = (160.0, 88.0, 80.0, 60.0);

fn alpha(mut colour: Rgba, a: f32) -> Rgba {
    colour.a *= a;
    colour
}

/// The empty space: the ⌘ key cap on its darker edge, six floating tab
/// cards around it (the favicon squares in curated space hues).
pub(super) fn empty_space(theme: &Theme, frame: SpaceFrame) -> Div {
    let p = theme.palette;
    let (key_x, key_y, key_w, key_h) = KEY;
    let cards = CARDS
        .iter()
        .zip(frame.cards)
        .map(|(&(x, y, hue, title), (dx, dy))| {
            let favicon = hue.map_or(p.ink, |hue| {
                space_tint(hue, TintIntensity::Standard, theme).hue
            });
            div()
                .absolute()
                .left(px(x + dx))
                .top(px(y + dy))
                .w(px(title + 48.0))
                .h(px(34.0))
                .pl(px(10.0))
                .gap(px(8.0))
                .flex()
                .flex_row()
                .items_center()
                .rounded(px(11.0))
                .bg(p.surface_overlay)
                .shadow(elevation::elev2(theme))
                .child(div().size(px(18.0)).rounded(px(5.0)).bg(favicon))
                .child(
                    div()
                        .w(px(title))
                        .h(px(8.0))
                        .rounded(Radius.full())
                        .bg(p.border_strong),
                )
        });
    let press = 2.0 * frame.press;
    let ink = p.ink;
    let glyph = canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let (cx, cy) = (key_w / 2.0, key_h / 2.0);
            let origin = bounds.origin;
            let at = |x: f32, y: f32| point(origin.x + px(x), origin.y + px(y));
            let stroke = px(2.0);
            let mut square = PathBuilder::stroke(stroke);
            square.move_to(at(cx - 4.5, cy - 4.5));
            square.line_to(at(cx + 4.5, cy - 4.5));
            square.line_to(at(cx + 4.5, cy + 4.5));
            square.line_to(at(cx - 4.5, cy + 4.5));
            square.close();
            paint(window, square, ink);
            // The loops: rings through each corner, centred outward.
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                ring(window, at(cx + sx * 8.5, cy + sy * 4.5), 4.0, stroke, ink);
            }
        },
    )
    .absolute()
    .size_full();
    div()
        .relative()
        .flex_none()
        .w(px(EMPTY_W))
        .h(px(EMPTY_H))
        .child(
            div()
                .absolute()
                .left(px(key_x))
                .top(px(key_y + 4.0))
                .w(px(key_w))
                .h(px(key_h + 2.0))
                .rounded(px(14.0))
                .bg(p.hairline),
        )
        .child(
            div()
                .absolute()
                .left(px(key_x))
                .top(px(key_y + press))
                .w(px(key_w))
                .h(px(key_h))
                .rounded(px(14.0))
                .bg(p.surface_canvas)
                .border_1()
                .border_color(alpha(p.ink, 0.08))
                .child(glyph),
        )
        .children(cards)
}

/// The heartbeat offset (pt, up is negative) at artboard `x` for the beat
/// travelling just behind `head`: one damped cycle 26 pt wide.
fn beat(x: f32, head: f32) -> f32 {
    let d = x - head;
    if !(-26.0 < d && d < 0.0) {
        return 0.0;
    }
    let p = (d + 26.0) / 26.0;
    if p >= 0.9 {
        return 0.0;
    }
    -38.0 * (p * std::f32::consts::TAU).sin() * (-3.0 * (p - 0.5).powi(2)).exp()
}

/// The crashed page: the line trails the travelling head in `accent`,
/// fading over 260 pt behind it; the rest of the line is faint; the head is
/// a dot with a soft halo.
pub(super) fn crashed(theme: &Theme, head: f32) -> impl IntoElement {
    let p = theme.palette;
    let (accent, faint) = (p.accent, alpha(p.ink_faint, 0.12));
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let origin = bounds.origin;
            let baseline = BEAT_H / 2.0;
            let at = |x: f32, y: f32| point(origin.x + px(x), origin.y + px(baseline + y));
            // Group the 2 pt steps into bands of one colour: the faint
            // line, and the trail in 20 pt opacity bands.
            let band = |x: f32| -> Option<i32> {
                let d = head - x;
                (0.0..260.0).contains(&d).then(|| (d / 20.0) as i32)
            };
            let mut x = 0.0;
            while x < BEAT_W {
                let key = band(x);
                let mut path = PathBuilder::stroke(px(2.4));
                path.move_to(at(x, beat(x, head)));
                while x < BEAT_W && band(x) == key {
                    x += 2.0;
                    path.line_to(at(x, beat(x, head)));
                }
                let colour = match key {
                    Some(step) => alpha(
                        accent,
                        (1.0 - (step as f32 * 20.0 + 10.0) / 260.0).max(0.12),
                    ),
                    None => faint,
                };
                paint(window, path, colour);
            }
            if (-10.0..BEAT_W + 10.0).contains(&head) {
                filled(window, at(head, 0.0), 10.0, alpha(accent, 0.15));
                filled(window, at(head, 0.0), 4.0, accent);
            }
        },
    )
    .flex_none()
    .w(px(BEAT_W))
    .h(px(BEAT_H))
}

/// The offline link: laptop ↔ globe in `ink`, the dotted link with its
/// gap, accent packets running into the gap, and the broken link pulsing in
/// the attention tone.
pub(super) fn offline(theme: &Theme, frame: OfflineFrame) -> impl IntoElement {
    let p = theme.palette;
    let (ink, dots, accent, warn) = (p.ink, p.border_strong, p.accent, p.status_attention);
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let origin = bounds.origin;
            // Artboard → drawing: the crop starts at (40, 92).
            let at = |x: f32, y: f32| point(origin.x + px(x - 40.0), origin.y + px(y - 92.0));
            let (lx, rx, cy) = (90.0, 310.0, 120.0);
            // Laptop: the screen (stroke centred on the outline) and the base.
            outline(
                window,
                at(lx - 31.5, cy - 23.5),
                (63.0, 43.0),
                7.5,
                px(3.0),
                ink,
            );
            rect(window, at(lx - 41.5, cy + 24.5), (83.0, 3.0), 1.5, ink);
            // Globe: the outline, the meridian and the equator.
            ring(window, at(rx, cy), 28.0, px(3.0), ink);
            let mut meridian = PathBuilder::stroke(px(2.4));
            meridian.move_to(at(rx, cy - 28.0));
            let radii = point(px(12.0), px(28.0));
            meridian.arc_to(radii, px(0.0), false, true, at(rx, cy + 28.0));
            meridian.arc_to(radii, px(0.0), false, true, at(rx, cy - 28.0));
            paint(window, meridian, ink);
            rect(window, at(rx - 28.0, cy - 1.2), (56.0, 2.4), 0.0, ink);
            // The dotted link; the gap is the break.
            let (gap0, gap1) = (186.0, 214.0);
            for index in 0..12 {
                let x = lx + 48.0 + index as f32 * 12.0;
                if x > rx - 40.0 {
                    break;
                }
                if gap0 < x && x < gap1 {
                    continue;
                }
                filled(window, at(x, cy), 2.0, dots);
            }
            for (position, opacity) in frame.packets {
                let x = lx + 48.0 + (gap0 - lx - 48.0) * position;
                filled(window, at(x, cy), 3.6, alpha(accent, opacity));
            }
            // The broken link: two severed ends and a soft disc, pulsing.
            let pulse = frame.pulse;
            let strength = 0.45 + 0.55 * pulse;
            for (x, y, dx) in [(gap0 + 2.0, cy - 9.0, 6.0), (gap1 - 2.0, cy + 9.0, -6.0)] {
                let mut end = PathBuilder::stroke(px(2.6));
                end.move_to(at(x, y));
                end.line_to(at(x + dx, y + dx));
                paint(window, end, alpha(warn, strength));
            }
            let disc = alpha(warn, 0.10 * strength);
            filled(
                window,
                at((gap0 + gap1) / 2.0, cy),
                12.0 + 4.0 * pulse,
                disc,
            );
        },
    )
    .flex_none()
    .w(px(LINK_W))
    .h(px(LINK_H))
}

fn paint(window: &mut Window, path: PathBuilder, colour: Rgba) {
    // The static geometries tessellate unconditionally; on an unexpected
    // failure the stroke degrades away instead of panicking the paint phase.
    if let Ok(path) = path.build() {
        window.paint_path(path, colour);
    }
}

fn filled(window: &mut Window, center: Point<Pixels>, radius: f32, colour: Rgba) {
    let corner = point(center.x - px(radius), center.y - px(radius));
    rect(window, corner, (2.0 * radius, 2.0 * radius), radius, colour);
}

fn rect(window: &mut Window, origin: Point<Pixels>, (w, h): (f32, f32), corner: f32, colour: Rgba) {
    window.paint_quad(quad(
        Bounds::new(origin, size(px(w), px(h))),
        px(corner),
        colour,
        px(0.0),
        transparent_black(),
        BorderStyle::default(),
    ));
}

/// A ring whose stroke is centred on the circle of `radius`.
fn ring(window: &mut Window, center: Point<Pixels>, radius: f32, stroke: Pixels, colour: Rgba) {
    let outer = radius + f32::from(stroke) / 2.0;
    let corner = point(center.x - px(outer), center.y - px(outer));
    outline(
        window,
        corner,
        (2.0 * outer, 2.0 * outer),
        outer,
        stroke,
        colour,
    );
}

fn outline(
    window: &mut Window,
    origin: Point<Pixels>,
    (w, h): (f32, f32),
    corner: f32,
    stroke: Pixels,
    colour: Rgba,
) {
    window.paint_quad(quad(
        Bounds::new(origin, size(px(w), px(h))),
        px(corner),
        transparent_black(),
        stroke,
        colour,
        BorderStyle::default(),
    ));
}
