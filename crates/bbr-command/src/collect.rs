//! Private result collection and matching for [`rank`](crate::rank).
//!
//! The public contract (types, key, entry point) lives in
//! [`crate::rank`]; this module owns the per-group collection passes, the
//! deterministic secondary keys, identity labeling, history deduplication,
//! and the match-quality ladder. Everything here is private to the crate.

use std::collections::HashMap;

use crate::corpus::{Corpus, HistoryEntry, SpaceEntry, TabEntry};
use crate::id::SpaceId;
use crate::parse::{Destination, Query, QueryContext};
use crate::rank::{Activation, CommandResult, MatchQuality, RankKey, ResultGroup};
use crate::registry::{Action, CommandRegistry};

/// The secondary key of a tab result: current space first, then corpus space
/// order, then the tab's index inside its space. The index must stay below
/// 2^20 (mask `0x000F_FFFF`) and the space order below 2^36 so the three
/// fields cannot overlap; real corpora respect both.
fn tab_secondary(tab: &TabEntry, current_space: Option<SpaceId>, space_order: u64) -> u64 {
    let current_bias = if Some(tab.space) == current_space {
        0
    } else {
        1
    };
    (current_bias << 56) | (space_order << 20) | (tab.index & 0x000F_FFFF)
}

pub(crate) fn collect_tab_matches(
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
        rows.push((key, tab_result(corpus, context, tab, group, key)));
    }
}

pub(crate) fn collect_empty_current_tabs(
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
        rows.push((key, tab_result(corpus, context, tab, group, key)));
    }
}

fn tab_result(
    corpus: &Corpus,
    context: &QueryContext,
    tab: &TabEntry,
    group: ResultGroup,
    score: RankKey,
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
        score,
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

pub(crate) fn collect_space_matches(
    corpus: &Corpus,
    context: &QueryContext,
    needle: &str,
    rows: &mut Vec<(RankKey, CommandResult)>,
) {
    for space in &corpus.spaces {
        if Some(space.id) == context.current_space {
            continue;
        }
        let Some(quality) = match_quality([space.name_lower()], needle) else {
            continue;
        };
        let key = RankKey {
            group: ResultGroup::Space.rank(),
            quality,
            secondary: space.index,
            tie: space.id.0,
        };
        rows.push((key, space_result(corpus, context, space, key)));
    }
}

fn space_result(
    corpus: &Corpus,
    context: &QueryContext,
    space: &SpaceEntry,
    score: RankKey,
) -> CommandResult {
    let title = if space.name.is_empty() {
        format!("space {}", space.id.0)
    } else {
        space.name.clone()
    };
    let subtitle = if Some(space.profile) == context.current_profile {
        "Space".to_owned()
    } else {
        let profile_name = corpus
            .profile_name(space.profile)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("profile {}", space.profile.0));
        format!("Profile {profile_name}")
    };
    CommandResult {
        activation: Activation::SelectSpace { space: space.id },
        group: ResultGroup::Space,
        title,
        subtitle: Some(subtitle),
        profile: Some(space.profile),
        key: format!("space:{}", space.id.0),
        score,
    }
}

pub(crate) fn collect_history_matches(
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
fn survivor_preferred(history: &[HistoryEntry], candidate: usize, current: usize) -> bool {
    let c = &history[candidate];
    let k = &history[current];
    (c.last_visited, c.visit_count) > (k.last_visited, k.visit_count)
}

pub(crate) fn collect_action_matches(
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

pub(crate) fn collect_all_actions(
    registry: &CommandRegistry,
    rows: &mut Vec<(RankKey, CommandResult)>,
) {
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

pub(crate) fn collect_fallback(query: &Query, rows: &mut Vec<(RankKey, CommandResult)>) {
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
