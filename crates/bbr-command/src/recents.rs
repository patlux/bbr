//! Blueprint D04 (DESIGN §11.4): what an empty ⌘T offers before the user
//! types — the recently used tabs (never the one on screen), then the most
//! recent history pages not already listed, then Reopen Closed Tab when
//! there is something to reopen — so ⌘T ↵ returns to the last thing.

use std::collections::HashSet;

use crate::collect::{action_result, tab_result};
use crate::corpus::Corpus;
use crate::id::ActionId;
use crate::parse::QueryContext;
use crate::rank::{Activation, CommandResult, MatchQuality, RankKey, ResultGroup};
use crate::registry::CommandRegistry;

/// Recent tabs listed on an empty ⌘T.
pub const RECENT_TABS: usize = 4;
/// Recent history pages listed after them.
pub const RECENT_HISTORY: usize = 3;
/// The registry's Reopen Closed Tab command.
const REOPEN_CLOSED_TAB: ActionId = ActionId(3);

/// The recents run in one fixed order, so the rank key's group is the same
/// for every row and the secondary key is the list position.
fn key(position: u64) -> RankKey {
    RankKey {
        group: 0,
        quality: MatchQuality::Exact,
        secondary: position,
        tie: 0,
    }
}

pub(crate) fn collect_recents(
    corpus: &Corpus,
    registry: &CommandRegistry,
    context: &QueryContext,
    rows: &mut Vec<(RankKey, CommandResult)>,
) {
    let mut position = 0_u64;
    let mut listed_urls: HashSet<String> = HashSet::new();

    let mut tabs: Vec<_> = corpus
        .tabs
        .iter()
        .filter(|tab| Some(tab.id) != context.current_tab && tab.last_used > 0)
        .collect();
    tabs.sort_by(|a, b| b.last_used.cmp(&a.last_used).then(a.id.0.cmp(&b.id.0)));
    for tab in tabs.into_iter().take(RECENT_TABS) {
        let group = if Some(tab.space) == context.current_space {
            ResultGroup::CurrentTab
        } else {
            ResultGroup::OtherSpace
        };
        listed_urls.insert(tab.url.to_lowercase());
        let mut result = tab_result(corpus, context, tab, group, key(position));
        result.score = key(position);
        rows.push((key(position), result));
        position += 1;
    }

    let mut history: Vec<_> = corpus.history.iter().collect();
    history.sort_by(|a, b| {
        b.last_visited
            .cmp(&a.last_visited)
            .then(b.visit_count.cmp(&a.visit_count))
    });
    let mut listed_history = 0;
    for entry in history {
        if listed_history == RECENT_HISTORY {
            break;
        }
        if !listed_urls.insert(entry.url.to_lowercase()) {
            continue;
        }
        let title = if entry.title.is_empty() {
            entry.url.clone()
        } else {
            entry.title.clone()
        };
        rows.push((
            key(position),
            CommandResult {
                activation: Activation::OpenHistory { history: entry.id },
                group: ResultGroup::History,
                title,
                subtitle: None,
                url: Some(entry.url.clone()),
                collection: None,
                profile: None,
                key: format!("history:{}", entry.id.0),
                score: key(position),
            },
        ));
        position += 1;
        listed_history += 1;
    }

    if corpus.reopen_available {
        if let Some(action) = registry.actions().find(|a| a.id == REOPEN_CLOSED_TAB) {
            let mut result = action_result(action, MatchQuality::Exact, position);
            result.score = key(position);
            rows.push((key(position), result));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::corpus::{Corpus, HistoryEntry, TabEntry};
    use crate::id::{HistoryId, ProfileId, SpaceId, TabId};
    use crate::parse::{Mode, Query, QueryContext};
    use crate::rank::{rank, Activation};
    use crate::registry::CommandRegistry;

    fn tab(id: u64, url: &str, last_used: u64) -> TabEntry {
        TabEntry::new(
            TabId(id),
            format!("T{id}"),
            url,
            SpaceId(1),
            ProfileId(1),
            false,
            id,
        )
        .with_last_used(last_used)
    }

    fn corpus() -> Corpus {
        let mut corpus = Corpus::new();
        corpus.tabs = vec![
            tab(1, "https://a.example/", 5),
            tab(2, "https://b.example/", 9),
            tab(3, "https://c.example/", 7),
        ];
        corpus.history = vec![
            HistoryEntry::new(HistoryId(10), "Old", "https://old.example/", 1, 1),
            HistoryEntry::new(HistoryId(11), "B again", "https://b.example/", 9, 3),
            HistoryEntry::new(HistoryId(12), "News", "https://news.example/", 8, 2),
        ];
        corpus
    }

    fn keys(corpus: &Corpus, context: QueryContext) -> Vec<String> {
        rank(
            corpus,
            &CommandRegistry::builtin(),
            &Query::new(Mode::NewTab, ""),
            &context,
            16,
        )
        .results
        .into_iter()
        .map(|r| r.key)
        .collect()
    }

    #[test]
    fn empty_new_tab_lists_recent_tabs_then_history_and_skips_the_current_tab() {
        let context =
            QueryContext::with_current(SpaceId(1), ProfileId(1)).with_current_tab(Some(TabId(2)));
        assert_eq!(
            keys(&corpus(), context),
            vec!["tab:3", "tab:1", "history:11", "history:12", "history:10"]
        );
    }

    #[test]
    fn history_already_open_as_a_tab_is_not_repeated_and_reopen_comes_last() {
        let mut corpus = corpus();
        corpus.reopen_available = true;
        let context = QueryContext::with_current(SpaceId(1), ProfileId(1));
        let keys = keys(&corpus, context);
        assert_eq!(keys[..3], ["tab:2", "tab:3", "tab:1"]);
        assert!(
            !keys.contains(&"history:11".to_string()),
            "b.example is a tab"
        );
        assert_eq!(keys.last().map(String::as_str), Some("action:3"));
    }

    #[test]
    fn location_and_switcher_keep_their_empty_behaviour() {
        let context = QueryContext::with_current(SpaceId(1), ProfileId(1));
        let registry = CommandRegistry::builtin();
        let location = rank(
            &corpus(),
            &registry,
            &Query::new(Mode::Location, ""),
            &context,
            16,
        );
        assert!(location.results.is_empty());
    }

    #[test]
    fn a_leading_angle_bracket_switches_to_actions() {
        let query = Query::new(Mode::NewTab, "> pin");
        assert_eq!(query.mode, Mode::Action);
        assert_eq!(query.text, "pin");
        let context = QueryContext::with_current(SpaceId(1), ProfileId(1));
        let ranked = rank(&corpus(), &CommandRegistry::builtin(), &query, &context, 16);
        assert!(!ranked.results.is_empty());
        assert!(ranked
            .results
            .iter()
            .all(|r| matches!(r.activation, Activation::RunAction { .. })));
        let all = rank(
            &corpus(),
            &CommandRegistry::builtin(),
            &Query::new(Mode::NewTab, ">"),
            &context,
            64,
        );
        assert_eq!(
            all.results.len(),
            CommandRegistry::builtin().actions().count()
        );
    }
}
