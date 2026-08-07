//! The content-pane semantic branch (DESIGN.md §15): the typed node for the
//! native web slot, the empty new-tab surface, and the crash panel — the
//! mirror of the content region `Render::render` builds. Kept in its own
//! module so the tree composition in `super` stays within its size budget.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};

use super::node;

/// The content-pane branch of the semantic tree: the native web slot, the
/// empty new-tab surface, or the crash panel — mirroring the same
/// `SemanticContent` the renderer selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticContent {
    Native,
    Empty { profile_line: String },
    Crash,
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
    }
}
