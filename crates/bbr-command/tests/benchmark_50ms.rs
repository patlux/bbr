//! Optimized release-mode benchmark: first correct result for a corpus of
//! 500 tabs + 10,000 history entries stays below the 50 ms Phase-5 ceiling.
//!
//! This is a portable pure-core benchmark: it measures parsing + ranking
//! only, never fixture setup or test transport, and cannot substitute for
//! the later in-process real-app command-bar presentation measurement (which
//! the app-composition packet owns). The test runs in every profile because
//! it enforces the ceiling; run `cargo test -p bbr-command --release` and
//! pass `-- --nocapture` to retain the raw per-sample distribution.
//!
//! Every timed iteration re-parses the query text and ranks, then asserts
//! that the *first* result is the deterministic correct one, so the sample
//! is the honest first-correct-result latency, not a bare function call.

use std::hint::black_box;
use std::time::Instant;

use bbr_command::corpus::{Corpus, HistoryEntry, ProfileEntry, SpaceEntry, TabEntry};
use bbr_command::id::{ProfileId, SpaceId, TabId};
use bbr_command::parse::{Mode, Query, QueryContext};
use bbr_command::rank::{rank, ResultGroup};
use bbr_command::registry::CommandRegistry;

const PROFILE_COUNT: u64 = 5;
const SPACES_PER_PROFILE: u64 = 5;
const TABS_PER_SPACE: u64 = 20;
const TAB_COUNT: u64 = PROFILE_COUNT * SPACES_PER_PROFILE * TABS_PER_SPACE; // 500
const HISTORY_COUNT: u64 = 10_000;

const TOPICS: [&str; 20] = [
    "inbox",
    "docs",
    "work",
    "research",
    "notes",
    "dashboard",
    "calendar",
    "mail",
    "news",
    "code",
    "design",
    "finance",
    "travel",
    "music",
    "health",
    "projects",
    "meetings",
    "reports",
    "drafts",
    "archive",
];

const HISTORY_TOPICS: [&str; 50] = [
    "rust",
    "swift",
    "web",
    "design",
    "finance",
    "travel",
    "recipes",
    "news",
    "docs",
    "code",
    "music",
    "health",
    "sports",
    "science",
    "history",
    "art",
    "books",
    "movies",
    "games",
    "work",
    "study",
    "home",
    "garden",
    "cooking",
    "coffee",
    "keyboards",
    "browsers",
    "privacy",
    "linux",
    "macos",
    "audio",
    "video",
    "photos",
    "maps",
    "weather",
    "trains",
    "bikes",
    "runs",
    "hikes",
    "camps",
    "markets",
    "stocks",
    "python",
    "go",
    "crypto",
    "ai",
    "robots",
    "cars",
    "space",
    "oceans",
];

/// Builds the deterministic 500-tab / 10,000-history corpus.
///
/// Tab identity 1 is the pinned "Work Inbox" of the current space; a few
/// entries are planted so benchmark queries have unambiguous first results.
fn benchmark_corpus() -> Corpus {
    let mut corpus = Corpus::new();

    for p in 0..PROFILE_COUNT {
        corpus.profiles.push(ProfileEntry::new(
            ProfileId(p + 1),
            format!("Profile {p}"),
            p,
        ));
    }

    let mut tab_id: u64 = 0;
    for p in 0..PROFILE_COUNT {
        for sp in 0..SPACES_PER_PROFILE {
            let space = SpaceId(p * SPACES_PER_PROFILE + sp);
            let profile = ProfileId(p + 1);
            corpus.spaces.push(SpaceEntry::new(
                space,
                format!("Space {p}-{sp}"),
                profile,
                p * SPACES_PER_PROFILE + sp,
            ));
            for t in 0..TABS_PER_SPACE {
                let id = TabId(tab_id + 1);
                let (title, url, pinned): (String, String, bool) = if tab_id == 0 {
                    // The pinned first tab of the current space.
                    (
                        "Work Inbox".to_owned(),
                        "https://work.example.com/inbox".to_owned(),
                        true,
                    )
                } else if tab_id == 41 {
                    // Exact-URL target used by a benchmark query.
                    (
                        "Site 42 Work".to_owned(),
                        "https://site-42.example.com/work".to_owned(),
                        false,
                    )
                } else if tab_id == 122 {
                    // Exact-title target used by a benchmark query.
                    (
                        "Exact Tab 123".to_owned(),
                        "https://exact.example.com/123".to_owned(),
                        false,
                    )
                } else if tab_id == 2 {
                    // Unicode target in the current space (unpinned).
                    (
                        "Café Berlin".to_owned(),
                        "https://cafe.example.com/berlin".to_owned(),
                        false,
                    )
                } else {
                    let topic = TOPICS[((tab_id + p * 7) % 20) as usize];
                    (
                        format!("{topic} {tab_id}"),
                        format!("https://site-{tab_id}.example.com/{topic}"),
                        tab_id.is_multiple_of(5),
                    )
                };
                corpus
                    .tabs
                    .push(TabEntry::new(id, title, url, space, profile, pinned, t));
                tab_id += 1;
            }
        }
    }
    assert_eq!(tab_id, TAB_COUNT);

    for n in 0..HISTORY_COUNT {
        let topic = HISTORY_TOPICS[(n % 50) as usize];
        corpus.history.push(HistoryEntry::new(
            bbr_command::id::HistoryId(n),
            format!("History entry {n} about {topic}"),
            format!("https://site-{n}.example.com/pages/{topic}-{n}"),
            (n * 7) % 50 + 1,
            n,
        ));
    }
    corpus
}

/// (query text, mode, expected first result key, expected first group)
const QUERIES: [(&str, Mode, &str, ResultGroup); 10] = [
    (
        "exact tab 123",
        Mode::Switcher,
        "tab:123",
        ResultGroup::ExactTab,
    ),
    (
        "https://site-42.example.com/work",
        Mode::Switcher,
        "tab:42",
        ResultGroup::ExactTab,
    ),
    ("work", Mode::Switcher, "tab:1", ResultGroup::CurrentPinned),
    ("new tab", Mode::Switcher, "action:1", ResultGroup::Action),
    ("sidebar", Mode::Switcher, "action:11", ResultGroup::Action),
    (
        "https://mail.example.com",
        Mode::Switcher,
        "navigate:https://mail.example.com",
        ResultGroup::Fallback,
    ),
    ("café", Mode::Switcher, "tab:3", ResultGroup::CurrentTab),
    (
        "example.com",
        Mode::Switcher,
        "tab:1",
        ResultGroup::CurrentPinned,
    ),
    ("", Mode::Switcher, "tab:1", ResultGroup::CurrentPinned),
    ("reload", Mode::Location, "action:9", ResultGroup::Action),
];

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (((sorted.len() - 1) as f64) * p).round() as usize;
    sorted[idx]
}

#[test]
fn first_correct_result_below_50ms_for_500_tabs_and_10k_history() {
    let corpus = black_box(benchmark_corpus());
    let registry = black_box(CommandRegistry::builtin());
    let context = black_box(QueryContext::with_current(SpaceId(0), ProfileId(1)));

    let iterations: usize = 40;
    let mut samples: Vec<f64> = Vec::with_capacity(QUERIES.len() * iterations);

    // Warm up: page-fault the corpus and caches before sampling.
    for (text, mode, expected_key, expected_group) in QUERIES {
        let ranked = rank(&corpus, &registry, &Query::new(mode, text), &context, 16);
        assert_first(ranked.results.first(), expected_key, expected_group, text);
    }

    for _ in 0..iterations {
        for (text, mode, expected_key, expected_group) in QUERIES {
            let start = Instant::now();
            let ranked = rank(&corpus, &registry, &Query::new(mode, text), &context, 16);
            let elapsed = start.elapsed();
            assert_first(ranked.results.first(), expected_key, expected_group, text);
            samples.push(elapsed.as_secs_f64() * 1000.0);
        }
    }

    let mut sorted = samples.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("finite sample"));
    let p50 = percentile(&sorted, 0.50);
    let p95 = percentile(&sorted, 0.95);
    let max = sorted.last().copied().unwrap_or(0.0);
    let min = sorted.first().copied().unwrap_or(0.0);

    println!(
        "benchmark 500 tabs + 10,000 history entries, {} samples",
        samples.len()
    );
    for (i, sample) in samples.iter().enumerate() {
        println!("bench sample_ms[{i}]={sample:.6}");
    }
    println!(
        "bench summary p50_ms={p50:.6} p95_ms={p95:.6} max_ms={max:.6} min_ms={min:.6} samples={}",
        samples.len()
    );

    // The Phase-5 first-result ceiling is 50 ms; the raw distribution above
    // is the retained evidence.
    assert!(
        p95 < 50.0,
        "p95 first-result latency {p95:.3} ms exceeded the 50 ms ceiling (raw samples printed above)"
    );
    assert!(
        max < 50.0,
        "max first-result latency {max:.3} ms exceeded the 50 ms ceiling (raw samples printed above)"
    );
}

fn assert_first(
    first: Option<&bbr_command::rank::CommandResult>,
    expected_key: &str,
    expected_group: ResultGroup,
    query: &str,
) {
    let Some(result) = first else {
        panic!("query {query:?} produced no first result");
    };
    assert_eq!(
        result.key, expected_key,
        "wrong first result for query {query:?}"
    );
    assert_eq!(
        result.group, expected_group,
        "wrong first-result group for query {query:?}"
    );
}
