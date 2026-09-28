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
