//! v2 motion core, app side (DESIGN.md §6, §18.5): the D5 address progress
//! state and its finish timer. The timing of every moment lives in
//! `bbr_ui::theme::motion`; this module only decides *which* committed
//! state is being decorated, so a moment never delays or replaces a commit.

use std::time::Instant;

use bbr_ui::components::AddressProgress;
use bbr_ui::theme::motion::Moment;
use bbr_web::WebViewId;
use gpui::{AsyncApp, Context, Task};

use super::BbrApp;

/// The app-owned motion state: the in-flight D5 finish (which view, until
/// when, and the timer that removes it) plus the testkit render counter.
#[derive(Default)]
pub(crate) struct MotionState {
    /// The view whose load just finished and the instant its finish ends.
    finishing: Option<(WebViewId, Instant)>,
    /// The timer that clears the finish and re-renders once it has played.
    finish_task: Option<Task<()>>,
    /// Root-view renders since launch — the idle evidence (an animation
    /// frame re-renders the root view).
    #[cfg(feature = "testkit")]
    pub(crate) root_renders: u64,
}

impl BbrApp {
    /// D5: the active tab's load progress for the address field — the sweep
    /// while it loads, the fill-and-fade right after it finished (full
    /// motion only), nothing otherwise.
    pub(crate) fn address_progress(&self) -> AddressProgress {
        if self.active_load().is_some() {
            return AddressProgress::Loading;
        }
        match (
            self.motion.finishing,
            self.controller.snapshot().active_view,
        ) {
            (Some((view, until)), Some(active)) if view == active && Instant::now() < until => {
                AddressProgress::Finishing
            }
            _ => AddressProgress::Idle,
        }
    }

    /// The D5 line the address control actually renders: the progress
    /// while the rail shows the address control, or `Idle` — a finish under
    /// Reduce Motion renders nothing (the same decision the render makes).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub(crate) fn rendered_address_progress(&self) -> AddressProgress {
        if !self.sidebar_visible || !self.command_slot_is_address() {
            return AddressProgress::Idle;
        }
        match self.address_progress() {
            AddressProgress::Finishing
                if Moment::AddressProgressFinish
                    .plan(self.theme().motion_mode)
                    .is_none() =>
            {
                AddressProgress::Idle
            }
            progress => progress,
        }
    }

    /// The active view while the active tab is loading (read around an
    /// engine event, so the event that ends the load is recognised whatever
    /// its kind).
    pub(crate) fn active_load(&self) -> Option<WebViewId> {
        let snapshot = self.controller.snapshot();
        snapshot
            .active_index
            .and_then(|index| snapshot.tabs.get(index))
            .filter(|tab| tab.loading)
            .and_then(|tab| tab.web_view)
    }

    /// A load on `view` finished: when it is the active view and full
    /// motion applies, play the D5 fill-and-fade and remove it afterwards
    /// (Reduce Motion removes the line on finish, so nothing plays there).
    pub(crate) fn note_load_finished(&mut self, view: WebViewId, cx: &mut Context<Self>) {
        let Some(plan) = Moment::AddressProgressFinish.plan(self.theme().motion_mode) else {
            return;
        };
        if self.controller.snapshot().active_view != Some(view) {
            return;
        }
        let until = Instant::now() + plan.duration;
        self.motion.finishing = Some((view, until));
        let weak = cx.weak_entity();
        // GPUI's platform timer (see `command_rank`): a foreground task is
        // not driven by smol's reactor.
        self.motion.finish_task = Some(cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            async_cx.background_executor().timer(plan.duration).await;
            async_cx.update(|cx| {
                let _ = weak.update(cx, |app, cx| {
                    if app.motion.finishing.map(|(_, end)| end) == Some(until) {
                        app.motion.finishing = None;
                        cx.notify();
                    }
                });
            });
        }));
    }
}
