//! The tab-drag state machine (DESIGN.md §10.9): armed press → active drag
//! → typed target, plus the one-time activation growth and the pointer the
//! §10.9 drag proxy follows.
//!
//! A row press **arms** a drag; only after the pointer travels
//! [`DRAG_ACTIVATION_DISTANCE`] does the drag become active and its
//! presentation (dragged-row state, tick, drag offers, proxy) begin. At
//! activation the drag records the rail growth the offers commit above the
//! source row (the §11.10 drop cell's extra row, the §11.2 ghost row), so
//! slot resolution keeps addressing the rows the render shows. The pure
//! target math lives in `super::drag_target`; the production listeners in
//! `super::drag`.

use super::drag_target::{resolve_target, DragGeometry, DragTarget};

/// The in-flight drag state of the root view. Owned by `BbrApp`; the
/// production listeners drive it, and the render derives the §10.9
/// dragging/tick/target/offer/proxy visuals plus the E2E snapshot from it.
#[derive(Debug, Clone, Default)]
pub struct TabDrag {
    /// The press is armed but the pointer has not yet travelled the
    /// activation distance: no drag presentation renders for a plain click,
    /// so a click never shifts the rail layout.
    armed: bool,
    active: bool,
    /// The pressed row's active-space index — the dragged tab.
    source: usize,
    /// The pointer x (window coordinates) where the drag began.
    start_x: f32,
    /// The pointer y (window coordinates) where the drag began.
    start_y: f32,
    /// The press's y within the source row surface (0 at the top edge,
    /// `row` at the bottom). Rows step on the §8 pitch, so the absolute
    /// press position is `source * pitch + press_in_row`.
    press_in_row: f32,
    /// The rail growth the drag offers committed above the source row at
    /// activation (points). Fixed for the whole gesture.
    growth: f32,
    /// The latest pointer position (window coordinates) — the proxy anchor.
    pointer_x: f32,
    pointer_y: f32,
    /// The current typed target (`None` over no droppable zone).
    target: Option<DragTarget>,
    /// Whether the current target is a legal drop. An illegal boundary
    /// crossing keeps the target (for the tick position) but reports
    /// invalid, so the render can signal the rejection (§10.9).
    valid: bool,
}

/// The pointer distance (points, per axis) a pressed row must travel
/// before the armed press becomes an active drag with its §10.9
/// presentation. Below it, the press is a click: no tick, no offers, no
/// layout shift.
pub const DRAG_ACTIVATION_DISTANCE: f32 = 4.0;

impl TabDrag {
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The press is armed but the activation distance is not yet exceeded.
    pub fn is_armed(&self) -> bool {
        self.armed
    }

    /// Arm a press on row `source` at the pointer position: the drag state
    /// records the anchor but stays inactive (no presentation) until the
    /// pointer travels [`DRAG_ACTIVATION_DISTANCE`].
    pub fn arm(&mut self, source: usize, position_x: f32, position_y: f32, press_in_row: f32) {
        *self = Self {
            armed: true,
            source,
            start_x: position_x,
            start_y: position_y,
            press_in_row,
            pointer_x: position_x,
            pointer_y: position_y,
            ..Self::default()
        };
    }

    /// The pressed row while armed or active (the growth computation needs
    /// it before activation).
    pub fn pressed_row(&self) -> Option<usize> {
        (self.armed || self.active).then_some(self.source)
    }

    /// Whether a pointer position exceeds the activation distance from the
    /// armed press anchor.
    pub fn activation_exceeded(&self, position_x: f32, position_y: f32) -> bool {
        (position_x - self.start_x).abs() >= DRAG_ACTIVATION_DISTANCE
            || (position_y - self.start_y).abs() >= DRAG_ACTIVATION_DISTANCE
    }

    /// Promote the armed press into the active drag at pointer `(x, y)`
    /// (the §10.9 presentation begins). `growth` is the rail growth the
    /// offers commit above the source row in the next frame; it stays fixed
    /// for the whole gesture. The target stays on the source slot — a drop
    /// before the pointer moves again is a no-op — because the offers have
    /// not rendered yet: resolving now would hit-test the pre-growth rects.
    pub fn activate(&mut self, growth: f32, x: f32, y: f32) {
        debug_assert!(self.armed, "activate requires an armed press");
        self.armed = false;
        self.active = true;
        self.growth = growth;
        self.pointer_x = x;
        self.pointer_y = y;
        self.target = Some(DragTarget::Tab { slot: self.source });
        self.valid = true;
    }

    /// The dragged row's active-space index.
    pub fn source(&self) -> Option<usize> {
        self.active.then_some(self.source)
    }

    /// The committed activation growth while active (snapshot evidence).
    #[cfg(any(test, feature = "testkit"))]
    pub fn growth(&self) -> Option<f32> {
        self.active.then_some(self.growth)
    }

    /// The §10.9 proxy inputs while active: the pointer's horizontal travel
    /// from the press, the pointer y, and the press offset inside the row.
    pub fn proxy_anchor(&self) -> Option<(f32, f32, f32)> {
        self.active.then_some((
            self.pointer_x - self.start_x,
            self.pointer_y,
            self.press_in_row,
        ))
    }

    /// The window x the drag began at (the E2E seam anchors its moves on
    /// it). Available while armed too, so the seam's first move can cross
    /// the activation distance from the true press anchor.
    #[cfg(feature = "testkit")]
    pub fn start_x(&self) -> Option<f32> {
        (self.active || self.armed).then_some(self.start_x)
    }

    /// The window y the drag began at (the E2E seam anchors its moves on it).
    #[cfg(feature = "testkit")]
    pub fn start_y(&self) -> Option<f32> {
        (self.active || self.armed).then_some(self.start_y)
    }

    /// Replace the press anchor after `arm` (E2E seam). No resolution has
    /// run at arm time, so overriding the anchor is consistent.
    #[cfg(feature = "testkit")]
    pub fn set_press_in_row(&mut self, press_in_row: f32) {
        self.press_in_row = press_in_row;
    }

    /// The current typed target (`None` over no droppable zone).
    pub fn target(&self) -> Option<DragTarget> {
        self.target
    }

    /// The tab insertion slot target, when the drag currently targets one.
    pub fn tab_slot(&self) -> Option<usize> {
        match self.target {
            Some(DragTarget::Tab { slot }) => Some(slot),
            _ => None,
        }
    }

    /// The §11.3 chip target: the exact persistent space the pointer
    /// hovers, when the drag currently targets a strip chip.
    pub fn chip_target(&self) -> Option<bbr_browser::SpaceId> {
        match self.target {
            Some(DragTarget::Chip { space }) => Some(space),
            _ => None,
        }
    }

    /// The §11.11 folder target, when the drag currently targets a folder
    /// row.
    pub fn folder_target(&self) -> Option<bbr_domain::FolderId> {
        match self.target {
            Some(DragTarget::Folder { folder }) => Some(folder),
            _ => None,
        }
    }

    /// Whether the current target is a legal drop.
    pub fn valid(&self) -> bool {
        self.valid
    }

    /// Test-only immediate begin: arm + activate (no growth) in one step.
    #[cfg(test)]
    pub fn begin(&mut self, source: usize, position_x: f32, position_y: f32, press_in_row: f32) {
        self.arm(source, position_x, position_y, press_in_row);
        self.activate(0.0, position_x, position_y);
    }

    /// Resolve the drag target for a new pointer position against the
    /// rendered geometry. The committed activation growth is subtracted
    /// from the vertical displacement: the source row moved down by it, so
    /// the slot math stays anchored on the row the render shows. Absolute
    /// hits (chips, the Favorites band, the ghost row) use the pointer
    /// itself.
    pub fn move_to(&mut self, position_x: f32, position_y: f32, geometry: &DragGeometry) {
        self.pointer_x = position_x;
        self.pointer_y = position_y;
        let dy = position_y - self.start_y - self.growth;
        let (target, valid) = resolve_target(
            self.source,
            position_x,
            position_y,
            dy,
            self.press_in_row,
            geometry,
        );
        self.target = target;
        self.valid = valid;
    }

    /// Blueprint D19: the pointer is over another window (or outside every
    /// window): no target in this one.
    pub fn clear_target(&mut self) {
        self.target = None;
        self.valid = false;
    }

    /// Clear the drag state (commit already happened or was cancelled).
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::TabDrag;
    use crate::app::drag_target::{ChipSpan, DragGeometry, DragTarget, DropRect};
    use bbr_browser::SpaceId;

    fn chip(space: u64, x_left: f32, x_right: f32, y_top: f32, y_bottom: f32) -> ChipSpan {
        ChipSpan {
            space: SpaceId(space),
            x_left,
            x_right,
            y_top,
            y_bottom,
        }
    }

    fn geometry<'a>(
        favorites_band: Option<(f32, f32)>,
        pin_offer: Option<(f32, f32)>,
        chips: &'a [ChipSpan],
    ) -> DragGeometry<'a> {
        let band = |(top, bottom): (f32, f32)| DropRect {
            x: 0.0,
            y: top,
            width: 300.0,
            height: bottom - top,
        };
        DragGeometry {
            pinned: 0,
            len: 3,
            pitch: 40.0,
            favorites_band: favorites_band.map(band),
            pin_offer: pin_offer.map(band),
            folders: &[],
            rail: None,
            chips,
            loose: None,
        }
    }

    #[test]
    fn move_to_accumulates_dy_from_the_press_position() {
        let chips = [chip(9, 60.0, 150.0, 640.0, 676.0)];
        let mut drag = TabDrag::default();
        // Row-surface-center press anchor (offset 18 in a 36 pt surface).
        drag.begin(1, 10.0, 200.0, 18.0);
        // One 40 pt pitch step down from the press → slot 2.
        drag.move_to(10.0, 240.0, &geometry(None, None, &chips));
        assert_eq!(drag.tab_slot(), Some(2));
        assert!(drag.valid());
        // Back to the press position → the source slot.
        drag.move_to(10.0, 200.0, &geometry(None, None, &chips));
        assert_eq!(drag.tab_slot(), Some(1));
        // A pointer over the rendered favorites band targets the grid.
        drag.move_to(10.0, 130.0, &geometry(Some((100.0, 160.0)), None, &chips));
        assert_eq!(drag.target(), Some(DragTarget::Favorites));
        assert!(drag.valid());
        // A pointer over the rendered ghost row targets the pin offer.
        drag.move_to(10.0, 175.0, &geometry(None, Some((160.0, 196.0)), &chips));
        assert_eq!(drag.target(), Some(DragTarget::PinOffer));
        assert!(drag.valid());
        // A pointer entering the rendered chip targets that exact space.
        drag.move_to(100.0, 658.0, &geometry(None, None, &chips));
        assert_eq!(drag.target(), Some(DragTarget::Chip { space: SpaceId(9) }));
        // A pointer below the list outside every chip is a no-target zone.
        drag.move_to(20.0, 658.0, &geometry(None, None, &chips));
        assert_eq!(drag.target(), None);
        assert!(!drag.valid());
    }

    #[test]
    fn activation_growth_keeps_slots_on_the_rendered_rows() {
        // The offers pushed the source row down one pitch at activation:
        // the pointer must travel that pitch to stay on the source slot,
        // and one more pitch reaches the next slot — the rows the render
        // shows, not the pre-activation layout.
        let chips = [];
        let mut drag = TabDrag::default();
        drag.arm(1, 10.0, 200.0, 18.0);
        assert_eq!(drag.pressed_row(), Some(1));
        assert!(drag.activation_exceeded(10.0, 206.0));
        drag.activate(40.0, 10.0, 206.0);
        assert_eq!(drag.growth(), Some(40.0));
        assert_eq!(
            drag.tab_slot(),
            Some(1),
            "activation keeps the no-op source slot until the next move"
        );
        drag.move_to(10.0, 240.0, &geometry(None, None, &chips));
        assert_eq!(
            drag.tab_slot(),
            Some(1),
            "the source row moved under the pointer"
        );
        drag.move_to(10.0, 280.0, &geometry(None, None, &chips));
        assert_eq!(drag.tab_slot(), Some(2));
    }

    #[test]
    fn the_proxy_follows_the_pointer_only_while_active() {
        let chips = [];
        let mut drag = TabDrag::default();
        drag.arm(0, 30.0, 300.0, 18.0);
        assert_eq!(drag.proxy_anchor(), None, "a plain press has no proxy");
        drag.activate(0.0, 34.0, 306.0);
        assert_eq!(drag.proxy_anchor(), Some((4.0, 306.0, 18.0)));
        drag.move_to(42.0, 360.0, &geometry(None, None, &chips));
        assert_eq!(drag.proxy_anchor(), Some((12.0, 360.0, 18.0)));
        drag.clear();
        assert_eq!(drag.proxy_anchor(), None);
        assert_eq!(drag.growth(), None);
    }
}
