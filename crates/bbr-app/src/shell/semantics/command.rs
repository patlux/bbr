//! Address-control, command-staging, and floating command-surface semantic
//! nodes.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};

// The floating-command id constants live in `bbr_ui::surfaces::command`
// (shared by the renderer and this semantic mirror so §15 ids never
// drift); `CommandSurface` stays app-owned.
use bbr_ui::surfaces::command::{
    OVERLAY_CURSOR_ID, OVERLAY_FIELD_ID, OVERLAY_ID, OVERLAY_TEXT_ID, PALETTE_CURSOR_ID,
    PALETTE_FIELD_ID, PALETTE_ID, PALETTE_MODE_ID,
};

use crate::app::palette::CommandSurface;

use super::{node, SemanticInput};

/// The sidebar command slot node: the §11.5 address pill (display-only
/// Button carrying the derived domain) whenever no non-address staging mode
/// is active — at rest and behind an open palette/overlay presentation; the
/// `bbr-command-bar` staging input during New Space / Move Tab / the empty
/// New Tab surface. The address control is never a Field and never carries
/// a cursor or selection (DESIGN.md §11.5).
pub(super) fn command_field_node(input: &SemanticInput<'_>) -> SemanticNode {
    let staging = input.command_surface == CommandSurface::None && input.command.mode != "inactive";
    if staging {
        staging_field_node(input)
    } else {
        address_control_node(input)
    }
}

/// The §11.5 address control node: a display-only Button whose name is the
/// derived domain, with the domain title and the reserved accessory slot as
/// children. The §11.3 cross-context profile hint renders at the leading
/// edge for its one-interaction lifetime (it follows whichever command
/// surface is actually visible).
fn address_control_node(input: &SemanticInput<'_>) -> SemanticNode {
    let mut children = vec![
        node(
            "bbr-address-domain",
            SemanticRole::Title,
            Some(input.command.address.clone()),
            None,
            false,
            false,
            vec![],
        ),
        SemanticNode {
            id: "bbr-address-accessory".into(),
            // The §17.14 accessory content decision is open; v1 ships the
            // slot empty, so it is not an activatable control.
            role: SemanticRole::Button,
            name: Some("Address actions".into()),
            value: None,
            enabled: false,
            selected: false,
            focused: false,
            drag_target: false,
            children: vec![],
        },
    ];
    if let Some(hint) = &input.command.profile_hint {
        children.insert(0, profile_hint_node(hint));
    }
    SemanticNode {
        id: "bbr-address".into(),
        role: SemanticRole::Button,
        name: Some(input.command.address.clone()),
        // Blueprint D03: the identity is spoken, not only drawn.
        value: address_identity_value(&input.command.address_identity),
        enabled: true,
        selected: false,
        focused: false,
        drag_target: false,
        children,
    }
}

/// Blueprint D03: the address control's spoken identity (`None` for a
/// secure site and for placeholders).
fn address_identity_value(kind: &str) -> Option<String> {
    match kind {
        "not-secure" => Some("Not secure".into()),
        "internal" => Some("BBR page".into()),
        "file" => Some("Local file".into()),
        _ => None,
    }
}

/// The §11.3 cross-context transient: the destination profile chip + label
/// shown for one interaction after a cross-profile space activation.
fn profile_hint_node(hint: &super::SemanticProfileHint) -> SemanticNode {
    SemanticNode {
        id: "bbr-command-profile-hint".into(),
        role: SemanticRole::Chip,
        name: Some(format!("in Profile · {}", hint.title)),
        value: None,
        enabled: true,
        selected: false,
        focused: false,
        drag_target: false,
        children: vec![SemanticNode {
            id: format!("bbr-profile-chip-{}", hint.profile_index),
            role: SemanticRole::Chip,
            name: Some(format!("Profile · {}", hint.monogram)),
            value: None,
            enabled: true,
            selected: false,
            focused: false,
            drag_target: false,
            children: vec![],
        }],
    }
}

/// The command **staging** surface node (`bbr-command-bar`): the input shown
/// only while a non-address staged mode is active without a floating
/// presentation (New Space / Move Tab / the §11.8 empty New Tab surface).
/// It keeps the field role, mode value, block cursor, the §11.3 hints, and
/// the §11.5 in-rail result list.
fn staging_field_node(input: &SemanticInput<'_>) -> SemanticNode {
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
    // §11.3 cross-context transient on the staging surface.
    if let Some(hint) = &input.command.profile_hint {
        children.push(profile_hint_node(hint));
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
    // §11.5 result surface: the in-rail palette renders beneath the staging
    // input while the location/new-tab surface is open; the typed state
    // rows expose their stable ids too. While a §11.4 palette/overlay
    // presentation is open the floating surface renders the same engine's
    // results itself, so the in-rail list mirrors nothing (it is hidden,
    // never duplicated).
    if input.command_surface == CommandSurface::None && !input.command.results.is_empty() {
        children.push(SemanticNode {
            id: "bbr-command-results".into(),
            role: SemanticRole::List,
            name: Some("Command results".into()),
            value: None,
            enabled: true,
            selected: false,
            focused: false,
            drag_target: false,
            children: input
                .command
                .results
                .iter()
                .map(|row| SemanticNode {
                    id: format!("bbr-command-result-{}", row.key),
                    role: SemanticRole::Result,
                    name: Some(row.title.clone()),
                    value: row.subtitle.clone(),
                    enabled: true,
                    selected: row.selected,
                    focused: false,
                    drag_target: false,
                    children: vec![],
                })
                .collect(),
        });
    }
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

/// The §11.4 command-surface presentation node (DESIGN.md §15): the
/// centered Cmd-T palette or the field-anchored Cmd-L/click overlay,
/// mirrored as a root child (elev:2, topmost) while open. The address pill
/// stays a sidebar child either way. `None` when no palette/overlay
/// presentation is open.
pub(super) fn command_surface_node(input: &SemanticInput<'_>) -> Option<SemanticNode> {
    let mode = input.command.mode.clone();
    let text = if input.command.text.is_empty() {
        "Type a URL…".to_string()
    } else {
        input.command.text.clone()
    };
    match input.command_surface {
        CommandSurface::None => None,
        CommandSurface::Palette => {
            let mut field_children = vec![node(
                PALETTE_MODE_ID,
                SemanticRole::Chip,
                Some(mode),
                None,
                false,
                false,
                vec![],
            )];
            if input.focus.command {
                field_children.push(node(
                    PALETTE_CURSOR_ID,
                    SemanticRole::Cursor,
                    None,
                    None,
                    false,
                    true,
                    vec![],
                ));
            }
            // The published §11.5 result rows mirror under the palette
            // field with the surface's own id family.
            let mut children = vec![SemanticNode {
                id: PALETTE_FIELD_ID.into(),
                role: SemanticRole::Field,
                name: Some(text),
                value: None,
                enabled: true,
                selected: false,
                focused: input.focus.command,
                drag_target: false,
                children: field_children,
            }];
            children.extend(surface_result_nodes(input, "bbr-palette-result"));
            Some(SemanticNode {
                id: PALETTE_ID.into(),
                role: SemanticRole::Palette,
                name: Some("Command palette".into()),
                value: None,
                enabled: true,
                selected: false,
                focused: input.focus.command,
                drag_target: false,
                children,
            })
        }
        CommandSurface::Overlay => {
            // The overlay's own field row (`bbr-command-overlay-field`) with
            // the full URL as its name and the structural cursor while the
            // overlay owns chrome focus (§11.4 — one keystroke replaces the
            // preselected URL).
            let mut field_children = vec![node(
                OVERLAY_TEXT_ID,
                SemanticRole::Title,
                Some(text.clone()),
                None,
                false,
                false,
                vec![],
            )];
            if input.focus.command {
                field_children.push(node(
                    OVERLAY_CURSOR_ID,
                    SemanticRole::Cursor,
                    None,
                    None,
                    false,
                    true,
                    vec![],
                ));
            }
            let mut children = vec![SemanticNode {
                id: OVERLAY_FIELD_ID.into(),
                role: SemanticRole::Field,
                name: Some(text.clone()),
                value: None,
                enabled: true,
                selected: false,
                focused: input.focus.command,
                drag_target: false,
                children: field_children,
            }];
            children.extend(surface_result_nodes(input, "bbr-command-result"));
            Some(SemanticNode {
                id: OVERLAY_ID.into(),
                role: SemanticRole::Palette,
                name: Some("Command overlay".into()),
                value: Some(text),
                enabled: true,
                selected: false,
                focused: input.focus.command,
                drag_target: false,
                children,
            })
        }
    }
}

/// The §11.5 ranked result rows mirrored on the open surface node with the
/// surface's id prefix (`bbr-palette-result` / `bbr-command-result`); empty
/// while the surface shows no published results.
fn surface_result_nodes(input: &SemanticInput<'_>, id_prefix: &str) -> Vec<SemanticNode> {
    if input.command.results.is_empty() {
        return Vec::new();
    }
    vec![SemanticNode {
        id: format!("{id_prefix}-list"),
        role: SemanticRole::List,
        name: Some("Command results".into()),
        value: None,
        enabled: true,
        selected: false,
        focused: false,
        drag_target: false,
        children: input
            .command
            .results
            .iter()
            .map(|row| SemanticNode {
                id: format!("{id_prefix}-{}", row.key),
                role: SemanticRole::Result,
                name: Some(row.title.clone()),
                value: row.subtitle.clone(),
                enabled: true,
                selected: row.selected,
                focused: false,
                drag_target: false,
                children: vec![],
            })
            .collect(),
    }]
}
