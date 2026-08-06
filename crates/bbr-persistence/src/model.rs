use std::path::{Path, PathBuf};

/// The complete ordered structural state owned by the persistence repository.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub profiles: Vec<Profile>,
    pub windows: Vec<WindowState>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub id: u64,
    pub title: String,
    pub hue: ProfileHue,
    pub spaces: Vec<Space>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileHue {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

impl ProfileHue {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Orange => "orange",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Blue => "blue",
            Self::Purple => "purple",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "red" => Some(Self::Red),
            "orange" => Some(Self::Orange),
            "yellow" => Some(Self::Yellow),
            "green" => Some(Self::Green),
            "blue" => Some(Self::Blue),
            "purple" => Some(Self::Purple),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Space {
    pub id: u64,
    pub title: String,
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
    fn default() -> Self {
        Self {
            width: 260.0,
            collapsed: false,
            last_expanded_width: 260.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RestoreOutcome {
    pub snapshot: Snapshot,
    pub warnings: Vec<RecoveryWarning>,
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
}

impl RecoveryWarning {
    pub(crate) fn sort_key(&self) -> (u64, u8) {
        match self {
            Self::ActiveTab { space_id, .. } => (*space_id, 0),
            Self::Split { space_id, .. } => (*space_id, 1),
            Self::WindowGeometry { window_id } => (*window_id, 2),
            Self::Sidebar { window_id } => (*window_id, 3),
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
    /// Six deterministic child-first DELETE statements plus one INSERT execution per row.
    pub sql_statements: u64,
    pub deleted_rows: u64,
    pub inserted_rows: u64,
}
