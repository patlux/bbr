//! U2 §15 semantic node builders (DESIGN.md §11.3/§11.4/§11.10/§11.11):
//! the footer row with its permanent space strip and identity chips, the
//! plus menu, favorites grid, and folders list nodes the semantic-tree
//! composition step folds into `sidebar_node` once the U2 surfaces render.
//! The node shapes below are the typed §15 contract the §15 tests lock.
//!
//! The footer row node carries the two fixed controls (`bbr-downloads` /
//! `bbr-plus`) with the §11.3 space strip between them: one `bbr-space-strip`
//! list node with one `bbr-space-chip-<persistent-space-id>` row per chip
//! (and its `bbr-space-identity-<persistent-space-id>` child) in canonical
//! display order, `selected` exactly on the active space.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};

use super::SemanticInput;
use bbr_ui::sidebar::{space_chip_id, space_identity_id};
use bbr_ui::state::SpaceChipVisuals;

/// The §11.4 footer row node: the container, the two fixed controls
/// (`bbr-downloads` / `bbr-plus`) with their §7 text alternatives, and the
/// permanent §11.3 space strip between them (one chip node per space in
/// canonical display order; the chip ids derive from the persistent space
/// id).
pub fn footer_node(input: &SemanticInput<'_>) -> SemanticNode {
    let mut footer = node(
        bbr_ui::components::footer::FOOTER_ID,
        SemanticRole::Footer,
        Some("Footer".into()),
        None,
        false,
        false,
        vec![],
    );
    footer.children.push(node(
        bbr_ui::components::footer::FOOTER_DOWNLOADS_ID,
        SemanticRole::Button,
        Some("Downloads".into()),
        None,
        false,
        false,
        vec![],
    ));
    footer.children.push(space_strip_node(input));
    footer.children.push(node(
        bbr_ui::components::footer::FOOTER_PLUS_ID,
        SemanticRole::Button,
        Some(bbr_ui::components::footer::FOOTER_PLUS_NAME.into()),
        None,
        false,
        false,
        vec![],
    ));
    footer
}

/// The §11.3 permanent strip node: one `bbr-space-strip` list carrying one
/// `bbr-space-chip-<persistent-space-id>` row per chip in canonical display
/// order, each with its `bbr-space-identity-<persistent-space-id>` child.
/// `selected` sits exactly on the active space's chip (and the selected chip
/// of the focused strip owns `focused`), so the strip's selection evidence
/// can never drift from the render.
fn space_strip_node(input: &SemanticInput<'_>) -> SemanticNode {
    let children: Vec<SemanticNode> = input
        .space_chips
        .iter()
        .map(|chip| space_chip_node(chip, input.focus.spaces))
        .collect();
    node(
        bbr_ui::components::footer::FOOTER_STRIP_ID,
        SemanticRole::List,
        Some("Spaces".into()),
        None,
        false,
        input.focus.spaces,
        children,
    )
}

/// One §11.3 chip node: the persistent `bbr-space-chip-<SpaceId>` identity
/// with its `bbr-space-identity-<SpaceId>` child. `selected` mirrors the
/// chip's active-space evidence; the identity node carries the validated
/// curated identity string so identity is never color-only. While a drag
/// targets this exact chip (§10.9), the node reports `drag_target` and
/// carries the §12 destination-profile companion chip as a child, so the
/// drag-target evidence is typed and derives from the same
/// `space_chip_visuals` the render and the snapshot consume.
fn space_chip_node(chip: &SpaceChipVisuals, strip_focused: bool) -> SemanticNode {
    let mut children: Vec<SemanticNode> = vec![node(
        space_identity_id(chip.space),
        SemanticRole::Title,
        Some(chip.identity.clone()),
        None,
        chip.selected,
        false,
        vec![],
    )];
    if let Some(companion) = &chip.companion {
        children.push(node(
            bbr_ui::components::space::profile_chip_id(companion.profile_index),
            SemanticRole::Chip,
            Some(companion.letter.clone()),
            None,
            false,
            false,
            vec![],
        ));
    }
    SemanticNode {
        id: space_chip_id(chip.space).to_string(),
        role: SemanticRole::Row,
        // §11.3: the chip's stable accessible name carries the Space and
        // Profile names (the footer is icon-only at rest, so the accessible
        // name is the non-color identity companion).
        name: Some(format!(
            "{} · in Profile · {}",
            chip.title, chip.profile_title
        )),
        value: None,
        enabled: true,
        selected: chip.selected,
        focused: strip_focused && chip.selected,
        drag_target: chip.drag_target,
        children,
    }
}

/// The §11.4 plus menu node while open: the `bbr-plus-menu` surface with
/// the two v1 items, `selected` on the keyboard selection. `None` while the
/// menu is closed (the surface is not rendered).
pub fn plus_menu_node(state: &bbr_ui::components::menu::PlusMenuState) -> Option<SemanticNode> {
    if !state.open {
        return None;
    }
    let selected = state.selected_item();
    let children = bbr_ui::components::plus_menu_items()
        .iter()
        .map(|item| {
            node(
                item.id,
                SemanticRole::MenuItem,
                Some(item.label.into()),
                None,
                item.id == selected.id,
                false,
                vec![],
            )
        })
        .collect();
    Some(node(
        bbr_ui::components::menu::PLUS_MENU_ID,
        SemanticRole::Menu,
        Some("Plus menu".into()),
        None,
        false,
        false,
        children,
    ))
}

/// The §11.10 favorites band node: the stable section wrapper contains the
/// `bbr-favorites` grid without painting redundant label text; each tile is
/// a §7.2 destination with its monogram
/// icon child. `None` while the region is empty at rest (label hidden).
/// While a tab drag targets the region — including an empty grid, when the
/// region renders as the compact drop band — the section carries the
/// §10.9 `drag_target` flag so the drop affordance is part of the semantic
/// tree (§15: during a drag both the source row and any target are
/// reported). Per DESIGN.md §11.10 the tile's semantic name includes the
/// owning space/profile when they differ from the current context, so
/// cross-space favorites stay distinguishable without color.
pub fn favorites_node(
    tiles: &[bbr_ui::components::FavoriteTileSpec],
    browser: &bbr_browser::ControllerSnapshot,
    band_targeted: bool,
) -> Option<SemanticNode> {
    if tiles.is_empty() && !band_targeted {
        return None;
    }
    let children = tiles
        .iter()
        .map(|tile| SemanticNode {
            id: bbr_ui::components::favorite_tile_id(tile.index).to_string(),
            role: SemanticRole::Tile,
            name: Some(favorite_tile_name(tile, browser)),
            value: None,
            enabled: true,
            selected: false,
            focused: false,
            drag_target: false,
            children: vec![node(
                bbr_ui::components::favorite_tile_icon_id(tile.index),
                SemanticRole::Favicon,
                Some(tile.letter.to_string()),
                None,
                false,
                false,
                vec![],
            )],
        })
        .collect();
    let mut section = node(
        bbr_ui::components::favorites::FAVORITES_SECTION_ID,
        SemanticRole::Section,
        Some("Favorites".into()),
        None,
        false,
        false,
        vec![],
    );
    section.drag_target = band_targeted;
    section.children.push(node(
        bbr_ui::components::favorites::FAVORITES_ID,
        SemanticRole::Grid,
        Some("Favorites grid".into()),
        None,
        false,
        false,
        children,
    ));
    Some(section)
}

/// The §11.10 tile semantic name: the page title, plus the owning
/// space/profile when they differ from the current context (a favorite is a
/// window-global presentation of one concrete tab, so the owner context
/// must be readable from any space/profile).
fn favorite_tile_name(
    tile: &bbr_ui::components::FavoriteTileSpec,
    browser: &bbr_browser::ControllerSnapshot,
) -> String {
    let mut name = tile.title.clone();
    if tile.space != browser.active_space.0 || tile.profile != browser.active_profile.0 {
        let space = browser
            .spaces
            .iter()
            .find(|s| s.id.0 == tile.space)
            .map(|s| s.title.clone())
            .unwrap_or_else(|| format!("{}", tile.space));
        let profile = browser
            .profiles
            .iter()
            .find(|p| p.id.0 == tile.profile)
            .map(|p| p.title.clone())
            .unwrap_or_else(|| format!("{}", tile.profile));
        name.push_str(&format!(" · in Space · {space} · in Profile · {profile}"));
    }
    name
}

/// The §11.11 folders node while folders exist: the section label over the
/// `bbr-folders` list with one folder row per spec — disclosure
/// (`expanded`/`collapsed` value), name, and count — plus the nested §11.1
/// rows of expanded folders under their owning row (`bbr-folder-item-<n>-<m>`
/// with `-title`/`-favicon`/`-close` slots).
pub fn folders_node(
    rows: &[bbr_ui::components::FolderRowSpec],
    members: &[bbr_ui::components::FolderMemberVisual],
) -> Option<SemanticNode> {
    if rows.is_empty() {
        return None;
    }
    let children = rows
        .iter()
        .map(|row| {
            let mut node_children = vec![
                node(
                    bbr_ui::components::folder_toggle_id(row.index),
                    SemanticRole::Disclosure,
                    Some("Toggle folder".into()),
                    Some(
                        if row.expanded {
                            "expanded"
                        } else {
                            "collapsed"
                        }
                        .into(),
                    ),
                    false,
                    false,
                    vec![],
                ),
                node(
                    bbr_ui::components::folder_name_id(row.index),
                    SemanticRole::Title,
                    Some(row.title.clone()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    bbr_ui::components::folder_count_id(row.index),
                    SemanticRole::Count,
                    None,
                    Some(row.count.to_string()),
                    false,
                    false,
                    vec![],
                ),
            ];
            node_children.extend(members.iter().filter(|member| member.folder == row.id).map(
                |member| {
                    folder_item_node(
                        member.folder_index,
                        member.member,
                        &member.title,
                        member.letter,
                    )
                },
            ));
            SemanticNode {
                id: bbr_ui::components::folder_row_id(row.index).to_string(),
                role: SemanticRole::Folder,
                name: Some(row.title.clone()),
                value: Some(row.count.to_string()),
                enabled: true,
                selected: false,
                focused: false,
                drag_target: false,
                children: node_children,
            }
        })
        .collect();
    let mut section = section_node("Folders", false);
    section.children.push(node(
        bbr_ui::components::folders::FOLDERS_ID,
        SemanticRole::List,
        Some("Folders".into()),
        None,
        false,
        false,
        children,
    ));
    Some(section)
}

/// One nested §11.1 row node inside an expanded folder (`bbr-folder-item-\
/// <n>-<m>` + `-title`/`-favicon`/`-close`): the member's rendered identity
/// as the semantic row, so the nested tree mirrors the render.
pub fn folder_item_node(folder: usize, member: usize, title: &str, letter: char) -> SemanticNode {
    SemanticNode {
        id: bbr_ui::components::folder_item_id(folder, member).to_string(),
        role: SemanticRole::Row,
        name: Some(title.to_string()),
        value: None,
        enabled: true,
        selected: false,
        focused: false,
        drag_target: false,
        children: vec![
            node(
                bbr_ui::components::folder_item_favicon_id(folder, member),
                SemanticRole::Favicon,
                Some(letter.to_string()),
                None,
                false,
                false,
                vec![],
            ),
            node(
                bbr_ui::components::folder_item_title_id(folder, member),
                SemanticRole::Title,
                Some(title.to_string()),
                None,
                false,
                false,
                vec![],
            ),
            node(
                bbr_ui::components::folder_item_close_id(folder, member),
                SemanticRole::Button,
                Some("Close tab".into()),
                None,
                false,
                false,
                vec![],
            ),
        ],
    }
}

fn section_node(text: &str, _focused: bool) -> SemanticNode {
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
        drag_target: false,
        children,
    }
}
