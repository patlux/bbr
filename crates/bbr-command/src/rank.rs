//! Deterministic local result ranking across Phase-5 priority groups.
//!
//! [`rank`] resolves a parsed [`Query`] against a [`Corpus`] and
//! [`CommandRegistry`] into typed, ordered [`CommandResult`]s. The output is
//! a pure function of its inputs: identical inputs always yield identical
//! output, ties never depend on hash-map iteration order, and no global
//! state or render path is touched, which later allows cancellation-aware
//! orchestration to re-run ranking freely.
//!
//! Group precedence follows Phase 5 (highest first):
//!
//! 1. exact/open tab matches ([`ResultGroup::ExactTab`]);
//! 2. pinned tabs of the current space ([`ResultGroup::CurrentPinned`]);
//! 3. other tabs of the current space ([`ResultGroup::CurrentTab`]);
//! 4. tabs in other spaces, labeled with profile/space identity
//!    ([`ResultGroup::OtherSpace`]);
//! 5. standalone spaces ([`ResultGroup::Space`]);
//! 6. local history ([`ResultGroup::History`]);
//! 7. browser commands ([`ResultGroup::Action`]);
//! 8. navigate/search fallback ([`ResultGroup::Fallback`]).
//!
//! The per-group collection passes live in the private [`collect`] module.

use std::cmp::Ordering;

use crate::collect::{
    collect_action_matches, collect_all_actions, collect_empty_current_tabs, collect_fallback,
    collect_history_matches, collect_space_matches, collect_tab_matches,
};

use crate::corpus::Corpus;
use crate::id::{ActionId, HistoryId, ProfileId, SpaceId, TabId};
use crate::parse::{Mode, Query, QueryContext};
use crate::registry::CommandRegistry;

/// The stable priority group a result belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResultGroup {
    /// A tab whose title or URL matches the query exactly.
    ExactTab,
    /// A pinned tab of the current space.
    CurrentPinned,
    /// An unpinned tab of the current space.
    CurrentTab,
    /// A tab in another space; labeled with profile/space identity.
    OtherSpace,
    /// A selectable standalone space.
    Space,
    /// A local history record.
    History,
    /// A registered browser command.
    Action,
    /// The navigate/search fallback row.
    Fallback,
}

impl ResultGroup {
    /// The Phase-5 precedence rank (lower sorts first).
    pub fn rank(self) -> u8 {
        match self {
            ResultGroup::ExactTab => 0,
            ResultGroup::CurrentPinned => 1,
            ResultGroup::CurrentTab => 2,
            ResultGroup::OtherSpace => 3,
            ResultGroup::Space => 4,
            ResultGroup::History => 5,
            ResultGroup::Action => 6,
            ResultGroup::Fallback => 7,
        }
    }
}

/// How well one record matched the query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchQuality {
    /// The needle occurs somewhere inside the haystack.
    Substring,
    /// The haystack starts with the needle.
    Prefix,
    /// The haystack equals the needle.
    Exact,
}

/// The deterministic sort key of one result.
///
/// Ordering is: group ascending, quality descending, secondary ascending,
/// tie ascending. The secondary and tie fields carry per-group semantics
/// (documented at their use sites) and make the total order total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankKey {
    /// Phase-5 group precedence.
    pub group: u8,
    /// Match quality inside the group.
    pub quality: MatchQuality,
    /// Per-group secondary order.
    pub secondary: u64,
    /// Absolute tie-break.
    pub tie: u64,
}

impl Ord for RankKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.group
            .cmp(&other.group)
            .then(other.quality.cmp(&self.quality))
            .then(self.secondary.cmp(&other.secondary))
            .then(self.tie.cmp(&other.tie))
    }
}

impl PartialOrd for RankKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The typed action a user activating a result triggers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Activation {
    /// Select the given tab inside its space.
    SelectTab { tab: TabId, space: SpaceId },
    /// Activate the given space.
    SelectSpace { space: SpaceId },
    /// Open the given local history record.
    OpenHistory { history: HistoryId },
    /// Run the given browser command.
    RunAction { action: ActionId },
    /// Navigate the active pane to the given address.
    Navigate { url: String },
    /// Perform a web search for the given terms.
    Search { terms: String },
}

/// One ranked, typed command-bar result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    /// The typed activation this result commits.
    pub activation: Activation,
    /// The Phase-5 priority group.
    pub group: ResultGroup,
    /// Primary display title.
    pub title: String,
    /// Secondary display line (cross-space identity label, shortcut hint).
    pub subtitle: Option<String>,
    /// Owning profile when the result has one (tabs always do).
    pub profile: Option<ProfileId>,
    /// Stable external key suitable for later snapshot/testkit identity.
    pub key: String,
    /// The deterministic sort key (useful for tests and diagnostics).
    pub score: RankKey,
}

/// The ordered output of one ranking pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedResults {
    /// Results in display order, truncated to the requested limit.
    pub results: Vec<CommandResult>,
    /// Number of matches before truncation.
    pub total: usize,
}

/// Ranks the corpus against a typed query with a deterministic group order.
///
/// `limit` caps the returned rows; `total` still reports the full match
/// count. The first element of `results` is the "first correct result" the
/// Phase-5 latency budget applies to.
pub fn rank(
    corpus: &Corpus,
    registry: &CommandRegistry,
    query: &Query,
    context: &QueryContext,
    limit: usize,
) -> RankedResults {
    let mut rows: Vec<(RankKey, CommandResult)> = Vec::new();
    let needle = query.lower();

    if query.mode != Mode::Action {
        if query.is_empty() {
            if query.mode == Mode::Switcher {
                collect_empty_current_tabs(corpus, context, &mut rows);
            }
        } else {
            collect_tab_matches(corpus, context, needle, &mut rows);
            collect_space_matches(corpus, context, needle, &mut rows);
            collect_history_matches(corpus, needle, &mut rows);
            collect_action_matches(registry, needle, &mut rows);
            collect_fallback(query, &mut rows);
        }
    } else if query.is_empty() {
        collect_all_actions(registry, &mut rows);
    } else {
        collect_action_matches(registry, needle, &mut rows);
    }

    rows.sort_by_key(|row| row.0);
    let total = rows.len();
    let results = rows.into_iter().take(limit).map(|(_, r)| r).collect();
    RankedResults { results, total }
}
