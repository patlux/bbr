//! §18.5 after-commit moments, app side (task packet `v2-motion-moments`):
//! the favorite pop, the download check, the ⌘T result entrance and cursor
//! glide, and the D4 space-tint fade.
//!
//! A moment is started by the code path that has **already committed** its
//! state (the favorite exists, the download succeeded, the result set or
//! cursor is published, the destination space is active) and is only a
//! decoration of the next frames: it is keyed, bounded by its
//! `bbr_ui::theme::motion` plan, and removed by a timer afterwards. Nothing
//! here can delay or replace a commit.

use std::time::Instant;

use bbr_ui::theme::motion::Moment;
use gpui::{AsyncApp, Context, Rgba, Task};

use super::BbrApp;

/// What a running moment decorates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MomentData {
    /// The favorite tile of this tab id pops.
    Favorite { tab: u64 },
    /// The downloads row of this download id draws its check.
    Download { id: u64 },
    /// The ⌘T result set enters.
    Results,
    /// The ⌘T cursor glides from this row index.
    Cursor { from: usize },
    /// The chrome tint fades from this source colour.
    Tint { from: Rgba },
}

/// One running moment.
#[derive(Debug, Clone)]
pub(crate) struct ActiveMoment {
    pub moment: Moment,
    pub data: MomentData,
    /// Unique per start: the animation element key, so every start plays
    /// from its first frame.
    pub key: u64,
    pub until: Instant,
    /// The §15 id the moment decorates (evidence).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub target: String,
    /// The variant chosen at start (`animated` / `reduced`) — evidence
    /// stays truthful if the motion mode changes while it plays.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub variant: &'static str,
}

/// The registry of running after-commit moments.
#[derive(Default)]
pub(crate) struct Moments {
    next_key: u64,
    active: Vec<ActiveMoment>,
    timers: Vec<(u64, Task<()>)>,
    /// Set by the swipe commit so the destination tint (already reached by
    /// the gesture) does not fade a second time (D4).
    pub(crate) swipe_commit: bool,
    /// The most recent moment starts, newest last (testkit evidence that a
    /// moment began, independent of how long it runs).
    #[cfg(feature = "testkit")]
    pub(crate) started: std::collections::VecDeque<(Moment, String, &'static str)>,
}

/// How many recent starts the evidence keeps.
#[cfg(feature = "testkit")]
const STARTED_HISTORY: usize = 16;

impl BbrApp {
    /// Start `moment` decorating `data` (after its state is committed).
    /// Under Reduce Motion a moment without a reduced animation is recorded
    /// as `static` and draws its committed state only.
    pub(crate) fn start_moment(
        &mut self,
        moment: Moment,
        data: MomentData,
        target: String,
        cx: &mut Context<Self>,
    ) {
        let mode = self.theme().motion_mode;
        #[cfg(feature = "testkit")]
        {
            let started = &mut self.moments.started;
            started.push_back((moment, target.clone(), moment.variant(mode)));
            while started.len() > STARTED_HISTORY {
                started.pop_front();
            }
        }
        // One instance per moment kind: a newer commit supersedes the
        // older decoration — even a static (Reduce Motion) one, so a stale
        // fade or glide never runs over the newer committed state.
        let replaced: Vec<u64> = self
            .moments
            .active
            .iter()
            .filter(|active| active.moment == moment)
            .map(|active| active.key)
            .collect();
        self.moments.active.retain(|active| active.moment != moment);
        self.moments
            .timers
            .retain(|(key, _)| !replaced.contains(key));
        let Some(plan) = moment.plan(mode) else {
            return;
        };
        self.moments.next_key += 1;
        let key = self.moments.next_key;
        self.moments.active.push(ActiveMoment {
            moment,
            data,
            key,
            until: Instant::now() + plan.duration,
            target,
            variant: moment.variant(mode),
        });
        let weak = cx.weak_entity();
        // GPUI's platform timer (see `command_rank`).
        let timer = cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            async_cx.background_executor().timer(plan.duration).await;
            async_cx.update(|cx| {
                let _ = weak.update(cx, |app, cx| {
                    app.moments.active.retain(|active| active.key != key);
                    app.moments.timers.retain(|(timer, _)| *timer != key);
                    cx.notify();
                });
            });
        });
        self.moments.timers.push((key, timer));
    }

    /// §18.5: a favorite was just added (drop, menu or seam) — its tile pops.
    pub(crate) fn start_favorite_pop(&mut self, tab: bbr_domain::TabId, cx: &mut Context<Self>) {
        let index = self
            .organization
            .organization_snapshot(&self.controller)
            .favorites
            .iter()
            .position(|favorite| favorite.tab == tab);
        let Some(index) = index else {
            return;
        };
        let target = bbr_ui::components::favorites::favorite_tile_id(index).to_string();
        self.start_moment(
            Moment::FavoritePop,
            MomentData::Favorite { tab: tab.0 },
            target,
            cx,
        );
    }

    /// §18.5: a download just succeeded — its row's check draws.
    pub(crate) fn start_download_check(
        &mut self,
        id: bbr_web::downloads::DownloadId,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.downloads.row_ids().iter().position(|row| *row == id) else {
            return;
        };
        let target = bbr_ui::surfaces::downloads::download_slot_id("status", index);
        self.start_moment(
            Moment::DownloadCheck,
            MomentData::Download { id: id.0 },
            target,
            cx,
        );
    }

    /// §18.5: a result set was just published into the open ⌘T palette.
    pub(crate) fn start_command_entrance(&mut self, cx: &mut Context<Self>) {
        if self.command_surface == super::palette::CommandSurface::Palette
            && self.command_ranking.state == super::command_rank::CommandResultState::Results
        {
            self.start_moment(
                Moment::CommandResults,
                MomentData::Results,
                bbr_ui::surfaces::PALETTE_ID.to_string(),
                cx,
            );
        }
    }

    /// §18.5: the ⌘T cursor just moved from `from` (the selection is
    /// already committed).
    pub(crate) fn start_command_glide(&mut self, from: usize, cx: &mut Context<Self>) {
        if self.command_surface == super::palette::CommandSurface::Palette {
            self.start_moment(
                Moment::CommandCursor,
                MomentData::Cursor { from },
                bbr_ui::surfaces::PALETTE_ID.to_string(),
                cx,
            );
        }
    }

    /// The running instance of `moment`, if any.
    pub(crate) fn active_moment(&self, moment: Moment) -> Option<&ActiveMoment> {
        let now = Instant::now();
        self.moments
            .active
            .iter()
            .find(|active| active.moment == moment && now < active.until)
    }

    /// §18.5 favorite pop: `(tab id, key)` while it plays.
    pub(crate) fn favorite_pop(&self) -> Option<(u64, u64)> {
        match self.active_moment(Moment::FavoritePop)? {
            ActiveMoment {
                data: MomentData::Favorite { tab },
                key,
                ..
            } => Some((*tab, *key)),
            _ => None,
        }
    }

    /// §18.5 download check: `(download id, key)` while it plays.
    pub(crate) fn download_check(&self) -> Option<(u64, u64)> {
        match self.active_moment(Moment::DownloadCheck)? {
            ActiveMoment {
                data: MomentData::Download { id },
                key,
                ..
            } => Some((*id, *key)),
            _ => None,
        }
    }

    /// §18.5 ⌘T result entrance key while it plays.
    pub(crate) fn command_entrance(&self) -> Option<u64> {
        self.active_moment(Moment::CommandResults)
            .map(|active| active.key)
    }

    /// §18.5 ⌘T cursor glide: `(from row, key)` while it plays.
    pub(crate) fn command_glide(&self) -> Option<(usize, u64)> {
        match self.active_moment(Moment::CommandCursor)? {
            ActiveMoment {
                data: MomentData::Cursor { from },
                key,
                ..
            } => Some((*from, *key)),
            _ => None,
        }
    }

    /// D4 space-tint fade: `(source chrome, key)` while it plays.
    pub(crate) fn tint_fade(&self) -> Option<(Rgba, u64)> {
        match self.active_moment(Moment::SpaceTintFade)? {
            ActiveMoment {
                data: MomentData::Tint { from },
                key,
                ..
            } => Some((*from, *key)),
            _ => None,
        }
    }

    /// The running after-commit moments (testkit evidence).
    #[cfg(feature = "testkit")]
    pub(crate) fn running_moments(&self) -> impl Iterator<Item = &ActiveMoment> {
        let now = Instant::now();
        self.moments
            .active
            .iter()
            .filter(move |active| now < active.until)
    }
}
