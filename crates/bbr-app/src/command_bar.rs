//! Command-bar modes and restoration state.

/// The command bar either edits the current tab or stages a future tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBarMode {
    /// Browser chrome may be focused, but the command bar is not editing.
    Inactive,
    /// `Cmd-L`: edit or replace the active tab's location.
    Location,
    /// `Cmd-T`: collect a destination without creating a tab or web session.
    NewTab,
}

impl CommandBarMode {
    pub fn is_active(self) -> bool {
        self != Self::Inactive
    }
}

#[cfg(feature = "testkit")]
impl From<CommandBarMode> for bbr_testkit::protocol::CommandBarMode {
    fn from(mode: CommandBarMode) -> Self {
        match mode {
            CommandBarMode::Inactive => Self::Inactive,
            CommandBarMode::Location => Self::Location,
            CommandBarMode::NewTab => Self::NewTab,
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
