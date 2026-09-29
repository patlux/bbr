//! Blueprint D23 (DESIGN §11.7): the geometry of a split group — a
//! `bbr_domain::tiling` tree projected onto the content card.
//!
//! [`project`] lays a group out in whole points: each branch shares its
//! length (minus the 2 pt dividers between its children) by the children's
//! weights, so a pane's rect is a pure function of the tree and the card.
//! The projection answers the questions the shell asks: whether every pane
//! keeps the 320 × 200 pt minimum ([`Tiling::fits`]), which pane lies next
//! to the focused one ([`Tiling::neighbour`], ⌃⌘+arrow), which divider is
//! under the pointer and where a drag leaves it ([`drag_weight`]), and
//! which pane edge a dragged tab would drop on ([`Tiling::drop_target`]).

use bbr_domain::tiling::{Axis, Branch, Edge, SplitNode, WEIGHT_TOTAL};
use bbr_domain::TabId;
use bbr_web::WebRect;

/// The narrowest a pane may get before the group collapses.
pub const PANE_MIN_WIDTH: f32 = 320.0;
/// The lowest a pane may get before the group collapses.
pub const PANE_MIN_HEIGHT: f32 = 200.0;
/// The divider between two panes.
pub const TILING_DIVIDER: f32 = 2.0;
/// The share of a pane's width (or height) its edge drop zones take.
pub const DROP_EDGE_SHARE: f32 = 0.24;

/// One pane at its rect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TiledPane {
    pub tab: TabId,
    pub rect: WebRect,
}

/// One divider: between children `index` and `index + 1` of the branch at
/// `path`, laid along `axis` (a row's dividers are vertical lines).
#[derive(Debug, Clone, PartialEq)]
pub struct TiledDivider {
    pub path: Vec<usize>,
    pub index: usize,
    pub axis: Axis,
    pub rect: WebRect,
    /// Along the axis: where the leading neighbour starts and the trailing
    /// neighbour ends.
    pub start: f32,
    pub end: f32,
    /// The two neighbours' combined weight.
    pub combined: u32,
}

/// Where a dragged tab would land on a pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropZone {
    /// Beside the pane, at this edge.
    Edge(Edge),
    /// In place of the pane.
    Center,
}

/// A projected group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tiling {
    /// In reading order.
    pub panes: Vec<TiledPane>,
    pub dividers: Vec<TiledDivider>,
}

/// Lay `root` out over `content` with `divider`-wide dividers.
pub fn project(root: &Branch, content: WebRect, divider: f32) -> Tiling {
    let mut tiling = Tiling::default();
    let mut path = Vec::new();
    layout(root, content, divider, &mut path, &mut tiling);
    tiling
}

fn layout(branch: &Branch, rect: WebRect, d: f32, path: &mut Vec<usize>, out: &mut Tiling) {
    let n = branch.children.len();
    let (origin, length) = match branch.axis {
        Axis::Row => (rect.x, rect.width),
        Axis::Column => (rect.y, rect.height),
    };
    let available = (length - d * (n.saturating_sub(1)) as f32).max(0.0);
    let mut cumulative = 0u64;
    let mut bounds = Vec::with_capacity(n + 1);
    bounds.push(0.0f32);
    for child in &branch.children {
        cumulative += u64::from(child.weight);
        bounds.push((available * cumulative as f32 / WEIGHT_TOTAL as f32).round());
    }
    let span = |i: usize| {
        let start = origin + bounds[i] + d * i as f32;
        let end = origin + bounds[i + 1] + d * i as f32;
        (start, end)
    };
    let child_rect = |start: f32, end: f32| match branch.axis {
        Axis::Row => WebRect::new(start, rect.y, end - start, rect.height),
        Axis::Column => WebRect::new(rect.x, start, rect.width, end - start),
    };
    for (i, child) in branch.children.iter().enumerate() {
        let (start, end) = span(i);
        let child_rect = child_rect(start, end);
        match &child.node {
            SplitNode::Pane(tab) => out.panes.push(TiledPane {
                tab: *tab,
                rect: child_rect,
            }),
            SplitNode::Branch(inner) => {
                path.push(i);
                layout(inner, child_rect, d, path, out);
                path.pop();
            }
        }
        if i + 1 < n {
            let next_end = span(i + 1).1;
            out.dividers.push(TiledDivider {
                path: path.clone(),
                index: i,
                axis: branch.axis,
                rect: match branch.axis {
                    Axis::Row => WebRect::new(end, rect.y, d, rect.height),
                    Axis::Column => WebRect::new(rect.x, end, rect.width, d),
                },
                start,
                end: next_end,
                combined: child.weight + branch.children[i + 1].weight,
            });
        }
    }
}

/// The leading neighbour's weight that puts `divider` at `pointer` (along
/// its axis), keeping both neighbours at least `min_len` long (or equal
/// when they cannot both be).
pub fn drag_weight(divider: &TiledDivider, pointer: f32, min_len: f32, divider_width: f32) -> u32 {
    let content = (divider.end - divider.start - divider_width).max(1.0);
    let wanted = pointer - divider_width / 2.0 - divider.start;
    let leading = if content < 2.0 * min_len {
        content / 2.0
    } else {
        wanted.clamp(min_len, content - min_len)
    };
    let weight = (f64::from(divider.combined) * f64::from(leading) / f64::from(content)).round();
    (weight as u32).clamp(1, divider.combined.saturating_sub(1).max(1))
}

fn contains(rect: &WebRect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// How much two spans overlap.
fn overlap(a0: f32, a1: f32, b0: f32, b1: f32) -> f32 {
    (a1.min(b1) - a0.max(b0)).max(0.0)
}

impl Tiling {
    /// Whether every pane keeps the minimum size.
    pub fn fits(&self) -> bool {
        self.panes
            .iter()
            .all(|p| p.rect.width >= PANE_MIN_WIDTH && p.rect.height >= PANE_MIN_HEIGHT)
    }

    pub fn pane(&self, tab: TabId) -> Option<&TiledPane> {
        self.panes.iter().find(|p| p.tab == tab)
    }

    /// The pane next to `from` toward `edge`: the nearest one past that
    /// edge that overlaps it across the axis (the larger overlap, then
    /// reading order, breaks ties).
    pub fn neighbour(&self, from: TabId, edge: Edge) -> Option<TabId> {
        let f = self.pane(from)?.rect;
        let (fl, ft, fr, fb) = (f.x, f.y, f.x + f.width, f.y + f.height);
        let mut best: Option<(f32, f32, TabId)> = None;
        for pane in self.panes.iter().filter(|p| p.tab != from) {
            let r = pane.rect;
            let (l, t, rr, b) = (r.x, r.y, r.x + r.width, r.y + r.height);
            let (gap, shared) = match edge {
                Edge::Right if l >= fr - 0.5 => (l - fr, overlap(ft, fb, t, b)),
                Edge::Left if rr <= fl + 0.5 => (fl - rr, overlap(ft, fb, t, b)),
                Edge::Bottom if t >= fb - 0.5 => (t - fb, overlap(fl, fr, l, rr)),
                Edge::Top if b <= ft + 0.5 => (ft - b, overlap(fl, fr, l, rr)),
                _ => continue,
            };
            if shared <= 0.0 {
                continue;
            }
            let better = match best {
                None => true,
                Some((g, s, _)) => gap < g - 0.5 || ((gap - g).abs() <= 0.5 && shared > s),
            };
            if better {
                best = Some((gap, shared, pane.tab));
            }
        }
        best.map(|(_, _, tab)| tab)
    }

    /// The pane under (`x`, `y`) and the zone of it: the outer 24 % strips
    /// are its edges (the nearest edge wins in a corner), the rest its
    /// centre.
    pub fn drop_target(&self, x: f32, y: f32) -> Option<(TabId, DropZone)> {
        let pane = self.panes.iter().find(|p| contains(&p.rect, x, y))?;
        let r = pane.rect;
        let fx = (x - r.x) / r.width.max(1.0);
        let fy = (y - r.y) / r.height.max(1.0);
        let candidates = [
            (fx, Edge::Left),
            (1.0 - fx, Edge::Right),
            (fy, Edge::Top),
            (1.0 - fy, Edge::Bottom),
        ];
        let nearest = candidates
            .into_iter()
            .filter(|(distance, _)| *distance < DROP_EDGE_SHARE)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        Some((
            pane.tab,
            nearest.map_or(DropZone::Center, |(_, edge)| DropZone::Edge(edge)),
        ))
    }

    /// The divider under (`x`, `y`), with `slop` points of grab room on
    /// each side.
    pub fn divider_at(&self, x: f32, y: f32, slop: f32) -> Option<&TiledDivider> {
        self.dividers.iter().find(|d| {
            let r = d.rect;
            let grown = match d.axis {
                Axis::Row => WebRect::new(r.x - slop, r.y, r.width + 2.0 * slop, r.height),
                Axis::Column => WebRect::new(r.x, r.y - slop, r.width, r.height + 2.0 * slop),
            };
            contains(&grown, x, y)
        })
    }
}

#[cfg(test)]
#[path = "tests_tiling.rs"]
mod tests;
