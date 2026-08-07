//! U2 §15 semantic node builders (DESIGN.md §11.4/§11.10/§11.11): the
//! footer row, plus menu, favorites grid, and folders list nodes the
//! semantic-tree composition step folds into `sidebar_node` once the U2
//! surfaces render. The render currently ships none of these surfaces, so
//! the builders stay un-wired (and dead-code-allowed) until then; the node
//! shapes below are the typed §15 contract the §15 tests lock.

use bbr_testkit::protocol::{SemanticNode, SemanticRole};

/// The §11.4 footer row node: the container plus exactly the three controls
/// (`bbr-downloads` / `bbr-spaces` / `bbr-plus`) with their §7 text
/// alternatives.
pub fn footer_node() -> SemanticNode {
    let mut footer = node(
        crate::components::footer::FOOTER_ID,
        SemanticRole::Footer,
        Some("Footer".into()),
        None,
        false,
        false,
        vec![],
    );
    footer.children = crate::components::FooterControl::all()
        .iter()
        .map(|control| {
            node(
                control.id(),
                SemanticRole::Button,
                Some(control.semantic_name().into()),
                None,
                false,
                false,
                vec![],
            )
        })
        .collect();
    footer
}

/// The §11.4 plus menu node while open: the `bbr-plus-menu` surface with
/// the two v1 items, `selected` on the keyboard selection. `None` while the
/// menu is closed (the surface is not rendered).
pub fn plus_menu_node(state: &crate::components::menu::PlusMenuState) -> Option<SemanticNode> {
    if !state.open {
        return None;
    }
    let selected = state.selected_item();
    let children = crate::components::plus_menu_items()
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
        crate::components::menu::PLUS_MENU_ID,
        SemanticRole::Menu,
        Some("Plus menu".into()),
        None,
        false,
        false,
        children,
    ))
}

/// The §11.10 favorites grid node while tiles exist: the section label over
/// the `bbr-favorites` grid, each tile a §7.2 destination with its monogram
/// icon child. `None` while the region is empty (label hidden). Per
/// DESIGN.md §11.10 the tile's semantic name includes the owning
/// space/profile when they differ from the current context, so cross-space
/// favorites stay distinguishable without color.
pub fn favorites_node(
    tiles: &[crate::components::FavoriteTileSpec],
    browser: &bbr_browser::ControllerSnapshot,
) -> Option<SemanticNode> {
    if tiles.is_empty() {
        return None;
    }
    let children = tiles
        .iter()
        .map(|tile| SemanticNode {
            id: crate::components::favorite_tile_id(tile.index).to_string(),
            role: SemanticRole::Tile,
            name: Some(favorite_tile_name(tile, browser)),
            value: None,
            enabled: true,
            selected: false,
            focused: false,
            children: vec![node(
                crate::components::favorite_tile_icon_id(tile.index),
                SemanticRole::Favicon,
                Some(tile.letter.to_string()),
                None,
                false,
                false,
                vec![],
            )],
        })
        .collect();
    let mut section = section_node("Favorites", false);
    section.children.push(node(
        crate::components::favorites::FAVORITES_ID,
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
    tile: &crate::components::FavoriteTileSpec,
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
    rows: &[crate::components::FolderRowSpec],
    members: &[crate::components::FolderMemberVisual],
) -> Option<SemanticNode> {
    if rows.is_empty() {
        return None;
    }
    let children = rows
        .iter()
        .map(|row| {
            let mut node_children = vec![
                node(
                    crate::components::folder_toggle_id(row.index),
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
                    crate::components::folder_name_id(row.index),
                    SemanticRole::Title,
                    Some(row.title.clone()),
                    None,
                    false,
                    false,
                    vec![],
                ),
                node(
                    crate::components::folder_count_id(row.index),
                    SemanticRole::Count,
                    None,
                    Some(row.count.to_string()),
                    false,
                    false,
                    vec![],
                ),
            ];
            node_children.extend(
                members
                    .iter()
                    .filter(|member| member.folder == row.index)
                    .map(|member| {
                        folder_item_node(member.folder, member.member, &member.title, member.letter)
                    }),
            );
            SemanticNode {
                id: crate::components::folder_row_id(row.index).to_string(),
                role: SemanticRole::Folder,
                name: Some(row.title.clone()),
                value: Some(row.count.to_string()),
                enabled: true,
                selected: false,
                focused: false,
                children: node_children,
            }
        })
        .collect();
    let mut section = section_node("Folders", false);
    section.children.push(node(
        crate::components::folders::FOLDERS_ID,
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
        id: crate::components::folder_item_id(folder, member).to_string(),
        role: SemanticRole::Row,
        name: Some(title.to_string()),
        value: None,
        enabled: true,
        selected: false,
        focused: false,
        children: vec![
            node(
                crate::components::folder_item_favicon_id(folder, member),
                SemanticRole::Favicon,
                Some(letter.to_string()),
                None,
                false,
                false,
                vec![],
            ),
            node(
                crate::components::folder_item_title_id(folder, member),
                SemanticRole::Title,
                Some(title.to_string()),
                None,
                false,
                false,
                vec![],
            ),
            node(
                crate::components::folder_item_close_id(folder, member),
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
        children,
    }
}
