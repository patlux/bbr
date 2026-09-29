//! Blueprint D23 (DESIGN §11.7, §19): a split group — several tabs of one
//! space shown at once as a tiling of rows and columns.
//!
//! A group is a tree: branches lay their children out side by side
//! ([`Axis::Row`]) or stacked ([`Axis::Column`]), leaves are tabs. Each
//! child of a branch carries an integer weight; a branch's weights sum to
//! [`WEIGHT_TOTAL`], so the stored layout is exact and comparable. The tree
//! is always normalized: every branch holds at least two children, a child
//! branch never repeats its parent's axis (it would be the same row or
//! column), and every tab appears once. The root is a branch, so a group
//! always shows at least two panes; removing the second-to-last pane
//! dissolves it.
//!
//! Operations work on tab identities only; pixel sizes (the 320 × 200 pt
//! minimum, divider drags) are the geometry layer's (`bbr-split`), which
//! hands resolved weights back through [`SplitGroup::set_divider`].

use crate::TabId;

/// The sum of a branch's child weights.
pub const WEIGHT_TOTAL: u32 = 10_000;

/// How a branch lays out its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Side by side, leading to trailing.
    Row,
    /// Stacked, top to bottom.
    Column,
}

/// The edge of a pane a new pane is placed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl Edge {
    /// The axis a pane placed at this edge shares with its target.
    pub fn axis(self) -> Axis {
        match self {
            Edge::Left | Edge::Right => Axis::Row,
            Edge::Top | Edge::Bottom => Axis::Column,
        }
    }

    /// Whether the new pane follows its target in reading order.
    pub fn after(self) -> bool {
        matches!(self, Edge::Right | Edge::Bottom)
    }
}

/// One node of the tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SplitNode {
    Pane(TabId),
    Branch(Branch),
}

/// A row or column of weighted children.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Branch {
    pub axis: Axis,
    pub children: Vec<Child>,
}

/// A weighted child of a branch.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Child {
    pub weight: u32,
    pub node: SplitNode,
}

/// Why a tiling operation or a stored tree was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilingError {
    /// The named tab is not a pane of the group.
    UnknownPane(TabId),
    /// The tab is already a pane of the group.
    AlreadyMember(TabId),
    /// A pane cannot be placed beside itself.
    SamePane(TabId),
    /// No branch at this path, or no divider at this index.
    UnknownDivider,
    /// A stored tree breaks an invariant (fewer than two children, a
    /// repeated axis, weights not summing to the total, a zero weight, a
    /// duplicate tab).
    Malformed,
}

/// What removing a pane left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Removal {
    /// The group still holds at least two panes.
    Kept,
    /// Only this pane was left: the group is gone.
    Dissolved(TabId),
}

/// A split group: the root branch of a normalized tiling tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SplitGroup {
    root: Branch,
}

impl SplitGroup {
    /// Two panes on `axis`, `first` leading, equal shares.
    pub fn pair(first: TabId, second: TabId, axis: Axis) -> Result<Self, TilingError> {
        if first == second {
            return Err(TilingError::SamePane(first));
        }
        Ok(Self {
            root: Branch {
                axis,
                children: vec![
                    Child {
                        weight: WEIGHT_TOTAL / 2,
                        node: SplitNode::Pane(first),
                    },
                    Child {
                        weight: WEIGHT_TOTAL - WEIGHT_TOTAL / 2,
                        node: SplitNode::Pane(second),
                    },
                ],
            },
        })
    }

    /// A stored tree, checked against every invariant.
    pub fn from_root(root: Branch) -> Result<Self, TilingError> {
        let group = Self { root };
        group.validate()?;
        Ok(group)
    }

    /// The root branch (for projection and persistence).
    pub fn root(&self) -> &Branch {
        &self.root
    }

    /// The panes in reading order (depth first, leading to trailing, top
    /// to bottom).
    pub fn members(&self) -> Vec<TabId> {
        let mut out = Vec::new();
        collect(&self.root, &mut out);
        out
    }

    pub fn contains(&self, tab: TabId) -> bool {
        self.members().contains(&tab)
    }

    pub fn len(&self) -> usize {
        self.members().len()
    }

    /// Never true: a group holds at least two panes.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Place `new` at `edge` of the pane `beside`. In a row (or column)
    /// of the same axis it joins that row next to the target; otherwise the
    /// target becomes a new branch of the two. The new pane takes half of
    /// the target's share.
    pub fn insert(&mut self, beside: TabId, new: TabId, edge: Edge) -> Result<(), TilingError> {
        if beside == new {
            return Err(TilingError::SamePane(new));
        }
        if self.contains(new) {
            return Err(TilingError::AlreadyMember(new));
        }
        let (path, index) = self
            .locate(beside)
            .ok_or(TilingError::UnknownPane(beside))?;
        let parent = self.branch_mut(&path).expect("located");
        if parent.axis == edge.axis() {
            let weight = parent.children[index].weight;
            let given = weight / 2;
            if given == 0 {
                // A sliver share: make room by evening out the row.
                parent.children.insert(
                    index + usize::from(edge.after()),
                    Child {
                        weight: 0,
                        node: SplitNode::Pane(new),
                    },
                );
                equalize_branch(parent);
                return Ok(());
            }
            parent.children[index].weight = weight - given;
            parent.children.insert(
                index + usize::from(edge.after()),
                Child {
                    weight: given,
                    node: SplitNode::Pane(new),
                },
            );
        } else {
            let target = parent.children[index].node.clone();
            let mut pair = vec![
                Child {
                    weight: WEIGHT_TOTAL / 2,
                    node: target,
                },
                Child {
                    weight: WEIGHT_TOTAL - WEIGHT_TOTAL / 2,
                    node: SplitNode::Pane(new),
                },
            ];
            if !edge.after() {
                pair.reverse();
                pair[0].weight = WEIGHT_TOTAL / 2;
                pair[1].weight = WEIGHT_TOTAL - WEIGHT_TOTAL / 2;
            }
            parent.children[index].node = SplitNode::Branch(Branch {
                axis: edge.axis(),
                children: pair,
            });
        }
        Ok(())
    }

    /// Remove the pane of `tab`. Its share goes to the neighbour before it
    /// (or after it when it led); a branch left with one child is replaced
    /// by that child, merged into its parent when they share an axis.
    pub fn remove(&mut self, tab: TabId) -> Result<Removal, TilingError> {
        let (path, index) = self.locate(tab).ok_or(TilingError::UnknownPane(tab))?;
        {
            let parent = self.branch_mut(&path).expect("located");
            let removed = parent.children.remove(index);
            let heir = index.saturating_sub(1).min(parent.children.len() - 1);
            parent.children[heir].weight += removed.weight;
        }
        self.normalize_at(&path);
        if self.root.children.len() == 1 {
            match self.root.children[0].node.clone() {
                SplitNode::Pane(last) => return Ok(Removal::Dissolved(last)),
                SplitNode::Branch(inner) => self.root = inner,
            }
        }
        Ok(Removal::Kept)
    }

    /// Show `new` in the pane of `old` (same place, same share).
    pub fn replace(&mut self, old: TabId, new: TabId) -> Result<(), TilingError> {
        if old == new {
            return Err(TilingError::SamePane(new));
        }
        if self.contains(new) {
            return Err(TilingError::AlreadyMember(new));
        }
        let (path, index) = self.locate(old).ok_or(TilingError::UnknownPane(old))?;
        self.branch_mut(&path).expect("located").children[index].node = SplitNode::Pane(new);
        Ok(())
    }

    /// Move the pane of `tab` to `edge` of the pane `beside`.
    pub fn move_to(&mut self, tab: TabId, beside: TabId, edge: Edge) -> Result<(), TilingError> {
        if tab == beside {
            return Err(TilingError::SamePane(tab));
        }
        if !self.contains(beside) {
            return Err(TilingError::UnknownPane(beside));
        }
        match self.remove(tab)? {
            Removal::Kept => self.insert(beside, tab, edge),
            Removal::Dissolved(last) => {
                let (first, second) = if edge.after() {
                    (last, tab)
                } else {
                    (tab, last)
                };
                *self = Self::pair(first, second, edge.axis())?;
                Ok(())
            }
        }
    }

    /// Set the divider `divider` (between children `divider` and
    /// `divider + 1`) of the branch at `path`: the leading neighbour gets
    /// `leading` of the two neighbours' combined weight, the trailing one
    /// the rest (each at least 1). No other child changes.
    pub fn set_divider(
        &mut self,
        path: &[usize],
        divider: usize,
        leading: u32,
    ) -> Result<(), TilingError> {
        let branch = self.branch_mut(path).ok_or(TilingError::UnknownDivider)?;
        if divider + 1 >= branch.children.len() {
            return Err(TilingError::UnknownDivider);
        }
        let combined = branch.children[divider].weight + branch.children[divider + 1].weight;
        let leading = leading.clamp(1, combined - 1);
        branch.children[divider].weight = leading;
        branch.children[divider + 1].weight = combined - leading;
        Ok(())
    }

    /// Give every child of the branch at `path` the same share.
    pub fn equalize(&mut self, path: &[usize]) -> Result<(), TilingError> {
        let branch = self.branch_mut(path).ok_or(TilingError::UnknownDivider)?;
        equalize_branch(branch);
        Ok(())
    }

    /// The branch at `path` (child indices from the root).
    pub fn branch(&self, path: &[usize]) -> Option<&Branch> {
        let mut branch = &self.root;
        for &index in path {
            match &branch.children.get(index)?.node {
                SplitNode::Branch(inner) => branch = inner,
                SplitNode::Pane(_) => return None,
            }
        }
        Some(branch)
    }

    fn branch_mut(&mut self, path: &[usize]) -> Option<&mut Branch> {
        let mut branch = &mut self.root;
        for &index in path {
            match &mut branch.children.get_mut(index)?.node {
                SplitNode::Branch(inner) => branch = inner,
                SplitNode::Pane(_) => return None,
            }
        }
        Some(branch)
    }

    /// The path of the branch holding `tab` and its index there.
    fn locate(&self, tab: TabId) -> Option<(Vec<usize>, usize)> {
        fn walk(branch: &Branch, tab: TabId, path: &mut Vec<usize>) -> Option<usize> {
            for (index, child) in branch.children.iter().enumerate() {
                match &child.node {
                    SplitNode::Pane(pane) if *pane == tab => return Some(index),
                    SplitNode::Pane(_) => {}
                    SplitNode::Branch(inner) => {
                        path.push(index);
                        if let Some(found) = walk(inner, tab, path) {
                            return Some(found);
                        }
                        path.pop();
                    }
                }
            }
            None
        }
        let mut path = Vec::new();
        let index = walk(&self.root, tab, &mut path)?;
        Some((path, index))
    }

    /// After a removal in the branch at `path`: a single-child branch
    /// below the root is replaced by its child, and a child branch sharing
    /// its new parent's axis is spliced into it (shares scaled).
    fn normalize_at(&mut self, path: &[usize]) {
        let Some((&last, parent_path)) = path.split_last() else {
            return;
        };
        let parent = self.branch_mut(parent_path).expect("parent exists");
        let SplitNode::Branch(branch) = &parent.children[last].node else {
            return;
        };
        if branch.children.len() != 1 {
            return;
        }
        let only = branch.children[0].node.clone();
        match only {
            SplitNode::Branch(inner) if inner.axis == parent.axis => {
                let share = parent.children[last].weight;
                let spliced = scale(&inner.children, share);
                parent.children.splice(last..=last, spliced);
            }
            node => parent.children[last].node = node,
        }
    }

    fn validate(&self) -> Result<(), TilingError> {
        fn check(branch: &Branch, parent: Option<Axis>) -> Result<(), TilingError> {
            if branch.children.len() < 2 || Some(branch.axis) == parent {
                return Err(TilingError::Malformed);
            }
            let sum: u64 = branch.children.iter().map(|c| u64::from(c.weight)).sum();
            if sum != u64::from(WEIGHT_TOTAL) || branch.children.iter().any(|c| c.weight == 0) {
                return Err(TilingError::Malformed);
            }
            for child in &branch.children {
                if let SplitNode::Branch(inner) = &child.node {
                    check(inner, Some(branch.axis))?;
                }
            }
            Ok(())
        }
        check(&self.root, None)?;
        let members = self.members();
        let mut sorted = members.clone();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != members.len() {
            return Err(TilingError::Malformed);
        }
        Ok(())
    }
}

fn collect(branch: &Branch, out: &mut Vec<TabId>) {
    for child in &branch.children {
        match &child.node {
            SplitNode::Pane(tab) => out.push(*tab),
            SplitNode::Branch(inner) => collect(inner, out),
        }
    }
}

fn equalize_branch(branch: &mut Branch) {
    let count = branch.children.len() as u32;
    let base = WEIGHT_TOTAL / count;
    let extra = WEIGHT_TOTAL % count;
    for (index, child) in branch.children.iter_mut().enumerate() {
        child.weight = base + u32::from((index as u32) < extra);
    }
}

/// `children`'s weights rescaled to sum to `share` (each at least 1).
fn scale(children: &[Child], share: u32) -> Vec<Child> {
    let mut out: Vec<Child> = children
        .iter()
        .map(|child| Child {
            weight: ((u64::from(child.weight) * u64::from(share)) / u64::from(WEIGHT_TOTAL)).max(1)
                as u32,
            node: child.node.clone(),
        })
        .collect();
    let sum: u32 = out.iter().map(|c| c.weight).sum();
    // Rounding lands on the largest child, which can absorb it.
    if let Some(largest) = out.iter_mut().max_by_key(|c| c.weight) {
        largest.weight = (largest.weight + share).saturating_sub(sum);
    }
    out
}

#[cfg(test)]
#[path = "tests_tiling.rs"]
mod tests;
