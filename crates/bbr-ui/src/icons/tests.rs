use super::*;
use crate::theme::{ContrastMode, ThemeMode};
use glyphs::{
    arc_radius, chassis_radii, chevron_arm, close_arm, folder_geometry, folder_open_geometry,
    lifering_radii, nav_arrow_geometry, plus_arm, reload_geometry, sidebar_geometry,
};

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
    assert_eq!(Icon::Folder { open: false }.name(), "folder");
    assert_eq!(Icon::Folder { open: true }.name(), "folder-open");
    assert_eq!(Icon::Disclosure { expanded: false }.name(), "disclosure");
    assert_eq!(
        Icon::Disclosure { expanded: true }.name(),
        "disclosure-expanded"
    );
    assert_eq!(Icon::LoadingArc.name(), "loading-arc");
    assert_eq!(Icon::Bbr(bbr_domain::BbrGlyph::Code).name(), "code");
    assert_eq!(Icon::Bbr(bbr_domain::BbrGlyph::Globe).name(), "globe");
    assert_eq!(Icon::ChevronLeft.name(), "chevron-left");
    assert_eq!(Icon::ChevronRight.name(), "chevron-right");
    assert_eq!(Icon::Sidebar { expanded: true }.name(), "sidebar-expanded");
    assert_eq!(
        Icon::Sidebar { expanded: false }.name(),
        "sidebar-collapsed"
    );
    assert_eq!(Icon::NavBack.name(), "nav-back");
    assert_eq!(Icon::NavForward.name(), "nav-forward");
    assert_eq!(Icon::Reload.name(), "reload");
}

#[test]
fn every_curated_identity_glyph_resolves_to_a_catalog_name() {
    // DESIGN.md §11.3/§7: every curated `BbrGlyph` resolves to a drawn
    // §7 path (the `bbr` paint module owns the paths) and round-trips
    // through its canonical name — the footer chips and the space-config
    // picker render from these; only a non-curated value falls back to
    // the two-letter badge.
    for glyph in bbr_domain::BbrGlyph::ALL {
        assert_eq!(
            Icon::Bbr(glyph).name(),
            glyph.name(),
            "{glyph:?} catalog name"
        );
        assert_eq!(
            bbr_domain::BbrGlyph::from_name(glyph.name()),
            Some(glyph),
            "{glyph:?} name round-trips"
        );
    }
    assert_eq!(
        bbr_domain::BbrGlyph::from_name("not-curated"),
        None,
        "non-curated names are the badge fallback"
    );
}

#[test]
fn stroke_weight_follows_the_density_contract() {
    // DESIGN.md §7: 1.5 pt stroke, 1.25 pt at S density.
    assert_eq!(icon_stroke(crate::theme::Density::Small), px(1.25));
    assert_eq!(icon_stroke(crate::theme::Density::Normal), px(1.5));
    assert_eq!(icon_stroke(crate::theme::Density::ExtraLarge), px(1.5));
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
        // The closed folder stays inside its box; the open variant's
        // mouth and lid also fit (the lid flap ends at 55% width, the
        // mouth sits below the closed top).
        let (half, _tab_depth, tab_width) = folder_geometry(size);
        assert!(half <= size / 2.0, "folder body fits at {size}pt");
        assert!(tab_width <= size, "folder tab fits at {size}pt");
        let (ohalf, _otab_depth, otab_width, mouth) = folder_open_geometry(size);
        assert!(ohalf <= size / 2.0, "open folder body fits at {size}pt");
        assert!(otab_width <= size, "open folder tab fits at {size}pt");
        assert!(
            mouth > 0.0 && mouth < ohalf,
            "open folder mouth is inside the body at {size}pt"
        );
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
        let (nav_head, nav_stem) = nav_arrow_geometry(size);
        assert!(nav_head <= size / 2.0, "nav head fits at {size}pt");
        assert!(
            nav_head + nav_stem <= size / 2.0,
            "nav stem fits at {size}pt"
        );
        let (reload_radius, reload_head) = reload_geometry(size);
        assert!(reload_radius <= size / 2.0, "reload radius fits");
        assert!(
            reload_radius * 0.866 + reload_head <= size / 2.0,
            "reload arrowhead fits at {size}pt"
        );
        for expanded in [true, false] {
            let (half, divider) = sidebar_geometry(size, expanded);
            assert!(half <= size / 2.0, "sidebar frame fits at {size}pt");
            assert!(
                divider.abs() <= half,
                "sidebar divider inside the frame at {size}pt"
            );
        }
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
    let theme = Theme::new(
        ThemeMode::Dark,
        ContrastMode::Standard,
        crate::theme::Density::Normal,
    );
    let p = theme.palette;
    assert_eq!(Icon::Close.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Plus.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Downloads.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Check.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Stopped.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Folder { open: false }.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Folder { open: true }.rest_color(p), p.ink_secondary);
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
    assert_eq!(
        Icon::Bbr(bbr_domain::BbrGlyph::Star).rest_color(p),
        p.ink_secondary
    );
    assert_eq!(Icon::Lifering.rest_color(p), p.status_error);
    assert_eq!(Icon::ChevronLeft.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::ChevronRight.rest_color(p), p.ink_secondary);
    assert_eq!(
        Icon::Sidebar { expanded: true }.rest_color(p),
        p.ink_secondary
    );
    assert_eq!(
        Icon::Sidebar { expanded: false }.rest_color(p),
        p.ink_secondary
    );
    assert_eq!(Icon::NavBack.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::NavForward.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Reload.rest_color(p), p.ink_secondary);
}
