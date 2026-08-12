//! App-owned browser-state persistence mapping (Phase 7, warm-restore
//! slice): the one portable mapping between the persistence [`BrowserState`]
//! DTOs and the controller's [`RestoreIntent`], plus the derived window
//! identity rule. Pure functions — no repository, no GPUI — so the mapping
//! is unit-testable without the app.
//!
//! The two shapes are the same identity graph; the deltas are mechanical:
//! the persistence `Space` carries curated appearance identity (tint/glyph,
//! schema v2) that the domain has no per-space field for in this slice, and
//! the persistence `BrowserState` additionally owns the window/sidebar
//! presentation rows the app composes.
//!
//! ## Appearance truth
//!
//! The space-appearance contract derives deterministic defaults from the
//! owning profile identity hue when no record exists (`SpaceTheme::
//! from_profile_hue`), and the identity defaults to the first curated glyph.
//! This slice therefore writes exactly those derived defaults (the effective
//! appearance is the same with or without a persisted row) and restore
//! ignores the persisted tint/glyph (the domain has no per-space field; the
//! appearance slice owns rendering them). No appearance fact is invented:
//! both values are the canonical derived defaults of the integrated
//! appearance contract.
//!
//! ## Truthful persistence boundary (§11.3 configuration surface)
//!
//! The Space-creation-and-configuration surface (packet
//! `space-creation-appearance-ui.md`) commits a rename and a curated
//! theme/identity for the exact bound `SpaceId`. Under the current schema:
//!
//! - **The rename persists truthfully.** The space title flows through this
//!   mapping (`IntentSpace.title` → `Space.title`) on every structural
//!   write, so a committed rename survives a graceful restart.
//! - **The curated theme/identity is session-valid only.** The domain's
//!   `SpaceAppearanceStore` is in-memory, and the persisted `Space` row
//!   always stores the canonical derived defaults (`tint_for_hue` +
//!   `DEFAULT_GLYPH`) while restore drops tint/glyph entirely. A curated
//!   theme/identity therefore reverts to the derived default after a
//!   restart. No code path claims otherwise: the surface's `Done` pokes the
//!   browser-state writer (the rename is durable), and the appearance is
//!   never mislabeled persistent. A future appearance-persistence slice
//!   would extend the schema — never this mapping.
//!
//! ## Window identity
//!
//! The persistence identity contract is global uniqueness across every kind
//! (profiles, spaces, tabs, recently-closed entries, **and** windows).
//! The app's single window therefore derives its identity per write as
//! `max(all domain ids) + 1` — above the current model, so a fresh creation
//! can never collide with the window row, and deterministic across restarts
//! (the restore's domain ids are fixed).

use bbr_browser::ProfileHue as DomainHue;
use bbr_browser::{
    HydrationPolicy, IntentClosedTab, IntentProfile, IntentSpace, IntentSplit, IntentTab,
    RestoreIntent,
};
use bbr_persistence::{GlyphName, Profile, Space, SpaceGlyph, SpaceTint, Tab};

/// The canonical default space glyph identity (the first curated BBR glyph).
pub(crate) const DEFAULT_GLYPH: GlyphName = GlyphName::Code;

/// The domain hue for a persisted hue (same names; the crates cannot share
/// the enum without a dependency).
pub(crate) fn to_domain_hue(hue: bbr_persistence::ProfileHue) -> DomainHue {
    use bbr_persistence::ProfileHue as Persisted;
    match hue {
        Persisted::Cobalt => DomainHue::Cobalt,
        Persisted::Violet => DomainHue::Violet,
        Persisted::Fuchsia => DomainHue::Fuchsia,
        Persisted::Amber => DomainHue::Amber,
        Persisted::Terracotta => DomainHue::Terracotta,
        Persisted::Slate => DomainHue::Slate,
    }
}

/// The persisted hue for a domain hue (see [`to_domain_hue`]).
pub(crate) fn to_persisted_hue(hue: DomainHue) -> bbr_persistence::ProfileHue {
    use bbr_persistence::ProfileHue as Persisted;
    match hue {
        DomainHue::Cobalt => Persisted::Cobalt,
        DomainHue::Violet => Persisted::Violet,
        DomainHue::Fuchsia => Persisted::Fuchsia,
        DomainHue::Amber => Persisted::Amber,
        DomainHue::Terracotta => Persisted::Terracotta,
        DomainHue::Slate => Persisted::Slate,
    }
}

/// Maps the persisted structural profiles onto the portable restore intent
/// the chrome-first hydration policy consumes. Appearance identity (tint/
/// glyph) is dropped: the domain has no per-space appearance field in this
/// slice, and the effective appearance derives the same defaults.
pub fn persisted_to_intent(profiles: &[Profile]) -> RestoreIntent {
    RestoreIntent {
        profiles: profiles
            .iter()
            .map(|profile| IntentProfile {
                id: profile.id,
                title: profile.title.clone(),
                hue: to_domain_hue(profile.hue),
                spaces: profile
                    .spaces
                    .iter()
                    .map(|space| IntentSpace {
                        id: space.id,
                        title: space.title.clone(),
                        tabs: space
                            .tabs
                            .iter()
                            .map(|tab| IntentTab {
                                id: tab.id,
                                title: tab.title.clone(),
                                url: tab.url.clone(),
                                pinned: tab.pinned,
                            })
                            .collect(),
                        active_tab: space.active_tab,
                        recently_closed: space
                            .recently_closed
                            .iter()
                            .map(|closed| IntentClosedTab {
                                tab_id: closed.tab_id,
                                title: closed.title.clone(),
                                url: closed.url.clone(),
                                pinned: closed.pinned,
                                former_position: closed.former_position,
                            })
                            .collect(),
                        split: space.split.map(|split| IntentSplit {
                            primary_tab: split.primary_tab,
                            secondary_tab: split.secondary_tab,
                        }),
                    })
                    .collect(),
            })
            .collect(),
    }
}

/// Maps the portable intent (the controller's live model export) onto the
/// persistence browser-state DTO. Space appearance is written as the
/// canonical derived defaults (see the module docs); the window row is
/// supplied by the caller with its derived identity.
pub fn intent_to_persisted(intent: &RestoreIntent) -> Vec<Profile> {
    intent
        .profiles
        .iter()
        .map(|profile| Profile {
            id: profile.id,
            title: profile.title.clone(),
            hue: to_persisted_hue(profile.hue),
            spaces: profile
                .spaces
                .iter()
                .map(|space| Space {
                    id: space.id,
                    title: space.title.clone(),
                    tint: tint_for_hue(profile.hue),
                    glyph: SpaceGlyph::Glyph(DEFAULT_GLYPH),
                    tabs: space
                        .tabs
                        .iter()
                        .map(|tab| Tab {
                            id: tab.id,
                            title: tab.title.clone(),
                            url: tab.url.clone(),
                            pinned: tab.pinned,
                        })
                        .collect(),
                    active_tab: space.active_tab,
                    recently_closed: space
                        .recently_closed
                        .iter()
                        .map(|closed| bbr_persistence::RecentlyClosedTab {
                            tab_id: closed.tab_id,
                            title: closed.title.clone(),
                            url: closed.url.clone(),
                            pinned: closed.pinned,
                            former_position: closed.former_position,
                        })
                        .collect(),
                    split: space.split.map(|split| bbr_persistence::Split {
                        primary_tab: split.primary_tab,
                        secondary_tab: split.secondary_tab,
                    }),
                })
                .collect(),
        })
        .collect()
}

/// The derived default space tint for a profile (the space-appearance
/// contract's `SpaceTheme::from_profile_hue`, mirrored onto the persistence
/// enum whose names are identical).
fn tint_for_hue(hue: DomainHue) -> SpaceTint {
    match hue {
        DomainHue::Cobalt => SpaceTint::Cobalt,
        DomainHue::Violet => SpaceTint::Violet,
        DomainHue::Fuchsia => SpaceTint::Fuchsia,
        DomainHue::Amber => SpaceTint::Amber,
        DomainHue::Terracotta => SpaceTint::Terracotta,
        DomainHue::Slate => SpaceTint::Slate,
    }
}

/// The derived window identity for a browser state: one above the maximum
/// identity of every kind in the model (global uniqueness across kinds).
/// An empty model yields 1 (the first positive identity).
pub fn derived_window_id(profiles: &[Profile]) -> u64 {
    let max = profiles
        .iter()
        .flat_map(|profile| {
            profile
                .spaces
                .iter()
                .flat_map(|space| {
                    space
                        .tabs
                        .iter()
                        .map(|tab| tab.id)
                        .chain(space.recently_closed.iter().map(|closed| closed.tab_id))
                        .chain(std::iter::once(space.id))
                })
                .chain(std::iter::once(profile.id))
        })
        .max()
        .unwrap_or(0);
    max + 1
}

/// The deterministic default active space for a restored intent (the policy
/// default: first space with an active tab, else the first space).
pub fn default_active_space(intent: &RestoreIntent) -> Option<bbr_browser::SpaceId> {
    HydrationPolicy::default_active_space(intent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_persistence::ProfileHue as PersistedHue;

    fn sample_profiles() -> Vec<Profile> {
        vec![Profile {
            id: 1,
            title: "Work".into(),
            hue: PersistedHue::Cobalt,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                tint: SpaceTint::Slate,
                glyph: SpaceGlyph::Emoji("⚙".into()),
                tabs: vec![
                    Tab {
                        id: 100,
                        title: "Pinned".into(),
                        url: "https://a.example".into(),
                        pinned: true,
                    },
                    Tab {
                        id: 101,
                        title: "One".into(),
                        url: "https://b.example".into(),
                        pinned: false,
                    },
                ],
                active_tab: Some(101),
                recently_closed: vec![bbr_persistence::RecentlyClosedTab {
                    tab_id: 900,
                    title: String::new(),
                    url: "https://closed.example".into(),
                    pinned: false,
                    former_position: 2,
                }],
                split: Some(bbr_persistence::Split {
                    primary_tab: 101,
                    secondary_tab: 100,
                }),
            }],
        }]
    }

    #[test]
    fn persisted_to_intent_keeps_ids_order_active_split_and_closed() {
        let intent = persisted_to_intent(&sample_profiles());
        assert_eq!(intent.profiles.len(), 1);
        let profile = &intent.profiles[0];
        assert_eq!(profile.id, 1);
        assert_eq!(profile.title, "Work");
        let space = &profile.spaces[0];
        assert_eq!(space.id, 10);
        assert_eq!(space.tabs[0].id, 100);
        assert!(space.tabs[0].pinned);
        assert_eq!(space.tabs[1].url, "https://b.example");
        assert_eq!(space.active_tab, Some(101));
        assert_eq!(space.recently_closed[0].tab_id, 900);
        assert_eq!(space.recently_closed[0].former_position, 2);
        assert_eq!(
            space.split,
            Some(IntentSplit {
                primary_tab: 101,
                secondary_tab: 100
            })
        );
        // Appearance is dropped: the intent has no tint/glyph.
        assert!(profile.hue == bbr_browser::ProfileHue::Cobalt);
    }

    #[test]
    fn intent_to_persisted_writes_derived_defaults_and_round_trips() {
        let intent = persisted_to_intent(&sample_profiles());
        let profiles = intent_to_persisted(&intent);
        // Identities and structure round-trip exactly.
        assert_eq!(
            profiles,
            sample_profiles()
                .into_iter()
                .map(|mut p| {
                    p.spaces[0].tint = SpaceTint::Cobalt;
                    p.spaces[0].glyph = SpaceGlyph::Glyph(DEFAULT_GLYPH);
                    p
                })
                .collect::<Vec<_>>()
        );
        // The derived default tint follows the profile hue (cobalt).
        assert_eq!(profiles[0].spaces[0].tint, SpaceTint::Cobalt);
        assert_eq!(
            profiles[0].spaces[0].glyph,
            SpaceGlyph::Glyph(GlyphName::Code)
        );
    }

    #[test]
    fn derived_window_id_clears_every_kind_and_empty_model() {
        assert_eq!(derived_window_id(&[]), 1);
        let profiles = sample_profiles();
        // Max identity across profile 1, space 10, tabs 100/101, closed 900.
        assert_eq!(derived_window_id(&profiles), 901);
    }

    #[test]
    fn default_active_space_prefers_the_first_space_with_an_active_tab() {
        let intent = persisted_to_intent(&sample_profiles());
        assert_eq!(
            default_active_space(&intent),
            Some(bbr_browser::SpaceId(10))
        );
    }
}
