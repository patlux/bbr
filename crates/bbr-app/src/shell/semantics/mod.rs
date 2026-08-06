//! The typed semantic tree of the chrome (DESIGN.md §15 / accessibility
//! track).
//!
//! Every shell element exposes a stable `bbr-*` id; this module derives the
//! *typed* semantic model — roles, names, values, enabled/selected/focused
//! state, and the parent/child hierarchy — from the same structures the
//! render consumes (rows via [`crate::components::RowVisuals`], space rows
//! via [`crate::shell::space::space_groups`], the content pane mirroring
//! `Render::render`), so the E2E semantic assertions cannot drift from the
//! shell.
//!
//! **Scope honesty:** this is the internal semantic contract shared with the
//! testkit and the future platform Accessibility bridge. GPUI 0.2.2
//! implements no platform accessibility layer (verified in its source: no
//! NSAccessibility, no accesskit), so these nodes are **not** exposed to
//! macOS Accessibility / VoiceOver today. The platform bridge and the
//! packaged-app VoiceOver/Appium/manual acceptance are Phase 9; the model
//! itself ships with the shell so roles/names/values/enabled/selected/
//! hierarchy are contract from Phase 4.

use bbr_browser::ControllerSnapshot;
use bbr_testkit::protocol::{SemanticNode, SemanticRole};

use crate::components::{RowVisuals, SpaceRowVisuals, SpaceVisualState, TabVisualState};

/// Which GPUI chrome surface owns keyboard focus (§9: the focused control
/// renders the ring; the model exposes the same fact typed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticFocus {
    pub command: bool,
    pub spaces: bool,
    pub tabs: bool,
    pub divider: bool,
}

/// The §11.3 cross-context command-field hint: the destination profile of a
/// cross-profile space activation, shown for one interaction as the §12 chip
/// plus the "in Profile · <title>" label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProfileHint {
    /// The destination profile title (the label text after "in Profile · ").
    pub title: String,
    /// The §12 chip monogram (the non-color identity companion).
    pub monogram: String,
    /// The destination profile's creation-order index (the §15 chip id).
    pub profile_index: usize,
}

/// The command-field semantic content (mode, text, and the §11.3 destination
/// hint when a move is staged).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCommand {
    pub mode: String,
    pub text: String,
    /// The rendered destination hint ("→ Space" / "in Profile · P → Space").
    pub hint: Option<String>,
    /// The §11.3 cross-context transient (chip + "in Profile · <title>")
    /// shown for one interaction after a cross-profile space activation.
    pub profile_hint: Option<SemanticProfileHint>,
}

/// The glass content area: the native slot, the empty pane, or the crash
/// panel — mirroring the branch `Render::render` builds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticContent {
    Native,
    Empty { profile_line: String },
    Crash,
}

/// All inputs the semantic tree needs, derived by the app snapshot from the
/// same sources the render uses.
#[derive(Debug, Clone)]
pub struct SemanticInput<'a> {
    pub browser: &'a ControllerSnapshot,
    pub focus: SemanticFocus,
    pub command: SemanticCommand,
    pub sidebar_visible: bool,
    /// The effective rail width (0 when collapsed).
    pub sidebar_width: f32,
    pub divider_label: String,
    /// The rendered tab rows (pinned block + bounded regular window), with
    /// their composed visual states.
    pub rows: &'a [RowVisuals],
    /// The rendered space rows in render order (grouped by profile).
    pub space_rows: &'a [SpaceRowVisuals],
    /// The content-pane branch.
    pub content: SemanticContent,
}

/// Build the root semantic tree of the current render.
pub fn semantic_tree(input: &SemanticInput<'_>) -> Vec<SemanticNode> {
    let mut root_children: Vec<SemanticNode> = Vec::new();
    if input.sidebar_visible {
        root_children.push(sidebar_node(input));
    }
    root_children.push(content_node(&input.content));
    let root = SemanticNode {
        id: "bbr-root".into(),
        role: SemanticRole::Root,
        name: Some("BBR window".into()),
        value: None,
        enabled: true,
        selected: false,
        focused: input.focus.command
            || input.focus.spaces
            || input.focus.tabs
            || input.focus.divider,
        children: root_children,
    };
    vec![root]
}

fn sidebar_node(input: &SemanticInput<'_>) -> SemanticNode {
    let mut children: Vec<SemanticNode> = vec![
        node(
            "bbr-brand",
            SemanticRole::Brand,
            Some("BBR".into()),
            None,
            false,
            false,
            vec![],
        ),
        command_field_node(input),
        spaces_list_node(input),
        tabs_list_node(input),
        node(
            "bbr-status",
            SemanticRole::Status,
            Some(crate::shell::status_line(input.browser)),
            None,
            false,
            false,
            vec![],
        ),
        node(
            "bbr-sidebar-toggle",
            SemanticRole::Button,
            Some("Sidebar (Cmd-B)".into()),
            Some(
                if input.sidebar_visible {
                    "visible"
                } else {
                    "collapsed"
                }
                .into(),
            ),
            false,
            false,
            vec![],
        ),
    ];
    children.push(SemanticNode {
        id: crate::sidebar::SIDEBAR_DIVIDER_ID.into(),
        role: SemanticRole::Divider,
        name: Some(input.divider_label.clone()),
        value: Some(format!("{:.0} pt", input.sidebar_width)),
        enabled: true,
        selected: false,
        focused: input.focus.divider,
        children: vec![],
    });
    SemanticNode {
        id: "bbr-sidebar".into(),
        role: SemanticRole::Sidebar,
        name: Some("Sidebar".into()),
        value: None,
        enabled: true,
        selected: false,
        focused: false,
        children,
    }
}

fn command_field_node(input: &SemanticInput<'_>) -> SemanticNode {
    let mut children = vec![];
    if input.focus.command {
        // The §9 cursor: a structural, non-color-only focus cue.
        children.push(node(
            "bbr-command-cursor",
            SemanticRole::Cursor,
            None,
            None,
            false,
            false,
            vec![],
        ));
    }
    // §11.3 cross-context transient: the destination profile chip + label
    // shown for one interaction after a cross-profile space activation.
    if let Some(hint) = &input.command.profile_hint {
        children.push(SemanticNode {
            id: "bbr-command-profile-hint".into(),
            role: SemanticRole::Chip,
            name: Some(format!("in Profile · {}", hint.title)),
            value: None,
            enabled: true,
            selected: false,
            focused: false,
            children: vec![SemanticNode {
                id: format!("bbr-profile-chip-{}", hint.profile_index),
                role: SemanticRole::Chip,
                name: Some(format!("Profile · {}", hint.monogram)),
                value: None,
                enabled: true,
                selected: false,
                focused: false,
                children: vec![],
            }],
        });
    }
    let name = if input.command.text.is_empty() {
        "Type a URL…".to_string()
    } else {
        input.command.text.clone()
    };
    let value = if let Some(hint) = &input.command.hint {
        format!("{} · {}", input.command.mode, hint)
    } else {
        input.command.mode.clone()
    };
    node(
        "bbr-command-bar",
        SemanticRole::Field,
        Some(name),
        Some(value),
        false,
        input.focus.command,
        children,
    )
}

fn spaces_list_node(input: &SemanticInput<'_>) -> SemanticNode {
    let mut children: Vec<SemanticNode> = input
        .space_rows
        .iter()
        .map(|row| space_row_node(row, input.focus.spaces))
        .collect();
    children.push(node(
        "bbr-space-new",
        SemanticRole::Button,
        Some("New Space".into()),
        None,
        false,
        false,
        vec![],
    ));
    node(
        "bbr-space-list",
        SemanticRole::List,
        Some("Spaces".into()),
        None,
        false,
        input.focus.spaces,
        children,
    )
}

fn space_row_node(row: &SpaceRowVisuals, list_focused: bool) -> SemanticNode {
    let selected = row.state == SpaceVisualState::Selected;
    let children = vec![
        node(
            format!("bbr-profile-chip-{}", row.profile_index),
            SemanticRole::Chip,
            Some(format!("Profile · {}", row.chip.letter)),
            None,
            false,
            false,
            vec![],
        ),
        node(
            format!("bbr-space-title-{}", row.index),
            SemanticRole::Title,
            Some(row.title.to_string()),
            None,
            selected,
            false,
            vec![],
        ),
        node(
            format!("bbr-space-count-{}", row.index),
            SemanticRole::Count,
            None,
            Some(row.count.to_string()),
            false,
            false,
            vec![],
        ),
    ];
    SemanticNode {
        id: format!("bbr-space-row-{}", row.index),
        role: SemanticRole::Row,
        name: Some(row.title.to_string()),
        value: Some(row.count.to_string()),
        enabled: true,
        selected,
        focused: list_focused && selected,
        children,
    }
}

fn tabs_list_node(input: &SemanticInput<'_>) -> SemanticNode {
    // Mirror the render's order: the Pinned section label + pinned rows,
    // then the Tabs section label + the bounded regular window rows (the
    // hairline between them is presence, not content — §2, skipped here).
    let pinned = crate::shell::pinned_count(&input.browser.tabs);
    let regular_count = input.browser.tabs.len().saturating_sub(pinned);
    let mut children: Vec<SemanticNode> = Vec::new();
    if pinned > 0 {
        children.push(section_node("Pinned", input.focus.tabs));
    }
    for row in input.rows {
        if row.index < pinned {
            children.push(tab_row_node(row, input.focus.tabs));
        }
    }
    if regular_count > 0 {
        children.push(section_node("Tabs", input.focus.tabs));
    }
    for row in input.rows {
        if row.index >= pinned {
            children.push(tab_row_node(row, input.focus.tabs));
        }
    }
    node(
        "bbr-tab-list",
        SemanticRole::List,
        Some("Tabs".into()),
        None,
        false,
        input.focus.tabs,
        children,
    )
}

fn tab_row_node(row: &RowVisuals, list_focused: bool) -> SemanticNode {
    let selected = row.state == TabVisualState::Selected;
    let dragging = row.state == TabVisualState::Dragging;
    let controls_enabled = !(dragging || row.state == TabVisualState::Disabled);
    let mut pin = node(
        format!("bbr-tab-pin-{}", row.index),
        SemanticRole::Button,
        Some(if row.pinned { "Unpin tab" } else { "Pin tab" }.into()),
        None,
        false,
        false,
        vec![],
    );
    pin.enabled = controls_enabled;
    let mut close = node(
        format!("bbr-tab-close-{}", row.index),
        SemanticRole::Button,
        Some("Close tab".into()),
        None,
        false,
        false,
        vec![],
    );
    close.enabled = controls_enabled;
    let children = vec![
        node(
            format!("bbr-tab-favicon-{}", row.index),
            SemanticRole::Favicon,
            row.favicon.map(|spec| spec.letter.to_string()),
            None,
            false,
            false,
            vec![],
        ),
        node(
            format!("bbr-tab-title-{}", row.index),
            SemanticRole::Title,
            Some(row.title.to_string()),
            None,
            selected,
            false,
            vec![],
        ),
        pin,
        node(
            format!("bbr-tab-state-{}", row.index),
            SemanticRole::State,
            Some(format!("{:?}", row.state).to_lowercase()),
            None,
            false,
            false,
            vec![],
        ),
        close,
    ];
    SemanticNode {
        id: format!("bbr-tab-row-{}", row.index),
        role: SemanticRole::Row,
        name: Some(row.title.to_string()),
        value: None,
        enabled: true,
        selected,
        focused: list_focused && selected,
        children,
    }
}

fn section_node(text: &str, _list_focused: bool) -> SemanticNode {
    // §9 skips section labels (and the brand) in the tab order: they never
    // own keyboard focus, so `focused` stays false even while the owning
    // list carries the ring. The label's text-step brightening on a focused
    // list is a visual side effect of the list's focus, not focus itself.
    node(
        format!("bbr-section-{}", text.to_lowercase()),
        SemanticRole::Section,
        Some(text.to_uppercase()),
        None,
        false,
        false,
        vec![],
    )
}

fn content_node(content: &SemanticContent) -> SemanticNode {
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

fn node(
    id: impl Into<String>,
    role: SemanticRole,
    name: Option<String>,
    value: Option<String>,
    selected: bool,
    focused: bool,
    children: Vec<SemanticNode>,
) -> SemanticNode {
    SemanticNode {
        id: id.into(),
        role,
        name,
        value,
        enabled: true,
        selected,
        focused,
        children,
    }
}

#[cfg(test)]
mod tests;
