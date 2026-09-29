//! Blueprint D24 (board S-8, DESIGN §7/§11.15): the media-card transport
//! marks — Previous and Next (a filled triangle against a bar), Play (a
//! filled triangle), Pause (two bars) and the speaker (sound waves, or the
//! §7 cross when muted). Drawn from the board's 18-pt geometry, scaled to
//! the slot; one stroke weight with the rest of the catalog.

use gpui::{point, px, PathBuilder, Pixels, Point, Rgba, Window};

use super::paint::paint_path;

/// The board's 18-pt grid mapped into a `size` slot centered at (cx, cy).
struct Grid {
    left: f32,
    top: f32,
    unit: f32,
}

impl Grid {
    fn new(cx: f32, cy: f32, size: f32) -> Self {
        Self {
            left: cx - size / 2.0,
            top: cy - size / 2.0,
            unit: size / 18.0,
        }
    }

    fn at(&self, x: f32, y: f32) -> Point<Pixels> {
        point(px(self.left + x * self.unit), px(self.top + y * self.unit))
    }

    fn filled(&self, window: &mut Window, color: Rgba, points: &[(f32, f32)]) {
        let mut path = PathBuilder::fill();
        let mut iter = points.iter();
        if let Some(&(x, y)) = iter.next() {
            path.move_to(self.at(x, y));
        }
        for &(x, y) in iter {
            path.line_to(self.at(x, y));
        }
        path.close();
        paint_path(window, path, color);
    }

    fn bars(&self, window: &mut Window, stroke: Pixels, color: Rgba, xs: &[f32]) {
        let mut path = PathBuilder::stroke(stroke);
        for &x in xs {
            path.move_to(self.at(x, 4.0));
            path.line_to(self.at(x, 14.0));
        }
        paint_path(window, path, color);
    }
}

/// Previous: a left-pointing triangle against a bar.
pub(super) fn paint_previous(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let grid = Grid::new(cx, cy, size);
    grid.filled(window, color, &[(13.0, 4.0), (13.0, 14.0), (6.0, 9.0)]);
    grid.bars(window, stroke, color, &[4.5]);
}

/// Next: a right-pointing triangle against a bar.
pub(super) fn paint_next(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let grid = Grid::new(cx, cy, size);
    grid.filled(window, color, &[(5.0, 4.0), (5.0, 14.0), (12.0, 9.0)]);
    grid.bars(window, stroke, color, &[13.5]);
}

/// Play: one right-pointing triangle.
pub(super) fn paint_play(window: &mut Window, color: Rgba, cx: f32, cy: f32, size: f32) {
    Grid::new(cx, cy, size).filled(window, color, &[(5.5, 3.5), (14.0, 9.0), (5.5, 14.5)]);
}

/// Pause: two bars.
pub(super) fn paint_pause(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    Grid::new(cx, cy, size).bars(window, stroke, color, &[6.0, 12.0]);
}

/// The speaker: waves while sound plays, the §7 cross when muted.
pub(super) fn paint_speaker(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
    muted: bool,
) {
    let grid = Grid::new(cx, cy, size);
    let mut body = PathBuilder::stroke(stroke);
    body.move_to(grid.at(9.0, 3.5));
    for (x, y) in [
        (5.5, 6.5),
        (3.0, 6.5),
        (3.0, 11.5),
        (5.5, 11.5),
        (9.0, 14.5),
    ] {
        body.line_to(grid.at(x, y));
    }
    body.close();
    paint_path(window, body, color);
    let mut marks = PathBuilder::stroke(stroke);
    if muted {
        marks.move_to(grid.at(12.0, 7.0));
        marks.line_to(grid.at(16.0, 11.0));
        marks.move_to(grid.at(16.0, 7.0));
        marks.line_to(grid.at(12.0, 11.0));
    } else {
        for (top, bottom, radius) in [(6.5, 11.5, 3.5), (4.5, 13.5, 6.25)] {
            let x = if radius < 4.0 { 11.75 } else { 13.75 };
            let r = px(radius * grid.unit);
            marks.move_to(grid.at(x, top));
            marks.arc_to(point(r, r), px(0.0), false, true, grid.at(x, bottom));
        }
    }
    paint_path(window, marks, color);
}
