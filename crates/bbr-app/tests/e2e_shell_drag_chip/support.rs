//! Drag-to-chip E2E support (DESIGN.md §11.3/§10.9): the deterministic
//! pointer entry into a rendered chip through the strip's real layout
//! evidence, the footer-invariant assertion, and the during-drag typed
//! target/companion assertions shared by the scenario modules. Split so the
//! suite files stay small.

use super::*;
use bbr_testkit::protocol::{dsl, Snapshot, SpaceStripScrollSnapshot};

/// The press anchor for every drag: window x 130 (the old seam's stable
/// anchor), window y 400 (inside the tab list, above the footer strip).
pub(super) const PRESS_X: f32 = 130.0;
pub(super) const PRESS_Y: f32 = 400.0;

/// The clipped visible span `[left, right, top, bottom]` of strip chip
/// `index` in window coordinates: the raw layout bounds from the production
/// `space_strip_scroll` handle plus the signed scroll offset, intersected
/// with the strip viewport — exactly the geometry the app's `chip_spans`
/// hit test consumes. `None` when the chip is fully scrolled out of the
/// viewport (an empty visible intersection): a fully hidden chip has no
/// hittable point, so the scenario must first bring it into view through
/// the production scroll-into-view authority plus a flushed frame, never
/// invent hidden hit geometry.
pub(super) fn clipped_span(scroll: &SpaceStripScrollSnapshot, index: usize) -> Option<[f32; 4]> {
    let raw = scroll.chip_bounds[index];
    let vp = scroll.viewport;
    let left = (raw[0] + scroll.offset[0]).max(vp[0]);
    let right = (raw[0] + scroll.offset[0] + raw[2]).min(vp[0] + vp[2]);
    let top = (raw[1] + scroll.offset[1]).max(vp[1]);
    let bottom = (raw[1] + scroll.offset[1] + raw[3]).min(vp[1] + vp[3]);
    if right <= left || bottom <= top {
        return None;
    }
    Some([left, right, top, bottom])
}

/// The visual center of strip chip `index` from its visible/clipped span: a
/// deterministic point strictly inside the non-empty visible intersection
/// of the chip with the strip viewport, so the pointer always enters the
/// exact geometry the production hit test resolves. A chip fully scrolled
/// out of view has no hittable point — the scenario must bring it into view
/// through the production scroll-into-view authority and a flushed frame
/// first (this panics loudly instead of inventing hidden hit geometry).
pub(super) fn chip_center(scroll: &SpaceStripScrollSnapshot, index: usize) -> (f32, f32) {
    let [left, right, top, bottom] = clipped_span(scroll, index)
        .expect("chip has a non-empty visible intersection; scroll it into view first");
    ((left + right) / 2.0, (top + bottom) / 2.0)
}

/// The visible span `[left, right, top, bottom]` of strip chip `index`
/// (window coordinates, clipped to the strip viewport) — used to prove a
/// drop lands in the gap between two rendered chips.
pub(super) fn chip_span(scroll: &SpaceStripScrollSnapshot, index: usize) -> [f32; 4] {
    clipped_span(scroll, index)
        .expect("chip has a non-empty visible intersection; scroll it into view first")
}

/// Begin a drag on row `index` at the shared press anchor, then move the
/// pointer to `(x, y)` (the deterministic chip entry point). Returns the
/// during-drag snapshot so the caller asserts the typed target evidence.
pub(super) fn drag_onto(client: &mut Client, index: usize, x: f32, y: f32) -> Snapshot {
    client
        .send_ok(&dsl::begin_tab_drag(index, PRESS_X, PRESS_Y, ROW_H / 2.0))
        .expect("begin drag");
    client
        .send_ok(&dsl::move_tab_drag(x - PRESS_X, y - PRESS_Y))
        .expect("move onto chip");
    client.snapshot().expect("during-drag snapshot")
}

/// The §11.4 footer composition is exactly `[Downloads][Space Strip][+]` —
/// the drag must never disturb the fixed controls or the strip.
pub(super) fn assert_footer_invariant(snapshot: &Snapshot) {
    assert_ids(
        snapshot,
        &["bbr-footer", "bbr-downloads", "bbr-space-strip", "bbr-plus"],
    );
}

/// The destination profile's `bbr-profile-chip-<n>` companion id (the §12
/// non-color identity the targeted chip renders during the drag).
pub(super) fn companion_id(
    snapshot: &Snapshot,
    space: bbr_testkit::protocol::TestSpaceId,
) -> String {
    let profile_id = snapshot
        .spaces
        .iter()
        .find(|s| s.space == space)
        .expect("destination space exists")
        .profile;
    let index = snapshot
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .expect("destination profile exists")
        .index;
    format!("bbr-profile-chip-{index}")
}

/// The index of the space `space` inside `snapshot.space_chips` (canonical
/// display order — the same vector the render, the snapshot, and the strip
/// scroll evidence share).
pub(super) fn chip_index(snapshot: &Snapshot, space: bbr_testkit::protocol::TestSpaceId) -> usize {
    snapshot
        .space_chips
        .iter()
        .position(|chip| chip.space == space)
        .expect("space chip renders")
}
