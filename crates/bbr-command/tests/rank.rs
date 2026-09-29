//! Ranking contract tests: every Phase-5 priority group, cross-profile
//! identity labels, deterministic tie-breaking, deduplication, fallback
//! semantics, unicode, empty input, and result limits.

use bbr_command::corpus::{
    Corpus, HistoryEntry, ProfileEntry, SavedPageEntry, SpaceEntry, TabEntry,
};
use bbr_command::id::{BookmarkId, HistoryId, ProfileId, SpaceId, TabId};
use bbr_command::parse::{Mode, Query, QueryContext};
use bbr_command::rank::{rank, Activation, ResultGroup};
use bbr_command::registry::CommandRegistry;

const P0: ProfileId = ProfileId(1);
const P1: ProfileId = ProfileId(2);

const S0: SpaceId = SpaceId(10);
const S1: SpaceId = SpaceId(11);
const S2: SpaceId = SpaceId(12);

fn fixture_corpus() -> Corpus {
    let mut corpus = Corpus::new();
    corpus.profiles.push(ProfileEntry::new(P0, "Personal", 0));
    corpus.profiles.push(ProfileEntry::new(P1, "Work", 1));
    corpus.spaces.push(SpaceEntry::new(S0, "General", P0, 0));
    corpus.spaces.push(SpaceEntry::new(S1, "Research", P0, 1));
    corpus.spaces.push(SpaceEntry::new(S2, "Work", P1, 2));

    let tabs = [
        // (id, title, url, space, profile, pinned, index)
        (0, "Inbox", "https://mail.example.com", S0, P0, true, 0),
        (1, "Docs", "https://docs.example.com", S0, P0, false, 1),
        (
            2,
            "Inbox Calendar",
            "https://calendar.example.com",
            S0,
            P0,
            false,
            2,
        ),
        (
            3,
            "Research Notes",
            "https://notes.example.com",
            S1,
            P0,
            false,
            0,
        ),
        (
            4,
            "Work Inbox",
            "https://work.example.com/inbox",
            S2,
            P1,
            false,
            0,
        ),
        (
            5,
            "Work Dashboard",
            "https://work.example.com",
            S2,
            P1,
            true,
            1,
        ),
        (
            6,
            "Inbox",
            "https://second.example.com/inbox",
            S1,
            P0,
            false,
            1,
        ),
        (
            7,
            "Inbox",
            "https://third.example.com/inbox",
            S0,
            P0,
            false,
            3,
        ),
        (
            8,
            "Café Berlin",
            "https://cafe.example.com",
            S0,
            P0,
            false,
            4,
        ),
    ];
    for (id, title, url, space, profile, pinned, index) in tabs {
        corpus.tabs.push(TabEntry::new(
            TabId(id),
            title,
            url,
            space,
            profile,
            pinned,
            index,
        ));
    }

    // history: (id, title, url, visits, last_visited)
    let history = [
        (
            0,
            "Inbox — Example Mail",
            "https://mail.example.com",
            5,
            100,
        ),
        (1, "Docs for Rust", "https://rust.example.com", 2, 90),
        (2, "Example Docs", "https://docs.example.com", 9, 80),
        (3, "Mail Example", "https://mail.example.com", 1, 95), // dup URL of 0
        (4, "Rust Docs", "https://rust.example.com", 7, 120),   // dup URL of 1
        (5, "Rust Guide", "https://guide.example.com", 1, 200),
        (6, "Rust Book", "https://book.example.com", 1, 150),
        (7, "", "https://empty.example.com", 1, 60), // no title
    ];
    for (id, title, url, visits, last) in history {
        corpus
            .history
            .push(HistoryEntry::new(HistoryId(id), title, url, visits, last));
    }

    // saved pages: (id, display title, canonical url, collection, captured title)
    let saved = [
        (
            1,
            "Inbox — Example Mail",
            "https://mail.example.com",
            Some("Work"),
            "Inbox — Example Mail",
        ),
        (
            2,
            "Rust Book",
            "https://book.example.com",
            None,
            "Rust Book",
        ),
        (
            3,
            "My Recipes",
            "https://recipes.example.com",
            Some("Cooking"),
            "Pasta Recipes",
        ),
    ];
    for (id, title, url, collection, captured) in saved {
        corpus.saved_pages.push(SavedPageEntry::new(
            BookmarkId(id),
            title,
            url,
            collection.map(str::to_owned),
            captured,
        ));
    }
    corpus
}

fn keys(results: &[bbr_command::rank::CommandResult]) -> Vec<String> {
    results.iter().map(|r| r.key.clone()).collect()
}

fn context() -> QueryContext {
    QueryContext::with_current(S0, P0)
}

fn rank_all(corpus: &Corpus, query: &str, mode: Mode) -> Vec<bbr_command::rank::CommandResult> {
    rank(
        corpus,
        &CommandRegistry::builtin(),
        &Query::new(mode, query),
        &context(),
        16,
    )
    .results
}

#[test]
fn empty_switcher_lists_current_tabs_pinned_first() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "", Mode::Switcher);
    assert_eq!(
        keys(&results),
        ["tab:0", "tab:1", "tab:2", "tab:7", "tab:8"]
    );
    let groups: Vec<ResultGroup> = results.iter().map(|r| r.group).collect();
    assert_eq!(
        groups,
        [
            ResultGroup::CurrentPinned,
            ResultGroup::CurrentTab,
            ResultGroup::CurrentTab,
            ResultGroup::CurrentTab,
            ResultGroup::CurrentTab,
        ]
    );
}

/// An empty ⌘L yields nothing; an empty ⌘T offers its recents instead
/// (blueprint D04 — covered by the `recents` unit tests).
#[test]
fn empty_location_yields_nothing() {
    let corpus = fixture_corpus();
    let ranked = rank(
        &corpus,
        &CommandRegistry::builtin(),
        &Query::new(Mode::Location, ""),
        &context(),
        16,
    );
    assert!(ranked.results.is_empty());
    assert_eq!(ranked.total, 0);
}

#[test]
fn empty_action_lists_all_actions_in_registry_order() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "", Mode::Action);
    let expected: Vec<String> = (1..=16).map(|id| format!("action:{id}")).collect();
    assert_eq!(keys(&results), expected);
    assert!(results.iter().all(|r| r.group == ResultGroup::Action));
}

#[test]
fn exact_title_beats_prefix_and_history() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "inbox", Mode::Switcher);
    assert_eq!(
        keys(&results),
        [
            "tab:0",
            "tab:7",
            "tab:6",
            "tab:2",
            "tab:4",
            "saved_page:1",
            "history:0",
            "search:inbox"
        ]
    );
    assert_eq!(results[0].group, ResultGroup::ExactTab);
    assert_eq!(results[1].group, ResultGroup::ExactTab);
    assert_eq!(results[2].group, ResultGroup::ExactTab);
    assert_eq!(results[3].group, ResultGroup::CurrentTab);
    assert_eq!(results[4].group, ResultGroup::OtherSpace);
    assert_eq!(results[5].group, ResultGroup::SavedPage);
    assert_eq!(results[6].group, ResultGroup::History);
    assert_eq!(results[7].group, ResultGroup::Fallback);
}

#[test]
fn exact_url_cross_profile_labels_identity() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "https://work.example.com", Mode::Switcher);
    // T5's URL is an exact match (ExactTab); T4's URL only prefixes it.
    assert_eq!(
        keys(&results),
        ["tab:5", "tab:4", "navigate:https://work.example.com"]
    );
    let first = &results[0];
    assert_eq!(
        first.activation,
        Activation::SelectTab {
            tab: TabId(5),
            space: S2
        }
    );
    assert_eq!(first.subtitle.as_deref(), Some("Profile Work · Space Work"));
    assert_eq!(first.profile, Some(P1));
    let second = &results[1];
    assert_eq!(
        second.subtitle.as_deref(),
        Some("Profile Work · Space Work")
    );
}

#[test]
fn same_profile_other_space_labels_space_only() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "research", Mode::Switcher);
    assert_eq!(keys(&results), ["tab:3", "space:11", "search:research"]);
    let first = &results[0];
    assert_eq!(first.group, ResultGroup::OtherSpace);
    assert_eq!(first.subtitle.as_deref(), Some("Space Research"));
    assert_eq!(first.profile, Some(P0));
}

#[test]
fn cross_space_tabs_ordered_by_position() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "work", Mode::Switcher);
    assert_eq!(
        keys(&results),
        ["tab:4", "tab:5", "space:12", "search:work"]
    );
    for result in &results[..2] {
        assert_eq!(result.group, ResultGroup::OtherSpace);
        assert_eq!(
            result.subtitle.as_deref(),
            Some("Profile Work · Space Work")
        );
    }
}

#[test]
fn history_deduplicates_by_url_keeping_most_recent() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "rust", Mode::Switcher);
    // The Saved Page record "Rust Book" ranks before the history group
    // (settled B2 order `SavedPage < History`); H5/H6/H4 survive, H1 is
    // dropped because H4 shares its URL and is newer.
    assert_eq!(
        keys(&results),
        [
            "saved_page:2",
            "history:5",
            "history:6",
            "history:4",
            "search:rust"
        ]
    );
    assert_eq!(results[0].group, ResultGroup::SavedPage);
    for result in &results[1..4] {
        assert_eq!(result.group, ResultGroup::History);
        match result.activation {
            Activation::OpenHistory { history } => assert_ne!(history, HistoryId(1)),
            ref other => panic!("expected history activation, got {other:?}"),
        }
    }
}

#[test]
fn saved_page_group_ranks_after_space_before_history() {
    // The settled B2 group order: `Space < SavedPage < History`. A dedicated
    // corpus where one query matches a standalone space, a Saved Page
    // record, and a history record at once makes the group precedence
    // directly observable.
    let mut corpus = Corpus::new();
    corpus.spaces.push(SpaceEntry::new(S1, "Alpha", P0, 0));
    corpus.saved_pages.push(SavedPageEntry::new(
        BookmarkId(1),
        "Alpha Page",
        "https://alpha.example.com",
        None,
        "Alpha Page",
    ));
    corpus.history.push(HistoryEntry::new(
        HistoryId(1),
        "Alpha History",
        "https://alpha-history.example.com",
        1,
        1,
    ));
    let results = rank_all(&corpus, "alpha", Mode::Switcher);
    let groups: Vec<ResultGroup> = results.iter().map(|r| r.group).collect();
    assert_eq!(
        groups,
        [
            ResultGroup::Space,
            ResultGroup::SavedPage,
            ResultGroup::History,
            ResultGroup::Fallback,
        ]
    );
    assert_eq!(
        keys(&results),
        ["space:11", "saved_page:1", "history:1", "search:alpha"]
    );
}

#[test]
fn saved_page_result_carries_bookmark_activation_url_and_collection() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "recipes", Mode::Switcher);
    assert_eq!(keys(&results), ["saved_page:3", "search:recipes"]);
    let page = &results[0];
    assert_eq!(page.group, ResultGroup::SavedPage);
    assert_eq!(
        page.activation,
        Activation::OpenSavedPage {
            bookmark: BookmarkId(3)
        }
    );
    assert_eq!(page.title, "My Recipes");
    assert_eq!(
        page.subtitle.as_deref(),
        Some("https://recipes.example.com")
    );
    assert_eq!(page.collection.as_deref(), Some("Cooking"));
    assert_eq!(page.profile, None, "Saved Pages carry no profile identity");
}

#[test]
fn saved_page_matches_captured_title_when_label_overrides_display() {
    // The display title is the user label ("My Recipes"), but the captured
    // title ("Pasta Recipes") stays a match surface, mirroring
    // `bbr-domain::SavedPage::matches`.
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "pasta", Mode::Switcher);
    assert_eq!(keys(&results), ["saved_page:3", "search:pasta"]);
    assert_eq!(results[0].group, ResultGroup::SavedPage);
}

#[test]
fn saved_page_ordering_preserves_provider_order() {
    // Two Saved Page records matching the same query keep the provider's
    // deterministic corpus order (most-recently-updated first), never a
    // hash-map or insertion-order guess.
    let mut corpus = fixture_corpus();
    corpus.saved_pages.push(SavedPageEntry::new(
        BookmarkId(9),
        "Inbox Archive",
        "https://archive.example.com",
        None,
        "Inbox Archive",
    ));
    let results = rank_all(&corpus, "inbox", Mode::Switcher);
    let saved: Vec<&str> = results
        .iter()
        .filter(|r| r.group == ResultGroup::SavedPage)
        .map(|r| r.key.as_str())
        .collect();
    assert_eq!(saved, ["saved_page:1", "saved_page:9"]);
}

#[test]
fn action_group_ranks_after_history_with_shortcut_hint() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "reload", Mode::Switcher);
    assert_eq!(keys(&results), ["action:9", "search:reload"]);
    let action = &results[0];
    assert_eq!(action.group, ResultGroup::Action);
    assert_eq!(
        action.activation,
        Activation::RunAction {
            action: bbr_command::id::ActionId(9)
        }
    );
    assert_eq!(action.subtitle.as_deref(), Some("⌘R"));
}

#[test]
fn action_keyword_match_ranks_exact() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "sidebar", Mode::Switcher);
    assert_eq!(keys(&results), ["action:11", "search:sidebar"]);
    assert_eq!(
        results[0].activation,
        Activation::RunAction {
            action: bbr_command::id::ActionId(11)
        }
    );
}

#[test]
fn action_mode_restricts_to_actions_without_fallback() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "new", Mode::Action);
    // Blueprint D06: New Profile… is the third "new" action.
    assert_eq!(keys(&results), ["action:1", "action:6", "action:15"]);
}

#[test]
fn action_mode_never_emits_fallback() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "xyzzy", Mode::Action);
    assert!(results.is_empty());
}

#[test]
fn fallback_navigates_for_url_text() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "example.org", Mode::Location);
    assert_eq!(keys(&results), ["navigate:example.org"]);
    assert_eq!(results[0].group, ResultGroup::Fallback);
    assert_eq!(
        results[0].activation,
        Activation::Navigate {
            url: "example.org".to_owned()
        }
    );
    assert_eq!(results[0].title, "Open example.org");
}

#[test]
fn fallback_searches_for_plain_text() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "random search terms", Mode::Switcher);
    assert_eq!(keys(&results), ["search:random search terms"]);
    assert_eq!(
        results[0].activation,
        Activation::Search {
            terms: "random search terms".to_owned()
        }
    );
    assert_eq!(results[0].title, "Search for random search terms");
}

#[test]
fn url_text_ranks_tabs_then_history_then_fallback() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "example.com", Mode::Switcher);
    // The Saved Page record (mail.example.com) ranks between the other-space
    // tabs and the history group; the limit-16 window then drops the
    // navigate fallback row.
    assert_eq!(
        keys(&results),
        [
            "tab:0",
            "tab:1",
            "tab:2",
            "tab:7",
            "tab:8", // current space
            "tab:3",
            "tab:6",
            "tab:4",
            "tab:5", // other spaces
            "saved_page:1",
            "saved_page:2",
            "saved_page:3",
            "history:5",
            "history:6",
            "history:4",
            "history:0",
        ]
    );
}

#[test]
fn limit_truncates_but_reports_total() {
    let corpus = fixture_corpus();
    let ranked = rank(
        &corpus,
        &CommandRegistry::builtin(),
        &Query::new(Mode::Switcher, "example.com"),
        &context(),
        3,
    );
    assert_eq!(ranked.results.len(), 3);
    assert_eq!(ranked.total, 19);
    assert_eq!(keys(&ranked.results), ["tab:0", "tab:1", "tab:2"]);
}

#[test]
fn ranking_is_deterministic_across_repeated_and_reordered_runs() {
    let corpus = fixture_corpus();
    let query = Query::new(Mode::Switcher, "inbox");
    let first = rank(&corpus, &CommandRegistry::builtin(), &query, &context(), 16);
    for _ in 0..5 {
        let again = rank(&corpus, &CommandRegistry::builtin(), &query, &context(), 16);
        assert_eq!(keys(&first.results), keys(&again.results));
    }

    // Same corpus with provider vectors in a different order must rank
    // identically (ordering comes from explicit index/id keys).
    let mut reversed = fixture_corpus();
    reversed.tabs.reverse();
    reversed.spaces.reverse();
    let reordered = rank(
        &reversed,
        &CommandRegistry::builtin(),
        &query,
        &context(),
        16,
    );
    assert_eq!(keys(&first.results), keys(&reordered.results));
}

#[test]
fn reversed_space_vector_preserves_other_space_tab_order() {
    let query = Query::new(Mode::Switcher, "work");
    let first = rank(
        &fixture_corpus(),
        &CommandRegistry::builtin(),
        &query,
        &context(),
        16,
    );
    let mut reversed = fixture_corpus();
    reversed.spaces.reverse();
    let reordered = rank(
        &reversed,
        &CommandRegistry::builtin(),
        &query,
        &context(),
        16,
    );
    assert_eq!(keys(&first.results), keys(&reordered.results));
}

#[test]
fn unicode_matching_is_case_folded() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "CAFÉ", Mode::Switcher);
    assert_eq!(keys(&results), ["tab:8", "search:CAFÉ"]);
    assert_eq!(results[0].title, "Café Berlin");
}

#[test]
fn empty_context_labels_every_space() {
    let corpus = fixture_corpus();
    let ranked = rank(
        &corpus,
        &CommandRegistry::builtin(),
        &Query::new(Mode::Switcher, "inbox"),
        &QueryContext::new(),
        16,
    );
    assert_eq!(
        keys(&ranked.results),
        [
            "tab:0",
            "tab:7",
            "tab:6",
            "tab:2",
            "tab:4",
            "saved_page:1",
            "history:0",
            "search:inbox"
        ]
    );
    // T6 belongs to S1 whose profile differs from None, so the label names
    // both profile and space.
    assert_eq!(
        ranked.results[2].subtitle.as_deref(),
        Some("Profile Personal · Space Research")
    );
}

#[test]
fn labels_fall_back_to_ids_when_names_are_missing() {
    let mut corpus = fixture_corpus();
    corpus.spaces.clear();
    corpus.profiles.clear();
    let results = rank_all(&corpus, "work", Mode::Switcher);
    assert_eq!(keys(&results), ["tab:4", "tab:5", "search:work"]);
    for result in &results[..2] {
        assert_eq!(
            result.subtitle.as_deref(),
            Some("Profile profile 2 · Space space 12")
        );
    }
}

#[test]
fn history_without_title_uses_url_as_title() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "empty.example.com", Mode::Switcher);
    assert_eq!(keys(&results), ["history:7", "navigate:empty.example.com"]);
    assert_eq!(results[0].title, "https://empty.example.com");
}

#[test]
fn select_tab_activation_carries_space() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "docs", Mode::Switcher);
    assert_eq!(
        keys(&results),
        ["tab:1", "history:4", "history:2", "search:docs"]
    );
    assert_eq!(
        results[0].activation,
        Activation::SelectTab {
            tab: TabId(1),
            space: S0
        }
    );
}

#[test]
fn score_model_is_exposed_deterministically() {
    let corpus = fixture_corpus();
    let results = rank_all(&corpus, "docs", Mode::Switcher);
    let first = &results[0];
    assert_eq!(first.score.group, ResultGroup::ExactTab.rank());
    assert_eq!(first.score.quality, bbr_command::rank::MatchQuality::Exact);
    let second = &results[1];
    assert_eq!(second.score.group, ResultGroup::History.rank());
}
