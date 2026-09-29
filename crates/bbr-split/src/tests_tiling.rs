use bbr_domain::tiling::SplitGroup;

use super::*;

fn t(id: u64) -> TabId {
    TabId(id)
}

fn card() -> WebRect {
    WebRect::new(10.0, 20.0, 1202.0, 800.0)
}

/// Row [1, Column [2, 3]] on a 1202 × 800 card.
fn l_shape() -> SplitGroup {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Bottom).unwrap();
    group
}

#[test]
fn a_pair_shares_the_card_around_one_divider() {
    let group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    let tiling = project(group.root(), card(), TILING_DIVIDER);
    assert_eq!(
        tiling.panes,
        [
            TiledPane {
                tab: t(1),
                rect: WebRect::new(10.0, 20.0, 600.0, 800.0)
            },
            TiledPane {
                tab: t(2),
                rect: WebRect::new(612.0, 20.0, 600.0, 800.0)
            },
        ]
    );
    assert_eq!(tiling.dividers.len(), 1);
    let divider = &tiling.dividers[0];
    assert_eq!(divider.rect, WebRect::new(610.0, 20.0, 2.0, 800.0));
    assert_eq!((divider.start, divider.end), (10.0, 1212.0));
    assert_eq!(divider.combined, WEIGHT_TOTAL);
    assert!(tiling.fits());
}

#[test]
fn nested_branches_tile_without_gaps() {
    let tiling = project(l_shape().root(), card(), TILING_DIVIDER);
    let rects: Vec<WebRect> = tiling.panes.iter().map(|p| p.rect).collect();
    assert_eq!(
        rects,
        [
            WebRect::new(10.0, 20.0, 600.0, 800.0),
            WebRect::new(612.0, 20.0, 600.0, 399.0),
            WebRect::new(612.0, 421.0, 600.0, 399.0),
        ]
    );
    assert_eq!(tiling.dividers.len(), 2);
    let inner = tiling
        .dividers
        .iter()
        .find(|d| d.axis == Axis::Column)
        .unwrap();
    assert_eq!(inner.path, [1]);
    assert_eq!(inner.rect, WebRect::new(612.0, 419.0, 600.0, 2.0));
}

#[test]
fn a_group_that_is_too_small_does_not_fit() {
    let mut group = SplitGroup::pair(t(1), t(2), Axis::Row).unwrap();
    group.insert(t(2), t(3), Edge::Right).unwrap();
    group.equalize(&[]).unwrap();
    // Three columns of ~400 pt fit; on a 900 pt card they would not.
    assert!(project(group.root(), card(), TILING_DIVIDER).fits());
    let narrow = WebRect::new(0.0, 0.0, 900.0, 800.0);
    assert!(!project(group.root(), narrow, TILING_DIVIDER).fits());
    // A stacked pair on a 380 pt tall card would not either.
    let stacked = SplitGroup::pair(t(1), t(2), Axis::Column).unwrap();
    let short = WebRect::new(0.0, 0.0, 800.0, 380.0);
    assert!(!project(stacked.root(), short, TILING_DIVIDER).fits());
}

#[test]
fn neighbours_follow_the_layout() {
    let tiling = project(l_shape().root(), card(), TILING_DIVIDER);
    assert_eq!(
        tiling.neighbour(t(1), Edge::Right),
        Some(t(2)),
        "reading order breaks a tie"
    );
    assert_eq!(tiling.neighbour(t(3), Edge::Left), Some(t(1)));
    assert_eq!(tiling.neighbour(t(2), Edge::Bottom), Some(t(3)));
    assert_eq!(tiling.neighbour(t(3), Edge::Top), Some(t(2)));
    assert_eq!(tiling.neighbour(t(1), Edge::Left), None);
    assert_eq!(tiling.neighbour(t(2), Edge::Right), None);
}

#[test]
fn dropping_on_a_pane_picks_the_nearest_edge_or_the_centre() {
    let tiling = project(l_shape().root(), card(), TILING_DIVIDER);
    // Pane 1 is x 10–610, y 20–820.
    assert_eq!(
        tiling.drop_target(20.0, 400.0),
        Some((t(1), DropZone::Edge(Edge::Left)))
    );
    assert_eq!(
        tiling.drop_target(600.0, 400.0),
        Some((t(1), DropZone::Edge(Edge::Right)))
    );
    assert_eq!(
        tiling.drop_target(300.0, 30.0),
        Some((t(1), DropZone::Edge(Edge::Top)))
    );
    assert_eq!(
        tiling.drop_target(300.0, 400.0),
        Some((t(1), DropZone::Center))
    );
    // Near the top-left corner the nearer edge wins.
    assert_eq!(
        tiling.drop_target(15.0, 60.0),
        Some((t(1), DropZone::Edge(Edge::Left)))
    );
    assert_eq!(tiling.drop_target(5.0, 5.0), None);
}

#[test]
fn dragging_a_divider_keeps_both_neighbours_at_the_minimum() {
    let tiling = project(
        SplitGroup::pair(t(1), t(2), Axis::Row).unwrap().root(),
        card(),
        TILING_DIVIDER,
    );
    let divider = &tiling.dividers[0];
    // Content 1200 pt: the pointer at x 311 (+1 for half the divider)
    // gives the leading pane 300 pt → clamped to 320.
    assert_eq!(
        drag_weight(divider, 311.0, PANE_MIN_WIDTH, TILING_DIVIDER),
        2_667
    );
    // At x 811 the leading pane gets 800 of 1200 pt.
    assert_eq!(
        drag_weight(divider, 811.0, PANE_MIN_WIDTH, TILING_DIVIDER),
        6_667
    );
    // Past the end: the trailing pane keeps 320 pt.
    assert_eq!(
        drag_weight(divider, 5_000.0, PANE_MIN_WIDTH, TILING_DIVIDER),
        7_333
    );
    assert!(tiling.divider_at(611.0, 300.0, 3.0).is_some());
    assert!(tiling.divider_at(615.0, 300.0, 3.0).is_none());
}
