//! The v2 motion helper (DESIGN.md §6, §18.5): the easing curves built from
//! the §6 tokens and the transient-moment descriptors every animated cue
//! resolves through, so each moment has exactly one timing authority and
//! one Reduce Motion variant.
//!
//! A moment only ever decorates a state that is already committed and
//! presented (§6 principle 1): the descriptors carry no delay, and the
//! caller renders the committed state on the commit frame whether or not
//! the moment animates. Loops exist only for loading and progress; a
//! descriptor of `None` means "render the static counterpart and request no
//! frames".

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use gpui::Animation;

/// Animation elements built by the moment builders since launch. Every
/// animated moment goes through [`MomentPlan::animation`], so this counter
/// is the render-side evidence that a frame built (or did not build) an
/// element that requests animation frames — observable even when the
/// display link does not tick (a hidden or locked-screen test window).
static ANIMATED_BUILT: AtomicU64 = AtomicU64::new(0);

/// How many animation elements the moment builders have built.
pub fn animated_elements_built() -> u64 {
    ANIMATED_BUILT.load(Ordering::Relaxed)
}

use crate::theme::{Motion, MotionMode};

/// The §6 easing tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// Loops and sweeps.
    Linear,
    /// `ease:standard` — `cubic-bezier(0.22, 1, 0.36, 1)`.
    Standard,
    /// `ease:accelerate` — `cubic-bezier(0.36, 0, 1, 1)`.
    Accelerate,
    /// `ease:decelerate` — `cubic-bezier(0, 0, 0.3, 1)` (D11).
    Decelerate,
}

impl Easing {
    /// The token's cubic-bezier control points `(x1, y1, x2, y2)`.
    pub const fn control_points(self) -> (f32, f32, f32, f32) {
        match self {
            Easing::Linear => (0.0, 0.0, 1.0, 1.0),
            Easing::Standard => (0.22, 1.0, 0.36, 1.0),
            Easing::Accelerate => (0.36, 0.0, 1.0, 1.0),
            Easing::Decelerate => (0.0, 0.0, 0.3, 1.0),
        }
    }

    /// Evaluate the curve at time `t` ∈ [0, 1]: solve the bezier's x for
    /// `t` (bisection — monotonic for every token), return its y clamped to
    /// [0, 1] (GPUI animations require the eased delta to stay in range).
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if self == Easing::Linear {
            return t;
        }
        let (x1, y1, x2, y2) = self.control_points();
        let bezier = |a: f32, b: f32, s: f32| {
            let inv = 1.0 - s;
            3.0 * inv * inv * s * a + 3.0 * inv * s * s * b + s * s * s
        };
        let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
        for _ in 0..32 {
            let mid = 0.5 * (lo + hi);
            if bezier(x1, x2, mid) < t {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        bezier(y1, y2, 0.5 * (lo + hi)).clamp(0.0, 1.0)
    }
}

/// The transient moments of the motion core (§18.5 table, S6 rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moment {
    /// §10.5: the favicon-slot loading arc.
    LoadingArc,
    /// §10.6: the attention dot's two-beat pulse.
    AttentionPulse,
    /// D5: the address field's indeterminate progress sweep.
    AddressProgress,
    /// D5: the address progress line filling, then fading, on finish.
    AddressProgressFinish,
    /// §18.5: a new Favorite tile pops and radiates specks.
    FavoritePop,
    /// §18.5: a finished download's check draws with one `status.ok` ring.
    DownloadCheck,
    /// §18.5: ⌘T results fade in with a 20 ms stagger and a 4 pt rise.
    CommandResults,
    /// §18.5: the ⌘T cursor highlight glides to the committed result.
    CommandCursor,
    /// D4: a committed space switch cross-fades the chrome tint.
    SpaceTintFade,
    /// D10: a drawn pane illustration plays two cycles, then rests.
    PaneIllustration,
}

/// One cycle of a D10 pane illustration.
pub const ILLUSTRATION_CYCLE: Duration = Duration::from_millis(1400);

/// The ⌘T stagger step between consecutive result rows.
pub const STAGGER_STEP: Duration = Duration::from_millis(20);
/// Rows past this index share the last stagger slot, so the whole moment
/// stays within `dur:emphasis` (7 × 20 ms + `dur:fine` = 260 ms).
pub const STAGGER_ROWS: usize = 7;

/// How a moment animates under the active motion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MomentPlan {
    pub duration: Duration,
    /// Loops restart until their state ends; one-shots rest on the final
    /// frame and request no further frames.
    pub repeat: bool,
    pub easing: Easing,
    /// The Reduce Motion variant: opacity only — no rotation, scale or
    /// travel.
    pub reduced: bool,
}

impl MomentPlan {
    /// The GPUI animation for this plan (counted as one built animation
    /// element — call it exactly once per element the render builds).
    pub fn animation(self) -> Animation {
        ANIMATED_BUILT.fetch_add(1, Ordering::Relaxed);
        let easing = self.easing;
        let animation = Animation::new(self.duration).with_easing(move |t| easing.apply(t));
        if self.repeat {
            animation.repeat()
        } else {
            animation
        }
    }
}

impl Moment {
    /// Every moment in §18.5 table order (the Gallery motion page and the
    /// evidence tests enumerate this).
    pub const ALL: [Moment; 10] = [
        Moment::LoadingArc,
        Moment::AttentionPulse,
        Moment::AddressProgress,
        Moment::AddressProgressFinish,
        Moment::FavoritePop,
        Moment::DownloadCheck,
        Moment::CommandResults,
        Moment::CommandCursor,
        Moment::SpaceTintFade,
        Moment::PaneIllustration,
    ];

    /// The stable wire/evidence name of the moment.
    pub const fn name(self) -> &'static str {
        match self {
            Moment::LoadingArc => "loading-arc",
            Moment::AttentionPulse => "attention-pulse",
            Moment::AddressProgress => "address-progress",
            Moment::AddressProgressFinish => "address-progress-finish",
            Moment::FavoritePop => "favorite-pop",
            Moment::DownloadCheck => "download-check",
            Moment::CommandResults => "command-results",
            Moment::CommandCursor => "command-cursor",
            Moment::SpaceTintFade => "space-tint-fade",
            Moment::PaneIllustration => "pane-illustration",
        }
    }

    /// The moment's plan for `mode`; `None` renders the static counterpart
    /// (§18.5 Reduced Motion column) and requests no frames.
    pub fn plan(self, mode: MotionMode) -> Option<MomentPlan> {
        let reduced = mode.is_reduced();
        match self {
            // 640 ms per turn, linear; Reduce Motion: static arc.
            Moment::LoadingArc => (!reduced).then_some(MomentPlan {
                duration: Duration::from_millis(640),
                repeat: true,
                easing: Easing::Linear,
                reduced: false,
            }),
            // Two beats of `dur:fine` (≤ `dur:emphasis`); Reduce Motion
            // keeps the opacity-only §10.6 pulse.
            Moment::AttentionPulse => Some(MomentPlan {
                duration: Motion::fine() * 2,
                repeat: false,
                easing: Easing::Standard,
                reduced,
            }),
            // `load:indeterminate` sweep; Reduce Motion: static line.
            Moment::AddressProgress => (!reduced).then_some(MomentPlan {
                duration: Motion::load_loop(),
                repeat: true,
                easing: Easing::Linear,
                reduced: false,
            }),
            // Fill for `dur:standard`, then fade for `dur:standard`;
            // Reduce Motion removes the line on finish instead.
            Moment::AddressProgressFinish => (!reduced).then_some(MomentPlan {
                duration: Motion::standard() * 2,
                repeat: false,
                easing: Easing::Linear,
                reduced: false,
            }),
            // Pop with overshoot and specks within `dur:emphasis`; Reduce
            // Motion: a `dur:fine` `tint.accent` flash on the tile.
            Moment::FavoritePop => Some(if reduced {
                MomentPlan {
                    duration: Motion::fine(),
                    repeat: false,
                    easing: Easing::Linear,
                    reduced: true,
                }
            } else {
                MomentPlan {
                    duration: Motion::emphasis(),
                    repeat: false,
                    easing: Easing::Linear,
                    reduced: false,
                }
            }),
            // Check stroke draw plus one ring; Reduce Motion: the check
            // simply appears.
            Moment::DownloadCheck => (!reduced).then_some(MomentPlan {
                duration: Motion::emphasis(),
                repeat: false,
                easing: Easing::Linear,
                reduced: false,
            }),
            // Stagger + rise; Reduce Motion keeps the fade without the rise.
            Moment::CommandResults => Some(MomentPlan {
                duration: Motion::emphasis(),
                repeat: false,
                easing: Easing::Linear,
                reduced,
            }),
            // `dur:micro` glide; Reduce Motion: the cursor snaps.
            Moment::CommandCursor => (!reduced).then_some(MomentPlan {
                duration: Motion::micro(),
                repeat: false,
                easing: Easing::Standard,
                reduced: false,
            }),
            // D4: `dur:standard` with `ease:standard`; Reduce Motion: instant.
            Moment::SpaceTintFade => (!reduced).then_some(MomentPlan {
                duration: Motion::standard(),
                repeat: false,
                easing: Easing::Standard,
                reduced: false,
            }),
            // D10: two cycles once, then the rest frame; Reduce Motion: the
            // rest frame only. Not a single-shot moment of the ≤ 260 ms
            // family: it decorates a whole pane that just appeared.
            Moment::PaneIllustration => (!reduced).then_some(MomentPlan {
                duration: ILLUSTRATION_CYCLE * 2,
                repeat: false,
                easing: Easing::Linear,
                reduced: false,
            }),
        }
    }

    /// The evidence variant of the moment under `mode`: `animated`,
    /// `reduced` (opacity-only animation) or `static` (no animation).
    pub fn variant(self, mode: MotionMode) -> &'static str {
        match self.plan(mode) {
            Some(plan) if plan.reduced => "reduced",
            Some(_) => "animated",
            None => "static",
        }
    }
}

/// §18.5 attention pulse sample at eased time `t`: `(dot scale, dot opacity,
/// ring diameter pt, ring opacity)`. Two beats — out and back — with one
/// ring expanding 4 → 16 pt and fading; the reduced variant keeps the
/// §10.6 opacity-only 1 → 0.45 → 1 at scale 1 and draws no ring.
pub fn attention_sample(t: f32, reduced: bool) -> (f32, f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    // Triangle 0 → 1 → 0 across the two beats.
    let beat = 1.0 - (2.0 * t - 1.0).abs();
    if reduced {
        (1.0, 1.0 - 0.55 * beat, 0.0, 0.0)
    } else {
        (1.0 + 0.45 * beat, 1.0, 4.0 + 12.0 * t, 1.0 - t)
    }
}

/// §18.5 favorite pop sample at (linear) time `t`: `(tile scale, speck
/// travel pt, speck opacity, flash opacity)`. Full motion: the tile pops
/// 0.86 → 1 with a back-ease overshoot while eight specks radiate 0 → 18 pt
/// and fade; reduced: a `tint.accent` flash fading 1 → 0 at scale 1.
pub fn favorite_pop_sample(t: f32, reduced: bool) -> (f32, f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    if reduced {
        return (1.0, 0.0, 0.0, 1.0 - t);
    }
    // Back-ease-out (overshoot ≈ 6 %), mapped 0.86 → 1.
    let (c1, c3) = (1.701_58_f32, 2.701_58_f32);
    let back = 1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2);
    let scale = 0.86 + 0.14 * back;
    let travel = 18.0 * Easing::Decelerate.apply(t);
    (scale, travel, 1.0 - t, 0.0)
}

/// §18.5 download check sample at time `t`: `(check stroke fraction, ring
/// growth pt, ring opacity)` — the stroke draws over the first half, the
/// ring expands 0 → 8 pt and fades over the whole moment.
pub fn download_check_sample(t: f32) -> (f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    let stroke = Easing::Decelerate.apply((t * 2.0).min(1.0));
    (stroke, 8.0 * Easing::Decelerate.apply(t), 1.0 - t)
}

/// §18.5 ⌘T result entrance at moment time `t` for row `index`: `(opacity,
/// rise pt)`. Row `i` starts `min(i, 7) × 20 ms` in and fades over
/// `dur:fine`; the full variant also rises 4 pt, the reduced variant only
/// fades.
pub fn result_entrance_sample(t: f32, index: usize, reduced: bool) -> (f32, f32) {
    let total = Motion::emphasis().as_secs_f32();
    let delay = STAGGER_STEP.as_secs_f32() * index.min(STAGGER_ROWS) as f32;
    let local =
        ((t.clamp(0.0, 1.0) * total - delay) / Motion::fine().as_secs_f32()).clamp(0.0, 1.0);
    let eased = Easing::Standard.apply(local);
    let rise = if reduced { 0.0 } else { 4.0 * (1.0 - eased) };
    (eased, rise)
}

/// D5 finish sample at time `t`: `(line width fraction, opacity)` — the
/// line fills during the first half, then fades during the second.
pub fn progress_finish_sample(t: f32, from_fraction: f32) -> (f32, f32) {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        let fill = Easing::Decelerate.apply(t * 2.0);
        (from_fraction + (1.0 - from_fraction) * fill, 1.0)
    } else {
        (1.0, 1.0 - Easing::Accelerate.apply((t - 0.5) * 2.0))
    }
}

/// D5 indeterminate sweep sample at time `t`: `(left fraction, width
/// fraction)` of a segment travelling across the field and wrapping.
pub fn progress_sweep_sample(t: f32) -> (f32, f32) {
    const SEGMENT: f32 = 0.3;
    let t = t.clamp(0.0, 1.0);
    let left = -SEGMENT + (1.0 + SEGMENT) * t;
    let start = left.max(0.0);
    let end = (left + SEGMENT).min(1.0);
    (start, (end - start).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn easing_curves_match_the_tokens_at_sample_points() {
        for easing in [
            Easing::Linear,
            Easing::Standard,
            Easing::Accelerate,
            Easing::Decelerate,
        ] {
            assert!(close(easing.apply(0.0), 0.0), "{easing:?} starts at 0");
            assert!(close(easing.apply(1.0), 1.0), "{easing:?} ends at 1");
            let mut last = 0.0;
            for step in 0..=20 {
                let y = easing.apply(step as f32 / 20.0);
                assert!((0.0..=1.0).contains(&y));
                assert!(y + 1e-4 >= last, "{easing:?} is monotonic");
                last = y;
            }
        }
        assert!(close(Easing::Linear.apply(0.5), 0.5));
        // ease:standard rushes out and settles: far past half at t = 0.5.
        assert!(Easing::Standard.apply(0.5) > 0.85);
        // ease:accelerate starts slow and ends fast.
        assert!(Easing::Accelerate.apply(0.5) < 0.35);
        // D11: ease:decelerate is (0, 0, 0.3, 1) — a real deceleration,
        // not the transposed near-linear ramp.
        assert_eq!(Easing::Decelerate.control_points(), (0.0, 0.0, 0.3, 1.0));
        assert!(Easing::Decelerate.apply(0.25) > 0.5);
    }

    #[test]
    fn moments_resolve_full_and_reduced_variants() {
        let full = MotionMode::Standard;
        let reduced = MotionMode::Reduced;
        let arc = Moment::LoadingArc.plan(full).expect("arc animates");
        assert_eq!(arc.duration, Duration::from_millis(640));
        assert!(arc.repeat);
        assert_eq!(Moment::LoadingArc.plan(reduced), None, "static arc");
        let pulse = Moment::AttentionPulse.plan(full).expect("pulse");
        assert!(!pulse.repeat && !pulse.reduced);
        assert!(pulse.duration <= Motion::emphasis(), "≤ dur:emphasis");
        let quiet = Moment::AttentionPulse.plan(reduced).expect("reduced pulse");
        assert!(quiet.reduced, "opacity-only under Reduce Motion");
        let sweep = Moment::AddressProgress.plan(full).expect("sweep");
        assert_eq!(sweep.duration, Motion::load_loop());
        assert!(sweep.repeat);
        assert_eq!(Moment::AddressProgress.plan(reduced), None);
        let finish = Moment::AddressProgressFinish.plan(full).expect("finish");
        assert_eq!(finish.duration, Motion::standard() * 2);
        assert!(!finish.repeat);
        assert_eq!(Moment::AddressProgressFinish.plan(reduced), None);
        assert_eq!(Moment::LoadingArc.variant(full), "animated");
        assert_eq!(Moment::LoadingArc.variant(reduced), "static");
        assert_eq!(Moment::AttentionPulse.variant(reduced), "reduced");
        // Only loading and progress loop.
        for moment in [
            Moment::LoadingArc,
            Moment::AttentionPulse,
            Moment::AddressProgress,
            Moment::AddressProgressFinish,
        ] {
            if let Some(plan) = moment.plan(full) {
                assert_eq!(
                    plan.repeat,
                    matches!(moment, Moment::LoadingArc | Moment::AddressProgress),
                    "{moment:?}"
                );
                assert!(plan.repeat || plan.duration <= Motion::emphasis() * 2);
            }
        }
    }

    #[test]
    fn after_commit_moments_stay_bounded_one_shots() {
        let full = MotionMode::Standard;
        let reduced = MotionMode::Reduced;
        for moment in [
            Moment::FavoritePop,
            Moment::DownloadCheck,
            Moment::CommandResults,
            Moment::CommandCursor,
            Moment::SpaceTintFade,
        ] {
            let plan = moment.plan(full).expect("animates under full motion");
            assert!(!plan.repeat, "{moment:?} never loops");
            assert!(plan.duration <= Motion::emphasis(), "{moment:?} ≤ 260 ms");
        }
        assert_eq!(
            Moment::CommandCursor.plan(full).unwrap().duration,
            Motion::micro()
        );
        assert_eq!(
            Moment::SpaceTintFade.plan(full).unwrap().duration,
            Motion::standard()
        );
        assert_eq!(
            Moment::SpaceTintFade.plan(full).unwrap().easing,
            Easing::Standard
        );
        // Reduced variants: flash, static check, fade-only results, snapping
        // cursor, instant tint.
        let flash = Moment::FavoritePop.plan(reduced).expect("flash");
        assert!(flash.reduced && flash.duration == Motion::fine());
        assert_eq!(Moment::DownloadCheck.plan(reduced), None);
        assert!(Moment::CommandResults.plan(reduced).unwrap().reduced);
        assert_eq!(Moment::CommandCursor.plan(reduced), None);
        assert_eq!(Moment::SpaceTintFade.plan(reduced), None);
    }

    #[test]
    fn after_commit_samples_follow_the_documented_shapes() {
        // Favorite: 0.86 → overshoot → 1; specks travel out and fade.
        let (start, travel, opacity, flash) = favorite_pop_sample(0.0, false);
        assert!(close(start, 0.86) && close(travel, 0.0));
        assert!(close(opacity, 1.0) && flash == 0.0);
        let peak = (1..20)
            .map(|step| favorite_pop_sample(step as f32 / 20.0, false).0)
            .fold(0.0, f32::max);
        assert!(peak > 1.0, "the pop overshoots");
        assert!(close(favorite_pop_sample(1.0, false).0, 1.0));
        assert!(close(favorite_pop_sample(1.0, false).1, 18.0));
        assert_eq!(favorite_pop_sample(0.0, true), (1.0, 0.0, 0.0, 1.0));
        // Download: the stroke is drawn by mid-moment; the ring fades out.
        assert!(close(download_check_sample(0.5).0, 1.0));
        assert!(close(download_check_sample(1.0).2, 0.0));
        // Results: row 0 enters first, later rows wait; everything is in by
        // the end; the reduced variant never rises.
        let (first, rise) = result_entrance_sample(0.2, 0, false);
        let (later, _) = result_entrance_sample(0.2, 5, false);
        assert!(first > later && rise < 4.0);
        for index in [0, 3, 7, 20] {
            let (opacity, rise) = result_entrance_sample(1.0, index, false);
            assert!(
                close(opacity, 1.0) && close(rise, 0.0),
                "row {index} settled"
            );
        }
        assert_eq!(result_entrance_sample(0.1, 0, true).1, 0.0);
    }

    #[test]
    fn samples_follow_the_documented_shapes() {
        // Attention: rest at both ends, peak scale 1.45 mid-way, ring 4→16.
        assert_eq!(attention_sample(0.0, false), (1.0, 1.0, 4.0, 1.0));
        let (scale, _, ring, fade) = attention_sample(0.5, false);
        assert!(close(scale, 1.45) && close(ring, 10.0) && close(fade, 0.5));
        assert_eq!(attention_sample(1.0, false).0, 1.0);
        assert!(close(attention_sample(1.0, false).2, 16.0));
        // Reduced: opacity 1 → 0.45 → 1, never scale, never a ring.
        let (scale, opacity, ring, _) = attention_sample(0.5, true);
        assert!(close(scale, 1.0) && close(opacity, 0.45) && ring == 0.0);
        // Finish: fills to full width, then fades out.
        assert!(close(progress_finish_sample(0.5, 0.3).0, 1.0));
        assert!(close(progress_finish_sample(1.0, 0.3).1, 0.0));
        // Sweep: the segment stays inside the field.
        for step in 0..=10 {
            let (left, width) = progress_sweep_sample(step as f32 / 10.0);
            assert!(left >= 0.0 && left + width <= 1.0 + 1e-6);
        }
    }
}
