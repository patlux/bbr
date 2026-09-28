//! Plain command-presentation data (DESIGN.md §11.4/§11.5/§13/§15).
//!
//! This module owns **only plain presentation data**: the UI-local
//! presentation discriminant (closed / centered palette / anchored
//! overlay), the theme, the chrome-focused flag, the text, the UI-local
//! result-state enum, the selected index, and the plain rows carrying the
//! rendered stable key / group / title / subtitle / destructive fields.
//! The app maps its app-owned `CommandSurface`, `CommandResultState`, and
//! `CommandResultRow` (including `Activation`) onto this model at render
//! time (`app/palette.rs`); ranking, activation, focus, and Portal
//! lifecycle never enter this crate.
//!
//! The model also owns the validation needed for the rejected combinations
//! (the Gallery matrix contract): at most one presentation discriminant
//! (mutual exclusion by construction — a raw value outside the three valid
//! discriminants is rejected) and no result row with a missing/empty
//! stable key.

use crate::theme::Theme;

/// The UI-local presentation discriminant of the floating command surface
/// (DESIGN.md §11.4). At most one presentation is open at a time; `Closed`
/// when the in-rail command bar is the active surface (or nothing is).
///
/// The valid raw values are pinned: `0` = closed, `1` = centered palette,
/// `2` = anchored overlay. `from_raw` rejects any other value, so the
/// "centered + anchored simultaneously" combination is impossible to
/// construct and any invalid discriminant is refused — the mutual
/// exclusion the packet's rejected-family row requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandPresentation {
    /// No palette/overlay presentation is open.
    #[default]
    Closed,
    /// The centered Cmd-T palette (`bbr-palette`) in New Tab staging mode.
    CenteredPalette,
    /// The field-anchored Cmd-L / field-click overlay
    /// (`bbr-command-overlay`) in Location mode.
    AnchoredOverlay,
}

/// Why a [`CommandModel`] is rejected (the matrix contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandModelError {
    /// A result row carries a missing or empty stable key.
    MissingRowKey,
    /// The presentation discriminant is invalid (no such presentation
    /// exists — including any hypothetical "centered + anchored
    /// simultaneously" raw value).
    InvalidPresentation,
}

impl CommandPresentation {
    /// The raw discriminant value (`0` closed / `1` palette / `2` overlay).
    pub const fn raw(self) -> u8 {
        match self {
            CommandPresentation::Closed => 0,
            CommandPresentation::CenteredPalette => 1,
            CommandPresentation::AnchoredOverlay => 2,
        }
    }

    /// Construct from a raw discriminant; rejects any value outside the
    /// three valid presentations (the mutual-exclusion contract).
    pub fn from_raw(raw: u8) -> Result<Self, CommandModelError> {
        match raw {
            0 => Ok(CommandPresentation::Closed),
            1 => Ok(CommandPresentation::CenteredPalette),
            2 => Ok(CommandPresentation::AnchoredOverlay),
            _ => Err(CommandModelError::InvalidPresentation),
        }
    }

    /// True while a palette/overlay presentation is open.
    pub const fn is_open(self) -> bool {
        !matches!(self, Self::Closed)
    }
}

/// The typed state of the command result surface (DESIGN.md §11.5),
/// UI-local: the app's ranking state machine maps onto this enum; no
/// ranking internals or generation live here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandResultState {
    /// The command bar is not ranking (no location/new-tab surface).
    #[default]
    Inactive,
    /// The ranking surface is open with no text to rank.
    Empty,
    /// A debounce generation is pending; ranking has not published yet.
    Loading,
    /// The latest published generation produced ranked results.
    Results,
    /// The last activation was rejected (unavailable destination).
    ActivationError,
}

/// One plain rendered result row: only the fields the renderer actually
/// presents (key / group / title / subtitle / destructive). The typed
/// `Activation`, the collection label (not rendered by the floating
/// surface), and every ranking/internal field stay app-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResultRow {
    /// The stable external key (`tab:<id>`, `space:<id>`, `action:<id>`,
    /// `history:<id>`, `saved_page:<id>`, or the fallback key) — the
    /// render id suffix `bbr-palette-result-<key>` /
    /// `bbr-command-result-<key>`. Never empty.
    pub key: String,
    /// The §11.5 group name (`exact_tab`, `current_pinned`,
    /// `current_tab`, `other_space`, `space`, `saved_page`, `history`,
    /// `action`, `fallback`) driving the group glyph.
    pub group: String,
    /// The row title (truncated at 140 pt, `type:title`).
    pub title: String,
    /// The optional mono subtitle (the URL / secondary line).
    pub subtitle: Option<String>,
    /// The row's destination URL (shown shortened in the mono column when
    /// no subtitle — a cross-space identity or a shortcut — takes it).
    pub url: Option<String>,
    /// Whether the row uses the error/destructive tint (the settled §11.5
    /// destructive set; no confirmation dialog is invented).
    pub destructive: bool,
}

/// The plain presentation model the floating command renderer consumes:
/// one presentation discriminant, the theme, the chrome-focus flag, the
/// field text, the UI-local result state, the plain rows, and the selected
/// index. Built by the app at render time; validated by
/// [`CommandModel::validate`].
#[derive(Debug, Clone)]
pub struct CommandModel {
    pub presentation: CommandPresentation,
    pub theme: Theme,
    pub chrome_focused: bool,
    pub text: String,
    pub state: CommandResultState,
    pub results: Vec<CommandResultRow>,
    pub selected: usize,
    /// §18.5 ⌘T: the key of the result-entrance moment while the freshly
    /// published rows fade in (staggered); `None` renders them at rest.
    pub entrance: Option<u64>,
    /// §18.5 ⌘T: `(previous row, moment key)` while the cursor highlight
    /// glides to `selected` (the selection itself is already committed).
    pub glide: Option<(usize, u64)>,
}

impl CommandModel {
    /// Validate the model's render contract: reject any result row with a
    /// missing/empty stable key. The presentation mutual exclusion is
    /// enforced by construction (one discriminant field).
    pub fn validate(&self) -> Result<(), CommandModelError> {
        if self.results.iter().any(|row| row.key.trim().is_empty()) {
            Err(CommandModelError::MissingRowKey)
        } else {
            Ok(())
        }
    }
}
