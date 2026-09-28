//! §11.8 / D10 drawn pane illustrations: the empty space (tab pills
//! floating around a ⌘ key), the crashed page (a heartbeat line) and the
//! offline load error (packets stopping at a broken link). Drawn from
//! tokens with GPUI quads and paths — never an image, never web content
//! (§13). Each plays two cycles after its pane appears and rests on its
//! final frame; Reduce Motion shows the final frame only.
//!
//! The frame functions are pure: `frame(1.0)` is exactly the rest frame, so
//! the animation ends where the static variant begins.

use gpui::{
    canvas, div, point, prelude::*, px, AnimationExt, AnyElement, Div, PathBuilder, Pixels, Rgba,
    SharedString,
};

use crate::theme::motion::Moment;
use crate::theme::{Radius, Theme};

/// One cycle of an illustration; D10 plays two.
pub const CYCLE: std::time::Duration = crate::theme::motion::ILLUSTRATION_CYCLE;

/// The three drawn panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Illustration {
    /// §11.8 empty space.
    EmptySpace,
    /// §11.8 crashed page.
    Crashed,
    /// §11.8 load error of the offline (network) class.
    Offline,
}

impl Illustration {
    /// The stable evidence name.
    pub const fn name(self) -> &'static str {
        match self {
            Illustration::EmptySpace => "empty-space",
            Illustration::Crashed => "crashed",
            Illustration::Offline => "offline",
        }
    }
}

/// The cycle phase at illustration time `t` ∈ [0, 1] (two cycles): 0 → 1
/// twice, and exactly 1 (the rest phase) at `t = 1`.
pub fn cycle_phase(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        1.0
    } else {
        (t * 2.0).fract()
    }
}

/// Empty space: the vertical float offsets (pt) of the three tab pills — a
/// gentle bob inside a sine envelope, so the pills start and rest exactly
/// in place.
pub fn empty_space_frame(t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    let envelope = (std::f32::consts::PI * t).sin();
    let mut offsets = [0.0; 3];
    for (index, offset) in offsets.iter_mut().enumerate() {
        let phase = 2.0 * std::f32::consts::TAU * t + index as f32 * std::f32::consts::TAU / 3.0;
        *offset = 3.0 * envelope * phase.sin();
    }
    offsets
}

/// Crashed: the revealed share of the heartbeat line; the rest frame is the
/// whole line.
pub fn heartbeat_frame(t: f32) -> f32 {
    cycle_phase(t)
}

/// Offline: each packet's position along the link (0 = the left node,
/// 1 = the break) — packets leave in turn and queue at the break, where the
/// rest frame leaves them.
pub fn offline_frame(t: f32) -> [f32; 3] {
    let phase = cycle_phase(t);
    let mut positions = [0.0; 3];
    for (index, position) in positions.iter_mut().enumerate() {
        let start = index as f32 * 0.18;
        let travel = ((phase - start) / (1.0 - start)).clamp(0.0, 1.0);
        let eased = crate::theme::motion::Easing::Decelerate.apply(travel);
        *position = eased * queue_position(index);
    }
    positions
}

/// Where packet `index` queues in front of the break (0..1 along the link).
fn queue_position(index: usize) -> f32 {
    1.0 - index as f32 * 0.16
}

/// The illustration at rest (`key = None`) or playing its two cycles once
/// (`key` = the pane appearance, so each appearance plays from the start).
pub fn illustration(theme: &Theme, kind: Illustration, key: Option<u64>) -> AnyElement {
    let theme = *theme;
    let draw = move |t: f32| -> AnyElement {
        match kind {
            Illustration::EmptySpace => {
                empty_space(&theme, empty_space_frame(t)).into_any_element()
            }
            Illustration::Crashed => crashed(&theme, heartbeat_frame(t)).into_any_element(),
            Illustration::Offline => offline(&theme, offline_frame(t)).into_any_element(),
        }
    };
    let plan = Moment::PaneIllustration.plan(theme.motion_mode);
    match (key, plan) {
        (Some(key), Some(plan)) => div()
            .with_animation(
                SharedString::from(format!("bbr-illustration-{}-{key}", kind.name())),
                plan.animation(),
                move |el, t| el.child(draw(t)),
            )
            .into_any_element(),
        _ => draw(1.0),
    }
}

/// A 3 pt-tall "tab pill": a white chip with a hairline edge, an accent dot
/// and a faint title line.
fn tab_pill(theme: &Theme, x: f32, y: f32) -> Div {
    let p = theme.palette;
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(34.0))
        .h(px(12.0))
        .px(px(4.0))
        .gap(px(3.0))
        .flex()
        .flex_row()
        .items_center()
        .rounded(Radius.full())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .child(div().size(px(4.0)).rounded(Radius.full()).bg(p.accent))
        .child(
            div()
                .h(px(2.0))
                .flex_1()
                .rounded(Radius.full())
                .bg(p.ink_faint),
        )
}

fn empty_space(theme: &Theme, offsets: [f32; 3]) -> Div {
    let p = theme.palette;
    div()
        .relative()
        .w(px(132.0))
        .h(px(88.0))
        .child(
            div()
                .absolute()
                .left(px(40.0))
                .top(px(18.0))
                .size(px(52.0))
                .rounded(theme.radius.medium())
                .bg(p.surface_overlay)
                .border_1()
                .border_color(p.hairline)
                .shadow(crate::theme::elevation::lift(theme))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(26.0))
                .text_color(p.ink_secondary)
                .child("⌘"),
        )
        .child(tab_pill(theme, 0.0, 12.0 + offsets[0]))
        .child(tab_pill(theme, 98.0, 22.0 + offsets[1]))
        .child(tab_pill(theme, 86.0, 70.0 + offsets[2]))
}

/// The heartbeat polyline in a 160 × 40 box (flat, a small bump, the
/// spike, the dip, flat).
const HEARTBEAT: [(f32, f32); 9] = [
    (0.0, 22.0),
    (50.0, 22.0),
    (58.0, 16.0),
    (66.0, 22.0),
    (76.0, 22.0),
    (84.0, 2.0),
    (94.0, 38.0),
    (102.0, 22.0),
    (160.0, 22.0),
];

fn crashed(theme: &Theme, reveal: f32) -> impl IntoElement {
    let colour = theme.palette.status_error;
    let stroke = px(2.0);
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let origin = bounds.origin;
            let at = |(x, y): (f32, f32)| point(origin.x + px(x), origin.y + px(y));
            let lengths: Vec<f32> = HEARTBEAT
                .windows(2)
                .map(|pair| {
                    ((pair[1].0 - pair[0].0).powi(2) + (pair[1].1 - pair[0].1).powi(2)).sqrt()
                })
                .collect();
            let mut remaining = reveal.clamp(0.0, 1.0) * lengths.iter().sum::<f32>();
            let mut path = PathBuilder::stroke(stroke);
            path.move_to(at(HEARTBEAT[0]));
            for (pair, length) in HEARTBEAT.windows(2).zip(&lengths) {
                if remaining <= 0.0 {
                    break;
                }
                let u = (remaining / length).min(1.0);
                path.line_to(at((
                    pair[0].0 + (pair[1].0 - pair[0].0) * u,
                    pair[0].1 + (pair[1].1 - pair[0].1) * u,
                )));
                remaining -= length;
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, colour);
            }
        },
    )
    .w(px(160.0))
    .h(px(40.0))
}

fn offline(theme: &Theme, positions: [f32; 3]) -> Div {
    let p = theme.palette;
    const WIDTH: f32 = 150.0;
    const NODE: f32 = 12.0;
    const BREAK_AT: f32 = 92.0;
    const GAP: f32 = 12.0;
    let link = |from: f32, to: f32, colour: Rgba| {
        div()
            .absolute()
            .left(px(from))
            .top(px(NODE / 2.0 - 1.0))
            .w(px(to - from))
            .h(px(2.0))
            .rounded(Radius.full())
            .bg(colour)
    };
    let node = |x: f32| {
        div()
            .absolute()
            .left(px(x))
            .top(px(0.0))
            .size(px(NODE))
            .rounded(Radius.full())
            .border_2()
            .border_color(p.ink_secondary)
    };
    // Packets travel from the left node's edge to the break.
    let lane = BREAK_AT - NODE - 6.0;
    div()
        .relative()
        .w(px(WIDTH))
        .h(px(NODE))
        .child(link(NODE, BREAK_AT, p.ink_faint))
        .child(link(BREAK_AT + GAP, WIDTH - NODE, p.ink_faint))
        .child(node(0.0))
        .child(node(WIDTH - NODE))
        .children(positions.iter().map(move |position| {
            div()
                .absolute()
                .left(px(NODE + lane * position - 2.0))
                .top(px(NODE / 2.0 - 2.0))
                .size(px(4.0))
                .rounded(Radius.full())
                .bg(p.accent)
        }))
}

/// The illustration's layout size (for the panes' fixed slot).
pub fn size(kind: Illustration) -> (Pixels, Pixels) {
    match kind {
        Illustration::EmptySpace => (px(132.0), px(88.0)),
        Illustration::Crashed => (px(160.0), px(40.0)),
        Illustration::Offline => (px(150.0), px(12.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_deterministic_and_end_on_the_rest_frame() {
        // Rest frames: pills in place, the whole heartbeat, packets queued.
        for offset in empty_space_frame(1.0) {
            assert!(offset.abs() < 1e-4, "pills rest in place");
        }
        for offset in empty_space_frame(0.0) {
            assert!(offset.abs() < 1e-4, "pills start in place");
        }
        assert_eq!(heartbeat_frame(1.0), 1.0);
        let rest = offline_frame(1.0);
        for (index, position) in rest.iter().enumerate() {
            assert!(
                (position - queue_position(index)).abs() < 1e-3,
                "packet {index} queued"
            );
        }
        // Deterministic: the same time gives the same frame.
        assert_eq!(offline_frame(0.3), offline_frame(0.3));
        assert_eq!(empty_space_frame(0.42), empty_space_frame(0.42));
        // Two cycles: the heartbeat restarts mid-way.
        assert!(heartbeat_frame(0.49) > 0.9 && heartbeat_frame(0.51) < 0.1);
        // Packets never pass the break.
        for step in 0..=40 {
            for position in offline_frame(step as f32 / 40.0) {
                assert!((0.0..=1.0).contains(&position));
            }
        }
    }

    #[test]
    fn the_illustration_plays_two_cycles_or_rests_under_reduce_motion() {
        use crate::theme::MotionMode;
        let plan = Moment::PaneIllustration
            .plan(MotionMode::Standard)
            .expect("animates");
        assert_eq!(plan.duration, CYCLE * 2);
        assert!(!plan.repeat, "stops requesting frames after two cycles");
        assert_eq!(Moment::PaneIllustration.plan(MotionMode::Reduced), None);
    }
}
