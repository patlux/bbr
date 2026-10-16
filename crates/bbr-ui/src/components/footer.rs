//! DESIGN.md §11.4 footer row — exactly `[Downloads] [Space Strip] [+]`.
//!
//! The footer is the rail's bottom dock (Paper S-7b): one band on the rail,
//! no paint and no rule. The fixed controls keep stable §15 ids;
//! `[Downloads]` opens the downloads surface, the middle flex region is the
//! permanent §11.3 space strip (built by the rail from [`crate::sidebar`]),
//! and `[+]` opens the plus menu (see [`super::menu`]). There is no
//! `[Spaces]` toggle and no revealable space-row region.
//!
//! This module owns the control identity contract (ids, labels, glyphs),
//! the band geometry, and the control states: hover and press washes, the
//! open state while the control's panel or menu shows, the §9 focus ring,
//! and the in-flight count badge. The rail wires the production listeners.

use bbr_domain::SpaceId;
use gpui::{
    div, point, prelude::*, px, App, BoxShadow, Div, MouseButton, MouseDownEvent, Pixels, Rgba,
    Stateful, Styled, Window,
};

use crate::icons::Icon;
use crate::theme::{Density, Theme};

/// The §15 id of the footer row container.
pub const FOOTER_ID: &str = "bbr-footer";
/// The §15 id of the `[Downloads]` control.
pub const FOOTER_DOWNLOADS_ID: &str = "bbr-downloads";
/// The §15 id of the in-flight count badge on `[Downloads]`.
pub const FOOTER_DOWNLOADS_COUNT_ID: &str = "bbr-downloads-count";
/// The §15 id of the permanent space strip (DESIGN.md §11.3/§11.4).
pub const FOOTER_STRIP_ID: &str = "bbr-space-strip";
/// The §15 id of the `[+]` control.
pub const FOOTER_PLUS_ID: &str = "bbr-plus";
/// The §7 text alternative of the glyph-only `[+]` control (the label is
/// the glyph itself; the semantic tree exposes this name).
#[allow(dead_code)] // §15 semantic seam
pub const FOOTER_PLUS_NAME: &str = "More actions";

/// One fixed footer control (§11.4: exactly two, the strip is the middle
/// flex region).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterControl {
    Downloads,
    Plus,
}

impl FooterControl {
    /// The stable §15 id.
    pub fn id(self) -> &'static str {
        match self {
            FooterControl::Downloads => FOOTER_DOWNLOADS_ID,
            FooterControl::Plus => FOOTER_PLUS_ID,
        }
    }

    /// The semantic label. Both fixed footer controls render glyph-only at
    /// rest; the typed semantic tree exposes this text alternative.
    pub fn label(self) -> &'static str {
        match self {
            FooterControl::Downloads => "Downloads",
            FooterControl::Plus => "+",
        }
    }

    /// The §7 glyph of the control.
    pub fn glyph(self) -> Icon {
        match self {
            FooterControl::Downloads => Icon::Downloads,
            FooterControl::Plus => Icon::Plus,
        }
    }
}

/// One pointer target of the footer dock: a fixed control or the chip of
/// one persistent space. The app keeps the hovered target as presentation
/// state through the production `on_hover` listeners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterTarget {
    Downloads,
    Plus,
    Space(SpaceId),
}

impl FooterTarget {
    /// The wire name (`downloads` / `plus` / `space-<id>`), the inverse of
    /// [`FooterTarget::parse`].
    pub fn name(self) -> String {
        match self {
            FooterTarget::Downloads => "downloads".into(),
            FooterTarget::Plus => "plus".into(),
            FooterTarget::Space(space) => format!("space-{}", space.0),
        }
    }

    /// Parse a wire name; unknown names are `None`.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "downloads" => Some(FooterTarget::Downloads),
            "plus" => Some(FooterTarget::Plus),
            other => other
                .strip_prefix("space-")
                .and_then(|id| id.parse().ok())
                .map(|id| FooterTarget::Space(SpaceId(id))),
        }
    }
}

/// Paper S-7b: a ring painted inside a target's square (an inset shadow),
/// so focus and drag outlines never change the target's layout box.
pub fn inset_ring(colour: Rgba, width: f32) -> BoxShadow {
    BoxShadow {
        color: colour.into(),
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(0.0),
        spread_radius: px(width),
        inset: true,
    }
}

/// The production hover listener of the dock: `(target, hovered)`.
pub type FooterHover = std::rc::Rc<dyn Fn(FooterTarget, bool, &mut Window, &mut App)>;

/// The §11.4 footer band geometry (Paper S-7b notes 1–3): the band height,
/// its asymmetric vertical insets, the inset from both visible rail edges,
/// the 8 pt gaps between the three parts and the 4 pt gap between chips,
/// and the one square every footer target shares.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FooterMetrics {
    pub height: f32,
    pub pad_top: f32,
    pub pad_bottom: f32,
    pub pad_x: f32,
    /// The trailing inset inside the rail column: the column ends at the
    /// transparent resize divider, so `pad_x` is measured from the visible
    /// rail edge as in Paper.
    pub pad_trailing: f32,
    pub gap: f32,
    pub chip_gap: f32,
    /// The square Downloads/Plus control and Space chip size.
    pub control: f32,
    /// The strip's height: the control plus 2 pt above and below, so a
    /// drag target's destination tag stays inside the scroll viewport.
    pub strip: f32,
}

impl FooterMetrics {
    pub fn for_density(density: Density) -> Self {
        let control = f32::from(density.hit_min_close());
        let (pad_top, pad_bottom, pad_x) = match density {
            Density::Small => (6.0, 8.0, 8.0),
            Density::Normal => (6.0, 10.0, 10.0),
            Density::ExtraLarge => (8.0, 12.0, 12.0),
        };
        FooterMetrics {
            height: pad_top + control + 4.0 + pad_bottom,
            pad_top,
            pad_bottom,
            pad_x,
            pad_trailing: pad_x - crate::sidebar::geometry::RESIZE_HANDLE_WIDTH,
            gap: 8.0,
            chip_gap: 4.0,
            control,
            strip: control + 4.0,
        }
    }
}

/// Downloads and Plus draw at the density's icon size (S 16 / N 18 /
/// XL 20) — the same family as the titlebar glyphs.
pub fn footer_glyph_size(density: Density) -> Pixels {
    density.icon()
}

/// Space identities draw 2 pt smaller than the fixed controls (S 14 /
/// N 16 / XL 18), so a lifted chip and a bare control read at one weight.
pub fn footer_identity_size(density: Density) -> Pixels {
    density.icon() - px(2.0)
}

/// The live state of the footer's fixed controls.
#[derive(Debug, Clone)]
pub struct FooterControls {
    /// The `[Downloads]` focus handle (the §11.12 dismissal return target).
    pub downloads_focus: gpui::FocusHandle,
    /// §9: `[Downloads]` owns keyboard focus (the accent ring).
    pub downloads_focused: bool,
    /// The downloads panel is open: `[Downloads]` stays pressed.
    pub downloads_open: bool,
    /// §11.4/§17.11: the in-flight count (`None` while nothing transfers).
    pub downloads_count: Option<usize>,
    /// The plus menu is open: `[+]` stays pressed.
    pub plus_open: bool,
    /// The pointer target under the cursor (hover wash).
    pub hovered: Option<FooterTarget>,
    /// The rail colour behind the dock (the count badge's cut-out ring).
    pub rail: Rgba,
}

/// The footer row: `[Downloads] [strip] [+]` on the rail. `strip` is the
/// rail-built space strip; `on_downloads`/`on_plus` are the production
/// activation listeners and `on_hover` the production hover listener
/// (`(target, hovered)`).
pub fn footer_surface(
    theme: &Theme,
    controls: &FooterControls,
    strip: Stateful<Div>,
    on_downloads: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_plus: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_hover: FooterHover,
) -> Stateful<Div> {
    let metrics = FooterMetrics::for_density(theme.density);
    // The listener is shared between the pointer path and the keyboard
    // activation without requiring `Clone`.
    let on_downloads = std::rc::Rc::new(on_downloads);
    let on_downloads_pointer = on_downloads.clone();
    let on_downloads_pointer = move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
        on_downloads_pointer(event, window, cx)
    };
    let on_downloads_keyboard = on_downloads.clone();
    let downloads_keyboard =
        move |event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut App| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                cx.stop_propagation();
                on_downloads_keyboard(&MouseDownEvent::default(), window, cx);
            }
        };
    let downloads = footer_control(
        theme,
        FooterControl::Downloads,
        ControlState {
            hovered: controls.hovered == Some(FooterTarget::Downloads),
            open: controls.downloads_open,
            focused: controls.downloads_focused,
        },
        on_downloads_pointer,
        on_hover.clone(),
    )
    .track_focus(&controls.downloads_focus)
    .on_key_down(downloads_keyboard)
    .when_some(controls.downloads_count, |el, count| {
        el.child(count_badge(theme, count, controls.rail))
    });
    let plus = footer_control(
        theme,
        FooterControl::Plus,
        ControlState {
            hovered: controls.hovered == Some(FooterTarget::Plus),
            open: controls.plus_open,
            focused: false,
        },
        on_plus,
        on_hover,
    );
    div()
        .id(FOOTER_ID)
        .debug_selector(|| FOOTER_ID.to_string())
        // S12: the dock sits directly on the (tinted) rail — no band paint
        // and no rule; `material:footer` is the media card's surface.
        .flex_shrink_0()
        .h(px(metrics.height))
        .pt(px(metrics.pad_top))
        .pb(px(metrics.pad_bottom))
        .pl(px(metrics.pad_x))
        .pr(px(metrics.pad_trailing))
        .gap(px(metrics.gap))
        .items_center()
        .flex()
        .flex_row()
        .child(downloads)
        // §11.3/§11.4: the permanent strip takes all remaining width,
        // clips to one line, and scrolls horizontally.
        .child(strip)
        .child(plus)
        .children(crate::rect_probe::rect_probe(FOOTER_ID))
}

/// The visible state of one fixed control.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ControlState {
    pub hovered: bool,
    /// The control's panel or menu is open (the pressed wash stays).
    pub open: bool,
    /// §9 keyboard focus (the 2 pt inset accent ring).
    pub focused: bool,
}

/// One fixed footer control: the §7 glyph centred in the shared square,
/// `density:radius`, the hover wash, the press wash, the open state
/// (`wash.active` + `ink`) and the §9 ring. The glyph box keeps a debug
/// selector for the geometry evidence.
pub fn footer_control(
    theme: &Theme,
    control: FooterControl,
    state: ControlState,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_hover: FooterHover,
) -> Stateful<Div> {
    let p = theme.palette;
    let glyph_size = footer_glyph_size(theme.density);
    let glyph_colour = if state.open { p.ink } else { p.ink_secondary };
    let glyph = div()
        .debug_selector(move || format!("{}-glyph", control.id()))
        .size(glyph_size)
        .flex()
        .items_center()
        .justify_center()
        .child(control.glyph().canvas_with(theme, glyph_size, glyph_colour));
    let size = px(FooterMetrics::for_density(theme.density).control);
    let target = match control {
        FooterControl::Downloads => FooterTarget::Downloads,
        FooterControl::Plus => FooterTarget::Plus,
    };
    div()
        .id(control.id())
        .debug_selector(move || control.id().to_string())
        .relative()
        .flex_shrink_0()
        .size(size)
        .rounded(theme.density.density_radius())
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .when(state.open, |el| el.bg(p.wash_active))
        .when(!state.open && state.hovered, |el| el.bg(p.wash_hover))
        .active(|style| style.bg(p.wash_active))
        .when(state.focused, |el| {
            el.shadow(vec![inset_ring(p.accent, 2.0)])
        })
        .child(glyph)
        .children(crate::rect_probe::rect_probe(control.id()))
        .on_mouse_down(MouseButton::Left, on_select)
        .on_hover(move |hovered: &bool, window: &mut Window, cx: &mut App| {
            on_hover(target, *hovered, window, cx)
        })
}

/// §11.4/§17.11: the in-flight count on `[Downloads]` — an accent badge at
/// the control's top-right, 14 pt tall, 10 pt semibold figures, cut out of
/// the glyph by a 2 pt ring in the rail colour. It never widens the
/// control, so the strip never moves.
pub fn count_badge(theme: &Theme, count: usize, rail: Rgba) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(FOOTER_DOWNLOADS_COUNT_ID)
        .debug_selector(|| FOOTER_DOWNLOADS_COUNT_ID.to_string())
        .absolute()
        .top(px(-3.0))
        .right(px(-4.0))
        .h(px(14.0))
        .min_w(px(14.0))
        .px(px(4.0))
        .rounded(px(7.0))
        .bg(p.accent)
        .shadow(vec![BoxShadow {
            color: rail.into(),
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(2.0),
            inset: false,
        }])
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.0))
        .line_height(px(14.0))
        .font_weight(gpui::FontWeight(600.0))
        .text_color(p.on_accent)
        .child(count.to_string())
}

/// The complete footer id set for the §15 `visible_ids` derivation (the
/// container, the two fixed controls, and the permanent strip; the count
/// badge is conditional and listed by the caller).
#[allow(dead_code)] // §15 visible-ids seam
pub fn footer_ids() -> Vec<String> {
    [
        FOOTER_ID,
        FOOTER_DOWNLOADS_ID,
        FOOTER_STRIP_ID,
        FOOTER_PLUS_ID,
    ]
    .map(String::from)
    .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_controls_are_exactly_the_two_fixed_contract_controls() {
        assert_eq!(FooterControl::Downloads.id(), FOOTER_DOWNLOADS_ID);
        assert_eq!(FooterControl::Plus.id(), FOOTER_PLUS_ID);
        assert_eq!(FooterControl::Downloads.label(), "Downloads");
        assert_eq!(FooterControl::Plus.label(), "+");
        assert_eq!(FooterControl::Plus.glyph(), Icon::Plus);
        assert_eq!(FooterControl::Downloads.glyph(), Icon::Downloads);
        // The glyph-only plus control keeps a non-empty text alternative.
        assert_eq!(FOOTER_PLUS_NAME, "More actions");
    }

    /// Paper S-7b note 2: fixed glyphs at the density icon size, identities
    /// 2 pt smaller.
    #[test]
    fn footer_glyphs_follow_the_density_icon_size() {
        for (density, glyph, identity) in [
            (Density::Small, 16.0, 14.0),
            (Density::Normal, 18.0, 16.0),
            (Density::ExtraLarge, 20.0, 18.0),
        ] {
            assert_eq!(footer_glyph_size(density), px(glyph), "{density:?}");
            assert_eq!(footer_identity_size(density), px(identity), "{density:?}");
        }
    }

    /// Paper S-7b note 1/3: S 42 / N 48 / XL 56 bands, one square per
    /// density, 8 pt part gaps and 4 pt chip gaps, the visible 10 pt inset
    /// at N on both rail edges, and a strip tall enough for the drag tag.
    #[test]
    fn footer_band_follows_the_paper_dock() {
        let n = FooterMetrics::for_density(Density::Normal);
        assert_eq!(
            n,
            FooterMetrics {
                height: 48.0,
                pad_top: 6.0,
                pad_bottom: 10.0,
                pad_x: 10.0,
                pad_trailing: 6.0,
                gap: 8.0,
                chip_gap: 4.0,
                control: 28.0,
                strip: 32.0,
            }
        );
        for (density, height, control) in [
            (Density::Small, 42.0, 24.0),
            (Density::Normal, 48.0, 28.0),
            (Density::ExtraLarge, 56.0, 32.0),
        ] {
            let m = FooterMetrics::for_density(density);
            assert_eq!((m.height, m.control), (height, control), "{density:?}");
            assert_eq!(
                m.pad_top + m.strip + m.pad_bottom,
                m.height,
                "{density:?}: the strip fills the band's content box"
            );
            assert!(m.control >= f32::from(footer_glyph_size(density)) + 6.0);
        }
    }

    #[test]
    fn footer_targets_round_trip_their_wire_names() {
        for target in [
            FooterTarget::Downloads,
            FooterTarget::Plus,
            FooterTarget::Space(SpaceId(7)),
        ] {
            assert_eq!(FooterTarget::parse(&target.name()), Some(target));
        }
        assert_eq!(FooterTarget::parse("space-x"), None);
        assert_eq!(FooterTarget::parse("spaces"), None);
    }

    #[test]
    fn footer_ids_are_stable_and_exact() {
        assert_eq!(
            footer_ids(),
            vec![
                "bbr-footer".to_string(),
                "bbr-downloads".to_string(),
                "bbr-space-strip".to_string(),
                "bbr-plus".to_string(),
            ]
        );
        assert_eq!(FOOTER_DOWNLOADS_COUNT_ID, "bbr-downloads-count");
        // No toggle control: the §11.4 footer is the two fixed controls
        // plus the permanent strip (the legacy `bbr-spaces` id is gone).
        assert!(!footer_ids().contains(&"bbr-spaces".to_string()));
    }
}
