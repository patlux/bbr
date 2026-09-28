use std::path::{Path, PathBuf};

use crate::fields::{
    CollectionRecord, CommandHistoryEntry, DownloadRecord, SavedPageRecord, SearchSettings,
    SpaceGlyph, SpaceTint, StoreLoad,
};

/// The browser structural state the app persists independently of the
/// global stores: the profile/space/tab graph (with recently-closed entries
/// and splits) plus the window/sidebar presentation state. The settings,
/// download, and command-history rows belong to their own store methods and
/// are never touched by a browser-state write.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BrowserState {
    pub profiles: Vec<Profile>,
    pub windows: Vec<WindowState>,
}

/// The complete ordered structural state owned by the persistence repository.
///
/// Schema v2 adds the four field groups the blocked features need: per-space
/// appearance identity, the global settings record, terminal download
/// history, and visited command-history entries (all defined in
/// [`crate::fields`]). Schema v3 adds the global Saved Pages tables
/// (`saved_pages`, `collections`). All collections keep the caller's order;
/// persistence derives dense per-table order keys from it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub profiles: Vec<Profile>,
    pub windows: Vec<WindowState>,
    /// The single global settings record (one persisted row, id 1).
    pub settings: SearchSettings,
    /// Terminal download attempts, in persisted history order.
    pub downloads: Vec<DownloadRecord>,
    /// Visited command-history entries, in persisted corpus order.
    pub command_history: Vec<CommandHistoryEntry>,
    /// Global Saved Page records (schema v3), in persisted order.
    pub saved_pages: Vec<SavedPageRecord>,
    /// Flat Saved Page collections (schema v3), in persisted order.
    pub collections: Vec<CollectionRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub id: u64,
    pub title: String,
    pub hue: ProfileHue,
    pub spaces: Vec<Space>,
}

/// The six DESIGN.md §12 profile hues.
///
/// Assignment is round-robin and stable for the profile's lifetime: the hue
/// is a portable identity fact, while the concrete tile/letter colors are a
/// presentation concern resolved by the shell's palette from this name. The
/// variant set and machine-readable names mirror `bbr-domain` exactly; the
/// persistence crate must not depend on `bbr-domain`, so the enum and its
/// serialization are duplicated here as the repository's persisted contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileHue {
    Cobalt,
    Violet,
    Fuchsia,
    Amber,
    Terracotta,
    Slate,
}

impl ProfileHue {
    /// All hues in round-robin assignment order (DESIGN.md §12).
    pub const ALL: [ProfileHue; 6] = [
        ProfileHue::Cobalt,
        ProfileHue::Violet,
        ProfileHue::Fuchsia,
        ProfileHue::Amber,
        ProfileHue::Terracotta,
        ProfileHue::Slate,
    ];

    /// The canonical machine-readable name used by persisted rows and E2E
    /// assertions (DESIGN.md §12 allowed values).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Cobalt => "cobalt",
            Self::Violet => "violet",
            Self::Fuchsia => "fuchsia",
            Self::Amber => "amber",
            Self::Terracotta => "terracotta",
            Self::Slate => "slate",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "cobalt" => Some(Self::Cobalt),
            "violet" => Some(Self::Violet),
            "fuchsia" => Some(Self::Fuchsia),
            "amber" => Some(Self::Amber),
            "terracotta" => Some(Self::Terracotta),
            "slate" => Some(Self::Slate),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Space {
    pub id: u64,
    pub title: String,
    /// Curated tint identity (schema v2; see [`SpaceTint`]).
    pub tint: SpaceTint,
    /// Curated glyph identity (schema v2; see [`SpaceGlyph`]).
    pub glyph: SpaceGlyph,
    /// Curated tint intensity and static grain (schema v4; see
    /// [`crate::SpacePaper`]).
    pub paper: crate::SpacePaper,
    pub tabs: Vec<Tab>,
    pub active_tab: Option<u64>,
    pub recently_closed: Vec<RecentlyClosedTab>,
    pub split: Option<Split>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub pinned: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentlyClosedTab {
    pub tab_id: u64,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    pub former_position: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Split {
    pub primary_tab: u64,
    pub secondary_tab: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowState {
    pub id: u64,
    pub geometry: Option<WindowGeometry>,
    pub sidebar: SidebarState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarState {
    pub width: f64,
    pub collapsed: bool,
    pub last_expanded_width: f64,
}

impl Default for SidebarState {
    /// Mirrors the shell's shipped normal-density rail default
    /// (`bbr_ui::sidebar::geometry::SIDEBAR_DEFAULT`, 300 pt). The portable
    /// persistence crate must not depend on the UI crate, so the value is
    /// duplicated here and pinned by the sidebar-contract test.
    fn default() -> Self {
        Self {
            width: 300.0,
            collapsed: false,
            last_expanded_width: 300.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RestoreOutcome {
    pub snapshot: Snapshot,
    pub warnings: Vec<RecoveryWarning>,
    /// Whether the persisted store was loaded as-is or discarded and
    /// recreated empty (see [`StoreLoad`]).
    pub store: StoreLoad,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryWarning {
    ActiveTab {
        space_id: u64,
        persisted_tab_id: u64,
    },
    Split {
        space_id: u64,
        primary_tab_id: u64,
        secondary_tab_id: u64,
    },
    WindowGeometry {
        window_id: u64,
    },
    Sidebar {
        window_id: u64,
    },
    /// The persisted settings record was incoherent (invalid custom search
    /// template, or a template without the custom engine) and fell back to
    /// defaults.
    Settings {
        detail: String,
    },
}

impl RecoveryWarning {
    pub(crate) fn sort_key(&self) -> (u64, u8) {
        match self {
            Self::ActiveTab { space_id, .. } => (*space_id, 0),
            Self::Split { space_id, .. } => (*space_id, 1),
            Self::WindowGeometry { window_id } => (*window_id, 2),
            Self::Sidebar { window_id } => (*window_id, 3),
            Self::Settings { .. } => (u64::MAX, 4),
        }
    }
}

/// Path-based owner of the portable SQLite repository.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    path: PathBuf,
}

impl Repository {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WriteStats {
    /// Complete-snapshot replacement always commits exactly one transaction.
    pub committed_transactions: u64,
    /// Nine deterministic child-first DELETE statements plus one INSERT execution per row.
    pub sql_statements: u64,
    pub deleted_rows: u64,
    pub inserted_rows: u64,
}
