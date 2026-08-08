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

use gpui::{
    canvas, point, px, quad, size, transparent_black, BorderStyle, Bounds, Canvas, PathBuilder,
    Pixels, Rgba, Styled, Window,
};

use crate::palette::Palette;
use crate::theme::{Density, Theme};

use super::u2_glyphs::{
    check_geometry, disclosure_arm, downloads_geometry, folder_geometry, stopped_half,
};

/// §7 icon catalog: the bounded set of BBR-drawn chrome glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// §11.1 tab close: two crossing diagonal strokes.
    Close,
    /// §11.2 pin: a filled dot when pinned, a hollow ring when not.
    Pin { pinned: bool },
    /// §10.7/§11.8 lifering: the crash/error ring (outer + inner ring).
    Lifering,
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
    Folder,
    /// §11.11 folder-row disclosure: a chevron that points right when
    /// collapsed and down when expanded.
    Disclosure { expanded: bool },
    /// §10.5 loading arc (static; rotation is a later polish pass).
    LoadingArc,
    /// §11.4 find bar: step to the previous match (left chevron).
    ChevronLeft,
    /// §11.4 find bar: step to the next match (right chevron).
    ChevronRight,
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
            Icon::ChassisMark => "chassis-mark",
            Icon::Plus => "plus",
            Icon::Downloads => "downloads",
            Icon::Check => "check",
            Icon::Stopped => "stopped",
            Icon::Folder => "folder",
            Icon::Disclosure { expanded } => {
                if expanded {
                    "disclosure-expanded"
                } else {
                    "disclosure"
                }
            }
            Icon::LoadingArc => "loading-arc",
            Icon::ChevronLeft => "chevron-left",
            Icon::ChevronRight => "chevron-right",
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
            | Icon::Folder
            | Icon::Disclosure { .. }
            | Icon::ChassisMark
            | Icon::Pin { pinned: false } => palette.ink_secondary,
            Icon::Pin { pinned: true } | Icon::LoadingArc => palette.accent,
            Icon::Lifering => palette.status_error,
            Icon::ChevronLeft | Icon::ChevronRight => palette.ink_secondary,
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

    fn paint_canvas(self, density: Density, size: Pixels, color: Rgba) -> Canvas<()> {
        canvas(
            |_bounds, _window, _cx| (),
            move |bounds, (), window, _cx| {
                Self::paint(self, bounds, window, density, color);
            },
        )
        .size(size)
    }

    fn paint(self, bounds: Bounds<Pixels>, window: &mut Window, density: Density, color: Rgba) {
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
            Icon::Folder => {
                let (half, tab_depth, tab_width) = folder_geometry(size);
                let mut path = PathBuilder::stroke(stroke);
                // Left edge up to the tab, the tab diagonal, then the body.
                path.move_to(point(px(cx - half), px(cy + half)));
                path.line_to(point(px(cx - half), px(cy - half + tab_depth)));
                path.line_to(point(px(cx - half + tab_width), px(cy - half)));
                path.line_to(point(px(cx + half), px(cy - half)));
                path.line_to(point(px(cx + half), px(cy + half)));
                path.line_to(point(px(cx - half), px(cy + half)));
                paint_path(window, path, color);
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
        }
    }
}

/// §7 stroke weight: 1.5 pt at N/XL density, 1.25 pt at S.
pub fn icon_stroke(density: Density) -> Pixels {
    px(match density {
        Density::Small => 1.25,
        Density::Normal | Density::ExtraLarge => 1.5,
    })
}

/// Close ×: half the diagonal stroke length (of the full stroke).
pub fn close_arm(size: f32) -> f32 {
    size * 0.28
}

/// New-space plus: half the stroke length.
pub fn plus_arm(size: f32) -> f32 {
    size * 0.28
}

/// Pin dot: the radius of the filled dot / hollow ring.
pub fn pin_radius(size: f32) -> f32 {
    size * 0.30
}

/// Lifering: (outer ring radius, inner ring radius).
pub fn lifering_radii(size: f32) -> (f32, f32) {
    (size * 0.44, size * 0.28)
}

/// Loading arc radius (§10.5: 14 pt arc inside the favicon slot).
pub fn arc_radius(size: f32) -> f32 {
    size * 0.40
}

/// Find-bar step chevron: half the arm length from the apex to either
/// corner — the same §7 geometry as the disclosure chevron, so one arm
/// contract covers both families.
pub fn chevron_arm(size: f32) -> f32 {
    disclosure_arm(size)
}

/// Chassis mark: (frame corner radius, aperture ring radius). The frame
/// corner keeps the §5 4–8 px chrome radius at the 64 pt mark size.
pub fn chassis_radii(size: f32) -> (f32, f32) {
    (size * 0.125, size * 0.30)
}

/// One rotated diagonal stroke (close × half). The line is built in local
/// coordinates, rotated about the glyph center, then translated there.
fn paint_stroke(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    arm: f32,
    angle: f32,
) {
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(-arm), px(0.0)));
    path.line_to(point(px(arm), px(0.0)));
    path.rotate(angle);
    path.translate(point(px(cx), px(cy)));
    paint_path(window, path, color);
}

fn paint_path(window: &mut Window, path: PathBuilder, color: Rgba) {
    // The static geometries tessellate unconditionally; on an unexpected
    // failure the glyph degrades away instead of panicking the paint phase.
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

/// A 1.5 pt ring at `radius` around the given center (border stroke centered
/// on the circle outline, transparent hole).
fn paint_ring(window: &mut Window, cx: f32, cy: f32, radius: f32, stroke: Pixels, color: Rgba) {
    let diameter = radius * 2.0;
    window.paint_quad(quad(
        Bounds::new(
            point(px(cx - radius), px(cy - radius)),
            size(px(diameter), px(diameter)),
        ),
        px(radius),
        transparent_black(),
        stroke,
        color,
        BorderStyle::default(),
    ));
}

/// A filled circle at `radius` around the given center.
fn paint_filled_circle(window: &mut Window, cx: f32, cy: f32, radius: f32, color: Rgba) {
    let diameter = radius * 2.0;
    window.paint_quad(quad(
        Bounds::new(
            point(px(cx - radius), px(cy - radius)),
            size(px(diameter), px(diameter)),
        ),
        px(radius),
        color,
        px(0.0),
        transparent_black(),
        BorderStyle::default(),
    ));
}

/// A rounded-square ring (the chassis frame): `half` is half the side, the
/// border stroke is centered on the outline.
fn paint_rounded_square_ring(
    window: &mut Window,
    cx: f32,
    cy: f32,
    half: f32,
    corner_radius: f32,
    stroke: Pixels,
    color: Rgba,
) {
    window.paint_quad(quad(
        Bounds::new(
            point(px(cx - half), px(cy - half)),
            size(px(half * 2.0), px(half * 2.0)),
        ),
        px(corner_radius),
        transparent_black(),
        stroke,
        color,
        BorderStyle::default(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, ThemeMode};

    #[test]
    fn catalog_names_are_stable() {
        assert_eq!(Icon::Close.name(), "close");
        assert_eq!(Icon::Pin { pinned: false }.name(), "pin");
        assert_eq!(Icon::Pin { pinned: true }.name(), "pin-filled");
        assert_eq!(Icon::Lifering.name(), "lifering");
        assert_eq!(Icon::ChassisMark.name(), "chassis-mark");
        assert_eq!(Icon::Plus.name(), "plus");
        assert_eq!(Icon::Downloads.name(), "downloads");
        assert_eq!(Icon::Check.name(), "check");
        assert_eq!(Icon::Stopped.name(), "stopped");
        assert_eq!(Icon::Folder.name(), "folder");
        assert_eq!(Icon::Disclosure { expanded: false }.name(), "disclosure");
        assert_eq!(
            Icon::Disclosure { expanded: true }.name(),
            "disclosure-expanded"
        );
        assert_eq!(Icon::LoadingArc.name(), "loading-arc");
        assert_eq!(Icon::ChevronLeft.name(), "chevron-left");
        assert_eq!(Icon::ChevronRight.name(), "chevron-right");
    }

    #[test]
    fn stroke_weight_follows_the_density_contract() {
        // DESIGN.md §7: 1.5 pt stroke, 1.25 pt at S density.
        assert_eq!(icon_stroke(Density::Small), px(1.25));
        assert_eq!(icon_stroke(Density::Normal), px(1.5));
        assert_eq!(icon_stroke(Density::ExtraLarge), px(1.5));
    }

    #[test]
    fn glyph_geometry_stays_inside_its_box() {
        // Every arm/radius plus the stroke must fit the glyph's bounding box
        // at every shipped size (16/18/20 rows, 32 crash, 64 chassis mark).
        for size in [16.0, 18.0, 20.0, 32.0, 64.0] {
            assert!(
                close_arm(size) * 1.5 < size / 2.0,
                "close arm fits at {size}pt"
            );
            assert!(plus_arm(size) < size / 2.0, "plus arm fits at {size}pt");
            let pin = pin_radius(size) + 1.5 / 2.0;
            assert!(pin <= size / 2.0, "pin ring fits at {size}pt");
            let (outer, inner) = lifering_radii(size);
            assert!(inner < outer, "lifering radii ordered at {size}pt");
            assert!(
                outer + 1.5 / 2.0 <= size / 2.0,
                "lifering outer ring fits at {size}pt"
            );
            assert!(
                arc_radius(size) + 1.5 / 2.0 <= size / 2.0,
                "loading arc fits at {size}pt"
            );
            assert!(
                chevron_arm(size) * 1.5 + 1.5 / 2.0 <= size / 2.0,
                "step chevron fits at {size}pt"
            );
        }
    }

    #[test]
    fn chassis_mark_frame_and_aperture_fit() {
        let size = 64.0;
        let (frame_corner, aperture) = chassis_radii(size);
        // DESIGN.md §5: chrome radii stay 4–8 px; at the 64 pt mark the
        // frame corner is exactly 8 px.
        assert_eq!(frame_corner, 8.0);
        assert!(
            (4.0..=8.0).contains(&frame_corner),
            "frame radius in §5 range"
        );
        // The aperture ring (stroke included) must sit inside the frame's
        // inner edge (the frame border is centered on its outline).
        let stroke = 1.5;
        let frame_inner_half = size / 2.0 - stroke;
        assert!(
            aperture + stroke / 2.0 <= frame_inner_half,
            "aperture fits inside the frame at {size}pt"
        );
    }

    #[test]
    fn rest_colors_come_from_the_palette() {
        let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
        let p = theme.palette;
        assert_eq!(Icon::Close.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::Plus.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::Downloads.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::Check.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::Stopped.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::Folder.rest_color(p), p.ink_secondary);
        assert_eq!(
            Icon::Disclosure { expanded: false }.rest_color(p),
            p.ink_secondary
        );
        assert_eq!(
            Icon::Disclosure { expanded: true }.rest_color(p),
            p.ink_secondary
        );
        assert_eq!(Icon::ChassisMark.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::Pin { pinned: false }.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::Pin { pinned: true }.rest_color(p), p.accent);
        assert_eq!(Icon::LoadingArc.rest_color(p), p.accent);
        assert_eq!(Icon::Lifering.rest_color(p), p.status_error);
        assert_eq!(Icon::ChevronLeft.rest_color(p), p.ink_secondary);
        assert_eq!(Icon::ChevronRight.rest_color(p), p.ink_secondary);
    }
}
