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
                None,
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
                Some("⎋"),
            ),
            Action::new(
                ActionId(11),
                "Toggle Sidebar",
                ["sidebar", "toggle", "collapse", "expand"],
                ActionCategory::View,
                Some("⌘B"),
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
    fn builtin_catalog_is_ordered_and_unique() {
        let registry = CommandRegistry::builtin();
        let mut seen = std::collections::HashSet::new();
        let mut last = 0;
        for action in registry.actions() {
            assert!(
                seen.insert(action.id),
                "duplicate action id {:?}",
                action.id
            );
            assert!(action.id.0 > last, "ids must be strictly increasing");
            last = action.id.0;
        }
        assert_eq!(registry.len(), 11);
        assert!(!registry.is_empty());
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
