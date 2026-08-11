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
    /// The §11.8a unresponsive pane: the active tab's renderer stopped
    /// answering liveness probes (the session and tab stay alive under the
    /// closed recovery set Wait / Reload / Close tab).
    Unresponsive,
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
    /// The §11.16 Saved Pages Library surface (B3 packet
    /// `saved-pages-library`): the internal chrome surface covers the
    /// content area while open — the active view's native slot is hidden
    /// (never destroyed) and the Library wins over the error surface. The
    /// node mirrors the `bbr-ui` surface anatomy from the same model the
    /// render draws; `focused` is the Library surface's real GPUI focus
    /// (the `library_handle`), so the search field (or the staged rename
    /// input) mirrors the true focus state.
    Library {
        model: bbr_ui::components::SavedPagesLibraryModel,
        focused: bool,
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
        SemanticContent::Unresponsive => node(
            "bbr-unresponsive-panel",
            SemanticRole::Content,
            Some("This page is not responding".into()),
            None,
            false,
            false,
            vec![
                node(
                    "bbr-unresponsive-wait",
                    SemanticRole::Button,
                    Some("Wait".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    "bbr-unresponsive-reload",
                    SemanticRole::Button,
                    Some("Reload".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    "bbr-unresponsive-close",
                    SemanticRole::Button,
                    Some("Close tab".into()),
                    None,
                    false,
                    false,
                    vec![],
                ),
            ],
        ),
        SemanticContent::Library { model, focused } => {
            super::library::library_node(model, *focused)
        }
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
