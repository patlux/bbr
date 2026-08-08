//! The content-pane semantic branch (DESIGN.md §15): the typed node for the
//! native web slot, the empty new-tab surface, and the crash panel — the
//! mirror of the content region `Render::render` builds. Kept in its own
//! module so the tree composition in `super` stays within its size budget.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};

use super::node;

/// The content-pane branch of the semantic tree: the native web slot, the
/// empty new-tab surface, the §11.8 crash panel, or the §11.8-style error
/// surface (task packet `error-surface-contract`) — mirroring the same
/// `SemanticContent` the renderer selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticContent {
    Native,
    Empty {
        profile_line: String,
    },
    Crash,
    /// The classified incident surface (TLS, navigation, offline): the
    /// calm title, the failing origin (when the contract knows one), the
    /// copy, and the contract-legal primary action plus Dismiss.
    Error {
        title: String,
        origin: Option<String>,
        copy: String,
        action: String,
        dismiss: String,
    },
}

/// The typed content-pane node.
pub fn content_node(content: &SemanticContent) -> SemanticNode {
    match content {
        SemanticContent::Native => node(
            "bbr-native-web-slot",
            SemanticRole::Content,
            Some("Web content".into()),
            None,
            false,
            false,
            vec![],
        ),
        SemanticContent::Empty { profile_line } => {
            let mut children = vec![node(
                "bbr-empty-new-tab-button",
                SemanticRole::Button,
                Some("New Tab  ⌘T".into()),
                None,
                false,
                false,
                vec![],
            )];
            let mut pane = node(
                "bbr-empty-new-tab",
                SemanticRole::Content,
                Some("No tabs in this space".into()),
                Some(profile_line.clone()),
                false,
                false,
                vec![],
            );
            pane.children.append(&mut children);
            pane
        }
        SemanticContent::Crash => node(
            "bbr-crash-panel",
            SemanticRole::Content,
            Some("This page crashed".into()),
            None,
            false,
            false,
            vec![
                node(
                    "bbr-crash-reload",
                    SemanticRole::Button,
                    Some("Reload".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    "bbr-crash-close",
                    SemanticRole::Button,
                    Some("Close tab".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
            ],
        ),
        SemanticContent::Error {
            title,
            origin,
            copy,
            action,
            dismiss,
        } => {
            let mut children = vec![
                node(
                    "bbr-error-title",
                    SemanticRole::Title,
                    Some(title.clone()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    "bbr-error-copy",
                    SemanticRole::State,
                    Some(copy.clone()),
                    None,
                    false,
                    false,
                    vec![],
                ),
            ];
            if let Some(origin) = origin {
                children.push(node(
                    "bbr-error-url",
                    SemanticRole::State,
                    Some(origin.clone()),
                    None,
                    false,
                    false,
                    vec![],
                ));
            }
            children.push(node(
                "bbr-error-action",
                SemanticRole::Button,
                Some(action.clone()),
                None,
                false,
                false,
                vec![],
            ));
            children.push(node(
                "bbr-error-dismiss",
                SemanticRole::Button,
                Some(dismiss.clone()),
                None,
                false,
                false,
                vec![],
            ));
            node(
                "bbr-error-panel",
                SemanticRole::Content,
                Some(title.clone()),
                None,
                false,
                false,
                children,
            )
        }
    }
}
