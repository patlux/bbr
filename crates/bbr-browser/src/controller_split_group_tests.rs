//! Blueprint D23: split-group commands on the controller — every pane of
//! the shown group owns a live session, the split layout names each pane's
//! view, and selection outside the group keeps it.

use bbr_domain::tiling::{Axis, Edge, SplitGroup};

use crate::harness::{open, plain};
use crate::ControllerEvent;

#[test]
fn a_grown_group_frames_every_pane() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    open(&mut c, "u3");
    assert_eq!(
        c.split_insert(0, 1, Edge::Right).unwrap(),
        ControllerEvent::SplitChanged
    );
    c.split_insert(1, 2, Edge::Bottom).unwrap();
    let layout = c.split_layout().expect("a shown group");
    assert_eq!(layout.views.len(), 3);
    let tabs: Vec<_> = layout.views.iter().map(|(tab, _)| *tab).collect();
    assert_eq!(layout.group.members(), tabs);
    let snapshot = c.snapshot();
    assert_eq!(snapshot.active_index, Some(2), "the new pane is focused");
    assert_eq!(layout.focused, snapshot.tabs[2].web_view.unwrap());
    for (_, view) in &layout.views {
        assert!(snapshot.tabs.iter().any(|t| t.web_view == Some(*view)));
    }
    // The legacy pane layout names the first two panes.
    let panes = c.pane_layout().unwrap();
    assert_eq!(panes.primary, layout.views[0].1);
    assert_eq!(panes.secondary, Some(layout.views[1].1));
}

#[test]
fn selecting_outside_keeps_the_group_for_later() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    open(&mut c, "u3");
    c.split_insert(0, 1, Edge::Right).unwrap();
    c.select_tab(2).unwrap();
    assert!(c.split_layout().is_none());
    assert_eq!(c.pane_layout().unwrap().secondary, None);
    c.select_tab(0).unwrap();
    assert_eq!(c.split_layout().unwrap().views.len(), 2);
}

#[test]
fn dividers_replace_remove_and_restore() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    open(&mut c, "u3");
    c.split_insert(0, 1, Edge::Right).unwrap();
    c.split_set_divider(&[], 0, 2_500).unwrap();
    let weights: Vec<u32> = c
        .split_layout()
        .unwrap()
        .group
        .root()
        .children
        .iter()
        .map(|child| child.weight)
        .collect();
    assert_eq!(weights, [2_500, 7_500]);
    c.split_equalize(&[]).unwrap();
    c.split_replace(1, 2).unwrap();
    let tabs = c.snapshot().tabs;
    let members = c.split_layout().unwrap().group.members();
    assert_eq!(members.len(), 2);
    assert!(tabs[2].web_view.is_some());
    c.split_remove(0).unwrap();
    assert!(c.split_layout().is_none(), "one pane left: dissolved");

    let space = c.snapshot().active_space;
    let ids: Vec<_> = c.snapshot().all_tabs.iter().map(|t| t.id).collect();
    let group = SplitGroup::pair(ids[0], ids[2], Axis::Column).unwrap();
    c.restore_split_groups(space, vec![group.clone()]).unwrap();
    assert_eq!(c.split_groups(), vec![(space, vec![group])]);
    c.select_tab(0).unwrap();
    assert_eq!(c.split_layout().unwrap().group.root().axis, Axis::Column);
}
