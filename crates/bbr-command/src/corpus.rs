//! Provider-neutral owned input corpus for command ranking.
//!
//! The corpus owns every searchable record the ranking consumes: open tabs,
//! spaces, profiles, and local history. Providers (later the domain/
//! persistence layers) construct these entries; the core never touches a
//! database or render path. Lowercase projections are precomputed once so
//! repeated ranking never re-folds case.

use crate::id::{HistoryId, ProfileId, SpaceId, TabId};

/// One open tab in the corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabEntry {
    /// Stable tab identity.
    pub id: TabId,
    /// Display title; may be empty when the page has not reported one.
    pub title: String,
    /// Display URL used for matching.
    pub url: String,
    /// The space the tab belongs to.
    pub space: SpaceId,
    /// The profile the owning space belongs to.
    pub profile: ProfileId,
    /// Whether the tab sits in the pinned region.
    pub pinned: bool,
    /// Deterministic ordering key within the space (lower renders first).
    pub index: u64,
    title_lower: String,
    url_lower: String,
}

impl TabEntry {
    /// Builds a tab entry, precomputing the lowercase match projections.
    pub fn new(
        id: TabId,
        title: impl Into<String>,
        url: impl Into<String>,
        space: SpaceId,
        profile: ProfileId,
        pinned: bool,
        index: u64,
    ) -> Self {
        let title = title.into();
        let url = url.into();
        let title_lower = title.to_lowercase();
        let url_lower = url.to_lowercase();
        TabEntry {
            id,
            title,
            url,
            space,
            profile,
            pinned,
            index,
            title_lower,
            url_lower,
        }
    }

    /// Precomputed lowercase title used for deterministic matching.
    pub fn title_lower(&self) -> &str {
        &self.title_lower
    }

    /// Precomputed lowercase URL used for deterministic matching.
    pub fn url_lower(&self) -> &str {
        &self.url_lower
    }
}

/// One workspace (space) entry used to label cross-space results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceEntry {
    /// Stable space identity.
    pub id: SpaceId,
    /// Display name of the space.
    pub name: String,
    /// The profile the space belongs to.
    pub profile: ProfileId,
    /// Deterministic corpus ordering key (lower renders first).
    pub index: u64,
    name_lower: String,
}

impl SpaceEntry {
    /// Builds a space entry, precomputing the lowercase name.
    pub fn new(id: SpaceId, name: impl Into<String>, profile: ProfileId, index: u64) -> Self {
        let name = name.into();
        let name_lower = name.to_lowercase();
        SpaceEntry {
            id,
            name,
            profile,
            index,
            name_lower,
        }
    }

    /// Precomputed lowercase name used for deterministic matching.
    pub fn name_lower(&self) -> &str {
        &self.name_lower
    }
}

/// One profile entry used to label cross-profile results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileEntry {
    /// Stable profile identity.
    pub id: ProfileId,
    /// Display name of the profile.
    pub name: String,
    /// Deterministic corpus ordering key (lower renders first).
    pub index: u64,
}

impl ProfileEntry {
    /// Builds a profile entry.
    pub fn new(id: ProfileId, name: impl Into<String>, index: u64) -> Self {
        ProfileEntry {
            id,
            name: name.into(),
            index,
        }
    }
}

/// One local history record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// Stable history-record identity.
    pub id: HistoryId,
    /// Display title; may be empty.
    pub title: String,
    /// Normalized URL of the visited page.
    pub url: String,
    /// Number of recorded visits (informational, used as a secondary key).
    pub visit_count: u64,
    /// Monotonic recency marker; **larger is more recent**.
    pub last_visited: u64,
    title_lower: String,
    url_lower: String,
}

impl HistoryEntry {
    /// Builds a history entry, precomputing the lowercase projections.
    pub fn new(
        id: HistoryId,
        title: impl Into<String>,
        url: impl Into<String>,
        visit_count: u64,
        last_visited: u64,
    ) -> Self {
        let title = title.into();
        let url = url.into();
        let title_lower = title.to_lowercase();
        let url_lower = url.to_lowercase();
        HistoryEntry {
            id,
            title,
            url,
            visit_count,
            last_visited,
            title_lower,
            url_lower,
        }
    }

    /// Precomputed lowercase title used for deterministic matching.
    pub fn title_lower(&self) -> &str {
        &self.title_lower
    }

    /// Precomputed lowercase URL used for deterministic matching.
    pub fn url_lower(&self) -> &str {
        &self.url_lower
    }
}

/// The owned, provider-neutral input set for one ranking pass.
///
/// All collections keep insertion order; ranking never re-orders them and
/// never iterates a hash map, so output depends only on this order plus the
/// explicit ordering keys above.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Corpus {
    /// Open tabs in corpus order.
    pub tabs: Vec<TabEntry>,
    /// Spaces in corpus order (used for cross-space labels).
    pub spaces: Vec<SpaceEntry>,
    /// Profiles in corpus order (used for cross-profile labels).
    pub profiles: Vec<ProfileEntry>,
    /// Local history records in corpus order.
    pub history: Vec<HistoryEntry>,
}

impl Corpus {
    /// An empty corpus.
    pub fn new() -> Self {
        Self::default()
    }

    /// The display name of a space, when present.
    pub fn space_name(&self, space: SpaceId) -> Option<&str> {
        self.spaces
            .iter()
            .find(|s| s.id == space)
            .map(|s| s.name.as_str())
    }

    /// The display name of a profile, when present.
    pub fn profile_name(&self, profile: ProfileId) -> Option<&str> {
        self.profiles
            .iter()
            .find(|p| p.id == profile)
            .map(|p| p.name.as_str())
    }

    /// The profile of a space, when the space is known.
    pub fn space_profile(&self, space: SpaceId) -> Option<ProfileId> {
        self.spaces
            .iter()
            .find(|s| s.id == space)
            .map(|s| s.profile)
    }

    /// The explicit deterministic order of a space, when present.
    pub fn space_order(&self, space: SpaceId) -> Option<u64> {
        self.spaces
            .iter()
            .find(|entry| entry.id == space)
            .map(|entry| entry.index)
    }
}
