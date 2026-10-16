use super::*;
use crate::theme::{ContrastMode, ThemeMode};
use glyphs::{
    arc_radius, chassis_radii, chevron_arm, close_arm, folder_shape, lifering_radii,
    nav_arrow_segments, plus_arm, reload_geometry, sidebar_geometry,
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
    assert_eq!(Icon::Search.name(), "search");
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
    assert_eq!(Icon::Bookmark.name(), "bookmark");
    assert_eq!(Icon::Pencil.name(), "pencil");
    assert_eq!(Icon::Warning.name(), "warning");
    assert_eq!(Icon::Gear.name(), "gear");
    assert_eq!(Icon::Document.name(), "document");
}

#[test]
fn the_not_secure_mark_carries_the_attention_status() {
    let p = crate::theme::Theme::dark().palette;
    assert_eq!(Icon::Warning.rest_color(p), p.status_attention);
    assert_eq!(Icon::Gear.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Document.rest_color(p), p.ink_secondary);
}

#[test]
fn clock_hands_start_at_the_center() {
    // Regression: both hands spanned through the center, drawing a cross
    // inside the ring instead of a clock face.
    for size in [16.0, 18.0, 20.0, 32.0] {
        let (cx, cy) = (size / 2.0, size / 2.0);
        let (ring, _, _) = glyphs::clock_geometry(size);
        for (from, to) in glyphs::clock_hands(size, cx, cy) {
            assert_eq!(from, (cx, cy), "a hand starts at the center at {size}pt");
            let length = ((to.0 - cx).powi(2) + (to.1 - cy).powi(2)).sqrt();
            assert!(
                length + 1.5 / 2.0 < ring,
                "a hand stays inside the ring at {size}pt"
            );
        }
    }
}

#[test]
fn bookmark_and_pencil_stay_inside_their_box() {
    for size in [16.0, 18.0, 20.0, 32.0] {
        let stroke = 1.5 / 2.0;
        let (half, top, bottom, notch) = glyphs::bookmark_geometry(size);
        assert!(half + stroke <= size / 2.0, "ribbon width at {size}pt");
        assert!(top + stroke <= size / 2.0, "ribbon top at {size}pt");
        assert!(bottom + stroke <= size / 2.0, "ribbon bottom at {size}pt");
        assert!(
            notch > 0.0 && notch < top + bottom,
            "notch inside the ribbon"
        );
        let (len, width, tip, band) = glyphs::pencil_geometry(size);
        // The farthest corner sits at (len + width) / √2 on both axes.
        let reach = (len + width) * std::f32::consts::FRAC_1_SQRT_2;
        assert!(reach + stroke <= size / 2.0, "pencil corner at {size}pt");
        assert!(tip < len && band < len, "tip and band inside the body");
    }
}

#[test]
fn identity_glyphs_share_one_optical_size() {
    // Regression: every glyph used the same `h`, so compact paths such as
    // compass or globe (farthest point 0.5 h) rendered at half the size of
    // code and folder (1.0 h). The painters divide `h` by `identity_extent`,
    // so the table must equal each path's real farthest per-axis point.
    // These values are measured independently from the path coordinates in
    // `icons/bbr.rs` (arc bulges included), so a wrong table entry fails
    // here instead of silently re-shrinking a glyph:
    // - code/folder reach the full h;
    // - terminal/star 0.55; camera: body 0.05 ± 0.5 → 0.55;
    // - person: head ring top 0.38 + 0.2 = 0.58;
    // - briefcase: body 0.1 ± 0.5 and the 0.2 handle semicircle at -0.4 → 0.60;
    // - moon ≈ 0.597; heart: 0.347 lobe semicircles centred at ±0.3 → 0.647;
    // - leaf: the stem end at (-0.65, 0.55) → 0.65; book ≈ 0.692;
    // - compass/palette/globe: rings of radius 0.5.
    let measured = |glyph: bbr_domain::BbrGlyph| -> f32 {
        use bbr_domain::BbrGlyph::*;
        match glyph {
            Code | Folder => 1.0,
            Terminal | Star | Camera => 0.55,
            Person => 0.58,
            Briefcase => 0.60,
            Moon => 0.597,
            Heart => 0.647,
            Leaf => 0.65,
            Book => 0.692,
            Compass | Palette | Globe => 0.50,
        }
    };
    for glyph in bbr_domain::BbrGlyph::ALL {
        let extent = bbr::identity_extent(glyph);
        assert!(
            (extent - measured(glyph)).abs() <= 0.01,
            "{glyph:?}: table {extent} vs measured path extent {}",
            measured(glyph)
        );
    }
    for size in [16.0_f32, 18.0, 20.0] {
        let target = size * 0.34;
        for glyph in bbr_domain::BbrGlyph::ALL {
            // The drawn farthest point after normalization.
            let reach = (target / bbr::identity_extent(glyph)) * measured(glyph);
            assert!(
                (reach / target - 1.0).abs() <= 0.02,
                "{glyph:?} reaches the shared box at {size}pt"
            );
            assert!(
                reach + 1.5 / 2.0 <= size / 2.0,
                "{glyph:?} fits at {size}pt"
            );
        }
    }
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
fn ported_strokes_end_in_paper_round_caps() {
    // A 1.5 pt stroke along +x: the outline sits 0.75 either side and its
    // round caps reach 0.75 past both ends (Paper `stroke-linecap: round`).
    let c = glyphs::capsule(((0.0, 0.0), (10.0, 0.0)), 0.75).expect("capsule");
    assert_eq!(c.start_plus, (0.0, 0.75));
    assert_eq!(c.end_plus, (10.0, 0.75));
    assert_eq!(c.end_tip, (10.75, 0.0));
    assert_eq!(c.end_minus, (10.0, -0.75));
    assert_eq!(c.start_minus, (0.0, -0.75));
    assert_eq!(c.start_tip, (-0.75, 0.0));
    assert_eq!((c.along, c.side), ((0.75, 0.0), (0.0, 0.75)));
    // Diagonal strokes keep the half-width on the normal.
    let d = glyphs::capsule(((0.0, 0.0), (3.0, 4.0)), 1.0).expect("capsule");
    assert_eq!(d.end_tip, (3.6, 4.8));
    assert_eq!(d.side, (-0.8, 0.6));
    assert!(glyphs::capsule(((1.0, 1.0), (1.0, 1.0)), 0.75).is_none());
}

#[test]
fn plus_and_minus_follow_paper_paths() {
    // Paper `M9 3.75v10.5M3.75 9h10.5` on the 18-unit box: 5.25 arms,
    // 12 pt end to end once the round caps are added.
    assert_eq!(
        glyphs::plus_segments(18.0),
        vec![((-5.25, 0.0), (5.25, 0.0)), ((0.0, -5.25), (0.0, 5.25))]
    );
    assert_eq!(
        glyphs::minus_segments(18.0),
        vec![((-5.25, 0.0), (5.25, 0.0))]
    );
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
        // The folder (closed and open share the silhouette) stays inside
        // its box.
        let folder = folder_shape(size);
        assert!(folder.half_w <= size / 2.0, "folder body fits at {size}pt");
        assert!(
            folder.half_h <= size / 2.0,
            "folder height fits at {size}pt"
        );
        assert!(
            folder.tab_w + folder.slant <= size,
            "folder tab fits at {size}pt"
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
        for right in [false, true] {
            for ((x0, y0), (x1, y1)) in nav_arrow_segments(size, right) {
                for v in [x0, y0, x1, y1] {
                    assert!(v.abs() <= size / 2.0, "nav arrow fits at {size}pt");
                }
            }
        }
        let (reload_radius, reload_head) = reload_geometry(size);
        assert!(reload_radius <= size / 2.0, "reload radius fits");
        assert!(
            reload_radius + reload_head <= size / 2.0,
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
    assert_eq!(Icon::Bookmark.rest_color(p), p.ink_secondary);
    assert_eq!(Icon::Pencil.rest_color(p), p.ink_secondary);
}
