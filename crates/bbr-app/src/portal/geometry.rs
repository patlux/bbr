//! Pure point-coordinate portal geometry (DESIGN.md §11.4, §13).
//!
//! Mirrors the placement math behind `gpui_component`'s
//! `deferred(anchored(...).snap_to_window_with_margin(...))` without copying
//! its widget system: the same eight-corner anchor model as `gpui::Anchor`,
//! `gpui::Bounds::from_anchor_and_size` origin math, `Anchor::other_side_along`
//! corner switching, and the edge-snap clamp from `gpui::Anchored::prepaint`
//! (verified against the pinned GPUI-CE source at the anchor commit).
//!
//! All values are plain `f32` points in host layout coordinates (top-left
//! origin, matching `bbr_web::WebRect`), so this module is unit-testable
//! without a display server, GPUI, or AppKit. Rounding the final element
//! offset is a render-time concern and is intentionally not applied here.

use std::ops::Add;

/// A point in host layout coordinates (top-left origin, points).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

impl Pt {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Pt {
    type Output = Pt;

    fn add(self, rhs: Pt) -> Pt {
        Pt {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// A size in points.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Sz {
    pub width: f32,
    pub height: f32,
}

impl Sz {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// An axis-aligned rect in host layout coordinates (top-left origin, points).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(origin: Pt, size: Sz) -> Self {
        Self {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn left(self) -> f32 {
        self.x
    }

    pub fn top(self) -> f32 {
        self.y
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
}

/// Per-edge insets in points (top/right/bottom/left), mirroring
/// `gpui::Edges<Pixels>`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn none() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

impl Add for Edges {
    type Output = Edges;

    fn add(self, rhs: Edges) -> Edges {
        Edges {
            top: self.top + rhs.top,
            right: self.right + rhs.right,
            bottom: self.bottom + rhs.bottom,
            left: self.left + rhs.left,
        }
    }
}

/// The axis used by [`PortalAnchor::other_side_along`] (mirrors
/// `gpui::Axis`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// The eight-corner anchor model, mirroring `gpui::Anchor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    LeftCenter,
    RightCenter,
}

impl PortalAnchor {
    /// Mirrors `gpui::Anchor::other_side_along`: the corner across from
    /// `self` along `axis` (center-anchored corners keep their side on the
    /// perpendicular axis, exactly like GPUI).
    pub fn other_side_along(self, axis: Axis) -> PortalAnchor {
        match axis {
            Axis::Vertical => match self {
                PortalAnchor::TopLeft => PortalAnchor::BottomLeft,
                PortalAnchor::TopRight => PortalAnchor::BottomRight,
                PortalAnchor::BottomLeft => PortalAnchor::TopLeft,
                PortalAnchor::BottomRight => PortalAnchor::TopRight,
                PortalAnchor::TopCenter => PortalAnchor::BottomCenter,
                PortalAnchor::BottomCenter => PortalAnchor::TopCenter,
                PortalAnchor::LeftCenter => PortalAnchor::LeftCenter,
                PortalAnchor::RightCenter => PortalAnchor::RightCenter,
            },
            Axis::Horizontal => match self {
                PortalAnchor::TopLeft => PortalAnchor::TopRight,
                PortalAnchor::TopRight => PortalAnchor::TopLeft,
                PortalAnchor::BottomLeft => PortalAnchor::BottomRight,
                PortalAnchor::BottomRight => PortalAnchor::BottomLeft,
                PortalAnchor::TopCenter => PortalAnchor::TopCenter,
                PortalAnchor::BottomCenter => PortalAnchor::BottomCenter,
                PortalAnchor::LeftCenter => PortalAnchor::RightCenter,
                PortalAnchor::RightCenter => PortalAnchor::LeftCenter,
            },
        }
    }
}

/// Overflow policy, mirroring `gpui::AnchoredFitMode`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FitMode {
    /// Switch the anchor corner when an edge would overflow, keeping the
    /// portal on-screen without clamping (GPUI `SwitchAnchor`; the switch
    /// attempt uses the raw anchor point without the placement offset,
    /// exactly like GPUI).
    SwitchAnchor,
    /// Snap to the window edge on overflow (GPUI `SnapToWindow`).
    SnapToWindow,
    /// Snap to the window edge keeping the given margins on overflow
    /// (GPUI `SnapToWindowWithMargin`; the DESIGN §11.4 overlays' default).
    SnapToWindowWithMargin(Edges),
}

/// Mirrors `gpui::Bounds::from_anchor_and_size`: place the `anchor` corner of
/// a `size` rect at `origin`.
pub fn from_anchor_and_size(anchor: PortalAnchor, origin: Pt, size: Sz) -> Rect {
    let x = match anchor {
        PortalAnchor::TopLeft | PortalAnchor::BottomLeft | PortalAnchor::LeftCenter => origin.x,
        PortalAnchor::TopCenter | PortalAnchor::BottomCenter => origin.x - size.width * 0.5,
        PortalAnchor::TopRight | PortalAnchor::BottomRight | PortalAnchor::RightCenter => {
            origin.x - size.width
        }
    };
    let y = match anchor {
        PortalAnchor::TopLeft | PortalAnchor::TopRight | PortalAnchor::TopCenter => origin.y,
        PortalAnchor::LeftCenter | PortalAnchor::RightCenter => origin.y - size.height * 0.5,
        PortalAnchor::BottomLeft | PortalAnchor::BottomRight | PortalAnchor::BottomCenter => {
            origin.y - size.height
        }
    };
    Rect::new(Pt::new(x, y), size)
}

/// Clamps `rect` into the viewport, mirroring the edge-snap step of
/// `gpui::Anchored::prepaint` (which GPUI runs for every fit mode; only
/// [`FitMode::SnapToWindowWithMargin`] contributes non-zero `margins`).
///
/// Right/bottom overflow shifts the rect inward by the overflow plus the
/// edge inset; left/top underflow pins the edge to the inset. A rect wider
/// than the viewport therefore aligns to the left, a taller one to the top —
/// GPUI's documented fallback.
pub fn clamp_to_viewport(rect: Rect, viewport: Sz, margins: Edges, client_inset: Edges) -> Rect {
    let mut desired = rect;
    let edges = margins + client_inset;

    // Horizontal (right overflow first, then left underflow — GPUI order).
    if desired.right() > viewport.width {
        desired.x -= desired.right() - viewport.width + edges.right;
    }
    if desired.left() < 0.0 {
        desired.x = edges.left;
    }

    // Vertical (bottom overflow first, then top underflow — GPUI order).
    if desired.bottom() > viewport.height {
        desired.y -= desired.bottom() - viewport.height + edges.bottom;
    }
    if desired.top() < 0.0 {
        desired.y = edges.top;
    }

    desired
}

/// Resolves an anchored placement to a concrete viewport rect, mirroring
/// `gpui::Anchored::prepaint` for its `Window` position mode: build the
/// bounds from the anchor corner at the (offset) anchor point, switch the
/// corner when a [`FitMode::SwitchAnchor`] fit overflows, then edge-snap
/// into the viewport.
pub fn anchored_rect(
    anchor_point: Pt,
    anchor: PortalAnchor,
    size: Sz,
    offset: Pt,
    fit: FitMode,
    viewport: Sz,
    client_inset: Edges,
) -> Rect {
    let mut anchor = anchor;
    let mut desired = from_anchor_and_size(anchor, anchor_point + offset, size);

    if fit == FitMode::SwitchAnchor {
        if desired.left() < 0.0 || desired.right() > viewport.width {
            let switched = from_anchor_and_size(
                anchor.other_side_along(Axis::Horizontal),
                anchor_point,
                size,
            );
            if !(switched.left() < 0.0 || switched.right() > viewport.width) {
                anchor = anchor.other_side_along(Axis::Horizontal);
                desired = switched;
            }
        }
        if desired.top() < 0.0 || desired.bottom() > viewport.height {
            let switched =
                from_anchor_and_size(anchor.other_side_along(Axis::Vertical), anchor_point, size);
            if !(switched.top() < 0.0 || switched.bottom() > viewport.height) {
                desired = switched;
            }
        }
    }

    let margins = match fit {
        FitMode::SnapToWindowWithMargin(edges) => edges,
        _ => Edges::none(),
    };
    clamp_to_viewport(desired, viewport, margins, client_inset)
}

/// The centered overlay rect (DESIGN §11.4 palette: `elev:2` centered over
/// the window content), snapped into the viewport with `margins` through the
/// same edge-snap math as `anchored_rect` with a centered origin.
pub fn centered_rect(viewport: Sz, size: Sz, margins: Edges, client_inset: Edges) -> Rect {
    let origin = Pt::new(
        (viewport.width - size.width) * 0.5,
        (viewport.height - size.height) * 0.5,
    );
    clamp_to_viewport(Rect::new(origin, size), viewport, margins, client_inset)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> Sz {
        Sz::new(1080.0, 700.0)
    }

    #[test]
    fn centered_over_window_content() {
        // DESIGN §11.4 palette: 480 pt wide, centered in the window.
        let rect = centered_rect(
            viewport(),
            Sz::new(480.0, 360.0),
            Edges::all(8.0),
            Edges::none(),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(300.0, 170.0), Sz::new(480.0, 360.0)),
            "centered origin leaves an equal margin on every side"
        );
    }

    #[test]
    fn below_start_anchored_top_left() {
        // DESIGN §11.4 field-anchored overlay: top edge flush below the
        // field's bottom, leading edge flush with the field's leading edge,
        // with the 8 pt design margin.
        let rect = anchored_rect(
            Pt::new(24.0, 300.0),
            PortalAnchor::TopLeft,
            Sz::new(320.0, 200.0),
            Pt::new(0.0, 4.0),
            FitMode::SnapToWindowWithMargin(Edges::all(8.0)),
            viewport(),
            Edges::none(),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(24.0, 304.0), Sz::new(320.0, 200.0)),
            "anchor point plus offset places the top-left corner"
        );
    }

    #[test]
    fn compact_window_centered_clamps_to_left_and_top() {
        // A 400 pt palette in a 320 pt window: wider than the viewport, so
        // GPUI's fallback aligns it left (and the centered vertical overflow
        // pins it to the top edge).
        let rect = centered_rect(
            Sz::new(320.0, 200.0),
            Sz::new(400.0, 200.0),
            Edges::none(),
            Edges::none(),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(0.0, 0.0), Sz::new(400.0, 200.0)),
            "wider-than-viewport portals align to the left/top"
        );
    }

    #[test]
    fn off_edge_anchor_snaps_with_margin() {
        // Anchor near the right edge: right overflow snaps the rect inward
        // so the right edge lands at viewport.width - 8, top stays put.
        let rect = anchored_rect(
            Pt::new(80.0, 30.0),
            PortalAnchor::TopLeft,
            Sz::new(60.0, 40.0),
            Pt::new(0.0, 0.0),
            FitMode::SnapToWindowWithMargin(Edges::all(8.0)),
            Sz::new(100.0, 100.0),
            Edges::none(),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(32.0, 30.0), Sz::new(60.0, 40.0)),
            "right edge snapped to 100 - 8 with the 8 pt margin"
        );
    }

    #[test]
    fn switch_anchor_flips_corner_to_stay_on_screen() {
        // TopLeft at the right edge overflows; the TopRight switch fits, so
        // the corner flips instead of clamping.
        let rect = anchored_rect(
            Pt::new(180.0, 10.0),
            PortalAnchor::TopLeft,
            Sz::new(60.0, 40.0),
            Pt::new(0.0, 0.0),
            FitMode::SwitchAnchor,
            Sz::new(200.0, 100.0),
            Edges::none(),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(120.0, 10.0), Sz::new(60.0, 40.0)),
            "horizontal overflow switched TopLeft to TopRight"
        );

        // Bottom overflow switches TopLeft to BottomLeft.
        let rect = anchored_rect(
            Pt::new(10.0, 80.0),
            PortalAnchor::TopLeft,
            Sz::new(60.0, 40.0),
            Pt::new(0.0, 0.0),
            FitMode::SwitchAnchor,
            Sz::new(200.0, 100.0),
            Edges::none(),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(10.0, 40.0), Sz::new(60.0, 40.0)),
            "vertical overflow switched TopLeft to BottomLeft"
        );
    }

    #[test]
    fn switch_anchor_refused_when_flipped_side_also_overflows() {
        // A portal wider than the remaining space overflows on both sides of
        // the flip; the switch is refused and the edge-snap fallback aligns
        // it to the window edge instead.
        let rect = anchored_rect(
            Pt::new(160.0, 10.0),
            PortalAnchor::TopLeft,
            Sz::new(200.0, 40.0),
            Pt::new(0.0, 0.0),
            FitMode::SwitchAnchor,
            Sz::new(200.0, 100.0),
            Edges::none(),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(0.0, 10.0), Sz::new(200.0, 40.0)),
            "flip refused; snap fallback aligns to the left edge"
        );
    }

    #[test]
    fn other_side_along_matches_gpui_anchor_table() {
        assert_eq!(
            PortalAnchor::TopLeft.other_side_along(Axis::Vertical),
            PortalAnchor::BottomLeft
        );
        assert_eq!(
            PortalAnchor::TopLeft.other_side_along(Axis::Horizontal),
            PortalAnchor::TopRight
        );
        assert_eq!(
            PortalAnchor::TopCenter.other_side_along(Axis::Horizontal),
            PortalAnchor::TopCenter,
            "center-anchored corners keep their side"
        );
        assert_eq!(
            PortalAnchor::LeftCenter.other_side_along(Axis::Vertical),
            PortalAnchor::LeftCenter
        );
        assert_eq!(
            PortalAnchor::LeftCenter.other_side_along(Axis::Horizontal),
            PortalAnchor::RightCenter
        );
    }

    #[test]
    fn client_inset_adds_to_margins_like_gpui() {
        // GPUI folds the window client inset into the snap edges per side.
        let rect = anchored_rect(
            Pt::new(80.0, 30.0),
            PortalAnchor::TopLeft,
            Sz::new(60.0, 40.0),
            Pt::new(0.0, 0.0),
            FitMode::SnapToWindowWithMargin(Edges::all(8.0)),
            Sz::new(100.0, 100.0),
            Edges::all(2.0),
        );
        assert_eq!(
            rect,
            Rect::new(Pt::new(30.0, 30.0), Sz::new(60.0, 40.0)),
            "right snap keeps 8 + 2 pt of inset"
        );
    }
}
