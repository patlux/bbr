//! Schema-v2 field groups: the persisted identity and record types.
//!
//! These are the four field groups the v2 shape carries for the blocked
//! features: per-space appearance identity, the global settings record,
//! terminal download history, and visited command-history entries, plus the
//! store-load disposition. The closed identity sets mirror the portable
//! feature contracts (space appearance, search settings, downloads,
//! command corpus) as the repository's persisted contract — the persistence
//! crate must not depend on `bbr-domain` or `bbr-command` (see
//! [`crate::model::ProfileHue`] for the precedent).

/// A curated space tint token (schema v2).
///
/// Identity data, not free-form styling: a space's tint is exactly one of
/// these stable tokens. The first six mirror the DESIGN.md §12 profile
/// identity hues so the existing round-robin identity "goes into" this
/// mechanism; `Pine`, `Plum`, and `Ink` are curated space-only tokens. The
/// canonical name set mirrors the delivered portable space-appearance
/// contract; concrete light/dark colors are a presentation concern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceTint {
    Cobalt,
    Violet,
    Fuchsia,
    Amber,
    Terracotta,
    Slate,
    Pine,
    Plum,
    Ink,
}

impl SpaceTint {
    /// All curated tints in stable palette order.
    pub const ALL: [SpaceTint; 9] = [
        SpaceTint::Cobalt,
        SpaceTint::Violet,
        SpaceTint::Fuchsia,
        SpaceTint::Amber,
        SpaceTint::Terracotta,
        SpaceTint::Slate,
        SpaceTint::Pine,
        SpaceTint::Plum,
        SpaceTint::Ink,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Cobalt => "cobalt",
            Self::Violet => "violet",
            Self::Fuchsia => "fuchsia",
            Self::Amber => "amber",
            Self::Terracotta => "terracotta",
            Self::Slate => "slate",
            Self::Pine => "pine",
            Self::Plum => "plum",
            Self::Ink => "ink",
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
            "pine" => Some(Self::Pine),
            "plum" => Some(Self::Plum),
            "ink" => Some(Self::Ink),
            _ => None,
        }
    }
}

/// A space's curated glyph identity (schema v2).
///
/// Exactly one of an allowlisted emoji scalar or a closed BBR icon-glyph
/// name; the enum makes "emoji or glyph, never both, never neither" a
/// type-level invariant, matching the delivered portable space-appearance
/// contract's `SpaceIdentity`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpaceGlyph {
    /// A curated emoji scalar (member of [`ALLOWED_EMOJI`]).
    Emoji(String),
    /// A closed BBR icon-glyph name.
    Glyph(GlyphName),
}

/// Curated emoji allowlist for space identity (schema v2).
///
/// Persisted-contract mirror of the space-appearance contract's curated v1
/// emoji allowlist: every entry is exactly one Unicode scalar value chosen
/// to render legibly at footer-indicator sizes. Membership is the
/// validation rule; no free-form emoji, no ZWJ/skin-tone sequences.
pub(crate) const ALLOWED_EMOJI: &[&str] = &[
    "🔥", "🌊", "🧭", "🚀", "🧠", "🌙", "⚡", "⭐", "🎯", "📚", "🎧", "🌱", "🎨", "🌐", "🏔", "💼",
    "🏠", "⚙",
];

/// A closed BBR-drawn icon glyph name used as a space identity (schema v2).
///
/// Stable names of the identity glyph catalog, mirroring the delivered
/// space-appearance contract's `BbrGlyph`; the actual stroke paths are
/// drawn by the shell's §7 icon module, never by persistence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphName {
    Code,
    Terminal,
    Folder,
    Person,
    Briefcase,
    Heart,
    Star,
    Leaf,
    Compass,
    Moon,
    Book,
    Camera,
    Palette,
    Globe,
}

impl GlyphName {
    /// All identity glyphs in stable catalog order.
    pub const ALL: [GlyphName; 14] = [
        GlyphName::Code,
        GlyphName::Terminal,
        GlyphName::Folder,
        GlyphName::Person,
        GlyphName::Briefcase,
        GlyphName::Heart,
        GlyphName::Star,
        GlyphName::Leaf,
        GlyphName::Compass,
        GlyphName::Moon,
        GlyphName::Book,
        GlyphName::Camera,
        GlyphName::Palette,
        GlyphName::Globe,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Terminal => "terminal",
            Self::Folder => "folder",
            Self::Person => "person",
            Self::Briefcase => "briefcase",
            Self::Heart => "heart",
            Self::Star => "star",
            Self::Leaf => "leaf",
            Self::Compass => "compass",
            Self::Moon => "moon",
            Self::Book => "book",
            Self::Camera => "camera",
            Self::Palette => "palette",
            Self::Globe => "globe",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "code" => Some(Self::Code),
            "terminal" => Some(Self::Terminal),
            "folder" => Some(Self::Folder),
            "person" => Some(Self::Person),
            "briefcase" => Some(Self::Briefcase),
            "heart" => Some(Self::Heart),
            "star" => Some(Self::Star),
            "leaf" => Some(Self::Leaf),
            "compass" => Some(Self::Compass),
            "moon" => Some(Self::Moon),
            "book" => Some(Self::Book),
            "camera" => Some(Self::Camera),
            "palette" => Some(Self::Palette),
            "globe" => Some(Self::Globe),
            _ => None,
        }
    }
}

/// The single global settings record (schema v2).
///
/// Carries the search engine choice and, for the custom engine, a validated
/// search URL template with exactly one `%s` placeholder. The template must
/// be present exactly when the engine is [`SearchEngineId::Custom`]; the
/// write path enforces that invariant and the restore path falls back to
/// defaults with a typed warning when a persisted record is incoherent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchSettings {
    pub engine: SearchEngineId,
    /// Validated `%s` search template; `Some` only for [`SearchEngineId::Custom`].
    pub custom_template: Option<String>,
}

impl Default for SearchSettings {
    /// The v1 default: DuckDuckGo, no custom template — byte-identical to
    /// today's hardcoded search fallback.
    fn default() -> Self {
        Self {
            engine: SearchEngineId::DuckDuckGo,
            custom_template: None,
        }
    }
}

/// A closed search engine id (schema v2).
///
/// Persisted-contract mirror of the delivered portable search-settings
/// contract's engine set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchEngineId {
    DuckDuckGo,
    Google,
    Bing,
    Ecosia,
    Kagi,
    Custom,
}

impl SearchEngineId {
    /// All engine ids in canonical settings order.
    pub const ALL: [SearchEngineId; 6] = [
        SearchEngineId::DuckDuckGo,
        SearchEngineId::Google,
        SearchEngineId::Bing,
        SearchEngineId::Ecosia,
        SearchEngineId::Kagi,
        SearchEngineId::Custom,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DuckDuckGo => "duckduckgo",
            Self::Google => "google",
            Self::Bing => "bing",
            Self::Ecosia => "ecosia",
            Self::Kagi => "kagi",
            Self::Custom => "custom",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "duckduckgo" => Some(Self::DuckDuckGo),
            "google" => Some(Self::Google),
            "bing" => Some(Self::Bing),
            "ecosia" => Some(Self::Ecosia),
            "kagi" => Some(Self::Kagi),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

/// One terminal download attempt surviving a restart (schema v2).
///
/// Only terminal attempts are persisted: the downloads contract's
/// `Succeeded`/`Failed`/`Cancelled` states are immutable end states, so the
/// history record captures the attempt identity, its terminal outcome, the
/// received size at the terminal event, the resolved destination (absent
/// when the attempt ended before a destination was decided), and the
/// deterministic monotonic terminal timestamp from the controller clock.
///
/// The settled post-MVP persistence decisions (DESIGN.md §17.10, §17.7,
/// §17.8) extend the terminal record with the facts a restored row needs:
/// the source `url` and display `filename` (so a restored Cancelled/Failed
/// row can Restart/Retry as a fresh attempt linked via `retry_of`), the
/// previous-attempt soft reference, and the 1-based `attempt` count. The
/// destination decision itself is not persisted: v1 uses the settled silent
/// suggested-destination policy, so a restored row re-runs the standard
/// suggested folder. `retry_of` is a soft reference (no foreign key):
/// deleting one terminal row leaves a child's link dangling rather than
/// cascading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadRecord {
    /// The attempt identity (`DownloadId`).
    pub id: u64,
    /// The terminal outcome of the attempt.
    pub outcome: DownloadOutcome,
    /// Received bytes at the terminal event (0 for a never-started attempt).
    pub size: u64,
    /// Resolved destination path; `None` when the attempt ended before a
    /// destination was decided.
    pub destination: Option<String>,
    /// Deterministic monotonic terminal timestamp (controller clock).
    pub terminal_at: u64,
    /// The source URL — required to Restart/Retry a restored row.
    pub url: String,
    /// The display filename (the panel's name line).
    pub filename: String,
    /// The previous attempt this row retries/restarts, when applicable
    /// (soft reference; may dangle after that row is deleted).
    pub retry_of: Option<u64>,
    /// 1-based attempt count (increments per retry/restart).
    pub attempt: u32,
}

/// The terminal outcome of a download attempt (schema v2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadOutcome {
    Succeeded,
    Failed,
    Cancelled,
}

impl DownloadOutcome {
    pub const ALL: [DownloadOutcome; 3] = [
        DownloadOutcome::Succeeded,
        DownloadOutcome::Failed,
        DownloadOutcome::Cancelled,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// One visited command-history entry (schema v2).
///
/// The persisted shape mirrors the provider-neutral command corpus's
/// `HistoryEntry`: a stable identity, display title (may be empty), the
/// normalized visited URL, the visit count, and a monotonic recency marker
/// where larger is more recent. That is the full input shape a later ranked
/// history provider needs; ranking itself stays `bbr-command`-owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandHistoryEntry {
    /// Stable history-record identity (`HistoryId`).
    pub id: u64,
    /// Display title; may be empty.
    pub title: String,
    /// Normalized URL of the visited page.
    pub url: String,
    /// Number of recorded visits (secondary ranking key).
    pub visit_count: u64,
    /// Monotonic recency marker; larger is more recent.
    pub last_visited: u64,
}

/// How the persisted store was treated during a load.
///
/// A discarded store is a normal, logged, non-fatal startup path (schema v2
/// product decision: no migrations, no backward compatibility — in
/// development, a store whose version stamp does not match the current
/// schema is deleted and recreated empty rather than loaded partially or
/// migrated). The caller logs the [`StoreDiscardReason`] and continues with
/// the empty snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreLoad {
    /// The store's version stamp matched the current schema (or the store
    /// was a fresh empty database) and it was loaded.
    Loaded,
    /// The store was discarded and recreated empty at the current schema.
    Discarded { reason: StoreDiscardReason },
}

/// Why a store was discarded and recreated empty (schema v2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreDiscardReason {
    /// The store carried no version stamp (`user_version = 0`) but contained
    /// data: a legacy pre-v2 store.
    Unstamped,
    /// The store carried a different schema version stamp.
    VersionMismatch { found: u32 },
    /// The stamp matched but the store failed structural validation
    /// (missing/misshaped tables or foreign-key violations).
    Corrupt { detail: String },
}
