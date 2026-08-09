//! DESIGN.md §11.11 Folders: session-local folder rows with a disclosure
//! control, name, and count, plus nested §11.1 rows for the expanded
//! folder.
//!
//! A folder is a session-local container of tab identities (in-memory in
//! v1, never persisted; Phase 7 owns restoration). Rows render
//! `[disclosure glyph icon:m 18; S icon:s 16] [name, trunc] [count,
//! type:meta tnum]` at `density:row` tall; nested rows indent one `sp:3`
//! step and reuse the §11.1 row anatomy. Toggling expands/collapses and
//! snaps (`dur:instant`); the app layer holds the presentation state only
//! ([`FolderPresentation`] — the domain model is L5-owned).
//!
//! This module owns the folder identity contract (ids, the row spec types,
//! the presentation state) and the folder-row anatomy. The snapshot→spec
//! derivations (`folder_row_specs`, `folder_member_visuals`) stay
//! app-owned adapters that build [`FolderRowSpec`]/[`FolderMemberVisual`]
//! from the controller read model; the nested-row rendering consumes member
//! tab visuals and is wired by the rail composition; the id derivation for
//! nested rows is already contract.

use gpui::{
    div, prelude::*, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled, Window,
};

use crate::icons::Icon;
use crate::palette::FaviconSlate;
use crate::theme::{Radius, Spacing, Theme, ThemeMode};

/// The §15 id of the folders list container.
pub const FOLDERS_ID: &str = "bbr-folders";
/// The §15 id of the "Folders" section label (hidden when empty).
#[allow(dead_code)] // §15 visible-ids seam; consumed by the app's snapshot
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
    /// The domain folder id (the L5 `MoveIntoFolder` command's target and
    /// the identity every folder command keys on).
    pub id: bbr_domain::FolderId,
    pub title: String,
    /// Member tab count (§11.11 count column, `type:meta` `tnum`).
    pub count: usize,
    /// Whether the nested row list is expanded (`dur:instant` snap).
    pub expanded: bool,
}

/// One nested §11.1 row's rendered identity inside an expanded folder:
/// the owning folder's domain id plus the folder/nested positions that key
/// the stable `bbr-folder-item-<n>-<m>` ids, and the member's display
/// title, §7.2 monogram letter, and slate hue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderMemberVisual {
    /// The owning folder's domain id (identity; the surface filter and the
    /// member commands key on it).
    pub folder: bbr_domain::FolderId,
    /// The owning folder's creation-order position (`<n>` in the
    /// `bbr-folder-item-<n>-<m>` ids — position-derived under the Z2-1
    /// default).
    pub folder_index: usize,
    /// The nested position within the folder, 0-based (`<m>`).
    pub member: usize,
    pub title: String,
    pub letter: char,
    /// §7.2 FNV slate-hue ramp index (0..6), from the member's URL.
    pub hue: usize,
}

/// The session-local folder presentation state the app layer owns
/// (DESIGN.md §11.11: "the app layer may hold folder presentation state
/// only"): one expanded/collapsed value per folder, keyed by the stable
/// domain [`bbr_domain::FolderId`], defaulting to collapsed. Entries are
/// kept in deterministic first-toggle order; unknown ids (a folder
/// created after the last toggle) read collapsed. Keying by `FolderId`
/// (not the creation-order index) keeps expansion state stable across any
/// future folder deletion or reorder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderPresentation {
    expanded: Vec<(bbr_domain::FolderId, bool)>,
}

impl FolderPresentation {
    /// Whether the folder with domain id `id` is expanded. Unknown ids
    /// (a folder created after the last toggle) read collapsed.
    pub fn is_expanded(&self, id: bbr_domain::FolderId) -> bool {
        self.expanded
            .iter()
            .find(|(folder, _)| *folder == id)
            .map(|(_, expanded)| *expanded)
            .unwrap_or(false)
    }

    /// Toggle the folder with domain id `id` (collapsed → expanded and
    /// back), appending a deterministic entry for folders toggled for the
    /// first time.
    pub fn toggle(&mut self, id: bbr_domain::FolderId) {
        if let Some((_, expanded)) = self.expanded.iter_mut().find(|(folder, _)| *folder == id) {
            *expanded = !*expanded;
        } else {
            self.expanded.push((id, true));
        }
    }
}

/// One §11.11 folder row: `[disclosure glyph icon:m 18; S icon:s 16]
/// [name, trunc] [count, type:meta tnum]` at `density:row` tall.
/// `on_toggle` is the production disclosure listener; the rail composition
/// wires it. Nested rows are appended below by the composition when
/// `spec.expanded` (they reuse the §11.1 row anatomy and are not part of
/// this module's anatomy).
pub fn folder_row(
    theme: &Theme,
    spec: &FolderRowSpec,
    on_toggle: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(folder_row_id(spec.index))
        .h(theme.density.row())
        .px(theme.density.row_padding_x())
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
                .size(theme.density.icon())
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, on_toggle)
                .child(
                    Icon::Disclosure {
                        expanded: spec.expanded,
                    }
                    .canvas(theme, theme.density.icon()),
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
#[allow(dead_code)] // §15 visible-ids seam; consumed by the app's snapshot
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
        ids.push(folder_item_id(member.folder_index, member.member).to_string());
        ids.push(folder_item_title_id(member.folder_index, member.member).to_string());
        ids.push(folder_item_favicon_id(member.folder_index, member.member).to_string());
        ids.push(folder_item_close_id(member.folder_index, member.member).to_string());
    }
    ids
}

/// One §11.11 nested row (`bbr-folder-item-<n>-<m>`): the §11.1 anatomy
/// `[favicon 18] · [title flex, trunc] · [close hit-min.close]` at
/// `density:row` tall, indented one `sp:3` step. `on_select` activates the
/// member tab, `on_close` closes it — the close listener is expected to
/// stop propagation so a close press never also selects the member row.
pub fn folder_item_row(
    theme: &Theme,
    visual: &FolderMemberVisual,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let slate = match theme.mode {
        ThemeMode::Dark => FaviconSlate::dark(visual.hue),
        ThemeMode::Light => FaviconSlate::light(visual.hue),
    };
    let monogram = div()
        .id(folder_item_favicon_id(visual.folder_index, visual.member))
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
        .id(folder_item_id(visual.folder_index, visual.member))
        .h(theme.density.row())
        .pl(Spacing::sp3())
        .pr(theme.density.row_padding_x())
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
                .id(folder_item_title_id(visual.folder_index, visual.member))
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
                .id(folder_item_close_id(visual.folder_index, visual.member))
                .size(theme.density.hit_min_close())
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, on_close)
                .child(Icon::Close.canvas(theme, theme.density.icon())),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<FolderRowSpec> {
        vec![
            FolderRowSpec {
                index: 0,
                id: bbr_domain::FolderId(1),
                title: "Work".into(),
                count: 2,
                expanded: false,
            },
            FolderRowSpec {
                index: 1,
                id: bbr_domain::FolderId(2),
                title: "Reading".into(),
                count: 0,
                expanded: false,
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
        assert!(!presentation.is_expanded(bbr_domain::FolderId(0)));
        assert!(
            !presentation.is_expanded(bbr_domain::FolderId(7)),
            "unknown reads collapsed"
        );
        presentation.toggle(bbr_domain::FolderId(2));
        assert!(presentation.is_expanded(bbr_domain::FolderId(2)));
        presentation.toggle(bbr_domain::FolderId(2));
        assert!(!presentation.is_expanded(bbr_domain::FolderId(2)));
        assert!(
            !presentation.is_expanded(bbr_domain::FolderId(1)),
            "toggling 2 leaves 1 alone"
        );
    }

    #[test]
    fn folder_rows_carry_counts_and_expansion() {
        let mut presentation = FolderPresentation::default();
        presentation.toggle(bbr_domain::FolderId(1));
        let mut rows = rows();
        rows[0].expanded = presentation.is_expanded(bbr_domain::FolderId(1));
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
        let rows = rows();
        let ids = folders_ids(&rows, &[]);
        assert!(ids.contains(&"bbr-section-folders".to_string()));
        assert!(ids.contains(&"bbr-folders".to_string()));
        assert!(ids.contains(&"bbr-folder-row-0".to_string()));
        assert!(
            !ids.iter().any(|id| id.starts_with("bbr-folder-item")),
            "collapsed folders expose no nested ids"
        );
        let members = [
            FolderMemberVisual {
                folder: bbr_domain::FolderId(1),
                folder_index: 0,
                member: 0,
                title: "Example".into(),
                letter: 'E',
                hue: 0,
            },
            FolderMemberVisual {
                folder: bbr_domain::FolderId(1),
                folder_index: 0,
                member: 1,
                title: "Local".into(),
                letter: 'L',
                hue: 1,
            },
        ];
        let ids = folders_ids(&rows, &members);
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
            "folder 1 exposes no nested ids"
        );
    }

    #[test]
    fn member_visuals_carry_folder_order_identity() {
        let members = [
            FolderMemberVisual {
                folder: bbr_domain::FolderId(1),
                folder_index: 0,
                member: 0,
                title: "Example".into(),
                letter: 'E',
                hue: 0,
            },
            FolderMemberVisual {
                folder: bbr_domain::FolderId(1),
                folder_index: 0,
                member: 1,
                title: "Local".into(),
                letter: 'L',
                hue: 1,
            },
        ];
        assert_eq!(members[0].folder, bbr_domain::FolderId(1));
        assert_eq!(members[0].folder_index, 0);
        assert_eq!(members[0].member, 0);
        assert_eq!(members[0].letter, 'E');
        assert_eq!(members[1].member, 1);
        assert_eq!(members[1].letter, 'L');
        assert!((0..6).contains(&members[0].hue));
    }
}
