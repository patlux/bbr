//! The organized area's row derivation (DESIGN.md §11.2/§11.10/§11.11): a
//! folder is the grouped presentation of pinned tabs, so a grouped
//! (folder-member) pinned tab renders inside its folder only; a Favorite is
//! a pinned tab whose tile is its only presentation, so it renders in the
//! grid only; the loose pinned block lists the remaining pinned tabs. Drag slots and keyboard traversal follow the
//! rendered rows. Pure functions over the controller and organization read
//! models, shared by the render, the drag resolver, the keyboard seam and the
//! testkit evidence.

use bbr_browser::{ControllerSnapshot, FavoriteSnapshot, FolderSnapshot};

/// The active-space tab indices that are members of a folder, in folder
/// order (members of other spaces are skipped).
pub fn grouped_indices(folders: &[FolderSnapshot], browser: &ControllerSnapshot) -> Vec<usize> {
    folders
        .iter()
        .flat_map(|folder| folder.members.iter())
        .filter_map(|id| {
            browser
                .all_tabs
                .iter()
                .find(|tab| tab.id == *id && tab.space == browser.active_space)
                .map(|tab| tab.index)
        })
        .collect()
}

/// The active-space pinned tab indices the loose pinned block does not list:
/// folder members (in folder order), then Favorites (in grid order) — each
/// presented only in its folder or its tile (DESIGN.md §11.10/§11.11).
pub fn hidden_pinned(
    folders: &[FolderSnapshot],
    favorites: &[FavoriteSnapshot],
    browser: &ControllerSnapshot,
) -> Vec<usize> {
    let mut hidden = grouped_indices(folders, browser);
    hidden.extend(favorites.iter().filter_map(|favorite| {
        browser
            .all_tabs
            .iter()
            .find(|tab| tab.id == favorite.tab && tab.space == browser.active_space)
            .map(|tab| tab.index)
    }));
    hidden
}

/// The active-space tab indices of the members of expanded folders, in
/// display order (the nested rows the folders list renders).
pub fn expanded_member_indices(
    folders: &[FolderSnapshot],
    browser: &ControllerSnapshot,
    is_expanded: impl Fn(bbr_domain::FolderId) -> bool,
) -> Vec<usize> {
    let expanded: Vec<FolderSnapshot> = folders
        .iter()
        .filter(|folder| is_expanded(folder.id))
        .cloned()
        .collect();
    grouped_indices(&expanded, browser)
}

/// The loose pinned rows: the pinned indices `0..pinned` that are not
/// hidden (grouped or favorite), in tab order.
pub fn loose_pinned(pinned: usize, grouped: &[usize]) -> Vec<usize> {
    (0..pinned)
        .filter(|index| !grouped.contains(index))
        .collect()
}

/// The keyboard traversal order of the rendered rows: loose pinned rows,
/// then the nested members of expanded folders (in display order), then the
/// regular region.
pub fn traversal_order(
    len: usize,
    pinned: usize,
    grouped: &[usize],
    expanded_members: &[usize],
) -> Vec<usize> {
    let mut order = loose_pinned(pinned.min(len), grouped);
    order.extend(
        expanded_members
            .iter()
            .copied()
            .filter(|index| *index < len),
    );
    order.extend(pinned.min(len)..len);
    order
}

/// The next row in `order` from `active` by `delta` (wrapping or clamping at
/// the edges). `None` when `active` is not a rendered row (a member of a
/// collapsed folder): the caller keeps its index-order fallback.
pub fn step(order: &[usize], active: usize, delta: isize, wrap: bool) -> Option<usize> {
    let position = order.iter().position(|index| *index == active)? as isize;
    let len = order.len() as isize;
    let next = if wrap {
        (position + delta).rem_euclid(len)
    } else {
        (position + delta).clamp(0, len - 1)
    };
    order.get(next as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_pinned_tabs_leave_the_loose_block() {
        // Pinned 0..5 with 1 and 3 grouped.
        assert_eq!(loose_pinned(5, &[3, 1]), vec![0, 2, 4]);
        assert_eq!(loose_pinned(3, &[]), vec![0, 1, 2]);
        assert!(loose_pinned(2, &[0, 1]).is_empty());
    }

    #[test]
    fn traversal_follows_the_rendered_rows() {
        // Pinned 0..4, 1 and 3 grouped; folder A (expanded) holds 3, folder
        // B (collapsed) holds 1; regular 4..6.
        let order = traversal_order(6, 4, &[3, 1], &[3]);
        assert_eq!(order, vec![0, 2, 3, 4, 5]);
        assert_eq!(step(&order, 2, 1, true), Some(3), "loose → member");
        assert_eq!(step(&order, 3, 1, true), Some(4), "member → regular");
        assert_eq!(step(&order, 5, 1, true), Some(0), "wraps");
        assert_eq!(step(&order, 0, -1, false), Some(0), "dense clamps");
        assert_eq!(step(&order, 1, 1, true), None, "collapsed member");
    }
}
