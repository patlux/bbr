//! Blueprint D23: the split groups of every space, kept in `splits.txt` in
//! the profile directory next to the store (the store's two-pane table
//! cannot hold a tree, and the layout is app state).
//!
//! The first line is the format version (`1`); every further line is one
//! group: the space id, a tab, and the tree — `R(…)` for a row, `C(…)` for
//! a column, children as `weight:tab` or `weight:branch`, e.g.
//! `7\tR(5000:12,5000:C(5000:13,5000:14))`. Unreadable lines are skipped;
//! the controller rejects groups whose tabs are gone.

use std::path::{Path, PathBuf};

use bbr_domain::tiling::{Axis, Branch, Child, SplitGroup, SplitNode};
use bbr_domain::{SpaceId, TabId};

const VERSION: &str = "1";

/// Every space's groups.
pub type SpaceGroups = Vec<(SpaceId, Vec<SplitGroup>)>;

/// The file of a profile directory.
pub fn path_in(profile_dir: &Path) -> PathBuf {
    profile_dir.join("splits.txt")
}

/// One group as text.
pub fn encode(group: &SplitGroup) -> String {
    let mut out = String::new();
    write_branch(&mut out, group.root());
    out
}

fn write_branch(out: &mut String, branch: &Branch) {
    out.push(match branch.axis {
        Axis::Row => 'R',
        Axis::Column => 'C',
    });
    out.push('(');
    for (index, child) in branch.children.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&child.weight.to_string());
        out.push(':');
        match &child.node {
            SplitNode::Pane(tab) => out.push_str(&tab.0.to_string()),
            SplitNode::Branch(inner) => write_branch(out, inner),
        }
    }
    out.push(')');
}

/// A group from its text (`None` when malformed).
pub fn decode(text: &str) -> Option<SplitGroup> {
    let mut rest = text.trim();
    let root = parse_branch(&mut rest)?;
    if !rest.is_empty() {
        return None;
    }
    SplitGroup::from_root(root).ok()
}

fn parse_branch(rest: &mut &str) -> Option<Branch> {
    let axis = match rest.chars().next()? {
        'R' => Axis::Row,
        'C' => Axis::Column,
        _ => return None,
    };
    *rest = rest[1..].strip_prefix('(')?;
    let mut children = Vec::new();
    loop {
        let colon = rest.find(':')?;
        let weight: u32 = rest[..colon].parse().ok()?;
        *rest = &rest[colon + 1..];
        let node = if rest.starts_with('R') || rest.starts_with('C') {
            SplitNode::Branch(parse_branch(rest)?)
        } else {
            let end = rest.find([',', ')'])?;
            let tab: u64 = rest[..end].parse().ok()?;
            *rest = &rest[end..];
            SplitNode::Pane(TabId(tab))
        };
        children.push(Child { weight, node });
        if let Some(after) = rest.strip_prefix(',') {
            *rest = after;
        } else {
            *rest = rest.strip_prefix(')')?;
            return Some(Branch { axis, children });
        }
    }
}

/// Every space's groups as the file body.
pub fn render(groups: &[(SpaceId, Vec<SplitGroup>)]) -> String {
    let mut out = format!("{VERSION}\n");
    for (space, list) in groups {
        for group in list {
            out.push_str(&format!("{}\t{}\n", space.0, encode(group)));
        }
    }
    out
}

/// The groups in a file body, per space.
pub fn parse(body: &str) -> Vec<(SpaceId, Vec<SplitGroup>)> {
    let mut lines = body.lines();
    if lines.next() != Some(VERSION) {
        return Vec::new();
    }
    let mut out: Vec<(SpaceId, Vec<SplitGroup>)> = Vec::new();
    for line in lines {
        let Some((space, tree)) = line.split_once('\t') else {
            continue;
        };
        let (Ok(space), Some(group)) = (space.parse::<u64>(), decode(tree)) else {
            continue;
        };
        let space = SpaceId(space);
        match out.iter_mut().find(|(id, _)| *id == space) {
            Some((_, list)) => list.push(group),
            None => out.push((space, vec![group])),
        }
    }
    out
}

/// Load the file (nothing when it is missing).
pub fn load(path: &Path) -> Vec<(SpaceId, Vec<SplitGroup>)> {
    std::fs::read_to_string(path)
        .map(|body| parse(&body))
        .unwrap_or_default()
}

/// Write the file atomically.
pub fn save(path: &Path, groups: &[(SpaceId, Vec<SplitGroup>)]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension("txt.tmp");
    std::fs::write(&temporary, render(groups))?;
    std::fs::rename(&temporary, path)
}

/// Restore the file's groups into a restored controller; a group the model
/// no longer supports (a tab gone, a space gone) is dropped.
pub fn restore(controller: &mut bbr_browser::BrowserController, path: &Path) {
    for (space, groups) in load(path) {
        if let Err(error) = controller.restore_split_groups(space, groups) {
            log::warn!("split groups of space {space:?} not restored: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_domain::tiling::Edge;

    fn l_shape() -> SplitGroup {
        let mut group = SplitGroup::pair(TabId(12), TabId(13), Axis::Row).unwrap();
        group.insert(TabId(13), TabId(14), Edge::Bottom).unwrap();
        group.set_divider(&[], 0, 4_000).unwrap();
        group
    }

    #[test]
    fn groups_round_trip_as_text() {
        let group = l_shape();
        assert_eq!(encode(&group), "R(4000:12,6000:C(5000:13,5000:14))");
        assert_eq!(decode(&encode(&group)), Some(group.clone()));
        let body = render(&[(SpaceId(7), vec![group.clone()])]);
        assert_eq!(body, "1\n7\tR(4000:12,6000:C(5000:13,5000:14))\n");
        assert_eq!(parse(&body), vec![(SpaceId(7), vec![group])]);
    }

    #[test]
    fn malformed_text_is_skipped() {
        for bad in [
            "",
            "R(",
            "R(5000:1)",
            "R(4000:1,5000:2)",
            "X(5000:1,5000:2)",
            "R(5000:1,5000:2)x",
            "R(5000:1,5000:R(5000:2,5000:3))",
        ] {
            assert_eq!(decode(bad), None, "{bad}");
        }
        assert!(
            parse("2\n7\tR(5000:1,5000:2)\n").is_empty(),
            "unknown version"
        );
        assert_eq!(parse("1\nnope\n7\tR(5000:1,5000:2)\n8\tbad\n").len(), 1);
    }
}
