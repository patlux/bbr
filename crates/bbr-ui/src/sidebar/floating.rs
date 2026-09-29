//! Blueprint D02 (DESIGN §2 §11.4, board W-3): the floating rail — the
//! whole rail composition on a panel over the content card, drawn by the
//! one native Portal so it sits above the page without moving it.
//!
//! The panel is inset 8 pt from the window's leading, top and bottom edges,
//! `radius:lg` (12) with the `elev:2` shadow and ring, on the active space's
//! rail material. The native traffic lights stay visible and clickable: the
//! panel leaves a notch around them (the Portal draws nothing there, so the
//! browser window's own lights and chassis show through). The entrance
//! slides in from the edge over `dur:fine` with the decelerate curve; the
//! exit slides back out and fades over `dur:fine` with the accelerate
//! curve (the panel no longer takes the pointer meanwhile). Reduce Motion
//! fades only.

use gpui::{
    canvas, div, point, prelude::*, px, Animation, AnimationExt, AnyElement, Bounds, PathBuilder,
    Pixels, Point, Rgba,
};

use crate::theme::elevation::elev2;
use crate::theme::motion::Easing;
use crate::theme::{Motion, Theme};

/// The §15 id of the floating panel.
pub const FLOATING_RAIL_ID: &str = "bbr-floating-rail";
/// The panel's inset from the window's leading, top and bottom edges.
pub const FLOATING_INSET: f32 = 8.0;
/// The panel's corner radius (`radius:lg`).
pub const FLOATING_RADIUS: f32 = 12.0;
/// How far the entrance slides in from the edge.
pub const FLOATING_SLIDE: f32 = 16.0;

/// The panel geometry in window points: the rail rect and the notch (in
/// panel-local points, anchored at its top-left corner) left open for the
/// native traffic lights (`[0, 0]` without lights, e.g. in fullscreen).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingRailGeometry {
    pub rect: [f32; 4],
    pub notch: [f32; 2],
}

impl FloatingRailGeometry {
    /// The panel for a window of `window_size` and a rail `width`, leaving
    /// a notch around the traffic-light region `lights` (window points;
    /// zero-sized when there are none).
    pub fn resolve(window_size: [f32; 2], width: f32, lights: [f32; 4]) -> Self {
        let rect = [
            FLOATING_INSET,
            FLOATING_INSET,
            width.min(window_size[0] - 2.0 * FLOATING_INSET).max(0.0),
            (window_size[1] - 2.0 * FLOATING_INSET).max(0.0),
        ];
        let notch = if lights[2] > 0.0 && lights[3] > 0.0 {
            [
                (lights[0] + lights[2] + 8.0 - rect[0]).max(0.0),
                (lights[1] + lights[3] + 6.0 - rect[1]).max(0.0),
            ]
        } else {
            [0.0, 0.0]
        };
        Self { rect, notch }
    }

    /// The pointer-capture regions (window points): the panel minus the
    /// notch, so a click on a traffic light reaches the browser window.
    pub fn capture_rects(&self) -> Vec<[f32; 4]> {
        let [x, y, w, h] = self.rect;
        let [nw, nh] = self.notch;
        if nw <= 0.0 || nh <= 0.0 {
            return vec![self.rect];
        }
        vec![
            [x + nw, y, (w - nw).max(0.0), nh],
            [x, y + nh, w, (h - nh).max(0.0)],
        ]
    }
}

/// The notched panel outline (panel-local points): a rounded rect whose
/// top-left corner is cut away by the `[nw, nh]` notch.
fn notched_outline(path: &mut PathBuilder, origin: Point<Pixels>, size: [f32; 2], notch: [f32; 2]) {
    const KAPPA: f32 = 0.552_284_8;
    let [w, h] = size;
    let r = FLOATING_RADIUS.min(w / 2.0).min(h / 2.0);
    let k = r * KAPPA;
    let [nw, nh] = notch;
    let p = |x: f32, y: f32| point(origin.x + px(x), origin.y + px(y));
    if nw > 0.0 && nh > 0.0 {
        let ri = 6.0_f32.min(nw).min(nh);
        let ki = ri * KAPPA;
        path.move_to(p(nw, 0.0));
        path.line_to(p(w - r, 0.0));
        path.cubic_bezier_to(p(w, r), p(w - r + k, 0.0), p(w, r - k));
        path.line_to(p(w, h - r));
        path.cubic_bezier_to(p(w - r, h), p(w, h - r + k), p(w - r + k, h));
        path.line_to(p(r, h));
        path.cubic_bezier_to(p(0.0, h - r), p(r - k, h), p(0.0, h - r + k));
        path.line_to(p(0.0, nh));
        path.line_to(p(nw - ri, nh));
        path.cubic_bezier_to(p(nw, nh - ri), p(nw - ri + ki, nh), p(nw, nh - ri + ki));
        path.close();
    } else {
        path.move_to(p(r, 0.0));
        path.line_to(p(w - r, 0.0));
        path.cubic_bezier_to(p(w, r), p(w - r + k, 0.0), p(w, r - k));
        path.line_to(p(w, h - r));
        path.cubic_bezier_to(p(w - r, h), p(w, h - r + k), p(w - r + k, h));
        path.line_to(p(r, h));
        path.cubic_bezier_to(p(0.0, h - r), p(r - k, h), p(0.0, h - r + k));
        path.line_to(p(0.0, r));
        path.cubic_bezier_to(p(r, 0.0), p(0.0, r - k), p(r - k, 0.0));
        path.close();
    }
}

/// The floating panel around `rail` (the rail composition rendered with a
/// transparent root, `model.floating`). `tint` is the rail material;
/// `entrance` keys the slide-in (a new key replays it).
pub fn floating_rail(
    theme: &Theme,
    rail: AnyElement,
    geometry: FloatingRailGeometry,
    tint: Rgba,
    entrance: u64,
) -> AnyElement {
    animated_panel(theme, rail, geometry, tint, entrance, false)
}

/// The panel leaving (blueprint D02 exit): the same panel sliding back out
/// and fading; `key` keys the exit (one per hide).
pub fn floating_rail_leaving(
    theme: &Theme,
    rail: AnyElement,
    geometry: FloatingRailGeometry,
    tint: Rgba,
    key: u64,
) -> AnyElement {
    animated_panel(theme, rail, geometry, tint, key, true)
}

fn animated_panel(
    theme: &Theme,
    rail: AnyElement,
    geometry: FloatingRailGeometry,
    tint: Rgba,
    key: u64,
    leaving: bool,
) -> AnyElement {
    let [x, y, w, h] = geometry.rect;
    let notch = geometry.notch;
    let background = canvas(
        |_, _, _| (),
        move |bounds: Bounds<Pixels>, (), window, _cx| {
            let mut path = PathBuilder::fill();
            notched_outline(&mut path, bounds.origin, [w, h], notch);
            if let Ok(path) = path.build() {
                window.paint_path(path, tint);
            }
        },
    )
    .absolute()
    .inset_0();
    let panel = div()
        .id(FLOATING_RAIL_ID)
        .debug_selector(|| FLOATING_RAIL_ID.to_string())
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(w))
        .h(px(h))
        .rounded(px(FLOATING_RADIUS))
        // The drop shadows paint outside the panel only, so the notch
        // stays clear for the native traffic lights.
        .shadow(elev2(theme))
        .child(background)
        .child(div().absolute().inset_0().child(rail));
    let reduce = theme.motion_mode.is_reduced();
    let (name, easing) = if leaving {
        ("bbr-floating-rail-exit", Easing::Accelerate)
    } else {
        ("bbr-floating-rail-entrance", Easing::Decelerate)
    };
    panel
        .with_animation(
            (name, key),
            Animation::new(Motion::fine()).with_easing(move |t| easing.apply(t)),
            move |panel, delta| {
                // Shown share: 0 → 1 on the way in, 1 → 0 on the way out.
                let shown = if leaving { 1.0 - delta } else { delta };
                let panel = panel.opacity(shown);
                if reduce {
                    panel
                } else {
                    panel.left(px(x - FLOATING_SLIDE * (1.0 - shown)))
                }
            },
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_inset_and_notched_around_the_traffic_lights() {
        let geometry =
            FloatingRailGeometry::resolve([1440.0, 900.0], 300.0, [20.0, 11.0, 52.0, 14.0]);
        assert_eq!(geometry.rect, [8.0, 8.0, 300.0, 884.0]);
        // Lights end at x 72 / y 25: the notch reaches 8 / 6 pt past them.
        assert_eq!(geometry.notch, [72.0, 23.0]);
        assert_eq!(
            geometry.capture_rects(),
            vec![[80.0, 8.0, 228.0, 23.0], [8.0, 31.0, 300.0, 861.0]]
        );
        // Without lights (fullscreen) the whole panel captures.
        let full = FloatingRailGeometry::resolve([1440.0, 900.0], 300.0, [0.0; 4]);
        assert_eq!(full.capture_rects(), vec![full.rect]);
    }
}
