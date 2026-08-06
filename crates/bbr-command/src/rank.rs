//! Deterministic local result ranking across Phase-5 priority groups.
//!
//! `rank` resolves a parsed [`Query`] against a [`Corpus`] and
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
//! 5. local history ([`ResultGroup::History`]);
//! 6. browser commands ([`ResultGroup::Action`]);
//! 7. navigate/search fallback ([`ResultGroup::Fallback`]).

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::corpus::{Corpus, TabEntry};
use crate::id::{ActionId, HistoryId, ProfileId, SpaceId, TabId};
use crate::parse::{Destination, Mode, Query, QueryContext};
use crate::registry::{Action, CommandRegistry};

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
            ResultGroup::History => 4,
            ResultGroup::Action => 5,
            ResultGroup::Fallback => 6,
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

/// The secondary key of a tab result: current space first, then corpus space
/// order, then the tab's index inside its space. The index must stay below
/// 2^20 and the space order below 2^36, which real corpora respect.
fn tab_secondary(tab: &TabEntry, current_space: Option<SpaceId>, space_order: u64) -> u64 {
    let current_bias = if Some(tab.space) == current_space {
        0
    } else {
        1
    };
    (current_bias << 56) | (space_order << 20) | (tab.index & 0x00FF_FFFF)
}

fn collect_tab_matches(
    corpus: &Corpus,
    context: &QueryContext,
    needle: &str,
    rows: &mut Vec<(RankKey, CommandResult)>,
) {
    for tab in &corpus.tabs {
        let Some(quality) = match_quality([tab.title_lower(), tab.url_lower()], needle) else {
            continue;
        };
        let group = if quality == MatchQuality::Exact {
            ResultGroup::ExactTab
        } else if Some(tab.space) == context.current_space {
            if tab.pinned {
                ResultGroup::CurrentPinned
            } else {
                ResultGroup::CurrentTab
            }
        } else {
            ResultGroup::OtherSpace
        };
        let space_order = corpus.space_order(tab.space).unwrap_or(tab.space.0);
        let key = RankKey {
            group: group.rank(),
            quality,
            secondary: tab_secondary(tab, context.current_space, space_order),
            tie: tab.id.0,
        };
        rows.push((key, tab_result(corpus, context, tab, group)));
    }
}

fn collect_empty_current_tabs(
    corpus: &Corpus,
    context: &QueryContext,
    rows: &mut Vec<(RankKey, CommandResult)>,
) {
    let Some(current) = context.current_space else {
        return;
    };
    for tab in &corpus.tabs {
        if tab.space != current {
            continue;
        }
        let group = if tab.pinned {
            ResultGroup::CurrentPinned
        } else {
            ResultGroup::CurrentTab
        };
        let space_order = corpus.space_order(tab.space).unwrap_or(tab.space.0);
        let key = RankKey {
            group: group.rank(),
            quality: MatchQuality::Exact,
            secondary: tab_secondary(tab, context.current_space, space_order),
            tie: tab.id.0,
        };
        rows.push((key, tab_result(corpus, context, tab, group)));
    }
}

fn tab_result(
    corpus: &Corpus,
    context: &QueryContext,
    tab: &TabEntry,
    group: ResultGroup,
) -> CommandResult {
    // Any tab outside the current space is a cross-space result and must
    // carry profile/space identity, including exact matches (Phase 5:
    // identity must never be hidden).
    let title = if tab.title.is_empty() {
        tab.url.clone()
    } else {
        tab.title.clone()
    };
    let subtitle = if Some(tab.space) != context.current_space {
        Some(other_space_label(corpus, context, tab))
    } else {
        None
    };
    CommandResult {
        activation: Activation::SelectTab {
            tab: tab.id,
            space: tab.space,
        },
        group,
        title,
        subtitle,
        profile: Some(tab.profile),
        key: format!("tab:{}", tab.id.0),
        score: RankKey {
            group: group.rank(),
            quality: MatchQuality::Exact,
            secondary: 0,
            tie: 0,
        },
    }
}

/// Identity label for a tab outside the current space. Same-profile results
/// name the space; cross-profile results also name the profile so identity
/// is never hidden behind color.
fn other_space_label(corpus: &Corpus, context: &QueryContext, tab: &TabEntry) -> String {
    let space_name = corpus
        .space_name(tab.space)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("space {}", tab.space.0));
    if Some(tab.profile) == context.current_profile {
        format!("Space {space_name}")
    } else {
        let profile_name = corpus
            .profile_name(tab.profile)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("profile {}", tab.profile.0));
        format!("Profile {profile_name} · Space {space_name}")
    }
}

fn collect_history_matches(
    corpus: &Corpus,
    needle: &str,
    rows: &mut Vec<(RankKey, CommandResult)>,
) {
    let mut candidates: Vec<(usize, MatchQuality)> = Vec::new();
    for (idx, entry) in corpus.history.iter().enumerate() {
        if let Some(quality) = match_quality([entry.title_lower(), entry.url_lower()], needle) {
            candidates.push((idx, quality));
        }
    }

    // Deduplicate by normalized URL, keeping the most recent (then most
    // visited) survivor so one page never appears twice.
    let mut keep: HashMap<&str, usize> = HashMap::new();
    let mut survivors: Vec<(usize, MatchQuality)> = Vec::new();
    for (idx, quality) in candidates {
        let url = corpus.history[idx].url_lower();
        match keep.get(url).copied() {
            Some(slot) => {
                if survivor_preferred(&corpus.history, idx, survivors[slot].0) {
                    survivors[slot] = (idx, quality);
                }
            }
            None => {
                keep.insert(url, survivors.len());
                survivors.push((idx, quality));
            }
        }
    }

    for (entry_idx, quality) in survivors {
        let entry = &corpus.history[entry_idx];
        let title = if entry.title.is_empty() {
            entry.url.clone()
        } else {
            entry.title.clone()
        };
        let key = RankKey {
            group: ResultGroup::History.rank(),
            quality,
            secondary: u64::MAX - entry.last_visited,
            tie: entry_idx as u64,
        };
        rows.push((
            key,
            CommandResult {
                activation: Activation::OpenHistory { history: entry.id },
                group: ResultGroup::History,
                title,
                subtitle: None,
                profile: None,
                key: format!("history:{}", entry.id.0),
                score: key,
            },
        ));
    }
}

/// True when `candidate` should replace `current` for the same URL: strictly
/// more recent, or equally recent and strictly more visited. Ties keep the
/// earlier corpus entry.
fn survivor_preferred(
    history: &[crate::corpus::HistoryEntry],
    candidate: usize,
    current: usize,
) -> bool {
    let c = &history[candidate];
    let k = &history[current];
    (c.last_visited, c.visit_count) > (k.last_visited, k.visit_count)
}

fn collect_action_matches(
    registry: &CommandRegistry,
    needle: &str,
    rows: &mut Vec<(RankKey, CommandResult)>,
) {
    for (idx, action) in registry.actions().enumerate() {
        let haystacks = action.keywords_lower();
        let Some(quality) = match_quality(haystacks.iter().map(String::as_str), needle) else {
            continue;
        };
        let key = RankKey {
            group: ResultGroup::Action.rank(),
            quality,
            secondary: idx as u64,
            tie: 0,
        };
        rows.push((key, action_result(action, quality, idx as u64)));
    }
}

fn collect_all_actions(registry: &CommandRegistry, rows: &mut Vec<(RankKey, CommandResult)>) {
    for (idx, action) in registry.actions().enumerate() {
        let key = RankKey {
            group: ResultGroup::Action.rank(),
            quality: MatchQuality::Exact,
            secondary: idx as u64,
            tie: 0,
        };
        rows.push((key, action_result(action, MatchQuality::Exact, idx as u64)));
    }
}

fn action_result(action: &Action, quality: MatchQuality, secondary: u64) -> CommandResult {
    let key = RankKey {
        group: ResultGroup::Action.rank(),
        quality,
        secondary,
        tie: 0,
    };
    CommandResult {
        activation: Activation::RunAction { action: action.id },
        group: ResultGroup::Action,
        title: action.title.clone(),
        subtitle: action.shortcut.map(str::to_owned),
        profile: None,
        key: format!("action:{}", action.id.0),
        score: key,
    }
}

fn collect_fallback(query: &Query, rows: &mut Vec<(RankKey, CommandResult)>) {
    let key_string = match &query.destination {
        Destination::Navigate(url) => format!("navigate:{url}"),
        Destination::Search(terms) => format!("search:{terms}"),
        Destination::Empty => return,
    };
    let (activation, title) = match &query.destination {
        Destination::Navigate(url) => (
            Activation::Navigate { url: url.clone() },
            format!("Open {url}"),
        ),
        Destination::Search(terms) => (
            Activation::Search {
                terms: terms.clone(),
            },
            format!("Search for {terms}"),
        ),
        Destination::Empty => unreachable!(),
    };
    let score = RankKey {
        group: ResultGroup::Fallback.rank(),
        quality: MatchQuality::Exact,
        secondary: 0,
        tie: 0,
    };
    rows.push((
        score,
        CommandResult {
            activation,
            group: ResultGroup::Fallback,
            title,
            subtitle: None,
            profile: None,
            key: key_string,
            score,
        },
    ));
}

/// Best match quality of `needle` against any haystack; `None` when the
/// needle is empty or matches nothing. Exact beats prefix beats substring.
fn match_quality<'a>(
    haystacks: impl IntoIterator<Item = &'a str>,
    needle: &str,
) -> Option<MatchQuality> {
    if needle.is_empty() {
        return None;
    }
    let mut best: Option<MatchQuality> = None;
    for haystack in haystacks {
        let quality = if haystack == needle {
            MatchQuality::Exact
        } else if haystack.starts_with(needle) {
            MatchQuality::Prefix
        } else if haystack.contains(needle) {
            MatchQuality::Substring
        } else {
            continue;
        };
        best = Some(match best {
            Some(current) => current.max(quality),
            None => quality,
        });
    }
    best
}
