//! The docked rail's slide (DESIGN.md §6 principle 1a, §11.4; Patrick
//! 2026-10-17): the pure presentation model of a committed sidebar toggle.
//!
//! The toggle commits first — `sidebar_visible`, persistence and focus
//! change on the commit frame — and the presentation then travels from the
//! share of the rail that was on screen to the committed one. The share
//! (*presence*) is `1` for the expanded rail and `0` for the collapsed one;
//! the rail width and every card inset are interpolated from it, so the
//! rail edge, the content card and the native slot always move together.
//! The rail keeps its full width and slides: its trailing edge is the
//! presented width, the content left of it leaves the window.
//!
//! A toggle while a slide runs reverses from the presented share and only
//! covers the remaining distance, so the rail never jumps. Under Reduce
//! Motion there is no transition and the committed geometry renders at once.

use std::time::{Duration, Instant};

use crate::card::CardInsets;
use crate::theme::motion::{Easing, MomentPlan};

/// The presence of a settled rail: `1` expanded, `0` collapsed.
pub fn presence_for(visible: bool) -> f32 {
    if visible {
        1.0
    } else {
        0.0
    }
}

/// One running slide from the presented share towards the committed one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarTransition {
    from: f32,
    to: f32,
    start: Instant,
    duration: Duration,
    easing: Easing,
}

impl SidebarTransition {
    /// Start sliding towards the committed `visible` state from the share
    /// `from` that is on screen at `now`. The plan's duration covers the
    /// whole travel; a reversal covers only the remaining distance in the
    /// matching share of it. `None` when nothing has to travel.
    pub fn start(from: f32, visible: bool, plan: MomentPlan, now: Instant) -> Option<Self> {
        let from = from.clamp(0.0, 1.0);
        let to = presence_for(visible);
        let distance = (to - from).abs();
        if distance <= f32::EPSILON {
            return None;
        }
        Some(Self {
            from,
            to,
            start: now,
            duration: plan.duration.mul_f32(distance),
            easing: plan.easing,
        })
    }

    /// The presented share at `now`: eased from the start share towards the
    /// committed share, resting on the committed share once the slide ended.
    pub fn presence(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start);
        let t = if self.duration.is_zero() {
            1.0
        } else {
            elapsed.as_secs_f32() / self.duration.as_secs_f32()
        };
        if t >= 1.0 {
            // Exactly the committed share (no f32 residue at rest).
            return self.to;
        }
        self.from + (self.to - self.from) * self.easing.apply(t)
    }

    /// Whether the slide has reached the committed share at `now`.
    pub fn finished(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.start) >= self.duration
    }

    /// How long this slide travels.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// The committed visibility the slide travels to.
    pub fn target_visible(&self) -> bool {
        self.to > 0.5
    }
}

/// The geometry one frame presents for a share of the rail.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarPresentation {
    /// The presented rail width: the rail's trailing edge in window points
    /// (`0` collapsed, the effective rail width expanded).
    pub rail_width: f32,
    /// The content card frame between the collapsed and the expanded frame.
    pub card: CardInsets,
}

impl SidebarPresentation {
    /// Present `presence` of a rail whose settled width is `full_width`,
    /// between the `collapsed` and `expanded` card frames. The settled ends
    /// are exact; in between every value is rounded to whole points so the
    /// rail edge, the card and the native slot stay on the pixel grid.
    pub fn resolve(
        presence: f32,
        full_width: f32,
        expanded: CardInsets,
        collapsed: CardInsets,
    ) -> Self {
        let presence = presence.clamp(0.0, 1.0);
        if presence >= 1.0 {
            return Self {
                rail_width: full_width,
                card: expanded,
            };
        }
        if presence <= 0.0 {
            return Self {
                rail_width: 0.0,
                card: collapsed,
            };
        }
        let mix = |closed: f32, open: f32| (closed + (open - closed) * presence).round();
        Self {
            rail_width: mix(0.0, full_width),
            card: CardInsets {
                top: mix(collapsed.top, expanded.top),
                trailing: mix(collapsed.trailing, expanded.trailing),
                bottom: mix(collapsed.bottom, expanded.bottom),
                leading: mix(collapsed.leading, expanded.leading),
                radius: mix(collapsed.radius, expanded.radius),
                shadow: expanded.shadow || collapsed.shadow,
            },
        }
    }

    /// Where the full-width rail sits inside the presented strip: its
    /// leading edge, `0` or negative while it slides.
    pub fn rail_offset(&self, full_width: f32) -> f32 {
        self.rail_width - full_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::motion::Transition;
    use crate::theme::MotionMode;

    fn plan() -> MomentPlan {
        Transition::SidebarSlide
            .plan(MotionMode::Standard)
            .expect("full motion slides")
    }

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn a_full_slide_travels_the_plan_and_rests_on_the_committed_share() {
        let now = Instant::now();
        let out = SidebarTransition::start(1.0, false, plan(), now).expect("collapse slides");
        assert_eq!(out.duration(), plan().duration);
        assert!(!out.target_visible());
        assert_eq!(out.presence(now), 1.0, "the first frame starts at the rail");
        let early = out.presence(now + ms(30));
        let mid = out.presence(now + ms(130));
        assert!(early < 1.0 && early > mid && mid > 0.0, "{early} {mid}");
        // ease:standard: most of the travel early, then a short settle.
        assert!(mid < 0.15, "far past half by the midpoint: {mid}");
        assert!(!out.finished(now + ms(259)));
        assert!(out.finished(now + plan().duration));
        assert_eq!(out.presence(now + plan().duration), 0.0);
        assert_eq!(out.presence(now + ms(5_000)), 0.0, "rests afterwards");

        let back = SidebarTransition::start(0.0, true, plan(), now).expect("expand slides");
        assert!(back.target_visible());
        assert!(back.presence(now + ms(60)) > 0.5);
        assert_eq!(back.presence(now + plan().duration), 1.0);
    }

    #[test]
    fn a_reversal_continues_from_the_presented_share_over_the_remaining_distance() {
        let now = Instant::now();
        let out = SidebarTransition::start(1.0, false, plan(), now).unwrap();
        let reversed_at = now + ms(20);
        let presented = out.presence(reversed_at);
        let back = SidebarTransition::start(presented, true, plan(), reversed_at).unwrap();
        assert_eq!(back.presence(reversed_at), presented, "no jump");
        // Odd start shares still end exactly on the committed share.
        let odd = SidebarTransition::start(0.3, true, plan(), now).unwrap();
        assert_eq!(odd.presence(now + odd.duration()), 1.0);
        let expected = plan().duration.mul_f32(1.0 - presented);
        assert!(back.duration().abs_diff(expected) < ms(1));
        assert!(back.duration() < plan().duration);
        assert_eq!(back.presence(reversed_at + back.duration()), 1.0);
        // Nothing to travel: no transition.
        assert_eq!(SidebarTransition::start(1.0, true, plan(), now), None);
        assert_eq!(SidebarTransition::start(0.0, false, plan(), now), None);
    }

    #[test]
    fn the_presentation_moves_the_rail_edge_and_the_card_together() {
        let expanded = CardInsets::resolve(true, false, 52.0);
        let collapsed = CardInsets::resolve(false, false, 52.0);
        let open = SidebarPresentation::resolve(1.0, 300.0, expanded, collapsed);
        assert_eq!((open.rail_width, open.card), (300.0, expanded));
        let closed = SidebarPresentation::resolve(0.0, 300.0, expanded, collapsed);
        assert_eq!((closed.rail_width, closed.card), (0.0, collapsed));

        let half = SidebarPresentation::resolve(0.5, 300.0, expanded, collapsed);
        assert_eq!(half.rail_width, 150.0);
        assert_eq!(half.card.top, 31.0);
        assert_eq!(half.card.leading, 5.0);
        assert_eq!((half.card.trailing, half.card.bottom), (10.0, 10.0));
        assert!(half.card.shadow);
        assert_eq!(
            half.rail_offset(300.0),
            -150.0,
            "the rail slides, full width"
        );
        // The native slot follows the presented edge: strictly between the
        // settled slots, never negative.
        let size = [1080.0, 700.0];
        let slot = half.card.inner_rect(size, half.rail_width);
        assert_eq!(slot[0], 155.0);
        assert!(slot[0] > expanded.inner_rect(size, 0.0)[0]);
        assert!(slot[0] < expanded.inner_rect(size, 300.0)[0]);
        // Intermediate values sit on whole points.
        let odd = SidebarPresentation::resolve(0.333, 301.0, expanded, collapsed);
        assert_eq!(odd.rail_width, odd.rail_width.round());
        assert_eq!(odd.card.top, odd.card.top.round());
        // A fractional settled width is presented exactly at rest.
        let settled = SidebarPresentation::resolve(1.0, 287.5, expanded, collapsed);
        assert_eq!(settled.rail_width, 287.5);
    }

    #[test]
    fn full_screen_keeps_the_full_bleed_card_while_the_rail_slides() {
        let expanded = CardInsets::resolve(true, true, 52.0);
        let collapsed = CardInsets::resolve(false, true, 52.0);
        let frame = SidebarPresentation::resolve(0.4, 300.0, expanded, collapsed);
        assert_eq!(frame.card, expanded, "full screen has one card frame");
        assert_eq!(frame.rail_width, 120.0);
    }
}
