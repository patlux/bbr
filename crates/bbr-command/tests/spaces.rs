//! Standalone space ranking and activation contracts.

use bbr_command::{
    rank, Activation, CommandRegistry, Corpus, MatchQuality, Mode, ProfileEntry, ProfileId, Query,
    QueryContext, ResultGroup, SpaceEntry, SpaceId,
};

const PERSONAL: ProfileId = ProfileId(1);
const WORK: ProfileId = ProfileId(2);
const CURRENT: SpaceId = SpaceId(10);

fn rank_spaces(
    corpus: &Corpus,
    query: &str,
    context: QueryContext,
) -> Vec<bbr_command::CommandResult> {
    rank(
        corpus,
        &CommandRegistry::new(),
        &Query::new(Mode::Switcher, query),
        &context,
        32,
    )
    .results
}

fn keys(results: &[bbr_command::CommandResult]) -> Vec<&str> {
    results.iter().map(|result| result.key.as_str()).collect()
}

fn corpus() -> Corpus {
    let mut corpus = Corpus::new();
    corpus
        .profiles
        .push(ProfileEntry::new(PERSONAL, "Personal", 0));
    corpus.profiles.push(ProfileEntry::new(WORK, "Work", 1));
    corpus
        .spaces
        .push(SpaceEntry::new(CURRENT, "General", PERSONAL, 0));
    corpus
        .spaces
        .push(SpaceEntry::new(SpaceId(11), "Research", PERSONAL, 30));
    corpus
        .spaces
        .push(SpaceEntry::new(SpaceId(12), "Research", WORK, 10));
    corpus
        .spaces
        .push(SpaceEntry::new(SpaceId(13), "Research", WORK, 10));
    corpus
}

#[test]
fn empty_space_is_matchable_and_activates_by_identity() {
    let results = rank_spaces(
        &corpus(),
        "research",
        QueryContext::with_current(CURRENT, PERSONAL),
    );
    assert_eq!(keys(&results)[..3], ["space:12", "space:13", "space:11"]);
    assert_eq!(results[0].group, ResultGroup::Space);
    assert_eq!(results[0].title, "Research");
    assert_eq!(
        results[0].activation,
        Activation::SelectSpace { space: SpaceId(12) }
    );
    assert_eq!(results[0].profile, Some(WORK));
}

#[test]
fn empty_switcher_remains_current_tabs_only() {
    let results = rank_spaces(&corpus(), "", QueryContext::with_current(CURRENT, PERSONAL));
    assert!(results.is_empty());
}

#[test]
fn current_space_is_excluded_even_when_its_name_matches() {
    let results = rank_spaces(
        &corpus(),
        "general",
        QueryContext::with_current(CURRENT, PERSONAL),
    );
    assert!(results.iter().all(|result| result.key != "space:10"));
}

#[test]
fn same_cross_and_unknown_profile_labels_are_explicit() {
    let corpus = corpus();
    let results = rank_spaces(
        &corpus,
        "research",
        QueryContext::with_current(CURRENT, PERSONAL),
    );
    let same = results
        .iter()
        .find(|result| result.key == "space:11")
        .unwrap();
    let cross = results
        .iter()
        .find(|result| result.key == "space:12")
        .unwrap();
    assert_eq!(same.subtitle.as_deref(), Some("Space"));
    assert_eq!(cross.subtitle.as_deref(), Some("Profile Work"));

    let unknown = rank_spaces(&corpus, "research", QueryContext::new());
    let personal = unknown
        .iter()
        .find(|result| result.key == "space:11")
        .unwrap();
    assert_eq!(personal.subtitle.as_deref(), Some("Profile Personal"));
}

#[test]
fn missing_profile_and_space_names_use_id_fallbacks() {
    let mut corpus = Corpus::new();
    corpus
        .spaces
        .push(SpaceEntry::new(SpaceId(21), "", ProfileId(9), 0));
    // An empty name cannot match a non-empty query; use an explicit matchable
    // name first, then clear only the display name after construction.
    corpus.spaces[0] = SpaceEntry::new(SpaceId(21), "profile", ProfileId(9), 0);
    corpus.spaces[0].name.clear();
    let results = rank_spaces(&corpus, "profile", QueryContext::new());
    assert_eq!(results[0].title, "space 21");
    assert_eq!(results[0].subtitle.as_deref(), Some("Profile profile 9"));
}

#[test]
fn exact_prefix_substring_quality_then_index_and_id_define_order() {
    let mut corpus = Corpus::new();
    corpus.spaces.extend([
        SpaceEntry::new(SpaceId(40), "Research", PERSONAL, 50),
        SpaceEntry::new(SpaceId(41), "Research Notes", PERSONAL, 5),
        SpaceEntry::new(SpaceId(42), "Deep Research", PERSONAL, 0),
        SpaceEntry::new(SpaceId(39), "Research", PERSONAL, 50),
    ]);
    let results = rank_spaces(&corpus, "research", QueryContext::new());
    assert_eq!(
        keys(&results)[..4],
        ["space:39", "space:40", "space:41", "space:42"]
    );
    assert_eq!(results[0].score.quality, MatchQuality::Exact);
    assert_eq!(results[2].score.quality, MatchQuality::Prefix);
    assert_eq!(results[3].score.quality, MatchQuality::Substring);
}

#[test]
fn reversing_spaces_vector_does_not_change_index_id_order() {
    let corpus = corpus();
    let first = rank_spaces(&corpus, "research", QueryContext::new());
    let mut reversed = corpus.clone();
    reversed.spaces.reverse();
    let second = rank_spaces(&reversed, "research", QueryContext::new());
    assert_eq!(keys(&first), keys(&second));
}
