use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistenceError {
    Io(String),
    /// The read-only restore path refuses a store whose version stamp does
    /// not match the current schema. The write path never produces this
    /// error: it discards the store instead (see [`crate::StoreLoad`]).
    SchemaMismatch {
        found: u32,
        supported: u32,
    },
    Schema(SchemaIssue),
    Corruption(CorruptionIssue),
    InvalidSnapshot(InvalidSnapshotIssue),
    Storage(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "repository I/O error: {message}"),
            Self::SchemaMismatch { found, supported } => write!(
                formatter,
                "persistence schema version {found} does not match supported version {supported}; \
                 the store must be discarded and recreated"
            ),
            Self::Schema(issue) => write!(formatter, "invalid persistence schema: {issue}"),
            Self::Corruption(issue) => write!(formatter, "persisted state is corrupt: {issue}"),
            Self::InvalidSnapshot(issue) => write!(formatter, "snapshot is invalid: {issue}"),
            Self::Storage(message) => write!(formatter, "SQLite operation failed: {message}"),
        }
    }
}

impl std::error::Error for PersistenceError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaIssue {
    MissingTable(String),
    UnexpectedColumns { table: String },
    InvalidColumnShape { table: String, column: String },
    UnexpectedForeignKeys { table: String },
}

impl fmt::Display for SchemaIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTable(table) => write!(formatter, "missing required table `{table}`"),
            Self::UnexpectedColumns { table } => {
                write!(formatter, "table `{table}` has unexpected columns")
            }
            Self::InvalidColumnShape { table, column } => write!(
                formatter,
                "column `{table}.{column}` has an invalid primary-key or nullability shape"
            ),
            Self::UnexpectedForeignKeys { table } => {
                write!(formatter, "table `{table}` has unexpected foreign keys")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorruptionIssue {
    ForeignKeyViolation,
    InvalidRequiredValue { table: String, detail: String },
    InvalidOrder { owner: String },
    DuplicateIdentity { id: u64 },
    InvalidPinnedRegion { space_id: u64 },
    InvalidRecentlyClosed { tab_id: u64, detail: String },
}

impl fmt::Display for CorruptionIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignKeyViolation => formatter.write_str("structural foreign-key violation"),
            Self::InvalidRequiredValue { table, detail } => {
                write!(formatter, "invalid required value in `{table}`: {detail}")
            }
            Self::InvalidOrder { owner } => write!(formatter, "invalid order keys for {owner}"),
            Self::DuplicateIdentity { id } => write!(formatter, "duplicate identity {id}"),
            Self::InvalidPinnedRegion { space_id } => {
                write!(formatter, "space {space_id} does not have a pinned prefix")
            }
            Self::InvalidRecentlyClosed { tab_id, detail } => {
                write!(
                    formatter,
                    "recently closed tab {tab_id} is invalid: {detail}"
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidSnapshotIssue {
    InvalidId {
        kind: &'static str,
        id: u64,
    },
    DuplicateIdentity {
        id: u64,
    },
    InvalidUrl {
        tab_id: u64,
    },
    InvalidPinnedRegion {
        space_id: u64,
    },
    InvalidActiveTab {
        space_id: u64,
        tab_id: u64,
    },
    InvalidSplit {
        space_id: u64,
    },
    RecentlyClosedIdentityCollision {
        tab_id: u64,
    },
    InvalidFormerPosition {
        tab_id: u64,
    },
    InvalidGeometry {
        window_id: u64,
    },
    InvalidSidebar {
        window_id: u64,
    },
    /// The space's glyph emoji is not a member of the curated allowlist.
    InvalidEmoji {
        space_id: u64,
    },
    /// The global settings record violates an engine/template rule.
    InvalidSettings {
        detail: String,
    },
    /// A download history record violates a persisted-shape rule.
    InvalidDownload {
        id: u64,
        detail: String,
    },
    /// A command-history record violates a persisted-shape rule.
    InvalidHistory {
        id: u64,
        detail: String,
    },
}

impl fmt::Display for InvalidSnapshotIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId { kind, id } => write!(formatter, "invalid {kind} id {id}"),
            Self::DuplicateIdentity { id } => write!(formatter, "duplicate identity {id}"),
            Self::InvalidUrl { tab_id } => write!(formatter, "invalid URL for tab {tab_id}"),
            Self::InvalidPinnedRegion { space_id } => {
                write!(formatter, "space {space_id} does not have a pinned prefix")
            }
            Self::InvalidActiveTab { space_id, tab_id } => {
                write!(formatter, "active tab {tab_id} is not in space {space_id}")
            }
            Self::InvalidSplit { space_id } => {
                write!(formatter, "invalid split in space {space_id}")
            }
            Self::RecentlyClosedIdentityCollision { tab_id } => {
                write!(
                    formatter,
                    "recently closed identity {tab_id} collides with live state"
                )
            }
            Self::InvalidFormerPosition { tab_id } => {
                write!(formatter, "invalid former position for tab {tab_id}")
            }
            Self::InvalidGeometry { window_id } => {
                write!(formatter, "invalid geometry for window {window_id}")
            }
            Self::InvalidSidebar { window_id } => {
                write!(formatter, "invalid sidebar for window {window_id}")
            }
            Self::InvalidEmoji { space_id } => write!(
                formatter,
                "space {space_id} has a glyph emoji outside the curated allowlist"
            ),
            Self::InvalidSettings { detail } => {
                write!(formatter, "invalid settings record: {detail}")
            }
            Self::InvalidDownload { id, detail } => {
                write!(formatter, "invalid download history record {id}: {detail}")
            }
            Self::InvalidHistory { id, detail } => {
                write!(formatter, "invalid command-history record {id}: {detail}")
            }
        }
    }
}
