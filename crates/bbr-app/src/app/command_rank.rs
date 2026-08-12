//! Cancellable, nonblocking command-result ranking orchestration
//! (DESIGN.md §11.5).
//!
//! The result surface consumes the portable `bbr-command` core through
//! [`corpus_from_snapshot`] + [`bbr_command::rank`]: a keystroke restarts a
//! debounce timer, the timer's task re-ranks from the current controller
//! snapshot on the main thread and publishes only if its generation is still
//! current (stale generations never publish). Ranking never runs from the
//! GPUI render method. The testkit gateway can flush the pending debounce
//! deterministically (`Request::FlushCommandRanking`) so E2E asserts burst
//! typing without waiting on real time.
//!
//! The typed state machine (inactive / empty / loading / results /
//! activation-error) and the selected result index (Up/Down move it without
//! leaving the field) live here; activation routing stays in the app
//! interaction layer so every activation goes through existing app/controller
//! methods.

use std::time::Duration;

use bbr_browser::ControllerSnapshot;
use bbr_command::corpus::HistoryEntry;
use bbr_command::parse::{Mode, Query, QueryContext};
use bbr_command::rank::rank;
use bbr_command::registry::CommandRegistry;
use bbr_persistence::{CollectionRecord, SavedPageRecord};
use gpui::{AsyncApp, Context, Task};

use super::command_corpus::{corpus_from_snapshot, RESULT_LIMIT};
use super::BbrApp;
use crate::command_bar::CommandBarMode;

/// The debounce delay between the last keystroke and a ranking pass.
///
/// The Phase-5 warm interaction budget is 50 ms p95 / 100 ms max and
/// `command_type` measures the real debounce plus ranking plus the rendered
/// result presentation, so the production delay must stay defensibly ≤ 30 ms
/// while remaining cancellable and nonblocking (each keystroke drops the
/// pending task and replaces it, so only the final generation ever
/// publishes). The render-inclusive measurement on this host measured
/// ~9.5 ms of rank + palette render/layout work after the timer, so a 40 ms
/// debounce pinned the p95 at the budget edge; 30 ms keeps the same
/// nonblocking cancellable burst semantics with comfortable headroom for
/// the actual rank/presentation work.
pub const DEBOUNCE_MS: u64 = 30;

/// The typed state of the command result surface (DESIGN.md §11.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandResultState {
    /// The command bar is not ranking (no location/new-tab surface).
    Inactive,
    /// The ranking surface is open with no text to rank.
    Empty,
    /// A debounce generation is pending; ranking has not published yet.
    Loading,
    /// The latest published generation produced ranked results.
    Results,
    /// The last activation was rejected (unavailable destination).
    ActivationError,
}

/// One published ranked row. The typed [`Activation`] stays app-local so the
/// activation path routes through existing controller/app methods; the
/// testkit snapshot mapping strips it onto the wire `CommandResultSnapshot`.
pub use super::command_result::CommandResultRow;

/// The result-surface state plus the cancellable debounce handle.
pub struct CommandRanking {
    pub state: CommandResultState,
    /// The latest published generation's rows (limit 16), in display order.
    pub results: Vec<CommandResultRow>,
    /// The truthful total match count of the latest published generation
    /// before the limit-16 window truncation (the full-search benchmark
    /// presentation evidence; reset whenever the results are cleared).
    pub total: usize,
    /// The selected row index (Up/Down move it within bounds).
    pub selected: usize,
    /// Monotonic generation; a publish only lands when it matches the
    /// current generation, so stale debounce tasks are never published.
    pub generation: u64,
    /// The last activation rejection (state `ActivationError`).
    pub activation_error: Option<String>,
    /// The debounce task of the current pending generation; replacing it
    /// drops (cancels) the previous one.
    pending: Option<Task<()>>,
}

impl Default for CommandRanking {
    fn default() -> Self {
        Self {
            state: CommandResultState::Inactive,
            results: Vec::new(),
            total: 0,
            selected: 0,
            generation: 0,
            activation_error: None,
            pending: None,
        }
    }
}

/// Map one portable ranked result onto the app row. Destructive actions
/// (the settled §11.5 destructive set) carry the destructive flag that
/// drives the error/destructive tint; no confirmation dialog is invented.
pub use super::command_result::row_from_result;

impl CommandRanking {
    /// The surface opened (`Cmd-L` / `Cmd-T`): reset error state and start a
    /// fresh generation for the current text.
    pub fn open(&mut self) {
        self.activation_error = None;
        self.generation += 1;
        self.state = CommandResultState::Empty;
        self.selected = 0;
        self.results.clear();
        self.total = 0;
    }

    /// The surface closed: cancel the pending debounce and return to
    /// `Inactive`. Dropping the pending task cancels its timer callback.
    pub fn close(&mut self) {
        self.pending = None;
        self.state = CommandResultState::Inactive;
        self.results.clear();
        self.selected = 0;
        self.total = 0;
        self.activation_error = None;
    }

    /// Schedule a debounced ranking for the current text. Bumps the
    /// generation (invalidating any in-flight task) and replaces the pending
    /// task, so burst typing cancels every previous debounce.
    pub fn schedule(&mut self, text: &str, cx: &mut Context<BbrApp>) {
        self.generation += 1;
        let generation = self.generation;
        let state = if text.trim().is_empty() {
            CommandResultState::Empty
        } else {
            CommandResultState::Loading
        };
        self.state = state;
        if state == CommandResultState::Empty {
            self.results.clear();
            self.selected = 0;
            self.total = 0;
            self.pending = None;
            return;
        }
        let weak = cx.weak_entity();
        self.pending = Some(cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI's platform-driven timer (dispatch_after), not `smol::Timer`:
            // a foreground-spawned task is polled by the main-thread executor,
            // which never drives smol's reactor, so a `smol::Timer` would
            // never fire in the real app. `background_executor().timer()`
            // schedules through the platform dispatcher and wakes the task
            // on time, retaining the cancellable nonblocking debounce.
            async_cx
                .background_executor()
                .timer(Duration::from_millis(DEBOUNCE_MS))
                .await;
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                let _ = weak.update(cx, |view, cx| view.publish_command_ranking(generation, cx));
            });
        }));
    }

    /// The testkit deterministic flush: cancel the pending timer so a
    /// synchronous publish of the current generation (app-owned) is the only
    /// publication that lands. The app method then ranks immediately from
    /// authoritative state, so burst typing asserts only the final
    /// generation without waiting on real time.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn flush(&mut self) {
        self.pending = None;
    }
}

/// Build the corpus from the authoritative controller snapshot and the
/// app-owned in-memory providers (history + Saved Pages) on demand, then
/// rank `text` in `mode`. Purely synchronous; used by the debounce publish,
/// the testkit flush, and the activation fallback derivation. `history` and
/// `saved_pages`/`collections` are the already-loaded provider data —
/// ranking never scans SQLite.
pub fn rank_text(
    snapshot: &ControllerSnapshot,
    history: &[HistoryEntry],
    saved_pages: &[SavedPageRecord],
    collections: &[CollectionRecord],
    mode: CommandBarMode,
    text: &str,
) -> (Vec<CommandResultRow>, usize) {
    let core_mode = match mode {
        CommandBarMode::Location => Mode::Location,
        CommandBarMode::NewTab => Mode::NewTab,
        _ => return (Vec::new(), 0),
    };
    let corpus = corpus_from_snapshot(snapshot, history, saved_pages, collections);
    let registry = CommandRegistry::builtin();
    let query = Query::new(core_mode, text);
    let context = QueryContext::with_current(
        bbr_command::id::SpaceId(snapshot.active_space.0),
        bbr_command::id::ProfileId(snapshot.active_profile.0),
    );
    let ranked = rank(&corpus, &registry, &query, &context, RESULT_LIMIT);
    let total = ranked.total;
    let rows = ranked.results.into_iter().map(row_from_result).collect();
    (rows, total)
}

impl BbrApp {
    /// Open the ranking surface for a location/new-tab mode and schedule the
    /// first debounced pass for the current text.
    pub fn open_command_ranking(&mut self, cx: &mut Context<Self>) {
        self.command_ranking.open();
        self.command_ranking.schedule(&self.command_text.text, cx);
        cx.notify();
    }

    /// Re-schedule ranking after the command text changed (every keystroke
    /// bumps the generation and cancels the previous debounce).
    pub fn on_command_text_changed(&mut self, cx: &mut Context<Self>) {
        self.command_ranking.schedule(&self.command_text.text, cx);
        cx.notify();
    }

    /// Rank synchronously from the authoritative controller snapshot and
    /// publish the current generation (shared by the debounce callback, the
    /// testkit flush, and the empty-state reset).
    pub fn rank_command_now(&mut self) {
        if !matches!(
            self.command_bar_mode,
            CommandBarMode::Location | CommandBarMode::NewTab
        ) {
            self.command_ranking.close();
            return;
        }
        let snapshot = self.controller.snapshot();
        let text = self.command_text.text.clone();
        if text.trim().is_empty() {
            self.command_ranking.state = CommandResultState::Empty;
            self.command_ranking.results.clear();
            self.command_ranking.selected = 0;
            self.command_ranking.total = 0;
            self.command_ranking.activation_error = None;
            return;
        }
        let (rows, total) = self.store.with_history(|history| {
            self.store.with_saved_pages(|saved_pages, collections| {
                rank_text(
                    &snapshot,
                    history,
                    saved_pages,
                    collections,
                    self.command_bar_mode,
                    &text,
                )
            })
        });
        // Location/NewTab queries always carry the navigate/search fallback
        // row (the portable core), so a published non-empty pass is never
        // empty; there is no production NoMatch state in this slice.
        debug_assert!(!rows.is_empty(), "fallback row always publishes");
        self.command_ranking.results = rows;
        self.command_ranking.total = total;
        self.command_ranking.state = CommandResultState::Results;
        self.command_ranking.selected = self
            .command_ranking
            .selected
            .min(self.command_ranking.results.len().saturating_sub(1));
        self.command_ranking.activation_error = None;
    }

    /// The debounce callback: publish only when `generation` is still the
    /// current one, so stale tasks never land. The publication makes the
    /// result state the render derives the §11.5 palette from, but it never
    /// emits a measurement itself: when a `CommandType` benchmark is pending,
    /// the sample commits only from a subsequent render of the result
    /// surface (the probe predicate enforces generation/result identity
    /// against the identity the render actually drew).
    pub fn publish_command_ranking(&mut self, generation: u64, cx: &mut Context<Self>) {
        if generation != self.command_ranking.generation {
            return;
        }
        self.rank_command_now();
        cx.notify();
        #[cfg(feature = "testkit")]
        self.schedule_command_text_render_commit(cx);
    }

    /// Deterministic testkit flush: cancel the pending debounce timer and
    /// publish the current generation synchronously.
    #[cfg(feature = "testkit")]
    pub fn flush_command_ranking(&mut self, cx: &mut Context<Self>) {
        self.command_ranking.flush();
        self.rank_command_now();
        cx.notify();
    }

    /// Up/Down selection inside the published results: clamps at the edges,
    /// never moves focus out of the field.
    pub fn command_move_selection(&mut self, delta: isize) {
        let len = self.command_ranking.results.len();
        if len == 0 {
            return;
        }
        let next = self.command_ranking.selected as isize + delta;
        self.command_ranking.selected = next.clamp(0, len as isize - 1) as usize;
    }
}

#[cfg(feature = "testkit")]
impl From<CommandResultState> for bbr_testkit::protocol::CommandResultState {
    fn from(state: CommandResultState) -> Self {
        match state {
            CommandResultState::Inactive => Self::Inactive,
            CommandResultState::Empty => Self::Empty,
            CommandResultState::Loading => Self::Loading,
            CommandResultState::Results => Self::Results,
            CommandResultState::ActivationError => Self::ActivationError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::command_result::group_name;
    use bbr_command::rank::{Activation, ResultGroup};

    fn snapshot() -> ControllerSnapshot {
        let tab = |index: usize, url: &str, title: &str, pinned: bool| bbr_browser::TabSnapshot {
            index,
            url: url.into(),
            title: title.into(),
            pinned,
            loading: false,
            crashed: false,
            unresponsive: false,
            active: index == 0,
            web_view: Some(bbr_web::WebViewId(index as u64 + 1)),
            lifecycle: bbr_browser::TabLifecycle::Warm,
            last_used: 0,
            favicon: bbr_browser::TabFavicon::Monogram,
        };
        let global =
            |id: u64, index: usize, url: &str, title: &str| bbr_browser::GlobalTabSnapshot {
                id: bbr_domain::TabId(id),
                space: bbr_domain::SpaceId(2),
                profile: bbr_web::ProfileId(1),
                index,
                url: url.into(),
                title: title.into(),
                pinned: index == 0,
                web_view: Some(bbr_web::WebViewId(id)),
                lifecycle: bbr_browser::TabLifecycle::Warm,
                last_used: 0,
            };
        ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: Some("Default".into()),
            active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
            active_space: bbr_browser::SpaceId(2),
            tabs: vec![
                tab(0, "https://a.example", "Alpha", true),
                tab(1, "https://b.example", "Beta", false),
            ],
            all_tabs: vec![
                global(1, 0, "https://a.example", "Alpha"),
                global(2, 1, "https://b.example", "Beta"),
            ],
            profiles: vec![bbr_browser::ProfileSnapshot {
                index: 0,
                id: bbr_web::ProfileId(1),
                title: "Default".into(),
                hue: bbr_browser::ProfileHue::Cobalt,
            }],
            spaces: vec![bbr_browser::SpaceSnapshot {
                index: 0,
                id: bbr_browser::SpaceId(2),
                profile: bbr_web::ProfileId(1),
                title: "Default".into(),
                active_tab: Some(0),
                active_view: Some(bbr_web::WebViewId(1)),
                active: true,
                tab_count: 2,
            }],
            active_index: Some(0),
            nav: Default::default(),
            web_view_hosted: true,
            engine_sessions: 2,
            active_view: Some(bbr_web::WebViewId(1)),
            lifecycle_counts: bbr_browser::LifecycleCounts::default(),
            preview: None,
        }
    }

    #[test]
    fn group_names_are_stable_snake_case() {
        assert_eq!(group_name(ResultGroup::ExactTab), "exact_tab");
        assert_eq!(group_name(ResultGroup::CurrentPinned), "current_pinned");
        assert_eq!(group_name(ResultGroup::CurrentTab), "current_tab");
        assert_eq!(group_name(ResultGroup::OtherSpace), "other_space");
        assert_eq!(group_name(ResultGroup::Space), "space");
        assert_eq!(group_name(ResultGroup::SavedPage), "saved_page");
        assert_eq!(group_name(ResultGroup::History), "history");
        assert_eq!(group_name(ResultGroup::Action), "action");
        assert_eq!(group_name(ResultGroup::Fallback), "fallback");
    }

    #[test]
    fn ranking_publishes_deterministic_rows_limited_to_16() {
        let (rows, _total) = rank_text(
            &snapshot(),
            &[],
            &[],
            &[],
            CommandBarMode::Location,
            "alpha",
        );
        assert!(!rows.is_empty());
        assert_eq!(rows[0].key, "tab:1");
        assert_eq!(rows[0].group, "exact_tab");
        assert!(!rows[0].destructive);
        let (rows, _) = rank_text(
            &snapshot(),
            &[],
            &[],
            &[],
            CommandBarMode::Location,
            "close",
        );
        let close = rows
            .iter()
            .find(|r| {
                r.activation
                    == Activation::RunAction {
                        action: bbr_command::id::ActionId(2),
                    }
            })
            .expect("close tab action row");
        assert!(close.destructive, "close tab is the destructive action");
        let (rows, _) = rank_text(
            &snapshot(),
            &[],
            &[],
            &[],
            CommandBarMode::Location,
            "nonexistent term",
        );
        assert!(
            rows.iter()
                .any(|r| matches!(r.activation, Activation::Search { .. })),
            "the duckduckgo search fallback row is present"
        );
        assert!(rows.len() <= RESULT_LIMIT);
    }

    #[test]
    fn state_machine_transitions_are_deterministic() {
        let mut ranking = CommandRanking::default();
        assert_eq!(ranking.state, CommandResultState::Inactive);
        ranking.open();
        assert_eq!(ranking.state, CommandResultState::Empty);
        assert_eq!(ranking.selected, 0);
        assert!(ranking.results.is_empty());
        // Non-empty text schedules a loading generation (the timer itself is
        // exercised through the app's publish path, not here).
        ranking.state = CommandResultState::Loading;
        assert_eq!(ranking.state, CommandResultState::Loading);
        ranking.close();
        assert_eq!(ranking.state, CommandResultState::Inactive);
        assert!(ranking.results.is_empty());
        assert_eq!(ranking.selected, 0);
        assert!(ranking.activation_error.is_none());
    }

    #[test]
    fn open_and_flush_keep_the_state_consistent() {
        let mut ranking = CommandRanking::default();
        ranking.open();
        ranking.flush();
        assert_eq!(ranking.state, CommandResultState::Empty);
        assert!(ranking.pending.is_none());
        ranking.close();
        assert_eq!(ranking.state, CommandResultState::Inactive);
    }
}
