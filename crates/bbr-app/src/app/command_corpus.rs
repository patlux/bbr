//! Provider-neutral local corpus from the controller snapshot.
//!
//! Maps the authoritative [`ControllerSnapshot`] onto the portable
//! [`bbr_command::Corpus`] on demand: tab/space/profile identity and
//! deterministic ordering are preserved, and the real history collection
//! stays empty until the Phase-7 persistence composition supplies it. The
//! corpus is rebuilt per ranking pass (a few hundred entries at most), never
//! cached, so it cannot drift from controller state.
//!
//! Tab identity comes from the controller's complete [`ControllerSnapshot`]
//! global tab collection (`all_tabs`), so every tab of every space/profile
//! ranks with its persistent id (`tab:<id>`), owning space, owning profile,
//! and owning-space index — the `OtherSpace` group is reachable and
//! activation routes by identity, never by active-space position.

use bbr_browser::ControllerSnapshot;
use bbr_command::corpus::{Corpus, HistoryEntry, ProfileEntry, SpaceEntry, TabEntry};
use bbr_command::id::{ProfileId, SpaceId, TabId};
use bbr_persistence::{CollectionRecord, SavedPageRecord};

use crate::store::saved_pages::saved_page_to_entry;

/// Build the ranking corpus from the current controller snapshot plus the
/// app-owned in-memory providers (Phase 7 history, B2 Saved Pages):
/// tab/space/profile identity and deterministic ordering are preserved, and
/// history/Saved Page entries rank in the exact persisted corpus order the
/// providers own. The corpus is rebuilt per ranking pass (a few hundred tab
/// entries at most plus the bounded providers), never cached, so it cannot
/// drift from controller or provider state — and it never touches SQLite:
/// `history` and `saved_pages` are the already-loaded provider data (see
/// [`crate::store::AppStore`]).
pub fn corpus_from_snapshot(
    snapshot: &ControllerSnapshot,
    history: &[HistoryEntry],
    saved_pages: &[SavedPageRecord],
    collections: &[CollectionRecord],
) -> Corpus {
    let mut corpus = Corpus::new();
    for (index, profile) in snapshot.profiles.iter().enumerate() {
        corpus.profiles.push(ProfileEntry::new(
            ProfileId(profile.id.0),
            profile.title.clone(),
            index as u64,
        ));
    }
    for (index, space) in snapshot.spaces.iter().enumerate() {
        corpus.spaces.push(SpaceEntry::new(
            SpaceId(space.id.0),
            space.title.clone(),
            ProfileId(space.profile.0),
            index as u64,
        ));
    }
    // Complete tab identity from the controller's global collection: every
    // tab of every space/profile keeps its persistent id (`tab:<id>`), its
    // owning space and profile, and its exact owning-space index. Activating
    // `SelectTab { tab, space }` therefore routes by identity: the owning
    // space is activated first (restoring its warm sessions) and the exact
    // index selected inside it — never an active-space position guess.
    for tab in &snapshot.all_tabs {
        corpus.tabs.push(
            TabEntry::new(
                TabId(tab.id.0),
                tab.title.clone(),
                tab.url.clone(),
                SpaceId(tab.space.0),
                ProfileId(tab.profile.0),
                tab.pinned,
                tab.index as u64,
            )
            // Blueprint D04: the recency the empty ⌘T recents order by.
            .with_last_used(tab.last_used),
        );
    }
    // History from the app-owned in-memory provider, in persisted corpus
    // order: the single runtime authority ranking consumes (Phase 7).
    corpus.history.extend(history.iter().cloned());
    // Saved Pages from the app-owned in-memory provider (B2 packet
    // `saved-pages-actions-command`): the provider's deterministic
    // presentation order (most recently updated first) is preserved
    // verbatim — ranking never re-sorts and never queries SQLite.
    corpus.saved_pages.extend(
        saved_pages
            .iter()
            .map(|page| saved_page_to_entry(page, collections)),
    );
    corpus
}

/// The settled result limit (Design §11.5 settled value).
pub const RESULT_LIMIT: usize = 16;

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-profile, two-space snapshot with two tabs in the active space
    /// (one pinned) and a warm tab in a second space of another profile.
    /// Tab ids are the persistent domain identities (1-based).
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
        let global = |id: u64,
                      space: u64,
                      profile: u64,
                      index: usize,
                      url: &str,
                      title: &str,
                      pinned: bool| bbr_browser::GlobalTabSnapshot {
            id: bbr_domain::TabId(id),
            space: bbr_domain::SpaceId(space),
            profile: bbr_web::ProfileId(profile),
            index,
            url: url.into(),
            title: title.into(),
            pinned,
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
                global(1, 2, 1, 0, "https://a.example", "Alpha", true),
                global(2, 2, 1, 1, "https://b.example", "Beta", false),
                global(3, 4, 3, 0, "https://c.example", "Gamma", false),
            ],
            profiles: vec![
                bbr_browser::ProfileSnapshot {
                    index: 0,
                    id: bbr_web::ProfileId(1),
                    title: "Default".into(),
                    hue: bbr_browser::ProfileHue::Cobalt,
                },
                bbr_browser::ProfileSnapshot {
                    index: 1,
                    id: bbr_web::ProfileId(3),
                    title: "Private".into(),
                    hue: bbr_browser::ProfileHue::Violet,
                },
            ],
            spaces: vec![
                bbr_browser::SpaceSnapshot {
                    index: 0,
                    id: bbr_browser::SpaceId(2),
                    profile: bbr_web::ProfileId(1),
                    title: "Default".into(),
                    active_tab: Some(0),
                    active_view: Some(bbr_web::WebViewId(1)),
                    active: true,
                    tab_count: 2,
                },
                bbr_browser::SpaceSnapshot {
                    index: 1,
                    id: bbr_browser::SpaceId(4),
                    profile: bbr_web::ProfileId(3),
                    title: "Private".into(),
                    active_tab: Some(0),
                    active_view: Some(bbr_web::WebViewId(3)),
                    active: false,
                    tab_count: 1,
                },
            ],
            active_index: Some(0),
            nav: Default::default(),
            web_view_hosted: true,
            engine_sessions: 3,
            active_view: Some(bbr_web::WebViewId(1)),
            lifecycle_counts: bbr_browser::LifecycleCounts::default(),
            preview: None,
        }
    }

    #[test]
    fn corpus_preserves_ids_order_pinned_and_labels() {
        let corpus = corpus_from_snapshot(&snapshot(), &[], &[], &[]);
        assert_eq!(corpus.profiles.len(), 2);
        assert_eq!(corpus.profiles[0].id, ProfileId(1));
        assert_eq!(corpus.profiles[0].name, "Default");
        assert_eq!(corpus.profiles[1].id, ProfileId(3));
        assert_eq!(corpus.spaces.len(), 2);
        assert_eq!(corpus.spaces[0].id, SpaceId(2));
        assert_eq!(corpus.spaces[1].id, SpaceId(4));
        assert_eq!(corpus.spaces[1].profile, ProfileId(3));
        // All three tabs rank with their persistent ids and owning
        // space/profile/index — the complete cross-space corpus.
        assert_eq!(corpus.tabs.len(), 3);
        let alpha = &corpus.tabs[0];
        assert_eq!(alpha.id, TabId(1));
        assert_eq!(alpha.title, "Alpha");
        assert_eq!(alpha.url, "https://a.example");
        assert_eq!(alpha.space, SpaceId(2));
        assert_eq!(alpha.profile, ProfileId(1));
        assert!(alpha.pinned);
        assert_eq!(alpha.index, 0);
        let beta = &corpus.tabs[1];
        assert_eq!(beta.id, TabId(2));
        assert!(!beta.pinned);
        let gamma = &corpus.tabs[2];
        assert_eq!(gamma.id, TabId(3));
        assert_eq!(gamma.space, SpaceId(4));
        assert_eq!(gamma.profile, ProfileId(3));
        assert_eq!(gamma.index, 0);
    }

    #[test]
    fn history_stays_an_empty_real_collection_without_the_provider() {
        let corpus = corpus_from_snapshot(&snapshot(), &[], &[], &[]);
        assert!(corpus.history.is_empty());
    }

    #[test]
    fn history_ranks_in_exact_provider_order() {
        // Phase 7: the corpus maps the app-owned provider data verbatim, in
        // the exact persisted corpus order — never re-sorted by the corpus
        // builder and never re-read from SQLite.
        use bbr_command::corpus::HistoryEntry;
        use bbr_command::id::HistoryId;
        let history = vec![
            HistoryEntry::new(
                HistoryId(7),
                "Seven".to_string(),
                "https://seven.test/".to_string(),
                1,
                5,
            ),
            HistoryEntry::new(
                HistoryId(3),
                "Three".to_string(),
                "https://three.test/".to_string(),
                4,
                9,
            ),
        ];
        let corpus = corpus_from_snapshot(&snapshot(), &history, &[], &[]);
        assert_eq!(corpus.history.len(), 2);
        assert_eq!(corpus.history[0].id, HistoryId(7));
        assert_eq!(corpus.history[1].id, HistoryId(3));
        assert_eq!(corpus.history[0].title, "Seven");
        assert_eq!(corpus.history[1].visit_count, 4);
    }

    #[test]
    fn corpus_ranking_produces_deterministic_local_results() {
        let corpus = corpus_from_snapshot(&snapshot(), &[], &[], &[]);
        let registry = bbr_command::CommandRegistry::builtin();
        // "alpha" matches the pinned tab exactly and nothing else.
        let query = bbr_command::Query::new(bbr_command::Mode::Location, "alpha");
        let context = bbr_command::QueryContext::with_current(
            bbr_command::id::SpaceId(2),
            bbr_command::id::ProfileId(1),
        );
        let ranked = bbr_command::rank(&corpus, &registry, &query, &context, RESULT_LIMIT);
        assert!(!ranked.results.is_empty());
        assert_eq!(ranked.results[0].key, "tab:1");
        assert_eq!(
            ranked.results[0].activation,
            bbr_command::Activation::SelectTab {
                tab: TabId(1),
                space: SpaceId(2),
            }
        );
        // A query matching only the fallback still produces the search row.
        let query = bbr_command::Query::new(bbr_command::Mode::Location, "nonexistent term");
        let ranked = bbr_command::rank(&corpus, &registry, &query, &context, RESULT_LIMIT);
        assert!(
            ranked
                .results
                .iter()
                .any(|r| matches!(r.activation, bbr_command::Activation::Search { .. })),
            "the settle duckduckgo search fallback row is present"
        );
        assert!(ranked.results.len() <= RESULT_LIMIT);
    }

    #[test]
    fn ranking_exposes_cross_profile_space_identity() {
        let corpus = corpus_from_snapshot(&snapshot(), &[], &[], &[]);
        let registry = bbr_command::CommandRegistry::builtin();
        let query = bbr_command::Query::new(bbr_command::Mode::Location, "private");
        let context = bbr_command::QueryContext::with_current(
            bbr_command::id::SpaceId(2),
            bbr_command::id::ProfileId(1),
        );
        let ranked = bbr_command::rank(&corpus, &registry, &query, &context, RESULT_LIMIT);
        let space = ranked
            .results
            .iter()
            .find(|r| r.key == "space:4")
            .expect("cross-profile space result");
        assert_eq!(space.profile, Some(ProfileId(3)));
        assert_eq!(
            space.subtitle.as_deref(),
            Some("Profile Private"),
            "cross-profile identity is never hidden"
        );
        assert_eq!(
            space.activation,
            bbr_command::Activation::SelectSpace { space: SpaceId(4) }
        );
    }

    #[test]
    fn other_space_tabs_rank_with_persistent_identity() {
        // The complete corpus exposes tabs in other spaces with their
        // persistent ids; the ranker labels them with profile/space identity
        // (same-profile: the space name; cross-profile: profile + space).
        let corpus = corpus_from_snapshot(&snapshot(), &[], &[], &[]);
        let registry = bbr_command::CommandRegistry::builtin();
        let query = bbr_command::Query::new(bbr_command::Mode::Location, "gamma");
        let context = bbr_command::QueryContext::with_current(
            bbr_command::id::SpaceId(2),
            bbr_command::id::ProfileId(1),
        );
        let ranked = bbr_command::rank(&corpus, &registry, &query, &context, RESULT_LIMIT);
        let tab = ranked
            .results
            .iter()
            .find(|r| r.key == "tab:3")
            .expect("other-space tab result");
        assert_eq!(
            tab.activation,
            bbr_command::Activation::SelectTab {
                tab: TabId(3),
                space: SpaceId(4),
            }
        );
        assert_eq!(
            tab.subtitle.as_deref(),
            Some("Profile Private · Space Private"),
            "cross-profile tab identity is never hidden"
        );
    }
}
