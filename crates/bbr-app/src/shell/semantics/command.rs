//! Command-field and floating command-surface semantic nodes.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};

use crate::app::palette::{
    CommandSurface, OVERLAY_ID, PALETTE_CURSOR_ID, PALETTE_FIELD_ID, PALETTE_ID, PALETTE_MODE_ID,
};

use super::{node, SemanticInput};

pub(super) fn command_field_node(input: &SemanticInput<'_>) -> SemanticNode {
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
    // §11.5 result surface: the palette renders beneath the field while the
    // location/new-tab surface is open; the typed state rows expose their
    // stable ids too. While a §11.4 palette/overlay presentation is open the
    // floating surface renders the same engine's results itself, so the
    // in-rail list mirrors nothing (it is hidden, never duplicated).
    if input.command_surface == CommandSurface::None && !input.command.results.is_empty() {
        children.push(SemanticNode {
            id: "bbr-command-results".into(),
            role: SemanticRole::List,
            name: Some("Command results".into()),
            value: None,
            enabled: true,
            selected: false,
            focused: false,
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
/// mirrored as a root child (elev:2, topmost) while open. The in-rail
/// command field stays a sidebar child either way. `None` when no
/// palette/overlay presentation is open.
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
                children,
            })
        }
        CommandSurface::Overlay => Some(SemanticNode {
            id: OVERLAY_ID.into(),
            role: SemanticRole::Palette,
            name: Some("Command overlay".into()),
            value: Some(text),
            enabled: true,
            selected: false,
            focused: input.focus.command,
            children: surface_result_nodes(input, "bbr-command-result"),
        }),
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
                children: vec![],
            })
            .collect(),
    }]
}
