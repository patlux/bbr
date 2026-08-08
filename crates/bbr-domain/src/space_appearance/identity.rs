//! Validated space identity: one curated emoji **or** one BBR icon glyph.

use crate::AppearanceError;

/// Curated v1 emoji allowlist for space identity (product spec §1).
///
/// Every entry is exactly one Unicode scalar value (a single Rust `char`),
/// chosen to render legibly at the small footer-indicator sizes of the
/// sidebar. Membership is the validation rule: v1 accepts no free-form
/// emoji, no normalization, no grapheme-segmentation dependency, and no
/// ZWJ/skin-tone sequences. Extending this list is the explicit, reviewed
/// way to add emoji identity options.
pub const ALLOWED_EMOJI: &[&str] = &[
    "🔥", "🌊", "🧭", "🚀", "🧠", "🌙", "⚡", "⭐", "🎯", "📚", "🎧", "🌱", "🎨", "🌐", "🏔", "💼",
    "🏠", "⚙",
];

/// A validated emoji identity: exactly one curated Unicode scalar value.
///
/// Construction is the only path into a `SpaceEmoji`; the inner string is
/// private, so an invalid or uncurated emoji cannot be forged through the
/// public API (type-level validity).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceEmoji(String);

impl SpaceEmoji {
    /// Validate `s` against the curated [`ALLOWED_EMOJI`] allowlist.
    ///
    /// Rejects the input **whole** with [`AppearanceError::InvalidEmoji`]
    /// when it is not an exact canonical member; there is no partial state.
    pub fn new(s: impl AsRef<str>) -> Result<Self, AppearanceError> {
        let s = s.as_ref();
        if ALLOWED_EMOJI.contains(&s) {
            Ok(Self(s.to_owned()))
        } else {
            Err(AppearanceError::InvalidEmoji(s.to_owned()))
        }
    }

    /// The canonical emoji scalar (always a member of [`ALLOWED_EMOJI`]).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SpaceEmoji {
    /// Deterministic default: the first curated emoji.
    fn default() -> Self {
        Self(ALLOWED_EMOJI[0].to_owned())
    }
}

impl std::fmt::Display for SpaceEmoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One BBR-drawn icon glyph (DESIGN §7 geometric family) used as a space
/// identity.
///
/// These are stable **names** of the identity glyph catalog; the actual
/// 1.5 pt stroke paths are drawn by the shell's §7 icon module (the
/// icon-system slice), never by the domain. The catalog is curated for
/// identity semantics and deliberately disjoint from profile-hue names and
/// status/accent families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BbrGlyph {
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

impl BbrGlyph {
    /// All identity glyphs in stable catalog order.
    pub const ALL: [BbrGlyph; 14] = [
        BbrGlyph::Code,
        BbrGlyph::Terminal,
        BbrGlyph::Folder,
        BbrGlyph::Person,
        BbrGlyph::Briefcase,
        BbrGlyph::Heart,
        BbrGlyph::Star,
        BbrGlyph::Leaf,
        BbrGlyph::Compass,
        BbrGlyph::Moon,
        BbrGlyph::Book,
        BbrGlyph::Camera,
        BbrGlyph::Palette,
        BbrGlyph::Globe,
    ];

    /// The canonical machine-readable name used by snapshots and E2E
    /// assertions (the same convention as [`ProfileHue::name`](crate::ProfileHue::name)).
    pub fn name(self) -> &'static str {
        match self {
            BbrGlyph::Code => "code",
            BbrGlyph::Terminal => "terminal",
            BbrGlyph::Folder => "folder",
            BbrGlyph::Person => "person",
            BbrGlyph::Briefcase => "briefcase",
            BbrGlyph::Heart => "heart",
            BbrGlyph::Star => "star",
            BbrGlyph::Leaf => "leaf",
            BbrGlyph::Compass => "compass",
            BbrGlyph::Moon => "moon",
            BbrGlyph::Book => "book",
            BbrGlyph::Camera => "camera",
            BbrGlyph::Palette => "palette",
            BbrGlyph::Globe => "globe",
        }
    }

    /// A short human-readable display label for pickers and settings.
    pub fn label(self) -> &'static str {
        match self {
            BbrGlyph::Code => "Code",
            BbrGlyph::Terminal => "Terminal",
            BbrGlyph::Folder => "Folder",
            BbrGlyph::Person => "Person",
            BbrGlyph::Briefcase => "Briefcase",
            BbrGlyph::Heart => "Heart",
            BbrGlyph::Star => "Star",
            BbrGlyph::Leaf => "Leaf",
            BbrGlyph::Compass => "Compass",
            BbrGlyph::Moon => "Moon",
            BbrGlyph::Book => "Book",
            BbrGlyph::Camera => "Camera",
            BbrGlyph::Palette => "Palette",
            BbrGlyph::Globe => "Globe",
        }
    }
}

impl Default for BbrGlyph {
    /// Deterministic default: the first glyph in catalog order.
    fn default() -> Self {
        BbrGlyph::Code
    }
}

/// Discriminant of a [`SpaceIdentity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityKind {
    Emoji,
    Glyph,
}

/// A space's visual identity: exactly one of a validated emoji or a BBR
/// icon glyph.
///
/// The enum shape makes "emoji **or** glyph, never both, never neither" a
/// type-level invariant: every value is one of the two variants and both
/// payloads are closed, validated types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpaceIdentity {
    Emoji(SpaceEmoji),
    Glyph(BbrGlyph),
}

impl SpaceIdentity {
    /// Stable machine-readable identity name for snapshots and E2E
    /// assertions: the emoji scalar itself, or the glyph name.
    pub fn name(&self) -> String {
        match self {
            SpaceIdentity::Emoji(emoji) => emoji.as_str().to_owned(),
            SpaceIdentity::Glyph(glyph) => glyph.name().to_owned(),
        }
    }

    /// The identity discriminant (for exhaustive UI branches such as the
    /// settings picker's emoji/glyph tabs).
    pub fn kind(&self) -> IdentityKind {
        match self {
            SpaceIdentity::Emoji(_) => IdentityKind::Emoji,
            SpaceIdentity::Glyph(_) => IdentityKind::Glyph,
        }
    }
}

impl Default for SpaceIdentity {
    /// Deterministic default: the first curated BBR glyph.
    fn default() -> Self {
        SpaceIdentity::Glyph(BbrGlyph::default())
    }
}

impl std::fmt::Display for SpaceIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}
