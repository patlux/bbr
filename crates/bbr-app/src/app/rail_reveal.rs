//! Blueprint D02 (DESIGN §2 §11.4): when the floating rail appears and when
//! it goes away — a pure model, fed with pointer samples from the native
//! observer (or the testkit seam) and the elapsed timers.
//!
//! - **Fast arrival:** the pointer reaches the left edge moving at least
//!   600 pt/s leftward over the last ~80 ms, horizontally dominant — the
//!   rail appears at once.
//! - **Slow arrival:** the pointer rests in the edge zone for 300 ms.
//! - **No trigger** while a mouse button is held, or while the app blocks
//!   it (a menu or the palette is open, the rail is docked).
//! - **Leaving** the rail's domain hides it unless an interaction holds it
//!   (a drag, an open menu or panel, an edit). A fast arrival that
//!   overshoots the window's left edge waits 1.2 s for the pointer to come
//!   back.
//! - After an explicit hide (Escape), the pointer must leave the edge zone
//!   before the rail can reveal again, so a resting pointer never reopens it.
//!
//! Samples carry their own timestamps (the native event time), so the
//! velocity is exact and the testkit can replay a fast or a slow arrival
//! deterministically.

use std::collections::VecDeque;

/// The dwell before a slow arrival reveals the rail.
pub const DWELL_MS: u64 = 300;
/// The horizontal speed (pt/s, leftward) that reveals the rail at once.
pub const FAST_SPEED: f32 = 600.0;
/// The sample window the velocity is measured over.
pub const VELOCITY_WINDOW_MS: f64 = 80.0;
/// How long an overshooting fast arrival waits for the pointer to return.
pub const OVERSHOOT_MS: u64 = 1200;
/// The edge zone in a window (the collapsed rail's chassis strip).
pub const EDGE_WINDOWED: f32 = 10.0;
/// The edge zone in fullscreen (a narrow native-observed edge).
pub const EDGE_FULL_SCREEN: f32 = 3.0;

/// One pointer sample in window points (top-left origin) with the event
/// time in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerSample {
    pub x: f32,
    pub y: f32,
    pub time_ms: f64,
    /// A mouse button is held.
    pub pressed: bool,
}

/// What the app must do after a model step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealAction {
    None,
    /// Start the dwell timer for `generation`.
    ArmDwell(u64),
    /// Present the floating rail.
    Show,
    /// Present the floating rail and start the overshoot timer.
    ShowOvershoot(u64),
    /// Hide the floating rail.
    Hide,
}

/// The reveal phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RevealPhase {
    #[default]
    Idle,
    Dwelling {
        generation: u64,
    },
    Floating {
        generation: u64,
        /// A fast arrival overshot the window; the rail waits for the
        /// pointer to come back.
        overshoot: bool,
    },
}

/// The app facts a sample is judged against.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RevealContext {
    /// The edge zone width (window or fullscreen).
    pub edge: f32,
    /// The floating rail's rect `[x, y, w, h]` (its domain while shown).
    pub rail: [f32; 4],
    /// The window height (samples below or above it do not count).
    pub window_height: f32,
    /// The app blocks a reveal (docked rail, menu or palette open).
    pub blocked: bool,
    /// An interaction holds the shown rail (drag, open menu or panel,
    /// an edit in its field).
    pub held: bool,
}

/// The reveal model.
#[derive(Debug, Clone, Default)]
pub struct RailReveal {
    phase: RevealPhase,
    generation: u64,
    samples: VecDeque<PointerSample>,
    /// After an explicit hide the pointer must leave the edge first.
    rearm_required: bool,
}

impl RailReveal {
    /// The phase (the testkit reports it).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn phase(&self) -> RevealPhase {
        self.phase
    }

    pub fn is_floating(&self) -> bool {
        matches!(self.phase, RevealPhase::Floating { .. })
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The horizontal and vertical speed (pt/s) over the sample window.
    pub fn velocity(&self) -> (f32, f32) {
        let (Some(first), Some(last)) = (self.samples.front(), self.samples.back()) else {
            return (0.0, 0.0);
        };
        let dt = (last.time_ms - first.time_ms) / 1000.0;
        if dt <= 0.0 {
            return (0.0, 0.0);
        }
        (
            ((last.x - first.x) as f64 / dt) as f32,
            ((last.y - first.y) as f64 / dt) as f32,
        )
    }

    fn fast_leftward(&self) -> bool {
        let (vx, vy) = self.velocity();
        vx <= -FAST_SPEED && vx.abs() > vy.abs()
    }

    fn record(&mut self, sample: PointerSample) {
        self.samples.push_back(sample);
        while self
            .samples
            .front()
            .is_some_and(|first| sample.time_ms - first.time_ms > VELOCITY_WINDOW_MS)
        {
            self.samples.pop_front();
        }
    }

    /// One pointer sample.
    pub fn on_sample(&mut self, sample: PointerSample, context: RevealContext) -> RevealAction {
        self.record(sample);
        let in_height = sample.y >= 0.0 && sample.y < context.window_height;
        let at_edge = in_height && sample.x <= context.edge;
        if !at_edge {
            self.rearm_required = false;
        }
        match self.phase {
            RevealPhase::Idle | RevealPhase::Dwelling { .. } => {
                if context.blocked || sample.pressed || self.rearm_required || !at_edge {
                    self.phase = RevealPhase::Idle;
                    return RevealAction::None;
                }
                if self.fast_leftward() {
                    return self.show(sample.x < 0.0);
                }
                if sample.x < 0.0 {
                    // Outside the window, not fast: no dwell there.
                    self.phase = RevealPhase::Idle;
                    return RevealAction::None;
                }
                if let RevealPhase::Dwelling { .. } = self.phase {
                    return RevealAction::None;
                }
                self.generation += 1;
                self.phase = RevealPhase::Dwelling {
                    generation: self.generation,
                };
                RevealAction::ArmDwell(self.generation)
            }
            RevealPhase::Floating {
                generation,
                overshoot,
            } => {
                let [x, y, w, h] = context.rail;
                let inside = (sample.x >= 0.0 && sample.x <= context.edge && in_height)
                    || (sample.x >= x && sample.x < x + w && sample.y >= y && sample.y < y + h);
                if inside {
                    self.phase = RevealPhase::Floating {
                        generation,
                        overshoot: false,
                    };
                    return RevealAction::None;
                }
                if context.held || (overshoot && sample.x < 0.0) {
                    return RevealAction::None;
                }
                self.phase = RevealPhase::Idle;
                RevealAction::Hide
            }
        }
    }

    fn show(&mut self, overshoot: bool) -> RevealAction {
        self.generation += 1;
        self.phase = RevealPhase::Floating {
            generation: self.generation,
            overshoot,
        };
        if overshoot {
            RevealAction::ShowOvershoot(self.generation)
        } else {
            RevealAction::Show
        }
    }

    /// The dwell timer of `generation` elapsed: reveal if the pointer is
    /// still resting at the edge.
    pub fn on_dwell_elapsed(&mut self, generation: u64) -> RevealAction {
        match self.phase {
            RevealPhase::Dwelling { generation: armed } if armed == generation => self.show(false),
            _ => RevealAction::None,
        }
    }

    /// The overshoot timer of `generation` elapsed: hide unless the pointer
    /// came back into the rail.
    pub fn on_overshoot_elapsed(&mut self, generation: u64, held: bool) -> RevealAction {
        match self.phase {
            RevealPhase::Floating {
                generation: shown,
                overshoot: true,
            } if shown == generation && !held => {
                self.phase = RevealPhase::Idle;
                RevealAction::Hide
            }
            _ => RevealAction::None,
        }
    }

    /// Show on an explicit command (⌘L or ⌥⌘S on a hidden rail).
    pub fn show_now(&mut self) -> RevealAction {
        if self.is_floating() {
            return RevealAction::None;
        }
        self.show(false)
    }

    /// Hide on an explicit command (Escape): the pointer must leave the
    /// edge before the rail can reveal again.
    pub fn hide_now(&mut self) -> RevealAction {
        let was = self.is_floating();
        self.phase = RevealPhase::Idle;
        self.rearm_required = true;
        if was {
            RevealAction::Hide
        } else {
            RevealAction::None
        }
    }

    /// End every phase without a hide presentation (docking, deactivation,
    /// minimize, close).
    pub fn reset(&mut self) {
        self.phase = RevealPhase::Idle;
        self.samples.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> RevealContext {
        RevealContext {
            edge: EDGE_WINDOWED,
            rail: [8.0, 8.0, 300.0, 884.0],
            window_height: 900.0,
            blocked: false,
            held: false,
        }
    }

    fn at(x: f32, time_ms: f64) -> PointerSample {
        PointerSample {
            x,
            y: 400.0,
            time_ms,
            pressed: false,
        }
    }

    #[test]
    fn a_fast_leftward_arrival_reveals_at_once() {
        let mut reveal = RailReveal::default();
        // 400 pt → 4 pt in 40 ms: ~9900 pt/s leftward.
        assert_eq!(
            reveal.on_sample(at(400.0, 0.0), context()),
            RevealAction::None
        );
        assert_eq!(
            reveal.on_sample(at(200.0, 20.0), context()),
            RevealAction::None
        );
        assert_eq!(
            reveal.on_sample(at(4.0, 40.0), context()),
            RevealAction::Show
        );
        assert!(reveal.is_floating());
    }

    #[test]
    fn a_slow_arrival_dwells_300_ms_and_leaving_the_edge_cancels() {
        let mut reveal = RailReveal::default();
        reveal.on_sample(at(30.0, 0.0), context());
        // 30 → 6 pt over 200 ms: 120 pt/s, slow.
        let action = reveal.on_sample(at(6.0, 200.0), context());
        let RevealAction::ArmDwell(generation) = action else {
            panic!("slow arrival arms the dwell: {action:?}");
        };
        // Still resting: the dwell reveals.
        assert_eq!(
            reveal.on_sample(at(5.0, 300.0), context()),
            RevealAction::None
        );
        assert_eq!(reveal.on_dwell_elapsed(generation), RevealAction::Show);
        // A second dwell, cancelled by leaving the edge before it elapses.
        let mut reveal = RailReveal::default();
        let RevealAction::ArmDwell(generation) = reveal.on_sample(at(6.0, 0.0), context()) else {
            panic!("arms");
        };
        reveal.on_sample(at(40.0, 100.0), context());
        assert_eq!(reveal.on_dwell_elapsed(generation), RevealAction::None);
        assert!(!reveal.is_floating());
    }

    #[test]
    fn a_held_button_or_a_blocking_surface_never_reveals() {
        let mut reveal = RailReveal::default();
        let mut pressed = at(4.0, 40.0);
        pressed.pressed = true;
        reveal.on_sample(at(400.0, 0.0), context());
        assert_eq!(reveal.on_sample(pressed, context()), RevealAction::None);
        let blocked = RevealContext {
            blocked: true,
            ..context()
        };
        let mut reveal = RailReveal::default();
        reveal.on_sample(at(400.0, 0.0), blocked);
        assert_eq!(reveal.on_sample(at(4.0, 40.0), blocked), RevealAction::None);
    }

    #[test]
    fn leaving_the_rail_hides_it_unless_an_interaction_holds_it() {
        let mut reveal = RailReveal::default();
        reveal.show_now();
        assert_eq!(
            reveal.on_sample(at(200.0, 0.0), context()),
            RevealAction::None
        );
        let held = RevealContext {
            held: true,
            ..context()
        };
        assert_eq!(reveal.on_sample(at(700.0, 50.0), held), RevealAction::None);
        assert!(reveal.is_floating());
        assert_eq!(
            reveal.on_sample(at(700.0, 100.0), context()),
            RevealAction::Hide
        );
        assert!(!reveal.is_floating());
    }

    #[test]
    fn an_overshoot_waits_for_the_pointer_to_come_back() {
        let mut reveal = RailReveal::default();
        reveal.on_sample(at(300.0, 0.0), context());
        let RevealAction::ShowOvershoot(generation) = reveal.on_sample(at(-40.0, 40.0), context())
        else {
            panic!("a fast exit past the edge reveals with the overshoot wait");
        };
        // Still outside: the rail stays.
        assert_eq!(
            reveal.on_sample(at(-60.0, 60.0), context()),
            RevealAction::None
        );
        assert_eq!(
            reveal.on_overshoot_elapsed(generation, false),
            RevealAction::Hide
        );
        // Coming back cancels the wait.
        let mut reveal = RailReveal::default();
        reveal.on_sample(at(300.0, 0.0), context());
        let RevealAction::ShowOvershoot(generation) = reveal.on_sample(at(-40.0, 40.0), context())
        else {
            panic!("reveals");
        };
        reveal.on_sample(at(100.0, 400.0), context());
        assert_eq!(
            reveal.on_overshoot_elapsed(generation, false),
            RevealAction::None
        );
        assert!(reveal.is_floating());
    }

    #[test]
    fn after_escape_the_pointer_must_leave_the_edge_first() {
        let mut reveal = RailReveal::default();
        reveal.show_now();
        assert_eq!(reveal.hide_now(), RevealAction::Hide);
        // Resting at the edge: no dwell.
        assert_eq!(
            reveal.on_sample(at(4.0, 1000.0), context()),
            RevealAction::None
        );
        // Leave, come back: the dwell arms again.
        reveal.on_sample(at(80.0, 2000.0), context());
        assert!(matches!(
            reveal.on_sample(at(4.0, 3000.0), context()),
            RevealAction::ArmDwell(_)
        ));
    }
}
