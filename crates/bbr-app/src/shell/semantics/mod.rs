//! The typed semantic tree of the chrome (DESIGN.md §15 / accessibility
//! track).
//!
//! Every shell element exposes a stable `bbr-*` id; this module derives the
//! *typed* semantic model — roles, names, values, enabled/selected/focused
//! state, and the parent/child hierarchy — from the same structures the
//! render consumes (rows via [`crate::components::RowVisuals`], the §11.3
//! strip chips via [`crate::components::SpaceChipVisuals`], the content pane
//! mirroring `Render::render`), so the E2E semantic assertions cannot drift
//! from the shell.
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

use crate::app::palette::CommandSurface;

mod command;
use crate::components::{RowVisuals, SpaceChipVisuals, TabVisualState};
use command::{command_field_node, command_surface_node};
mod content;
use content::content_node;
pub use content::SemanticContent;

/// Which GPUI chrome surface owns keyboard focus (§9: the focused control
/// renders the ring; the model exposes the same fact typed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticFocus {
    pub command: bool,
    pub spaces: bool,
    pub tabs: bool,
    pub divider: bool,
    /// The §11.11 folders region (Enter/Space toggles the focused row).
    pub folders: bool,
    /// The §11.4 plus menu surface (keyboard selection while open).
    pub plus_menu: bool,
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

/// One §11.5 command result row on the semantic tree: the stable key, the
/// display title/subtitle, and the selection/destructive state the palette
/// renders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticResultRow {
    pub key: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub selected: bool,
    pub destructive: bool,
}

/// The command-field semantic content (mode, text, the §11.3 destination
/// hint when a move is staged, and the §11.5 ranked result surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCommand {
    pub mode: String,
    /// The §11.5 address pill's derived domain (display-only — the pill
    /// never holds the editable text or a selection).
    pub address: String,
    pub text: String,
    /// The rendered destination hint ("→ Space" / "in Profile · P → Space").
    pub hint: Option<String>,
    /// The §11.3 cross-context transient (chip + "in Profile · <title>")
    /// shown for one interaction after a cross-profile space activation.
    pub profile_hint: Option<SemanticProfileHint>,
    /// The published §11.5 result rows in display order (empty when the
    /// surface shows no results).
    pub results: Vec<SemanticResultRow>,
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
    /// The open §11.4 command-surface presentation (centered palette or
    /// field-anchored overlay); `None` when the in-rail command bar is the
    /// active surface.
    pub command_surface: CommandSurface,
    /// The rendered tab rows (pinned block + bounded regular window), with
    /// their composed visual states.
    pub rows: &'a [RowVisuals],
    /// The rendered §11.3 strip chips in canonical display order (one chip
    /// per space; the chip/identity ids derive from the persistent space
    /// id, and `selected` mirrors the active space exactly).
    pub space_chips: &'a [SpaceChipVisuals],
    /// The rendered §11.10 favorite destination tiles (empty → region
    /// hidden with its label).
    pub favorites: &'a [crate::components::FavoriteTileSpec],
    /// The rendered §11.11 folder rows (empty → region hidden).
    pub folder_rows: &'a [crate::components::FolderRowSpec],
    /// The expanded folders' nested member rows (title/letter per position).
    pub folder_members: &'a [crate::components::FolderMemberVisual],
    /// The §11.4 plus-menu state (the menu node renders only while open).
    pub plus_menu: crate::components::menu::PlusMenuState,
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
    // The open palette/overlay presentation mirrors on the tree as the
    // topmost node (elev:2, §13) while it is open.
    if let Some(surface) = command_surface_node(input) {
        root_children.push(surface);
    }
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
            || input.focus.divider
            || input.focus.folders
            || input.focus.plus_menu,
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
    ];
    // §11.4 region order: command field → favorites grid → pinned → folders
    // → regular tabs → status row → footer row (the permanent §11.3 space
    // strip is the footer's flex region — always part of the rail). Hidden
    // regions render no nodes (§15 mirrors the render).
    if let Some(favorites) = u2::favorites_node(input.favorites, input.browser) {
        children.push(favorites);
    }
    children.push(tabs_list_node(input));
    children.push(node(
        "bbr-status",
        SemanticRole::Status,
        Some(crate::shell::status_line(input.browser)),
        None,
        false,
        false,
        vec![],
    ));
    children.push(node(
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
    ));
    if let Some(menu) = u2::plus_menu_node(&input.plus_menu) {
        children.push(menu);
    }
    children.push(u2::footer_node(input));
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

fn tabs_list_node(input: &SemanticInput<'_>) -> SemanticNode {
    // Mirror the render's order: the Pinned section label + pinned rows,
    // then the Folders section (when folders exist), then the Tabs section
    // label + the bounded regular window rows (the hairline between them is
    // presence, not content — §2, skipped here). The folders section carries
    // the nested member rows of expanded folders.
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
    if let Some(folders) = u2::folders_node(input.folder_rows, input.folder_members) {
        children.push(folders);
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

// U2 §15 node builders live in [`u2`] (footer row, plus menu, favorites
// grid, folders list) so no module outgrows the size budget.
pub mod u2;

// The content-pane branch lives in [`content`] (native slot / empty
// new-tab / crash panel), mirroring `Render::render`'s content selection.

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
