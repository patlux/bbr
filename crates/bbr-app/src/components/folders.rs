//! DESIGN.md §11.11 Folders: session-local folder rows with a disclosure
//! control, name, and count, plus nested §11.1 rows for the expanded
//! folder.
//!
//! A folder is a session-local container of tab identities (in-memory in
//! v1, never persisted; Phase 7 owns restoration). Rows render
//! `[disclosure glyph 14 §7] [name, trunc] [count, type:meta tnum]` at
//! `density:row` tall; nested rows indent one `sp:2` step and reuse the
//! §11.1 row anatomy. Toggling expands/collapses and snaps (`dur:instant`);
//! the app layer holds the presentation state only ([`FolderPresentation`]
//! — the domain model is L5-owned).
//!
//! This module owns the folder identity contract (ids, the row spec
//! derivation, the presentation state) and the folder-row anatomy. The
//! nested-row rendering consumes member tab visuals and is wired by the
//! rail composition; the id derivation for nested rows is already contract.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use crate::theme::{Radius, Spacing, Theme};

use super::Icon;

/// The §15 id of the folders list container.
pub const FOLDERS_ID: &str = "bbr-folders";
/// The §15 id of the "Folders" section label (hidden when empty).
pub const FOLDERS_SECTION_ID: &str = "bbr-section-folders";

/// The stable folder-row id `bbr-folder-row-<n>` (DESIGN.md §11.11).
pub fn folder_row_id(index: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-row-{index}"))
}

/// The stable folder-name id `bbr-folder-name-<n>`.
pub fn folder_name_id(index: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-name-{index}"))
}

/// The stable folder-count id `bbr-folder-count-<n>`.
pub fn folder_count_id(index: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-count-{index}"))
}

/// The stable disclosure-control id `bbr-folder-toggle-<n>`.
pub fn folder_toggle_id(index: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-toggle-{index}"))
}

/// The stable nested-row id `bbr-folder-item-<n>-<m>`: folder `n`, nested
/// position `m` (0-based, deterministic within the folder).
pub fn folder_item_id(folder: usize, member: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-item-{folder}-{member}"))
}

/// The stable nested title id `bbr-folder-item-<n>-<m>-title`.
pub fn folder_item_title_id(folder: usize, member: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-item-{folder}-{member}-title"))
}

/// The stable nested favicon id `bbr-folder-item-<n>-<m>-favicon`.
pub fn folder_item_favicon_id(folder: usize, member: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-item-{folder}-{member}-favicon"))
}

/// The stable nested close id `bbr-folder-item-<n>-<m>-close`.
pub fn folder_item_close_id(folder: usize, member: usize) -> SharedString {
    SharedString::from(format!("bbr-folder-item-{folder}-{member}-close"))
}

/// One folder row's rendered identity, derived from the L5
/// `FolderSnapshot` plus the app-local presentation state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRowSpec {
    /// Position in creation order — the `bbr-folder-row-<n>` id.
    pub index: usize,
    /// The domain folder id (the L5 `MoveIntoFolder` command's target).
    pub id: u64,
    pub title: String,
    /// Member tab count (§11.11 count column, `type:meta` `tnum`).
    pub count: usize,
    /// Whether the nested row list is expanded (`dur:instant` snap).
    pub expanded: bool,
}

/// Derive the rendered folder rows in creation order (the same source the
/// folders list renders). Member counts come from the L5 snapshot; the
/// expanded/collapsed value from the app-local presentation state.
pub fn folder_row_specs(
    folders: &[bbr_browser::FolderSnapshot],
    presentation: &FolderPresentation,
) -> Vec<FolderRowSpec> {
    folders
        .iter()
        .map(|folder| FolderRowSpec {
            index: folder.index,
            id: folder.id.0,
            title: folder.title.clone(),
            count: folder.members.len(),
            expanded: presentation.is_expanded(folder.index),
        })
        .collect()
}

/// One nested §11.1 row's rendered identity inside an expanded folder:
/// the folder/nested positions that key the stable `bbr-folder-item-<n>-<m>`
/// ids plus the member's display title, §7.2 monogram letter, and slate hue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderMemberVisual {
    /// The owning folder's creation-order index (`<n>`).
    pub folder: usize,
    /// The nested position within the folder, 0-based (`<m>`).
    pub member: usize,
    pub title: String,
    pub letter: char,
    /// §7.2 FNV slate-hue ramp index (0..6), from the member's URL.
    pub hue: usize,
}

/// Resolve the expanded folders' member rows from the L5 snapshot: each
/// member `TabId` maps through the controller's global read model to its
/// display title and monogram letter, in folder order, for expanded folders
/// only (collapsed folders render no nested rows). Deterministic: the folder
/// creation-order index and the member's folder position are the ids.
pub fn folder_member_visuals(
    rows: &[FolderRowSpec],
    folders: &[bbr_browser::FolderSnapshot],
    tabs: &[bbr_browser::GlobalTabSnapshot],
) -> Vec<FolderMemberVisual> {
    let mut visuals = Vec::new();
    for spec in rows {
        if !spec.expanded {
            continue;
        }
        let folder = folders
            .iter()
            .find(|folder| folder.index == spec.index)
            .expect("folder row derives from the snapshot");
        for (member, tab_id) in folder.members.iter().enumerate() {
            let tab = tabs
                .iter()
                .find(|tab| tab.id == *tab_id)
                .expect("reconciled folder member exists in the controller");
            visuals.push(FolderMemberVisual {
                folder: spec.index,
                member,
                title: crate::shell::display_title(&tab.url, &tab.title),
                letter: crate::shell::monogram_letter(&tab.url),
                hue: crate::shell::monogram_hue(&tab.url),
            });
        }
    }
    visuals
}

/// The session-local folder presentation state the app layer owns
/// (DESIGN.md §11.11: "the app layer may hold folder presentation state
/// only"): one expanded/collapsed value per folder in creation order,
/// defaulting to collapsed. Deterministic id stability across toggles:
/// the vector index is the folder's creation-order index, which never
/// changes in v1 (folders are never deleted).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderPresentation {
    expanded: Vec<bool>,
}

impl FolderPresentation {
    /// Whether the folder at creation-order `index` is expanded. Unknown
    /// indices (a folder created after the last toggle) read collapsed.
    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded.get(index).copied().unwrap_or(false)
    }

    /// Toggle the folder at `index` (collapsed → expanded and back),
    /// growing the state vector deterministically for folders created
    /// after the last toggle.
    pub fn toggle(&mut self, index: usize) {
        if index >= self.expanded.len() {
            self.expanded.resize(index + 1, false);
        }
        self.expanded[index] = !self.expanded[index];
    }
}

/// One §11.11 folder row: `[disclosure glyph 14 §7] [name, trunc] [count,
/// type:meta tnum]` at `density:row` tall. `on_toggle` is the production
/// disclosure listener; the rail composition wires it. Nested rows are
/// appended below by the composition when `spec.expanded` (they reuse the
/// §11.1 row anatomy and are not part of this module's anatomy).
pub fn folder_row(
    theme: &Theme,
    spec: &FolderRowSpec,
    on_toggle: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(folder_row_id(spec.index))
        .h(theme.density.row())
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .bg(p.surface_sidebar)
        .cursor_pointer()
        .child(
            div()
                .id(folder_toggle_id(spec.index))
                .size(px(14.0))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, on_toggle)
                .child(
                    Icon::Disclosure {
                        expanded: spec.expanded,
                    }
                    .canvas(theme, px(14.0)),
                ),
        )
        .child(
            div()
                .id(folder_name_id(spec.index))
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink_secondary)
                .child(spec.title.clone()),
        )
        .child(
            div()
                .id(folder_count_id(spec.index))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child(spec.count.to_string()),
        )
}

/// The folders id set for the §15 `visible_ids` derivation: the section
/// label only while folders exist, then the list, the row/name/count/toggle
/// ids, and — for expanded folders — the nested row ids plus their
/// title/favicon/close slots (`bbr-folder-item-<n>-<m>` + `-title`/
/// `-favicon`/`-close`, deterministic 0-based nested positions).
#[cfg_attr(not(feature = "testkit"), allow(dead_code))] // §15 visible-ids seam
pub fn folders_ids(rows: &[FolderRowSpec], members: &[FolderMemberVisual]) -> Vec<String> {
    let mut ids = Vec::new();
    if rows.is_empty() {
        return ids;
    }
    ids.push(FOLDERS_SECTION_ID.to_string());
    ids.push(FOLDERS_ID.to_string());
    for row in rows {
        ids.push(folder_row_id(row.index).to_string());
        ids.push(folder_name_id(row.index).to_string());
        ids.push(folder_count_id(row.index).to_string());
        ids.push(folder_toggle_id(row.index).to_string());
    }
    for member in members {
        ids.push(folder_item_id(member.folder, member.member).to_string());
        ids.push(folder_item_title_id(member.folder, member.member).to_string());
        ids.push(folder_item_favicon_id(member.folder, member.member).to_string());
        ids.push(folder_item_close_id(member.folder, member.member).to_string());
    }
    ids
}

/// One §11.11 nested row (`bbr-folder-item-<n>-<m>`): the §11.1 anatomy
/// `[favicon 16] · [title flex, trunc] · [close 16]` at `density:row` tall,
/// indented one `sp:2` step. `on_select` activates the member tab,
/// `on_close` closes it — the close listener is expected to stop
/// propagation so a close press never also selects the member row.
pub fn folder_item_row(
    theme: &Theme,
    visual: &FolderMemberVisual,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let slate = match theme.mode {
        crate::theme::ThemeMode::Dark => crate::palette::FaviconSlate::dark(visual.hue),
        crate::theme::ThemeMode::Light => crate::palette::FaviconSlate::light(visual.hue),
    };
    let monogram = div()
        .id(folder_item_favicon_id(visual.folder, visual.member))
        .size(theme.density.icon())
        .rounded(Radius.small())
        .bg(slate)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(p.surface_glass)
        .child(SharedString::from(visual.letter.to_string()));
    div()
        .id(folder_item_id(visual.folder, visual.member))
        .h(theme.density.row())
        .pl(Spacing::sp4())
        .pr(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_select)
        .child(monogram)
        .child(
            div()
                .id(folder_item_title_id(visual.folder, visual.member))
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink_secondary)
                .child(visual.title.clone()),
        )
        .child(
            div()
                .id(folder_item_close_id(visual.folder, visual.member))
                .size(px(16.0))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, on_close)
                .child(Icon::Close.canvas(theme, px(14.0))),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_browser::FolderSnapshot;
    use bbr_browser::SpaceId;
    use bbr_domain::{FolderId, TabId};

    fn folders() -> Vec<FolderSnapshot> {
        vec![
            FolderSnapshot {
                index: 0,
                id: FolderId(1),
                space: SpaceId(1),
                title: "Work".into(),
                members: vec![TabId(1), TabId(2)],
            },
            FolderSnapshot {
                index: 1,
                id: FolderId(2),
                space: SpaceId(1),
                title: "Reading".into(),
                members: vec![],
            },
        ]
    }

    #[test]
    fn folder_ids_are_deterministic_per_index() {
        assert_eq!(folder_row_id(0).to_string(), "bbr-folder-row-0");
        assert_eq!(folder_name_id(3).to_string(), "bbr-folder-name-3");
        assert_eq!(folder_count_id(3).to_string(), "bbr-folder-count-3");
        assert_eq!(folder_toggle_id(3).to_string(), "bbr-folder-toggle-3");
        assert_eq!(folder_item_id(1, 2).to_string(), "bbr-folder-item-1-2");
        assert_eq!(
            folder_item_title_id(1, 2).to_string(),
            "bbr-folder-item-1-2-title"
        );
        assert_eq!(
            folder_item_favicon_id(1, 2).to_string(),
            "bbr-folder-item-1-2-favicon"
        );
        assert_eq!(
            folder_item_close_id(1, 2).to_string(),
            "bbr-folder-item-1-2-close"
        );
    }

    #[test]
    fn presentation_defaults_to_collapsed_and_toggles_deterministically() {
        let mut presentation = FolderPresentation::default();
        assert!(!presentation.is_expanded(0));
        assert!(!presentation.is_expanded(7), "unknown reads collapsed");
        presentation.toggle(2);
        assert!(presentation.is_expanded(2));
        presentation.toggle(2);
        assert!(!presentation.is_expanded(2));
        assert!(!presentation.is_expanded(1), "toggling 2 leaves 1 alone");
    }

    #[test]
    fn folder_rows_derive_counts_and_expansion() {
        let mut presentation = FolderPresentation::default();
        presentation.toggle(0);
        let rows = folder_row_specs(&folders(), &presentation);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].index, 0);
        assert_eq!(rows[0].title, "Work");
        assert_eq!(rows[0].count, 2);
        assert!(rows[0].expanded);
        assert_eq!(rows[1].title, "Reading");
        assert_eq!(rows[1].count, 0);
        assert!(!rows[1].expanded);
    }

    #[test]
    fn folders_ids_hide_the_empty_region_and_mirror_expansion() {
        assert!(folders_ids(&[], &[]).is_empty());
        let mut presentation = FolderPresentation::default();
        let collapsed = folder_row_specs(&folders(), &presentation);
        let members = folder_member_visuals(&collapsed, &folders(), &tabs());
        let ids = folders_ids(&collapsed, &members);
        assert!(ids.contains(&"bbr-section-folders".to_string()));
        assert!(ids.contains(&"bbr-folders".to_string()));
        assert!(ids.contains(&"bbr-folder-row-0".to_string()));
        assert!(
            !ids.iter().any(|id| id.starts_with("bbr-folder-item")),
            "collapsed folders expose no nested ids"
        );
        presentation.toggle(0);
        let expanded = folder_row_specs(&folders(), &presentation);
        let members = folder_member_visuals(&expanded, &folders(), &tabs());
        let ids = folders_ids(&expanded, &members);
        for required in [
            "bbr-folder-item-0-0",
            "bbr-folder-item-0-0-title",
            "bbr-folder-item-0-0-favicon",
            "bbr-folder-item-0-0-close",
            "bbr-folder-item-0-1",
            "bbr-folder-item-0-1-title",
            "bbr-folder-item-0-1-favicon",
            "bbr-folder-item-0-1-close",
        ] {
            assert!(ids.contains(&required.to_string()), "missing {required}");
        }
        assert!(
            !ids.iter().any(|id| id.starts_with("bbr-folder-item-1")),
            "the collapsed folder 1 exposes no nested ids"
        );
    }

    #[test]
    fn member_visuals_resolve_ids_titles_and_monograms_in_folder_order() {
        let mut presentation = FolderPresentation::default();
        presentation.toggle(0);
        let rows = folder_row_specs(&folders(), &presentation);
        let visuals = folder_member_visuals(&rows, &folders(), &tabs());
        assert_eq!(visuals.len(), 2, "only the expanded folder's members");
        assert_eq!(visuals[0].folder, 0);
        assert_eq!(visuals[0].member, 0);
        assert_eq!(visuals[0].letter, 'E');
        assert_eq!(visuals[1].member, 1);
        assert_eq!(visuals[1].letter, 'L');
        assert!((0..6).contains(&visuals[0].hue));
        // Collapsed folder 1 contributes nothing even though it is empty.
        presentation.toggle(1);
        let rows = folder_row_specs(&folders(), &presentation);
        let visuals = folder_member_visuals(&rows, &folders(), &tabs());
        assert_eq!(visuals.len(), 2, "empty folder 1 adds no members");
    }

    fn tabs() -> Vec<bbr_browser::GlobalTabSnapshot> {
        vec![
            global_tab(1, 0, "https://example.com/a", "Example"),
            global_tab(2, 1, "http://localhost:8080/x", "Local"),
        ]
    }

    fn global_tab(id: u64, index: usize, url: &str, title: &str) -> bbr_browser::GlobalTabSnapshot {
        bbr_browser::GlobalTabSnapshot {
            id: bbr_domain::TabId(id),
            space: bbr_browser::SpaceId(1),
            profile: bbr_web::ProfileId(1),
            index,
            url: url.into(),
            title: title.into(),
            pinned: false,
            web_view: None,
            lifecycle: bbr_browser::TabLifecycle::Warm,
            last_used: 0,
        }
    }
}
