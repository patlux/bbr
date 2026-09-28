//! The §10.5 loading arc at an arbitrary rotation (DESIGN §18.5): the one
//! geometry the static glyph and the rotating spinner share, so Reduce
//! Motion's static arc is exactly the animated arc at turn 0.

use gpui::{canvas, point, px, Canvas, PathBuilder, Pixels, Rgba, Styled, Window};

use super::glyphs::{arc_radius, icon_stroke};
use super::paint::paint_path;
use crate::theme::{Density, Theme};

/// Paint the 270° accent arc into `bounds`, rotated clockwise by `turn`
/// revolutions (0 = from the top clockwise to the left).
pub(super) fn paint_arc(
    bounds: gpui::Bounds<Pixels>,
    window: &mut Window,
    density: Density,
    color: Rgba,
    turn: f32,
) {
    let size = f32::from(bounds.size.width);
    let cx = f32::from(bounds.center().x);
    let cy = f32::from(bounds.center().y);
    let radius = arc_radius(size);
    let at = |angle: f32| point(px(cx + radius * angle.sin()), px(cy - radius * angle.cos()));
    let start = turn.rem_euclid(1.0) * std::f32::consts::TAU;
    let end = start + 0.75 * std::f32::consts::TAU;
    let mut path = PathBuilder::stroke(icon_stroke(density));
    path.move_to(at(start));
    path.arc_to(point(px(radius), px(radius)), px(0.0), true, true, at(end));
    paint_path(window, path, color);
}

/// §10.5/§18.5: the accent loading arc rotated by `turn` revolutions — the
/// frame the favicon spinner draws at each animation delta.
pub fn loading_arc_canvas(theme: &Theme, size: Pixels, turn: f32) -> Canvas<()> {
    let density = theme.density;
    let color = theme.palette.accent;
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| paint_arc(bounds, window, density, color, turn),
    )
    .size(size)
}

/// §18.5: the §3.3 check mark drawn up to `progress` (0 → 1) of its stroke
/// length — the short inner stroke first, then the long ascending one — so
/// a finished download's check "draws" itself; at 1 it is exactly the
/// static `Icon::Check`.
pub fn check_progress_canvas(
    theme: &Theme,
    size: Pixels,
    color: Rgba,
    progress: f32,
) -> Canvas<()> {
    let density = theme.density;
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let size = f32::from(bounds.size.width);
            let cx = f32::from(bounds.center().x);
            let cy = f32::from(bounds.center().y);
            let (inner, outer) = super::glyphs::check_geometry(size);
            let a = (cx - outer, cy);
            let b = (cx - inner, cy + inner);
            let c = (cx + outer, cy - outer);
            let first = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let second = ((c.0 - b.0).powi(2) + (c.1 - b.1).powi(2)).sqrt();
            let drawn = progress.clamp(0.0, 1.0) * (first + second);
            if drawn <= 0.0 {
                return;
            }
            let lerp = |p: (f32, f32), q: (f32, f32), u: f32| {
                point(px(p.0 + (q.0 - p.0) * u), px(p.1 + (q.1 - p.1) * u))
            };
            let mut path = PathBuilder::stroke(icon_stroke(density));
            path.move_to(point(px(a.0), px(a.1)));
            if drawn <= first {
                path.line_to(lerp(a, b, drawn / first));
            } else {
                path.line_to(point(px(b.0), px(b.1)));
                path.line_to(lerp(b, c, (drawn - first) / second));
            }
            paint_path(window, path, color);
        },
    )
    .size(size)
}
