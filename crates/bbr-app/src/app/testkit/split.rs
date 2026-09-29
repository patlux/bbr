//! Blueprint D23 testkit seams: the shown split group on the wire and one
//! interaction through the method its control calls.

use bbr_domain::tiling::{Axis, Branch, Edge, SplitNode};
use bbr_testkit::protocol::{SplitInput, SplitPaneSnapshot, SplitSnapshot};
use gpui::{Context, Window};

use crate::app::BbrApp;

fn edge(name: &str) -> Option<Edge> {
    Some(match name {
        "left" => Edge::Left,
        "right" => Edge::Right,
        "top" => Edge::Top,
        "bottom" => Edge::Bottom,
        _ => return None,
    })
}

fn tree(branch: &Branch, view: &dyn Fn(bbr_domain::TabId) -> u64) -> String {
    let children: Vec<String> = branch
        .children
        .iter()
        .map(|child| match &child.node {
            SplitNode::Pane(tab) => format!("{}:{}", child.weight, view(*tab)),
            SplitNode::Branch(inner) => format!("{}:{}", child.weight, tree(inner, view)),
        })
        .collect();
    let axis = match branch.axis {
        Axis::Row => 'R',
        Axis::Column => 'C',
    };
    format!("{axis}({})", children.join(","))
}

impl BbrApp {
    pub(crate) fn split_snapshot(&self) -> Option<SplitSnapshot> {
        let (layout, tiling) = self.split_projection()?;
        let rect = |r: bbr_web::WebRect| [r.x, r.y, r.width, r.height];
        let groups = {
            let snapshot = self.controller.snapshot();
            self.controller
                .split_groups()
                .into_iter()
                .find(|(space, _)| *space == snapshot.active_space)
                .map_or(0, |(_, groups)| groups.len())
        };
        Some(SplitSnapshot {
            panes: tiling
                .panes
                .iter()
                .filter_map(|pane| {
                    Some(SplitPaneSnapshot {
                        view: layout.view_of(pane.tab)?,
                        rect: rect(pane.rect),
                    })
                })
                .collect(),
            dividers: tiling.dividers.iter().map(|d| rect(d.rect)).collect(),
            focused: layout.focused,
            tree: tree(layout.group.root(), &|tab| {
                layout.view_of(tab).map_or(0, |view| view.0)
            }),
            collapsed: self.split_collapsed(),
            list_open: self.split_view.list_open,
            groups,
            dragging: self.split_view.drag.is_some(),
        })
    }

    /// Returns whether the input applied.
    pub(crate) fn split_input(
        &mut self,
        input: SplitInput,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match input {
            SplitInput::Insert {
                beside,
                new,
                edge: name,
            } => {
                let Some(edge) = edge(&name) else {
                    return false;
                };
                match self.controller.split_insert(beside, new, edge) {
                    Ok(event) => {
                        self.emit_browser(&event);
                        if let Some(tab) = self.tab_id_at(new) {
                            self.keep_split_block(tab, cx);
                        }
                        self.persist_structure();
                        cx.notify();
                        true
                    }
                    Err(_) => false,
                }
            }
            SplitInput::DragDivider { index, to } => {
                let Some(divider) = self
                    .split_projection()
                    .and_then(|(_, tiling)| tiling.dividers.get(index).cloned())
                else {
                    return false;
                };
                let (x, y) = match divider.axis {
                    Axis::Row => (to, divider.rect.y + 1.0),
                    Axis::Column => (divider.rect.x + 1.0, to),
                };
                self.split_view.drag = Some(divider);
                self.split_divider_to(x, y, cx);
                self.split_view.drag = None;
                self.persist_structure();
                true
            }
            SplitInput::DoubleClickDivider { index } => {
                let Some(path) = self
                    .split_projection()
                    .and_then(|(_, tiling)| tiling.dividers.get(index).map(|d| d.path.clone()))
                else {
                    return false;
                };
                self.split_equalize(&path, cx);
                true
            }
            SplitInput::PointerDown { x, y } => {
                self.split_pointer_down(x, y, cx);
                true
            }
            SplitInput::Chip { row } => {
                if self.split_collapsed().is_none() {
                    return false;
                }
                self.split_chip_press(row, cx);
                true
            }
        }
    }
}
