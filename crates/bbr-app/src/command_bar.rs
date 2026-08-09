//! Command-bar modes and restoration state (DESIGN.md §11.4/§11.5).
//!
//! The sidebar command slot owns two presentations of the same §11.5 engine
//! (the centered Cmd-T palette and the field-anchored Cmd-L/click overlay);
//! the sidebar address control itself is a display-only pill (§11.5) that
//! shows the derived domain. This module owns the mode enum and the
//! Escape-restoration state. The pure display derivation the pill renders
//! (scheme/path/query dropping, `www.` elision, IDN decoding, port
//! handling, internal-page placeholders, the mid-ellipsis budget, and the
//! reserved accessory-slot width) moved to `bbr-ui::text` in the G1a
//! cutover; the app imports it from there and never redefines it.

/// The command bar either edits the current tab or stages a future tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBarMode {
    /// Browser chrome may be focused, but the command bar is not editing.
    Inactive,
    /// `Cmd-L`: edit or replace the active tab's location.
    Location,
    /// `Cmd-T`: collect a destination without creating a tab or web session.
    NewTab,
    /// `Cmd-Shift-N`: collect a new space name without creating anything
    /// until Enter commits it (the pointer half is the New Space row).
    NewSpace,
    /// `Cmd-Shift-M`: collect a destination space name for moving the active
    /// tab; Enter commits the move, Escape cancels it.
    MoveTab,
}

impl CommandBarMode {
    pub fn is_active(self) -> bool {
        !matches!(self, Self::Inactive)
    }
}

#[cfg(feature = "testkit")]
impl From<CommandBarMode> for bbr_testkit::protocol::CommandBarMode {
    fn from(mode: CommandBarMode) -> Self {
        match mode {
            CommandBarMode::Inactive => Self::Inactive,
            CommandBarMode::Location => Self::Location,
            CommandBarMode::NewTab => Self::NewTab,
            CommandBarMode::NewSpace => Self::NewSpace,
            CommandBarMode::MoveTab => Self::MoveTab,
        }
    }
}

/// State restored when a staged new-tab command is cancelled.
#[derive(Debug, Clone)]
pub struct CommandBarReturn {
    pub mode: CommandBarMode,
    pub text: String,
    pub chrome_focused: bool,
}
