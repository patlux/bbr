//! Blueprint D16 (boards L-3…L-9, DESIGN §7): the Settings toolbar marks —
//! person (Profiles), shield (Privacy), eye (Appearance) and keyboard
//! (Shortcuts). Same geometric family as the rest of §7: one stroke weight,
//! drawn paths scaled from the slot size.

use gpui::{point, px, PathBuilder, Pixels, Rgba, Window};

use super::paint::{paint_filled_circle, paint_path, paint_ring};

/// The person: a head ring over a shoulder arc.
pub(super) fn paint_person(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    paint_ring(window, cx, cy - size * 0.16, size * 0.17, stroke, color);
    let mut path = PathBuilder::stroke(stroke);
    let (half, top, bottom) = (size * 0.3, cy + size * 0.12, cy + size * 0.38);
    path.move_to(point(px(cx - half), px(bottom)));
    path.cubic_bezier_to(
        point(px(cx + half), px(bottom)),
        point(px(cx - half), px(top - size * 0.06)),
        point(px(cx + half), px(top - size * 0.06)),
    );
    paint_path(window, path, color);
}

/// The shield: a flat top with curved flanks meeting at the bottom point.
pub(super) fn paint_shield(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let (half, top, bottom) = (size * 0.32, cy - size * 0.38, cy + size * 0.4);
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(cx), px(top)));
    path.line_to(point(px(cx + half), px(top + size * 0.1)));
    path.cubic_bezier_to(
        point(px(cx), px(bottom)),
        point(px(cx + half), px(cy + size * 0.12)),
        point(px(cx + size * 0.14), px(bottom - size * 0.08)),
    );
    path.cubic_bezier_to(
        point(px(cx - half), px(top + size * 0.1)),
        point(px(cx - size * 0.14), px(bottom - size * 0.08)),
        point(px(cx - half), px(cy + size * 0.12)),
    );
    path.close();
    paint_path(window, path, color);
}

/// The eye: an almond outline around a filled pupil.
pub(super) fn paint_eye(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let half = size * 0.42;
    let lid = size * 0.34;
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(cx - half), px(cy)));
    path.cubic_bezier_to(
        point(px(cx + half), px(cy)),
        point(px(cx - half * 0.5), px(cy - lid)),
        point(px(cx + half * 0.5), px(cy - lid)),
    );
    path.cubic_bezier_to(
        point(px(cx - half), px(cy)),
        point(px(cx + half * 0.5), px(cy + lid)),
        point(px(cx - half * 0.5), px(cy + lid)),
    );
    path.close();
    paint_path(window, path, color);
    paint_filled_circle(window, cx, cy, size * 0.1, color);
}

/// The keyboard: a rounded key bed with two key rows and a space bar.
pub(super) fn paint_keyboard(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let (half_w, half_h) = (size * 0.42, size * 0.28);
    let mut path = PathBuilder::stroke(stroke);
    let r = size * 0.06;
    let (l, t, rt, b) = (cx - half_w, cy - half_h, cx + half_w, cy + half_h);
    path.move_to(point(px(l + r), px(t)));
    path.line_to(point(px(rt - r), px(t)));
    path.line_to(point(px(rt), px(t + r)));
    path.line_to(point(px(rt), px(b - r)));
    path.line_to(point(px(rt - r), px(b)));
    path.line_to(point(px(l + r), px(b)));
    path.line_to(point(px(l), px(b - r)));
    path.line_to(point(px(l), px(t + r)));
    path.close();
    paint_path(window, path, color);
    let dot = f32::from(stroke) * 0.55;
    for row in 0..2 {
        let y = cy - size * 0.12 + row as f32 * size * 0.13;
        for key in 0..4 {
            let x = cx - size * 0.24 + key as f32 * size * 0.16;
            paint_filled_circle(window, x, y, dot, color);
        }
    }
    let mut bar = PathBuilder::stroke(stroke);
    bar.move_to(point(px(cx - size * 0.16), px(cy + size * 0.15)));
    bar.line_to(point(px(cx + size * 0.16), px(cy + size * 0.15)));
    paint_path(window, bar, color);
}
