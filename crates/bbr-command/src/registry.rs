//! Browser command registry: typed actions with deterministic ordering.
//!
//! The registry is the portable catalog of browser commands the command bar
//! can run. It carries display titles, match keywords, a category, and a
//! shortcut hint; the ranker matches against title and keywords, and the
//! later app-composition slice maps [`ActionId`]s onto real controller
//! calls. The registry never executes anything itself.

use crate::id::ActionId;

/// Coarse grouping of browser commands for later surface organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    /// Tab lifecycle and organization commands.
    Tab,
    /// Space lifecycle commands.
    Space,
    /// Back/forward/reload navigation commands.
    Navigation,
    /// Window/chrome view commands.
    View,
}

/// One registered browser command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// Stable action identity.
    pub id: ActionId,
    /// Human-readable title (also the primary match surface).
    pub title: String,
    /// Additional match keywords.
    pub keywords: Vec<String>,
    /// Coarse category.
    pub category: ActionCategory,
    /// Display shortcut hint (informational only; the app owns keybinding).
    pub shortcut: Option<&'static str>,
    keywords_lower: Vec<String>,
}

impl Action {
    /// Builds a registered command, precomputing the lowercase match surface
    /// (title first, then the keywords).
    pub fn new(
        id: ActionId,
        title: impl Into<String>,
        keywords: impl IntoIterator<Item = impl Into<String>>,
        category: ActionCategory,
        shortcut: Option<&'static str>,
    ) -> Self {
        let title = title.into();
        let keywords: Vec<String> = keywords.into_iter().map(Into::into).collect();
        let mut keywords_lower = Vec::with_capacity(keywords.len() + 1);
        keywords_lower.push(title.to_lowercase());
        keywords_lower.extend(keywords.iter().map(|k| k.to_lowercase()));
        Action {
            id,
            title,
            keywords,
            category,
            shortcut,
            keywords_lower,
        }
    }

    /// Precomputed lowercase match surface (title + keywords).
    pub fn keywords_lower(&self) -> &[String] {
        &self.keywords_lower
    }
}

/// The ordered action catalog.
///
/// Insertion order is the deterministic tie-break when two actions match
/// equally well, so a built-in catalog must list actions in a stable order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandRegistry {
    actions: Vec<Action>,
}

impl CommandRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// The catalog of commands the current product contract defines.
    ///
    /// Order matters: it is the deterministic tie-break between equal
    /// matches. The set mirrors the existing commands in the browser
    /// interaction contract.
    pub fn builtin() -> Self {
        let actions = vec![
            Action::new(
                ActionId(1),
                "New Tab",
                ["new", "tab"],
                ActionCategory::Tab,
                Some("⌘T"),
            ),
            Action::new(
                ActionId(2),
                "Close Tab",
                ["close", "tab"],
                ActionCategory::Tab,
                Some("⌘W"),
            ),
            Action::new(
                ActionId(3),
                "Reopen Closed Tab",
                ["reopen", "closed", "tab", "undo"],
                ActionCategory::Tab,
                Some("⇧⌘T"),
            ),
            Action::new(
                ActionId(4),
                "Pin or Unpin Tab",
                ["pin", "unpin", "tab"],
                ActionCategory::Tab,
                Some("⇧⌘P"),
            ),
            Action::new(
                ActionId(5),
                "Move Tab to Space",
                ["move", "tab", "space"],
                ActionCategory::Tab,
                Some("⇧⌘M"),
            ),
            Action::new(
                ActionId(6),
                "New Space",
                ["new", "space"],
                ActionCategory::Space,
                Some("⇧⌘N"),
            ),
            Action::new(
                ActionId(7),
                "Back",
                ["back", "previous"],
                ActionCategory::Navigation,
                Some("⌘["),
            ),
            Action::new(
                ActionId(8),
                "Forward",
                ["forward", "next"],
                ActionCategory::Navigation,
                Some("⌘]"),
            ),
            Action::new(
                ActionId(9),
                "Reload",
                ["reload", "refresh"],
                ActionCategory::Navigation,
                Some("⌘R"),
            ),
            Action::new(
                ActionId(10),
                "Stop",
                ["stop"],
                ActionCategory::Navigation,
                None,
            ),
            Action::new(
                ActionId(11),
                "Toggle Sidebar",
                ["sidebar", "toggle", "collapse", "expand"],
                ActionCategory::View,
                Some("⌘S"),
            ),
            Action::new(
                ActionId(12),
                "Reorder Tab Up",
                ["reorder", "move", "tab", "up"],
                ActionCategory::Tab,
                Some("⇧⌘↑"),
            ),
            Action::new(
                ActionId(13),
                "Reorder Tab Down",
                ["reorder", "move", "tab", "down"],
                ActionCategory::Tab,
                Some("⇧⌘↓"),
            ),
            Action::new(
                ActionId(14),
                "Set as Default Browser",
                ["default", "browser", "handler", "open", "links"],
                ActionCategory::View,
                None,
            ),
            // Blueprint D06: a profile can be created from the command bar.
            Action::new(
                ActionId(15),
                "New Profile…",
                ["new", "profile", "create", "account", "separate"],
                ActionCategory::Space,
                None,
            ),
            // Blueprint D16: the Settings window.
            Action::new(
                ActionId(16),
                "Settings…",
                ["settings", "preferences", "options"],
                ActionCategory::View,
                Some("⌘,"),
            ),
            // Blueprint D10: History and Clear Browsing Data.
            Action::new(
                ActionId(17),
                "Show All History",
                ["history", "visited", "recently", "closed", "past"],
                ActionCategory::View,
                Some("⌘Y"),
            ),
            Action::new(
                ActionId(18),
                "Clear Browsing Data…",
                [
                    "clear", "delete", "browsing", "data", "cookies", "cache", "privacy",
                ],
                ActionCategory::View,
                Some("⇧⌘⌫"),
            ),
            // Blueprint D11: the active site's settings.
            Action::new(
                ActionId(19),
                "Site Settings…",
                [
                    "site",
                    "settings",
                    "permissions",
                    "camera",
                    "microphone",
                    "cookies",
                ],
                ActionCategory::View,
                None,
            ),
            // Blueprint D19: the Tab menu's tab commands.
            Action::new(
                ActionId(20),
                "Duplicate Tab",
                ["duplicate", "copy", "clone", "tab"],
                ActionCategory::Tab,
                None,
            ),
            Action::new(
                ActionId(21),
                "Select Next Tab",
                ["next", "tab", "switch", "cycle"],
                ActionCategory::Tab,
                Some("⌃⇥"),
            ),
            Action::new(
                ActionId(22),
                "Select Previous Tab",
                ["previous", "tab", "switch", "cycle"],
                ActionCategory::Tab,
                Some("⌃⇧⇥"),
            ),
            // Blueprint D23: split commands.
            Action::new(
                ActionId(23),
                "Split Right",
                ["split", "side", "right", "pane", "tile"],
                ActionCategory::Tab,
                Some("⌃⌘S"),
            ),
            Action::new(
                ActionId(24),
                "Split Below",
                ["split", "stack", "below", "pane", "tile"],
                ActionCategory::Tab,
                Some("⌃⌥⌘S"),
            ),
            Action::new(
                ActionId(25),
                "Close Split",
                ["close", "split", "dissolve", "unsplit", "panes"],
                ActionCategory::Tab,
                Some("⇧⌃⌘S"),
            ),
        ];
        CommandRegistry { actions }
    }

    /// The actions in registry (insertion) order.
    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        self.actions.iter()
    }

    /// Looks up one action by identity.
    pub fn get(&self, id: ActionId) -> Option<&Action> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// Number of registered actions.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// True when no action is registered.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_catalog_matches_the_interaction_contract() {
        let expected = [
            (1, "New Tab", ActionCategory::Tab, Some("⌘T")),
            (2, "Close Tab", ActionCategory::Tab, Some("⌘W")),
            (3, "Reopen Closed Tab", ActionCategory::Tab, Some("⇧⌘T")),
            (4, "Pin or Unpin Tab", ActionCategory::Tab, Some("⇧⌘P")),
            (5, "Move Tab to Space", ActionCategory::Tab, Some("⇧⌘M")),
            (6, "New Space", ActionCategory::Space, Some("⇧⌘N")),
            (7, "Back", ActionCategory::Navigation, Some("⌘[")),
            (8, "Forward", ActionCategory::Navigation, Some("⌘]")),
            (9, "Reload", ActionCategory::Navigation, Some("⌘R")),
            (10, "Stop", ActionCategory::Navigation, None),
            (11, "Toggle Sidebar", ActionCategory::View, Some("⌘S")),
            (12, "Reorder Tab Up", ActionCategory::Tab, Some("⇧⌘↑")),
            (13, "Reorder Tab Down", ActionCategory::Tab, Some("⇧⌘↓")),
            (14, "Set as Default Browser", ActionCategory::View, None),
            (15, "New Profile…", ActionCategory::Space, None),
            (16, "Settings…", ActionCategory::View, Some("⌘,")),
            (17, "Show All History", ActionCategory::View, Some("⌘Y")),
            (
                18,
                "Clear Browsing Data…",
                ActionCategory::View,
                Some("⇧⌘⌫"),
            ),
            (19, "Site Settings…", ActionCategory::View, None),
            (20, "Duplicate Tab", ActionCategory::Tab, None),
            (21, "Select Next Tab", ActionCategory::Tab, Some("⌃⇥")),
            (22, "Select Previous Tab", ActionCategory::Tab, Some("⌃⇧⇥")),
            (23, "Split Right", ActionCategory::Tab, Some("⌃⌘S")),
            (24, "Split Below", ActionCategory::Tab, Some("⌃⌥⌘S")),
            (25, "Close Split", ActionCategory::Tab, Some("⇧⌃⌘S")),
        ];
        let registry = CommandRegistry::builtin();
        let actual: Vec<_> = registry
            .actions()
            .map(|action| {
                (
                    action.id.0,
                    action.title.as_str(),
                    action.category,
                    action.shortcut,
                )
            })
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_finds_registered_action() {
        let registry = CommandRegistry::builtin();
        let reload = registry.get(ActionId(9)).expect("reload action");
        assert_eq!(reload.title, "Reload");
        assert_eq!(reload.category, ActionCategory::Navigation);
        assert_eq!(reload.shortcut, Some("⌘R"));
        assert!(registry.get(ActionId(999)).is_none());
    }

    #[test]
    fn keywords_lower_prepends_title() {
        let registry = CommandRegistry::builtin();
        let action = registry.get(ActionId(11)).expect("sidebar");
        assert_eq!(
            action.keywords_lower(),
            &["toggle sidebar", "sidebar", "toggle", "collapse", "expand"]
        );
    }
}
