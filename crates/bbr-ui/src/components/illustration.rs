//! §11.8 / D10 drawn pane illustrations (S12: the Paper posters): the
//! empty space (six tab cards floating around a ⌘ key cap), the crashed
//! page (an accent heartbeat swept by a travelling beat) and the offline
//! load error (laptop ↔ globe, packets queueing at a broken link). The
//! drawing lives in `illustration_draw`. Drawn from
//! tokens with GPUI quads and paths — never an image, never web content
//! (§13). Each plays two cycles after its pane appears and rests on its
//! final frame; Reduce Motion shows the final frame only.
//!
//! The frame functions are pure: `frame(1.0)` is exactly the rest frame, so
//! the animation ends where the static variant begins.

use gpui::{div, prelude::*, px, AnimationExt, AnyElement, Pixels, SharedString};

use crate::theme::motion::Moment;
use crate::theme::Theme;

use super::illustration_draw::{
    crashed, empty_space, offline, BEAT_H, BEAT_W, EMPTY_H, EMPTY_W, LINK_H, LINK_W,
};

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

/// Empty space: each tab card's drift `(dx, dy)` (pt) and the ⌘ key's
/// press (0 = up, 1 = down 2 pt).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpaceFrame {
    /// The six cards' drift from their poster positions.
    pub cards: [(f32, f32); 6],
    /// The key press.
    pub press: f32,
}

/// The cards' poster phases (the Paper drift offsets them per card).
const CARD_PHASES: [f32; 6] = [0.0, 1.1, 2.3, 3.1, 4.2, 5.0];

/// Empty space: one slow drift loop inside a sine envelope, so the cards
/// and the key start and rest exactly on the poster.
pub fn empty_space_frame(t: f32) -> SpaceFrame {
    let t = t.clamp(0.0, 1.0);
    let envelope = (std::f32::consts::PI * t).sin();
    let w = std::f32::consts::TAU * t;
    let mut cards = [(0.0, 0.0); 6];
    for (index, (card, phase)) in cards.iter_mut().zip(CARD_PHASES).enumerate() {
        let turn = if index % 2 == 1 { 1.0 } else { -1.0 };
        *card = (
            8.0 * envelope * (w + phase).sin(),
            7.0 * envelope * (w * turn + phase).cos(),
        );
    }
    let press = envelope * (0.5 + 0.5 * (2.0 * w).sin());
    SpaceFrame { cards, press }
}

/// Where the heartbeat head starts, ends and rests (artboard pt): it enters
/// off the left edge, leaves off the right, and rests mid-line (the poster).
pub const HEAD_START: f32 = -40.0;
/// See [`HEAD_START`].
pub const HEAD_END: f32 = 460.0;
/// See [`HEAD_START`].
pub const HEAD_REST: f32 = 210.0;

/// Crashed: the head's artboard x. The first cycle sweeps the whole line;
/// the second decelerates into the rest position, so the rest frame is the
/// poster.
pub fn heartbeat_frame(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        HEAD_START + (HEAD_END - HEAD_START) * (t * 2.0)
    } else {
        let u = crate::theme::motion::Easing::Decelerate.apply((t - 0.5) * 2.0);
        HEAD_START + (HEAD_REST - HEAD_START) * u
    }
}

/// Offline: each packet's `(position, opacity)` along the lane (0 = the
/// laptop, 1 = the break) and the broken link's pulse (0..1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OfflineFrame {
    /// The three packets.
    pub packets: [(f32, f32); 3],
    /// The broken link's pulse.
    pub pulse: f32,
}

/// Offline: the packets loop twice (staggered by a third, fading into the
/// break) and the link pulses four times; both loops close on the poster.
pub fn offline_frame(t: f32) -> OfflineFrame {
    let t = t.clamp(0.0, 1.0);
    let mut packets = [(0.0, 0.0); 3];
    for (index, packet) in packets.iter_mut().enumerate() {
        let u = (2.0 * t + index as f32 / 3.0).fract();
        let fade = ((u - 0.75) / 0.25).clamp(0.0, 1.0);
        *packet = (u, 1.0 - fade);
    }
    let pulse = 0.5 + 0.5 * (std::f32::consts::TAU * 4.0 * t).sin();
    OfflineFrame { packets, pulse }
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

/// The illustration's layout size (for the panes' fixed slot).
pub fn size(kind: Illustration) -> (Pixels, Pixels) {
    let (w, h) = match kind {
        Illustration::EmptySpace => (EMPTY_W, EMPTY_H),
        Illustration::Crashed => (BEAT_W, BEAT_H),
        Illustration::Offline => (LINK_W, LINK_H),
    };
    (px(w), px(h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_deterministic_and_end_on_the_rest_frame() {
        // Rest frames = the posters: cards in place with the key up, the
        // head mid-line, packets a third apart at full opacity.
        for at in [0.0, 1.0] {
            let frame = empty_space_frame(at);
            for (dx, dy) in frame.cards {
                assert!(dx.abs() < 1e-4 && dy.abs() < 1e-4, "cards on the poster");
            }
            assert!(frame.press.abs() < 1e-4, "key up");
        }
        assert_eq!(heartbeat_frame(1.0), HEAD_REST);
        assert_eq!(heartbeat_frame(0.0), HEAD_START);
        for at in [0.0, 1.0] {
            let rest = offline_frame(at);
            for (index, (position, opacity)) in rest.packets.iter().enumerate() {
                assert!(
                    (position - index as f32 / 3.0).abs() < 1e-3,
                    "packet {index} queued"
                );
                assert_eq!(*opacity, 1.0);
            }
            assert!((rest.pulse - 0.5).abs() < 1e-3);
        }
        // Deterministic: the same time gives the same frame.
        assert_eq!(offline_frame(0.3), offline_frame(0.3));
        assert_eq!(empty_space_frame(0.42), empty_space_frame(0.42));
        // Two cycles: the first sweep leaves the line, the second re-enters.
        assert!(heartbeat_frame(0.49) > 440.0 && heartbeat_frame(0.51) < 0.0);
        // The motion stays inside the drawing: drift ≤ 8 pt, packets on the
        // lane, the head never past the sweep.
        for step in 0..=40 {
            let t = step as f32 / 40.0;
            for (dx, dy) in empty_space_frame(t).cards {
                assert!(dx.abs() <= 8.0 && dy.abs() <= 7.0);
            }
            for (position, opacity) in offline_frame(t).packets {
                assert!((0.0..1.0).contains(&position));
                assert!((0.0..=1.0).contains(&opacity));
            }
            assert!((HEAD_START..=HEAD_END).contains(&heartbeat_frame(t)));
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
