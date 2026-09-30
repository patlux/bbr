//! DESIGN.md §11.11 Folders: session-local folder rows leading with the §7
//! folder glyph (open/closed variant carrying the disclosure state), name,
//! and a hover/focus-revealed `type:meta` count, plus nested §11.1 rows for
//! the expanded folder.
//!
//! A folder is a session-local container of tab identities (in-memory in
//! v1, never persisted; Phase 7 owns restoration). Rows render
//! `[folder glyph open/closed with the disclosure state, icon:m 18; S
//! icon:s 16] [name, trunc] [count, type:meta tnum, hover/focus-revealed]`
//! at `density:row` tall, stepping on the §8 pitch; nested rows indent one
//! `sp:3` step and reuse the §11.1 row anatomy. Toggling expands/collapses
//! and snaps (`dur:instant`); the app layer holds the presentation state
//! only ([`FolderPresentation`] — the domain model is L5-owned).
//!
//! This module owns the folder identity contract (ids, the row spec types,
//! the presentation state) and the folder-row anatomy. The snapshot→spec
//! derivations (`folder_row_specs`, `folder_member_visuals`) stay
//! app-owned adapters that build [`FolderRowSpec`]/[`FolderMemberVisual`]
//! from the controller read model; the nested-row rendering consumes member
//! tab visuals and is wired by the rail composition; the id derivation for
//! nested rows is already contract.

use gpui::{
    div, img, prelude::*, App, Div, Image, ImageSource, MouseButton, MouseDownEvent, ObjectFit,
    SharedString, Stateful, Styled, Window,
};
use std::sync::Arc;

use crate::icons::Icon;
use crate::palette::FaviconSlate;
use crate::theme::{elevation, Radius, Spacing, Theme, ThemeMode};

mod rename;
use rename::rename_slot;
pub use rename::FolderRenameField;

/// The §15 id of the folders list container.
pub const FOLDERS_ID: &str = "bbr-folders";
/// The §15 id of the "Folders" section label (hidden when empty).
#[allow(dead_code)] // §15 visible-ids seam; consumed by the app's snapshot
pub const FOLDERS_SECTION_ID: &str = "bbr-section-folders";

/// The §15 id of the inline rename field (blueprint D20). At most one
/// folder is in rename, so the id is singular; it replaces that row's
/// name slot while the rename is open.
pub const FOLDER_RENAME_ID: &str = "bbr-folder-rename";

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
    /// Whether the nested row list is expanded (`dur:instant` snap). The
    /// folder row's leading glyph carries this state (open/closed variant).
    pub expanded: bool,
    /// §11.11 hover/focus-reveal: the `type:meta` count paints only while
    /// the pointer hovers the row or the row is the keyboard focus anchor
    /// (`true`), and stays quiet (zero-opacity, id still present) at rest.
    pub reveal_count: bool,
    /// Blueprint D20: the open inline rename of this folder (`None` at
    /// rest). The row paints as a focused field and its name slot shows
    /// the draft instead of the committed title.
    pub rename: Option<FolderRenameField>,
    /// §10.9/§11.11: a dragged tab is over this row — it takes the
    /// `drag-target` language (1 pt accent outline on `tint.accent`) and a
    /// drop moves the tab into the folder.
    pub drop_target: bool,
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
    /// The resolved favicon image for the member's host in its owning
    /// profile's bounded store, if acquired (task packet
    /// `real-favicon-pipeline.md`): the nested row renders it instead of
    /// the monogram — the same record the member's tab row derives from,
    /// one acquisition, every presentation.
    pub image: Option<Arc<Image>>,
    /// Whether this member is the window's selected tab: the nested row then
    /// presents the §10.4 selected surface with the static `elev:lift` (D2),
    /// exactly like the tab row it mirrors.
    pub selected: bool,
    /// §10.12: the member is closed in place — favicon and title grey out
    /// and the row offers no close control; a press reopens it.
    pub closed: bool,
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

/// One §11.11 folder row at `density:row` tall, stepping on the §8 pitch
/// (`density:pitch` = surface + gap) like every other row: the §7 folder
/// glyph (open/closed variant carrying the disclosure state, never a
/// separate disclosure chevron), the truncated name, and the `type:meta`
/// `tnum` count that paints only on hover/focus (`spec.reveal_count`) —
/// the row stays quiet at rest, the `bbr-folder-count-<n>` id stays in
/// the tree either way. `on_hover`/`on_toggle` are the production
/// listeners; the rail composition wires them. Nested rows are appended
/// below by the composition when `spec.expanded` (they reuse the §11.1
/// row anatomy and are not part of this module's anatomy).
pub fn folder_row(
    theme: &Theme,
    spec: &FolderRowSpec,
    on_hover: impl Fn(&bool, &mut Window, &mut App) + 'static,
    on_toggle: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let reveal = spec.reveal_count && spec.rename.is_none();
    let row = div()
        .id(folder_row_id(spec.index))
        .debug_selector(move || folder_row_id(spec.index).to_string())
        .h(theme.density.row())
        // §8 pitch: the folder row steps on surface + gap like every
        // regular/pinned/nested/New Tab row, so the region extent stays an
        // exact pitch multiple (§11.6).
        .mb(theme.density.gap())
        .px(theme.density.row_padding_x())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(theme.density.density_radius())
        // Rest paints no surface: the (space-tinted) rail shows through.
        .cursor_pointer()
        .on_hover(on_hover)
        .child(
            div()
                .id(folder_toggle_id(spec.index))
                .debug_selector(move || folder_toggle_id(spec.index).to_string())
                .size(theme.density.icon())
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, on_toggle)
                .child(
                    Icon::Folder {
                        open: spec.expanded,
                    }
                    .canvas(theme, theme.density.icon()),
                ),
        )
        .child(match &spec.rename {
            Some(field) => rename_slot(theme, spec.index, field),
            None => div()
                .id(folder_name_id(spec.index))
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                // S12 (Paper): folder names read as group headings — the
                // semibold weight in `ink`.
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(p.ink)
                .child(spec.title.clone()),
        })
        .child(
            div()
                .id(folder_count_id(spec.index))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                // §11.11 hover/focus-reveal: the count paints only while
                // the row is hovered or the keyboard focus anchor; at rest
                // the id stays in the tree at zero opacity (quiet rest,
                // stable §15 id).
                .opacity(if reveal { 1.0 } else { 0.0 })
                .child(spec.count.to_string()),
        );
    // Blueprint D20 (board S-4 "Rename inline"): the renaming row reads as
    // a focused field — the `field` surface, a 1 pt accent edge and the
    // `tint-accent` halo — so the rename is never color-only (the caret
    // and the field shape carry it too).
    if spec.drop_target {
        // §10.9 drag-target: the accent outline on the `tint.accent` drop
        // fill (§18.2), the same language as every drop region.
        return row.bg(p.tint_accent).border_1().border_color(p.accent);
    }
    if spec.rename.is_some() {
        row.bg(p.surface_field)
            .border_1()
            .border_color(p.accent)
            .shadow(vec![gpui::BoxShadow {
                color: p.tint_accent.into(),
                offset: gpui::point(gpui::px(0.0), gpui::px(0.0)),
                blur_radius: gpui::px(0.0),
                spread_radius: gpui::px(2.0),
                inset: false,
            }])
    } else {
        row
    }
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
        if !member.closed {
            ids.push(folder_item_close_id(member.folder_index, member.member).to_string());
        }
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
    let size = theme.density.icon();
    let radius = Radius.small();
    let badge_size = theme.type_.badge.size;
    let badge_weight = theme.type_.badge.weight;
    let glass = FaviconSlate::letter(theme.mode);
    let monogram_style = MonogramStyle {
        size,
        radius,
        badge_size,
        badge_weight,
        slate,
        glass,
    };
    // Owned copies for the `'static` img fallback closure (the member
    // visual is borrowed for this render only).
    let folder_index = visual.folder_index;
    let member = visual.member;
    let letter = visual.letter;
    // The nested-row favicon slot: the §7.2 monogram, or the resolved
    // favicon image when the member's owning profile store holds one (the
    // same record the member's tab row derives from). A decode failure
    // falls back to the monogram silently (the acquisition contract's
    // fallback rule).
    let favicon_slot: gpui::AnyElement = match visual.image.clone() {
        Some(image) => img(ImageSource::Image(image))
            .id(folder_item_favicon_id(folder_index, member))
            .size(size)
            .rounded(radius)
            .object_fit(ObjectFit::Contain)
            .with_fallback(move || {
                folder_item_monogram(folder_index, member, monogram_style, letter).into_any()
            })
            .into_any(),
        None => folder_item_monogram(folder_index, member, monogram_style, letter).into_any(),
    };
    let closed = visual.closed;
    // §10.12: a closed member's favicon greys out.
    let favicon_slot = div()
        .flex_none()
        .opacity(if closed {
            crate::state::CLOSED_ICON_OPACITY
        } else {
            1.0
        })
        .child(favicon_slot);
    div()
        .id(folder_item_id(visual.folder_index, visual.member))
        .debug_selector(move || folder_item_id(visual.folder_index, visual.member).to_string())
        .h(theme.density.row())
        // §8 pitch: nested rows step on surface + gap like every other
        // row, so the expanded folder's extent stays an exact pitch
        // multiple (§11.6).
        .mb(theme.density.gap())
        // S12 (Paper): nested rows indent one `sp:3` step past the row
        // padding, so members sit visibly inside their folder.
        .pl(theme.density.row_padding_x() + Spacing::sp3())
        .pr(theme.density.row_padding_x())
        .group(folder_item_id(visual.folder_index, visual.member))
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .relative()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_select)
        // §10.4/D2: the selected member lifts on the same inset rounded
        // surface as the tab row it mirrors (`density:row-inset` from the
        // rail edges, `density:radius`); geometry is unchanged.
        .when(visual.selected, |el| {
            el.child(
                div()
                    .absolute()
                    .left(theme.density.row_inset())
                    .right(theme.density.row_inset())
                    .top(gpui::px(0.0))
                    .bottom(gpui::px(0.0))
                    .rounded(theme.density.density_radius())
                    .bg(p.tint_selected)
                    .shadow(elevation::lift(theme)),
            )
        })
        .child(favicon_slot)
        .child(
            div()
                .id(folder_item_title_id(visual.folder_index, visual.member))
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(if closed { p.ink_faint } else { p.ink })
                .child(visual.title.clone()),
        )
        // §10.12: a closed member is already closed — no close control.
        .when(!closed, |row| {
            row.child(
                div()
                    .id(folder_item_close_id(visual.folder_index, visual.member))
                    .debug_selector(move || {
                        folder_item_close_id(visual.folder_index, visual.member).to_string()
                    })
                    .size(theme.density.hit_min_close())
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_mouse_down(MouseButton::Left, on_close)
                    // S12 (Paper): the close control is revealed while the row
                    // is hovered or selected, like a tab row; at rest it keeps
                    // its id and hit square at zero opacity.
                    .opacity(if visual.selected { 1.0 } else { 0.0 })
                    .group_hover(
                        folder_item_id(visual.folder_index, visual.member),
                        |style| style.opacity(1.0),
                    )
                    .child(Icon::Close.canvas(theme, theme.density.icon())),
            )
        })
}

/// The §7.2 monogram tile geometry + palette inputs of one folder-member
/// row (a named bundle so the monogram builder and the image fallback
/// closure stay under the seven-argument lint budget).
#[derive(Clone, Copy)]
struct MonogramStyle {
    size: gpui::Pixels,
    radius: gpui::Pixels,
    badge_size: gpui::Pixels,
    badge_weight: gpui::FontWeight,
    slate: gpui::Rgba,
    glass: gpui::Rgba,
}

/// The §7.2 monogram tile of one nested folder-member row (also the image
/// decode-failure fallback): the letter on the slate tile at the density
/// icon size, `radius:sm`, with the stable `bbr-folder-item-<n>-<m>-favicon`
/// id.
fn folder_item_monogram(
    folder_index: usize,
    member: usize,
    style: MonogramStyle,
    letter: char,
) -> Stateful<Div> {
    div()
        .id(folder_item_favicon_id(folder_index, member))
        .size(style.size)
        .rounded(style.radius)
        .bg(style.slate)
        .flex()
        .items_center()
        .justify_center()
        .text_size(style.badge_size)
        .font_weight(style.badge_weight)
        .text_color(style.glass)
        .child(SharedString::from(letter.to_string()))
}

#[cfg(test)]
#[path = "folders/tests.rs"]
mod tests;
