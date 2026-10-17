//! §11.4 sidebar slide, app side (DESIGN.md §6 principle 1a; Patrick
//! 2026-10-17): the toggle commits first, then the presentation travels.
//!
//! `toggle_sidebar` flips `sidebar_visible`, persists and routes focus on
//! the commit frame exactly as before; [`BbrApp::start_sidebar_slide`]
//! then starts a [`SidebarTransition`] from the share of the rail that was
//! on screen. Every render resolves the presented rail edge and card frame
//! from it ([`BbrApp::sidebar_presentation`]) — the native slot follows
//! because GPUI hands the slot its interpolated bounds — and requests the
//! next frame while the slide travels. A timer renders the committed
//! geometry once the slide's duration has passed even if no display-link
//! frame arrives (hidden or occluded windows), so the presentation can
//! never rest between the two states. Reduce Motion and docking the
//! floating rail present the committed geometry at once.

use std::time::Instant;

use bbr_ui::card::CardInsets;
use bbr_ui::sidebar::{presence_for, SidebarPresentation, SidebarTransition};
use bbr_ui::theme::motion::Transition;
use gpui::{div, prelude::*, px, AnyElement, AsyncApp, Context, Div, Stateful, Task, Window};

use super::BbrApp;

/// How many frames of the last slide the testkit evidence keeps.
#[cfg(feature = "testkit")]
const FRAME_HISTORY: usize = 64;

/// The app-owned slide state.
#[derive(Default)]
pub(crate) struct SidebarMotion {
    /// The running slide (`None` at rest: the committed geometry renders).
    transition: Option<SidebarTransition>,
    /// Renders the committed geometry once the slide ends, whether or not a
    /// display-link frame arrives.
    settle_task: Option<Task<()>>,
    /// This frame moved the rail (a travelling or the settling frame): the
    /// docked command field follows with an open overlay.
    moving: bool,
    #[cfg(feature = "testkit")]
    pub(crate) evidence: SlideEvidence,
}

/// Testkit evidence of the last toggle's presentation.
#[cfg(feature = "testkit")]
#[derive(Default)]
pub(crate) struct SlideEvidence {
    pub(crate) variant: Option<&'static str>,
    pub(crate) toggles: u64,
    /// The last slide's start share and duration (`None` when static).
    pub(crate) slide: Option<(f32, f32)>,
    /// The last toggle's commit instant while its settle is unmeasured.
    committed_at: Option<Instant>,
    pub(crate) last_settle_ms: Option<f32>,
    pub(crate) frames: Vec<bbr_testkit::protocol::SidebarMotionFrame>,
    /// Animation frames requested by the slide since launch.
    pub(crate) frame_requests: u64,
}

impl BbrApp {
    /// The presented share of the docked rail at `now` (`1` expanded, `0`
    /// collapsed). Pure: a running slide answers its eased share; at rest
    /// the committed state does.
    pub(crate) fn sidebar_presence_at(&self, now: Instant) -> f32 {
        match self.sidebar_motion.transition {
            Some(transition) if transition.target_visible() == self.sidebar_visible => {
                transition.presence(now)
            }
            _ => presence_for(self.sidebar_visible),
        }
    }

    /// Whether the rail is presented but already committed hidden (the
    /// leaving rail takes no pointer and never floats).
    pub(crate) fn sidebar_sliding_out(&self) -> bool {
        !self.sidebar_visible && self.sidebar_presence_at(Instant::now()) > 0.0
    }

    /// Start presenting a toggle that has just been committed. `from` is the
    /// share that was on screen before the commit; `snap` presents the
    /// committed geometry at once (docking the floating rail, whose panel
    /// already shows the rail at full width).
    pub(crate) fn start_sidebar_slide(&mut self, from: f32, snap: bool, cx: &mut Context<Self>) {
        let now = Instant::now();
        let plan = Transition::SidebarSlide.plan(self.theme().motion_mode);
        let transition = plan
            .filter(|_| !snap)
            .and_then(|plan| SidebarTransition::start(from, self.sidebar_visible, plan, now));
        #[cfg(feature = "testkit")]
        {
            let evidence = &mut self.sidebar_motion.evidence;
            evidence.toggles += 1;
            evidence.variant = Some(if transition.is_some() {
                "animated"
            } else {
                "static"
            });
            evidence.slide = transition.map(|transition| {
                (
                    transition.presence(now),
                    transition.duration().as_secs_f32() * 1_000.0,
                )
            });
            evidence.committed_at = Some(now);
            evidence.last_settle_ms = None;
            evidence.frames.clear();
        }
        self.sidebar_motion.transition = transition;
        self.sidebar_motion.settle_task = transition.map(|transition| {
            let weak = cx.weak_entity();
            // GPUI's platform timer (see `command_rank`): a foreground task
            // is not driven by smol's reactor.
            cx.spawn(async move |_, async_cx: &mut AsyncApp| {
                async_cx
                    .background_executor()
                    .timer(transition.duration())
                    .await;
                async_cx.update(|cx| {
                    let _ = weak.update(cx, |_, cx| cx.notify());
                });
            })
        });
    }

    /// Whether this frame moves the docked rail (an open ⌘L overlay must
    /// follow its field).
    pub(crate) fn sidebar_slide_moving(&self) -> bool {
        self.sidebar_motion.moving
    }

    /// End a running slide now: the next frame presents the committed
    /// geometry (a floating rail takes over from a leaving docked rail).
    pub(crate) fn settle_sidebar_slide(&mut self) {
        self.sidebar_motion.transition = None;
        self.sidebar_motion.settle_task = None;
    }

    /// Resolve this frame's rail edge and card frame (render only): the
    /// presented share between the collapsed and the expanded frame. While
    /// the slide travels the next animation frame is requested; the first
    /// frame at the committed share ends it.
    pub(crate) fn sidebar_presentation(
        &mut self,
        full_width: f32,
        window: &mut Window,
    ) -> SidebarPresentation {
        let now = Instant::now();
        let travelling = self.sidebar_motion.transition.is_some_and(|transition| {
            transition.target_visible() == self.sidebar_visible && !transition.finished(now)
        });
        // At rest (and on the settling frame) exactly the committed share.
        let presence = if travelling {
            self.sidebar_presence_at(now)
        } else {
            presence_for(self.sidebar_visible)
        };
        self.sidebar_motion.moving = travelling;
        if travelling {
            window.request_animation_frame();
        } else if self.sidebar_motion.transition.take().is_some() {
            self.sidebar_motion.settle_task = None;
            self.sidebar_motion.moving = true;
        }
        let presentation = self.sidebar_presentation_for(presence, full_width, window);
        #[cfg(feature = "testkit")]
        self.record_slide_frame(now, presence, presentation.rail_width, travelling);
        presentation
    }

    /// Place the rail for this frame: nothing while collapsed, the rail
    /// itself at rest, and while it slides the full-width rail inside a
    /// strip as wide as the presented edge, offset so its trailing edge is
    /// that edge. A leaving rail is already committed hidden: an occluding
    /// layer keeps the pointer off it.
    pub(crate) fn sliding_rail(
        &self,
        rail: Stateful<Div>,
        presentation: SidebarPresentation,
        full_width: f32,
    ) -> Option<AnyElement> {
        let width = presentation.rail_width;
        if width <= 0.0 {
            return None;
        }
        if width >= full_width {
            return Some(rail.into_any_element());
        }
        Some(
            div()
                .id("bbr-sidebar-slide")
                .relative()
                .flex_none()
                .h_full()
                .w(px(width))
                .overflow_hidden()
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(px(presentation.rail_offset(full_width)))
                        .w(px(full_width))
                        .child(rail),
                )
                .when(!self.sidebar_visible, |strip| {
                    strip.child(
                        div()
                            .id("bbr-sidebar-slide-veil")
                            .absolute()
                            .inset_0()
                            .occlude(),
                    )
                })
                .into_any_element(),
        )
    }

    /// The presentation of `presence` for this window state.
    pub(crate) fn sidebar_presentation_for(
        &self,
        presence: f32,
        full_width: f32,
        window: &Window,
    ) -> SidebarPresentation {
        let full_screen = self.is_full_screen(window);
        let band = self.safe_area(window).height;
        SidebarPresentation::resolve(
            presence,
            full_width,
            CardInsets::resolve(true, full_screen, band),
            CardInsets::resolve(false, full_screen, band),
        )
    }

    #[cfg(feature = "testkit")]
    fn record_slide_frame(
        &mut self,
        now: Instant,
        presence: f32,
        rail_width: f32,
        travelling: bool,
    ) {
        let native_x = self.performance.content_rect().map_or(0.0, |rect| rect[0]);
        let evidence = &mut self.sidebar_motion.evidence;
        if travelling {
            evidence.frame_requests += 1;
        }
        let Some(committed) = evidence.committed_at else {
            return;
        };
        let at_ms = now.saturating_duration_since(committed).as_secs_f32() * 1_000.0;
        // The travelling frames are bounded; the settled frame always ends
        // the trace.
        if !travelling || evidence.frames.len() < FRAME_HISTORY - 1 {
            evidence
                .frames
                .push(bbr_testkit::protocol::SidebarMotionFrame {
                    at_ms,
                    presence,
                    rail_width,
                    native_x,
                });
        }
        if !travelling {
            evidence.last_settle_ms = Some(at_ms);
            evidence.committed_at = None;
        }
    }
}

#[cfg(feature = "testkit")]
impl BbrApp {
    /// The card frame presented at snapshot time (the committed frame at
    /// rest, the interpolated one while the rail slides).
    pub(crate) fn presented_card_frame(&self, window: &Window) -> CardInsets {
        let presence = self.sidebar_presence_at(Instant::now());
        let full_width = self
            .sidebar
            .effective_width(f32::from(window.bounds().size.width));
        self.sidebar_presentation_for(presence, full_width, window)
            .card
    }

    /// The typed §11.4 slide evidence at snapshot time.
    pub(crate) fn sidebar_motion_snapshot(
        &self,
        window: &Window,
    ) -> bbr_testkit::protocol::SidebarMotionSnapshot {
        let now = Instant::now();
        let presence = self.sidebar_presence_at(now);
        let full_width = self
            .sidebar
            .effective_width(f32::from(window.bounds().size.width));
        let evidence = &self.sidebar_motion.evidence;
        bbr_testkit::protocol::SidebarMotionSnapshot {
            variant: evidence.variant.map(str::to_string),
            animating: self.sidebar_motion.transition.is_some_and(|transition| {
                transition.target_visible() == self.sidebar_visible && !transition.finished(now)
            }),
            presence,
            presented_width: self
                .sidebar_presentation_for(presence, full_width, window)
                .rail_width,
            toggles: evidence.toggles,
            start_presence: evidence.slide.map(|(start, _)| start),
            duration_ms: evidence.slide.map(|(_, duration)| duration),
            last_settle_ms: evidence.last_settle_ms,
            frames: evidence.frames.clone(),
        }
    }
}
