//! DESIGN.md §7 geometric instrument glyphs — BBR-drawn, no Unicode/emoji
//! substitutes (§14).
//!
//! This bounded catalog replaces the placeholder text glyphs the first shell
//! slices shipped (close `×`, pin `●`/`○`, loading `●`, new-space `+`, crash
//! `!`, empty-pane brand `B`, and the status-line `⚠`). Every glyph is drawn
//! as GPUI vector paths/quads at the §7 stroke weight (1.5 pt at N/XL
//! density, 1.25 pt at S), with squared glyph corners and rest luminance
//! matched to `ink.secondary` exactly like the family contract requires.
//!
//! Icons are pure presentation: callers keep their own control ids, hit
//! squares, listeners, labels, and semantics on the owning div, so this
//! module never changes behavior, focus, or layout. The loading glyph is the
//! §10.5 circular arc in accent; it renders statically because rotation is
//! transient-indication motion that belongs to a later polish pass — this
//! slice replaces the placeholder, it does not introduce animation.
//!
//! The pure path geometry lives in [`glyphs`].

pub mod bbr;
pub mod glyphs;
mod paint;

pub use bbr::{classify_identity, IdentityForm};

use gpui::{canvas, point, px, Bounds, Canvas, PathBuilder, Pixels, Rgba, Styled, Window};

use bbr_domain::BbrGlyph;

use crate::palette::Palette;
use crate::theme::Theme;

use glyphs::{
    arc_radius, chassis_radii, check_geometry, chevron_arm, clock_geometry, close_arm,
    disclosure_arm, downloads_geometry, folder_geometry, folder_open_geometry, icon_stroke,
    lifering_radii, nav_arrow_geometry, pin_radius, plus_arm, reload_geometry, sidebar_geometry,
    stopped_half,
};
use paint::{paint_filled_circle, paint_path, paint_ring, paint_rounded_square_ring, paint_stroke};

/// §7 icon catalog: the bounded set of BBR-drawn chrome glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// §11.1 tab close: two crossing diagonal strokes.
    Close,
    /// §11.2 pin: a filled dot when pinned, a hollow ring when not.
    Pin { pinned: bool },
    /// §10.7/§11.8 lifering: the crash/error ring (outer + inner ring).
    Lifering,
    /// §10.11 unresponsive clock companion: a static ring with hour and
    /// minute hands next to the lifering on an unresponsive row.
    Clock,
    /// §11.8 empty-pane chassis mark: machined frame + glass aperture.
    ChassisMark,
    /// §11.3 New Space affordance: two crossing strokes.
    Plus,
    /// §11.4 footer `[Downloads]` control: an arrow dropping into an
    /// open-top tray.
    Downloads,
    /// §11.12 downloads status: the `status.ok` check on succeeded rows.
    Check,
    /// §11.12 downloads status: the neutral cancelled marker (a short
    /// horizontal bar at `ink.secondary`) on cancelled rows.
    Stopped,
    /// §11.11 folder identity: a folder silhouette with a top-left tab.
    /// `open` carries the folder's disclosure state: the closed variant is
    /// the plain silhouette, the open variant lifts the lid off the body so
    /// the §11.11 expanded state is never color-only.
    Folder { open: bool },
    /// §11.11 folder-row disclosure: a chevron that points right when
    /// collapsed and down when expanded.
    Disclosure { expanded: bool },
    /// §11.3 identity glyph: one curated [`BbrGlyph`] drawn in the §7
    /// geometric family (the catalog paths live in [`bbr`]). The footer
    /// Space chips and the space-config glyph picker render these paths;
    /// only a non-curated identity falls back to the two-letter badge.
    Bbr(BbrGlyph),
    /// §10.5 loading arc (static; rotation is a later polish pass).
    LoadingArc,
    /// §11.4 find bar: step to the previous match (left chevron).
    ChevronLeft,
    /// §11.4 find bar: step to the next match (right chevron).
    ChevronRight,
    /// §11.14 titlebar: the sidebar collapse/expand toggle. The glyph
    /// reflects the rail's state — the divider sits near the left edge
    /// when collapsed and toward the center when expanded.
    Sidebar { expanded: bool },
    /// §11.14 titlebar: go Back (a left-pointing §7 arrow).
    NavBack,
    /// §11.14 titlebar: go Forward (a right-pointing §7 arrow).
    NavForward,
    /// §11.14 titlebar: Reload (a drawn circular arrow).
    Reload,
}

impl Icon {
    /// The stable catalog name, for geometry/name tests and diagnostics.
    /// (Not referenced by the render path itself, so the lint is relaxed;
    /// the §7 catalog contract is exercised by the unit tests below.)
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            Icon::Close => "close",
            Icon::Pin { pinned } => {
                if pinned {
                    "pin-filled"
                } else {
                    "pin"
                }
            }
            Icon::Lifering => "lifering",
            Icon::Clock => "clock",
            Icon::ChassisMark => "chassis-mark",
            Icon::Plus => "plus",
            Icon::Downloads => "downloads",
            Icon::Check => "check",
            Icon::Stopped => "stopped",
            Icon::Folder { open } => {
                if open {
                    "folder-open"
                } else {
                    "folder"
                }
            }
            Icon::Disclosure { expanded } => {
                if expanded {
                    "disclosure-expanded"
                } else {
                    "disclosure"
                }
            }
            Icon::LoadingArc => "loading-arc",
            Icon::Bbr(glyph) => glyph.name(),
            Icon::ChevronLeft => "chevron-left",
            Icon::ChevronRight => "chevron-right",
            Icon::Sidebar { expanded } => {
                if expanded {
                    "sidebar-expanded"
                } else {
                    "sidebar-collapsed"
                }
            }
            Icon::NavBack => "nav-back",
            Icon::NavForward => "nav-forward",
            Icon::Reload => "reload",
        }
    }

    /// The §7 rest color: chrome glyphs sit at `ink.secondary` luminance;
    /// signal glyphs use their contract colors (accent pin/spinner, error
    /// lifering).
    fn rest_color(self, palette: Palette) -> Rgba {
        match self {
            Icon::Close
            | Icon::Plus
            | Icon::Downloads
            | Icon::Check
            | Icon::Stopped
            | Icon::Folder { .. }
            | Icon::Disclosure { .. }
            | Icon::ChassisMark
            | Icon::Pin { pinned: false } => palette.ink_secondary,
            Icon::Pin { pinned: true } | Icon::LoadingArc => palette.accent,
            Icon::Lifering => palette.status_error,
            Icon::Bbr(_) => palette.ink_secondary,
            Icon::Clock => palette.ink_secondary,
            Icon::ChevronLeft | Icon::ChevronRight => palette.ink_secondary,
            Icon::Sidebar { .. } => palette.ink_secondary,
            Icon::NavBack | Icon::NavForward | Icon::Reload => palette.ink_secondary,
        }
    }

    /// Render the icon in a `size`-pt square at its §7 rest color. The
    /// returned canvas is non-interactive; the owning control keeps its id,
    /// hit square, listeners, and label on the wrapping div.
    pub fn canvas(self, theme: &Theme, size: Pixels) -> Canvas<()> {
        Self::paint_canvas(self, theme.density, size, self.rest_color(theme.palette))
    }

    /// Render the icon in an explicit color (e.g. the disabled pin state
    /// drops to `ink.faint`).
    pub fn canvas_with(self, theme: &Theme, size: Pixels, color: Rgba) -> Canvas<()> {
        Self::paint_canvas(self, theme.density, size, color)
    }

    fn paint_canvas(self, density: crate::theme::Density, size: Pixels, color: Rgba) -> Canvas<()> {
        canvas(
            |_bounds, _window, _cx| (),
            move |bounds, (), window, _cx| {
                Self::paint(self, bounds, window, density, color);
            },
        )
        .size(size)
    }

    fn paint(
        self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        density: crate::theme::Density,
        color: Rgba,
    ) {
        let stroke = icon_stroke(density);
        let size = f32::from(bounds.size.width);
        let cx = f32::from(bounds.center().x);
        let cy = f32::from(bounds.center().y);
        match self {
            Icon::Close => {
                let arm = close_arm(size);
                paint_stroke(window, stroke, color, cx, cy, arm, 45.0);
                paint_stroke(window, stroke, color, cx, cy, arm, -45.0);
            }
            Icon::Pin { pinned } => {
                let radius = pin_radius(size);
                if pinned {
                    paint_filled_circle(window, cx, cy, radius, color);
                } else {
                    paint_ring(window, cx, cy, radius, stroke, color);
                }
            }
            Icon::Lifering => {
                let (outer, inner) = lifering_radii(size);
                paint_ring(window, cx, cy, outer, stroke, color);
                paint_ring(window, cx, cy, inner, stroke, color);
            }
            Icon::Clock => {
                let (ring, hour, minute) = clock_geometry(size);
                paint_ring(window, cx, cy, ring, stroke, color);
                // A static 10:00 reading: the hour hand points up, the
                // minute hand points right — never an animated hand.
                let mut path = PathBuilder::stroke(stroke);
                path.move_to(point(px(cx), px(cy - hour)));
                path.line_to(point(px(cx), px(cy + hour)));
                path.move_to(point(px(cx - minute), px(cy)));
                path.line_to(point(px(cx + minute), px(cy)));
                paint_path(window, path, color);
            }
            Icon::ChassisMark => {
                let (frame_corner, aperture) = chassis_radii(size);
                // The frame border is centered on its outline, so inset the
                // rounded square by half a stroke to keep it fully inside.
                let half = size / 2.0 - f32::from(stroke) / 2.0;
                paint_rounded_square_ring(window, cx, cy, half, frame_corner, stroke, color);
                paint_ring(window, cx, cy, aperture, stroke, color);
            }
            Icon::Plus => {
                let arm = plus_arm(size);
                let mut path = PathBuilder::stroke(stroke);
                path.move_to(point(px(cx - arm), px(cy)));
                path.line_to(point(px(cx + arm), px(cy)));
                path.move_to(point(px(cx), px(cy - arm)));
                path.line_to(point(px(cx), px(cy + arm)));
                paint_path(window, path, color);
            }
            Icon::Downloads => {
                let (arm, arrow, tray_half, tray_depth, tray_side) = downloads_geometry(size);
                let apex_y = cy + arrow;
                let mut path = PathBuilder::stroke(stroke);
                // The vertical stem down to the arrowhead apex.
                path.move_to(point(px(cx), px(cy - arm)));
                path.line_to(point(px(cx), px(apex_y)));
                // The arrowhead, then the open-top tray below it.
                path.move_to(point(px(cx), px(apex_y)));
                path.line_to(point(px(cx - arrow), px(apex_y + arrow)));
                path.move_to(point(px(cx), px(apex_y)));
                path.line_to(point(px(cx + arrow), px(apex_y + arrow)));
                let base_y = cy + tray_depth;
                path.move_to(point(px(cx - tray_half), px(base_y - tray_side)));
                path.line_to(point(px(cx - tray_half), px(base_y)));
                path.line_to(point(px(cx + tray_half), px(base_y)));
                path.line_to(point(px(cx + tray_half), px(base_y - tray_side)));
                paint_path(window, path, color);
            }
            Icon::Check => {
                let (inner, outer) = check_geometry(size);
                let mut path = PathBuilder::stroke(stroke);
                // The short inner stroke then the long ascending stroke of
                // the check mark.
                path.move_to(point(px(cx - outer), px(cy)));
                path.line_to(point(px(cx - inner), px(cy + inner)));
                path.move_to(point(px(cx - inner), px(cy + inner)));
                path.line_to(point(px(cx + outer), px(cy - outer)));
                paint_path(window, path, color);
            }
            Icon::Stopped => {
                let half = stopped_half(size);
                let mut path = PathBuilder::stroke(stroke);
                // A short horizontal bar: the neutral cancelled marker.
                path.move_to(point(px(cx - half), px(cy)));
                path.line_to(point(px(cx + half), px(cy)));
                paint_path(window, path, color);
            }
            Icon::Folder { open } => {
                if open {
                    let (half, tab_depth, tab_width, mouth) = folder_open_geometry(size);
                    let mut path = PathBuilder::stroke(stroke);
                    // The body with its open mouth: the top edge sits below
                    // the closed top (`mouth`), so the lid visibly lifts off
                    // the body.
                    path.move_to(point(px(cx - half), px(cy + half)));
                    path.line_to(point(px(cx - half), px(cy - mouth)));
                    path.line_to(point(px(cx + half), px(cy - mouth)));
                    path.line_to(point(px(cx + half), px(cy + half)));
                    path.line_to(point(px(cx - half), px(cy + half)));
                    // The lifted lid: the tab diagonal from the left edge up
                    // to the closed top, across to the flap end, then down
                    // into the mouth.
                    let flap_x = cx + half * 0.55;
                    path.move_to(point(px(cx - half), px(cy - half + tab_depth)));
                    path.line_to(point(px(cx - half + tab_width), px(cy - half)));
                    path.line_to(point(px(flap_x), px(cy - half)));
                    path.line_to(point(px(flap_x), px(cy - mouth)));
                    paint_path(window, path, color);
                } else {
                    let (half, tab_depth, tab_width) = folder_geometry(size);
                    let mut path = PathBuilder::stroke(stroke);
                    // Left edge up to the tab, the tab diagonal, then the
                    // body.
                    path.move_to(point(px(cx - half), px(cy + half)));
                    path.line_to(point(px(cx - half), px(cy - half + tab_depth)));
                    path.line_to(point(px(cx - half + tab_width), px(cy - half)));
                    path.line_to(point(px(cx + half), px(cy - half)));
                    path.line_to(point(px(cx + half), px(cy + half)));
                    path.line_to(point(px(cx - half), px(cy + half)));
                    paint_path(window, path, color);
                }
            }
            Icon::Disclosure { expanded } => {
                let arm = disclosure_arm(size);
                let mut path = PathBuilder::stroke(stroke);
                if expanded {
                    // Down chevron: apex at the bottom, arms to the sides.
                    path.move_to(point(px(cx - arm), px(cy)));
                    path.line_to(point(px(cx), px(cy + arm)));
                    path.line_to(point(px(cx + arm), px(cy)));
                } else {
                    // Right chevron: apex at the right, arms to the corners.
                    path.move_to(point(px(cx), px(cy - arm)));
                    path.line_to(point(px(cx + arm), px(cy)));
                    path.line_to(point(px(cx), px(cy + arm)));
                }
                paint_path(window, path, color);
            }
            Icon::LoadingArc => {
                let radius = arc_radius(size);
                let mut path = PathBuilder::stroke(stroke);
                // A 270° accent arc: from the top clockwise to the left.
                path.move_to(point(px(cx), px(cy - radius)));
                path.arc_to(
                    point(px(radius), px(radius)),
                    px(0.0),
                    true,
                    true,
                    point(px(cx - radius), px(cy)),
                );
                paint_path(window, path, color);
            }
            Icon::ChevronLeft | Icon::ChevronRight => {
                let arm = chevron_arm(size);
                let mut path = PathBuilder::stroke(stroke);
                // A single chevron pointing in the step direction: apex at
                // the trailing side, arms to the leading corners.
                let (apex_x, arm_x) = match self {
                    Icon::ChevronLeft => (cx - arm, cx),
                    _ => (cx + arm, cx),
                };
                path.move_to(point(px(arm_x), px(cy - arm)));
                path.line_to(point(px(apex_x), px(cy)));
                path.line_to(point(px(arm_x), px(cy + arm)));
                paint_path(window, path, color);
            }
            Icon::Sidebar { expanded } => {
                let (half, divider) = sidebar_geometry(size, expanded);
                // The frame: a rounded-square ring like the chassis mark.
                // The corner radius scales with the glyph so the small
                // titlebar frame keeps square §7 corners instead of
                // ballooning at a fixed 4 px.
                let corner = (size * 0.14).min(4.0);
                paint_rounded_square_ring(window, cx, cy, half, corner, stroke, color);
                // The vertical divider: near the left edge when collapsed,
                // toward the center when expanded.
                let mut path = PathBuilder::stroke(stroke);
                let inset = half - f32::from(stroke);
                path.move_to(point(px(cx + divider), px(cy - inset)));
                path.line_to(point(px(cx + divider), px(cy + inset)));
                paint_path(window, path, color);
            }
            Icon::NavBack | Icon::NavForward => {
                let (head, stem) = nav_arrow_geometry(size);
                let pointing_right = matches!(self, Icon::NavForward);
                // The head apex at the pointing side, arms to the corners,
                // and the stem running from the head base outward.
                let apex_x = if pointing_right { cx + head } else { cx - head };
                let stem_x = if pointing_right { cx - stem } else { cx + stem };
                let mut path = PathBuilder::stroke(stroke);
                path.move_to(point(px(apex_x), px(cy)));
                path.line_to(point(px(cx), px(cy - head)));
                path.move_to(point(px(apex_x), px(cy)));
                path.line_to(point(px(cx), px(cy + head)));
                path.move_to(point(px(cx), px(cy)));
                path.line_to(point(px(stem_x), px(cy)));
                paint_path(window, path, color);
            }
            Icon::Reload => {
                let (radius, head) = reload_geometry(size);
                // The reload ring: a 300° clockwise stroke from 60° past
                // the top back around to the top — the gap spans the
                // top-right 60°, exactly like the reference glyph. Drawn
                // as two ≤180° arcs so the arc flags are unambiguous.
                let at = |clock_deg: f32| {
                    let rad = clock_deg.to_radians();
                    (cx + radius * rad.sin(), cy - radius * rad.cos())
                };
                let (start_x, start_y) = at(60.0);
                let (mid_x, mid_y) = at(240.0);
                let tip_x = cx;
                let tip_y = cy - radius;
                let mut path = PathBuilder::stroke(stroke);
                path.move_to(point(px(start_x), px(start_y)));
                path.arc_to(
                    point(px(radius), px(radius)),
                    px(0.0),
                    false,
                    true,
                    point(px(mid_x), px(mid_y)),
                );
                path.arc_to(
                    point(px(radius), px(radius)),
                    px(0.0),
                    false,
                    true,
                    point(px(tip_x), px(tip_y)),
                );
                // The arrowhead at the top end: a V opening backward along
                // the stroke, pointing right — the clockwise travel
                // direction at the top of the ring.
                path.move_to(point(px(tip_x - head), px(tip_y - head)));
                path.line_to(point(px(tip_x), px(tip_y)));
                path.line_to(point(px(tip_x - head), px(tip_y + head)));
                paint_path(window, path, color);
            }
            Icon::Bbr(glyph) => {
                bbr::paint_identity(glyph, bounds, window, density, color);
            }
        }
    }
}
#[cfg(test)]
mod tests;
