use super::*;

fn t(id: u64) -> TabId {
    TabId(id)
}

fn weights(branch: &Branch) -> Vec<u32> {
    branch.children.iter().map(|c| c.weight).collect()
}

fn valid(group: &SplitGroup) {
    SplitGroup::from_root(group.root().clone()).expect("normalized");
}

#[test]
fn a_pair_is_two_equal_panes() {
    let group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    assert_eq!(group.members(), [t(1), t(2)]);
    assert_eq!(weights(group.root()), [5_000, 5_000]);
    assert_eq!(
        SplitGroup::pair(t(1), t(1), Axis::Row),
        Err(TilingError::SamePane(t(1)))
    );
    valid(&group);
}

#[test]
fn inserting_on_the_same_axis_joins_the_row() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Right).unwrap();
    assert_eq!(group.members(), [t(1), t(2), t(3)]);
    assert_eq!(group.root().axis, Axis::Row);
    // The new pane takes half of its target's share.
    assert_eq!(weights(group.root()), [5_000, 2_500, 2_500]);
    group.insert(t(1), t(4), Edge::Left).unwrap();
    assert_eq!(group.members(), [t(4), t(1), t(2), t(3)]);
    assert_eq!(weights(group.root()), [2_500, 2_500, 2_500, 2_500]);
    valid(&group);
}

#[test]
fn inserting_across_the_axis_stacks_inside_the_row() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Bottom).unwrap();
    assert_eq!(group.members(), [t(1), t(2), t(3)]);
    let SplitNode::Branch(column) = &group.root().children[1].node else {
        panic!("a column");
    };
    assert_eq!(column.axis, Axis::Column);
    assert_eq!(weights(column), [5_000, 5_000]);
    assert_eq!(
        weights(group.root()),
        [5_000, 5_000],
        "the row keeps its shares"
    );
    group.insert(t(2), t(4), Edge::Top).unwrap();
    assert_eq!(group.members(), [t(1), t(4), t(2), t(3)]);
    valid(&group);
}

#[test]
fn inserting_refuses_unknown_and_repeated_panes() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    assert_eq!(
        group.insert(t(9), t(3), Edge::Right),
        Err(TilingError::UnknownPane(t(9)))
    );
    assert_eq!(
        group.insert(t(1), t(2), Edge::Right),
        Err(TilingError::AlreadyMember(t(2)))
    );
    assert_eq!(
        group.insert(t(1), t(1), Edge::Right),
        Err(TilingError::SamePane(t(1)))
    );
}

#[test]
fn removing_hands_the_share_to_the_neighbour_and_collapses() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Right).unwrap();
    assert_eq!(group.remove(t(2)), Ok(Removal::Kept));
    assert_eq!(group.members(), [t(1), t(3)]);
    assert_eq!(weights(group.root()), [7_500, 2_500]);
    valid(&group);
    assert_eq!(group.remove(t(1)), Ok(Removal::Dissolved(t(3))));
}

#[test]
fn removing_from_a_column_merges_the_survivor_into_the_row() {
    // Row [1, Column [2, 3], 4]: removing 3 leaves 2 directly in the row.
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(4), Edge::Right).unwrap();
    group.insert(t(2), t(3), Edge::Bottom).unwrap();
    assert_eq!(group.remove(t(3)), Ok(Removal::Kept));
    assert_eq!(group.root().axis, Axis::Row);
    assert!(group
        .root()
        .children
        .iter()
        .all(|c| matches!(c.node, SplitNode::Pane(_))));
    assert_eq!(group.members(), [t(1), t(2), t(4)]);
    valid(&group);
}

#[test]
fn removing_splices_a_same_axis_grandchild() {
    // Row [1, Column [2, Row [3, 4]]]: removing 2 leaves Row [3, 4] in the
    // outer row — it is spliced in, shares scaled to the column's share.
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Bottom).unwrap();
    group.insert(t(3), t(4), Edge::Right).unwrap();
    assert_eq!(group.members(), [t(1), t(2), t(3), t(4)]);
    group.remove(t(2)).unwrap();
    assert_eq!(group.members(), [t(1), t(3), t(4)]);
    assert_eq!(group.root().axis, Axis::Row);
    assert_eq!(weights(group.root()), [5_000, 2_500, 2_500]);
    valid(&group);
}

#[test]
fn removing_the_root_row_down_to_a_column_keeps_the_column() {
    // Row [1, Column [2, 3]]: removing 1 leaves the column as the root.
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Bottom).unwrap();
    assert_eq!(group.remove(t(1)), Ok(Removal::Kept));
    assert_eq!(group.root().axis, Axis::Column);
    assert_eq!(group.members(), [t(2), t(3)]);
    valid(&group);
}

#[test]
fn replacing_keeps_place_and_share() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.set_divider(&[], 0, 3_000).unwrap();
    group.replace(t(1), t(5)).unwrap();
    assert_eq!(group.members(), [t(5), t(2)]);
    assert_eq!(weights(group.root()), [3_000, 7_000]);
    assert_eq!(
        group.replace(t(5), t(2)),
        Err(TilingError::AlreadyMember(t(2)))
    );
}

#[test]
fn moving_a_pane_to_another_edge() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Right).unwrap();
    group.move_to(t(1), t(3), Edge::Bottom).unwrap();
    assert_eq!(group.members(), [t(2), t(3), t(1)]);
    valid(&group);
    // A pair turns on its axis.
    let mut pair = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    pair.move_to(t(1), t(2), Edge::Bottom).unwrap();
    assert_eq!(pair.root().axis, Axis::Column);
    assert_eq!(pair.members(), [t(2), t(1)]);
}

#[test]
fn a_divider_moves_only_its_two_neighbours() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Right).unwrap();
    group.set_divider(&[], 1, 1_000).unwrap();
    assert_eq!(weights(group.root()), [5_000, 1_000, 4_000]);
    // Clamped to leave both neighbours a share.
    group.set_divider(&[], 1, 0).unwrap();
    assert_eq!(weights(group.root()), [5_000, 1, 4_999]);
    assert_eq!(
        group.set_divider(&[], 2, 10),
        Err(TilingError::UnknownDivider)
    );
    group.equalize(&[]).unwrap();
    assert_eq!(weights(group.root()), [3_334, 3_333, 3_333]);
    valid(&group);
}

#[test]
fn stored_trees_are_checked() {
    let pane = |id| Child {
        weight: 5_000,
        node: SplitNode::Pane(t(id)),
    };
    let one = Branch {
        axis: Axis::Row,
        children: vec![Child {
            weight: WEIGHT_TOTAL,
            node: SplitNode::Pane(t(1)),
        }],
    };
    assert_eq!(SplitGroup::from_root(one), Err(TilingError::Malformed));
    let duplicate = Branch {
        axis: Axis::Row,
        children: vec![pane(1), pane(1)],
    };
    assert_eq!(
        SplitGroup::from_root(duplicate),
        Err(TilingError::Malformed)
    );
    let same_axis = Branch {
        axis: Axis::Row,
        children: vec![
            pane(1),
            Child {
                weight: 5_000,
                node: SplitNode::Branch(Branch {
                    axis: Axis::Row,
                    children: vec![pane(2), pane(3)],
                }),
            },
        ],
    };
    assert_eq!(
        SplitGroup::from_root(same_axis),
        Err(TilingError::Malformed)
    );
    let short = Branch {
        axis: Axis::Row,
        children: vec![
            pane(1),
            Child {
                weight: 4_000,
                node: SplitNode::Pane(t(2)),
            },
        ],
    };
    assert_eq!(SplitGroup::from_root(short), Err(TilingError::Malformed));
    let fine = Branch {
        axis: Axis::Column,
        children: vec![pane(1), pane(2)],
    };
    assert!(SplitGroup::from_root(fine).is_ok());
}
