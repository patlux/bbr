//! The tree surface (packet `component-gallery-mvp.md`, live-review
//! finding 1): renders the `TreeState` as the keyboard-operable component
//! tree with stable ids (`bbr-gallery-tree`, `bbr-gallery-node-<n>`,
//! `bbr-gallery-story-<id>`, `bbr-gallery-disclosure-<group>`). Group rows
//! expose a real pointer target on the visible disclosure glyph (and the
//! whole row); story rows are clickable; every pointer route runs through
//! the same activation the keyboard uses (`GalleryApp::activate_node` →
//! `TreeState::activate`), so expansion stays one authority and focus
//! semantics stay one. The catalog header carries the honest scope copy
//! (finding 3).

use std::rc::Rc;

use bbr_ui::icons::Icon;

use crate::registry::{GroupId, NodeId};
use crate::tree::{NodeKind, TreeState};

/// The stable pointer target of a group's disclosure glyph: clicking it
/// (or the group row) toggles the group through the **same** `TreeState`
/// expansion authority the keyboard uses (`activate` → `expanded[]`),
/// visibly flipping the chevron and hiding/showing the group's story rows.
pub fn disclosure_id(group: GroupId) -> String {
    format!("bbr-gallery-disclosure-{}", group.0)
}

/// The activation callback the pointer targets route clicks through.
type ActivateCallback = Rc<dyn Fn(NodeId, &mut gpui::Window, &mut gpui::App)>;

pub fn tree_surface(
    tree: &TreeState,
    theme: &bbr_ui::theme::Theme,
    on_activate: impl Fn(NodeId, &mut gpui::Window, &mut gpui::App) + 'static,
) -> gpui::AnyElement {
    use crate::stories::{GALLERY_CONTINUATION, GALLERY_SCOPE};
    use gpui::{div, prelude::*, px, SharedString};
    let on_activate: ActivateCallback = Rc::new(on_activate);
    let visible = tree.visible();
    let rows: Vec<gpui::AnyElement> = visible
        .iter()
        .map(|&index| {
            let node = tree.nodes[index];
            let focused = index == tree.focus;
            let on_activate = on_activate.clone();
            let row = div()
                .id(SharedString::from(format!(
                    "bbr-gallery-node-{}",
                    node.id.0
                )))
                .debug_selector(move || format!("bbr-gallery-node-{}", node.id.0))
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .rounded(theme.radius.small())
                .cursor_pointer()
                .bg(if focused {
                    theme.palette.tint_selected
                } else {
                    gpui::rgba(0x00000000)
                })
                .border_1()
                .border_color(if focused {
                    theme.palette.accent
                } else {
                    gpui::rgba(0x00000000)
                })
                .text_size(theme.type_.label.size)
                .text_color(if focused {
                    theme.palette.ink
                } else {
                    theme.palette.ink_secondary
                });
            let row = match node.kind {
                NodeKind::Group(group) => {
                    // The group row is itself a pointer target: a click
                    // routes through the same expansion authority as
                    // Enter/Space/Left/Right on the focused group.
                    let row_on_activate = on_activate.clone();
                    let disclosure_on_activate = on_activate.clone();
                    let disclosure_target = disclosure_id(group);
                    row.on_click(move |_event, window, cx| {
                        row_on_activate(node.id, window, cx);
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .id(SharedString::from(disclosure_target.clone()))
                            .debug_selector(move || disclosure_target.clone())
                            .cursor_pointer()
                            .on_click(move |_event, window, cx| {
                                disclosure_on_activate(node.id, window, cx);
                                cx.stop_propagation();
                            })
                            .child(
                                Icon::Disclosure {
                                    expanded: tree.expanded[group.0 as usize],
                                }
                                .canvas(theme, px(12.0)),
                            ),
                    )
                    .child(node.label)
                }
                NodeKind::Story(id) => row
                    .px_4()
                    .on_click(move |_event, window, cx| {
                        on_activate(node.id, window, cx);
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .id(SharedString::from(format!("bbr-gallery-story-{}", id.0)))
                            .child(node.label),
                    ),
            };
            row.into_any()
        })
        .collect();
    div()
        .id("bbr-gallery-tree")
        .flex()
        .flex_col()
        .size_full()
        .bg(theme.palette.surface_sidebar)
        .child(
            div()
                .flex()
                .flex_col()
                .px_3()
                .py_2()
                .gap_1()
                .border_b_1()
                .border_color(theme.palette.hairline)
                .child(
                    div()
                        .id("bbr-gallery-title")
                        .debug_selector(|| "bbr-gallery-title".to_string())
                        .text_size(theme.type_.title.size)
                        .font_weight(theme.type_.title.weight)
                        .text_color(theme.palette.ink)
                        .child("BBR Gallery"),
                )
                .child(
                    div()
                        .id("bbr-gallery-scope")
                        .debug_selector(|| "bbr-gallery-scope".to_string())
                        .text_size(theme.type_.badge.size)
                        .font_weight(theme.type_.badge.weight)
                        .text_color(theme.palette.ink_secondary)
                        .child(GALLERY_SCOPE),
                )
                .child(
                    div()
                        .id("bbr-gallery-continuation")
                        .debug_selector(|| "bbr-gallery-continuation".to_string())
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(GALLERY_CONTINUATION),
                ),
        )
        .children(rows)
        .into_any()
}
