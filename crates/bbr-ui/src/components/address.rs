//! Sidebar address control (DESIGN.md §11.5) and the non-address command
//! staging surface — the plain presentation halves of the app's sidebar
//! command slot.
//!
//! The sidebar address is a **display control, not a text field**: at rest
//! (and behind an open palette/overlay presentation) the slot renders the
//! domain field `bbr-address` — a quiet bordered surface one step off the
//! rail (a veil that follows theme and space tint), hover wash, button
//! semantics, and a trailing accessory slot that shows the page zoom chip
//! only while zoom is not 100 % (DESIGN §11.5/§17.14). All address
//! text entry lives in the §11.4 palette presentations; the pill holds no
//! editing state, cursor or selection and never accepts keyboard text.
//!
//! The same slot renders a separate **command staging surface**
//! (`bbr-command-bar`) only for the non-address staged modes — New Space,
//! Move Tab, and the §11.8 empty New Tab surface — which collect a space
//! name (never an address) and keep their existing field styling, cursor,
//! and hints. The address control itself is never that surface.
//!
//! Both components are pure presentation: the caller (the app's
//! `sidebar_view` command slot) owns the mode authority, the domain
//! derivation ([`crate::text`]), the §11.4 field anchor, the controller
//! snapshot, and the `cx.listener` closures; it builds the DTOs below and
//! wires the callbacks. The ids and the element tree are byte-identical to
//! the pre-cutover app render, so §15 semantics never drift.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use super::space::{profile_chip, ProfileChipSpec};
use crate::text::accessory_slot_width;
use crate::theme::{Spacing, Theme};

/// The §11.3 cross-context profile hint rendered at the pill's leading edge
/// for its one-interaction lifetime: the §12 chip (creation-order index,
/// monogram, hue) plus the hint label. Derived by the app from the active
/// profile; never the only identity cue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressProfileHint {
    /// The profile's creation-order index (the `bbr-profile-chip-<n>` id).
    pub profile_index: usize,
    /// The §12 chip identity (monogram + hue).
    pub spec: ProfileChipSpec,
    /// The hint text rendered next to the chip.
    pub label: SharedString,
}

/// The §11.5 address pill's presentation model. The app derives the domain
/// label from the live URL via [`crate::text::address_domain`] +
/// [`crate::text::address_domain_budget_chars`] + [`crate::text::mid_ellipsis`];
/// the pill itself holds no editing state.
#[derive(Debug, Clone)]
pub struct AddressModel {
    /// The mid-ellipsis'd domain label.
    pub domain: SharedString,
    /// §10.2 hover wash: the pointer hovers the pill.
    pub hovered: bool,
    /// The §11.3 cross-context profile hint; `None` at rest.
    pub profile_hint: Option<AddressProfileHint>,
    /// §17.14 zoom accessory: the active site's zoom percent while it is
    /// not 100 % ([`zoom_accessory`]); `None` keeps the slot empty.
    pub zoom: Option<u16>,
    /// D5 (§18.1): the active tab's load progress on the field's inner
    /// bottom edge.
    pub progress: AddressProgress,
    /// Blueprint D03: the leading identity mark (`None` for a secure site
    /// and for placeholders).
    pub mark: Option<AddressMark>,
    /// Blueprint D11: the leading site slot — a live or blocked device
    /// mark, and whether a press opens site settings.
    pub site: AddressSite,
}

/// Blueprint D11: what the leading slot says about the site's devices and
/// whether it opens site settings (http and https pages only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AddressSite {
    pub indicator: Option<SiteIndicator>,
    pub interactive: bool,
}

/// Blueprint D11: the device mark in the leading slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteIndicator {
    /// The page is using the camera and/or the microphone.
    Live { camera: bool },
    /// A remembered Block refused the page's last request.
    Blocked { camera: bool },
}

/// The stable ids of the device marks (blueprint D11).
pub const ADDRESS_CAPTURE_ID: &str = "bbr-address-capture";
pub const ADDRESS_BLOCKED_ID: &str = "bbr-address-blocked";

/// The stable id of the leading identity mark (blueprint D03).
pub const ADDRESS_MARK_ID: &str = "bbr-address-mark";

/// Blueprint D03 (§11.5): the leading mark before the address label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMark {
    /// Plain http off loopback: the attention triangle.
    NotSecure,
    /// A BBR internal page's glyph.
    Internal(crate::text::InternalPage),
    /// A local file: the document mark.
    File,
}

impl AddressMark {
    /// The mark for an identity: secure sites and placeholders show none.
    pub fn for_identity(identity: &crate::text::AddressIdentity) -> Option<Self> {
        use crate::text::AddressIdentity;
        match identity {
            AddressIdentity::Site { secure: false, .. } => Some(Self::NotSecure),
            AddressIdentity::Internal(page) => Some(Self::Internal(*page)),
            AddressIdentity::File { .. } => Some(Self::File),
            _ => None,
        }
    }

    /// The §7 glyph the mark draws.
    pub fn icon(self) -> crate::icons::Icon {
        use crate::icons::Icon;
        use crate::text::InternalPage;
        match self {
            Self::NotSecure => Icon::Warning,
            Self::Internal(InternalPage::Library) => Icon::Bookmark,
            Self::Internal(InternalPage::Settings) => Icon::Gear,
            Self::Internal(InternalPage::History) => Icon::Clock,
            Self::Internal(InternalPage::Welcome) => Icon::Bbr(bbr_domain::BbrGlyph::Compass),
            Self::File => Icon::Document,
        }
    }
}

/// D5 load progress in the address field (§18.5): GPUI never paints above
/// the native web view, so the line lives in the chrome the user is
/// already looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddressProgress {
    /// No load in flight and no finish playing: nothing renders.
    #[default]
    Idle,
    /// The active tab is loading: the indeterminate sweep (a static 2 pt
    /// line under Reduce Motion).
    Loading,
    /// The load just finished: the line fills, then fades (Reduce Motion
    /// removes it on finish instead, so this phase never occurs there).
    Finishing,
}

/// The stable id of the D5 progress line (present only while shown).
pub const ADDRESS_PROGRESS_ID: &str = "bbr-address-progress";

/// The stable id of the §17.14 zoom chip in the address accessory slot.
pub const ADDRESS_ZOOM_ID: &str = "bbr-address-zoom";

/// §17.14: the zoom chip shows only while the page zoom is not 100 %.
pub fn zoom_accessory(level: u16) -> Option<u16> {
    (level != 100).then_some(level)
}

/// The §11.5 command **staging** surface's presentation model (`bbr-command-bar`):
/// the input shown only while a non-address staged mode is active without a
/// floating presentation — New Space, Move Tab, and the §11.8 empty New Tab
/// surface. The caller resolves the `CommandBarMode` authority (which stays
/// app-owned with its testkit protocol mapping) into the presentation fields
/// below; the surface itself only renders.
#[derive(Debug, Clone)]
pub struct StagingModel {
    /// §9: whether the browser chrome is focused (the field background and
    /// the accent border follow it).
    pub focused: bool,
    /// The staged command text. Empty renders the "Type a URL…" placeholder
    /// (the legacy field's hint), exactly like the pre-cutover surface.
    pub text: String,
    /// The §11.3 hint suffix, if any.
    pub hint: Option<SharedString>,
    /// Whether the caret renders — `chrome_focused && mode.is_active()`
    /// resolved by the caller.
    pub show_cursor: bool,
    /// Blueprint D02: the caret, the selection and the IME composition over
    /// `text` (byte offsets; the app's editing model owns them).
    pub edit: super::EditState,
    /// Where the app learns the laid-out line; `None` in static renders.
    pub line_sink: Option<super::EditLineSink>,
    /// The §11.3 cross-context profile hint; `None` at rest.
    pub profile_hint: Option<AddressProfileHint>,
}

/// The §11.5 address control: a quiet bordered field carrying the domain
/// only, with a trailing accessory slot. `type:title` domain text
/// (`bbr-address-domain`, mid-ellipsis), the slot (`bbr-address-accessory`)
/// reserving one §14 glyph's `density:hit-min` width so the zoom chip
/// changes no layout, hover wash on both chrome polarities (the border
/// never changes), and button semantics (pointer press opens the §11.4
/// overlay over the control; it never accepts text, shows no caret and no
/// selection). A press on the zoom chip resets the site to 100 % through
/// `on_zoom_reset` and never opens the overlay. The §11.3 cross-context
/// profile hint renders at the
/// leading edge for its one-interaction lifetime. `field_anchor` is the
/// caller-built invisible §11.4 field anchor overlaying the pill to record
/// its window-relative rect; `on_hover`/`on_open` are the production
/// `cx.listener` closures.
pub fn address_control(
    theme: &Theme,
    model: &AddressModel,
    field_anchor: gpui::AnyElement,
    on_hover: impl Fn(&bool, &mut Window, &mut App) + 'static,
    on_open: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_zoom_reset: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_site: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    // Blueprint D11: the leading slot opens site settings; the press never
    // reaches the field's overlay activation.
    let interactive = model.site.interactive;
    let on_site = std::rc::Rc::new(on_site);
    let site_press = move |slot: Stateful<Div>| {
        let on_site = on_site.clone();
        slot.when(interactive, move |slot| {
            slot.cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    cx.stop_propagation();
                    on_site(event, window, cx);
                })
        })
    };
    let indicator = model.site.indicator.map(|indicator| {
        let (id, icon, color) = match indicator {
            SiteIndicator::Live { camera: true } => (
                ADDRESS_CAPTURE_ID,
                crate::icons::Icon::Camera { blocked: false },
                p.status_attention,
            ),
            SiteIndicator::Live { camera: false } => (
                ADDRESS_CAPTURE_ID,
                crate::icons::Icon::Microphone { blocked: false },
                p.status_attention,
            ),
            SiteIndicator::Blocked { camera: true } => (
                ADDRESS_BLOCKED_ID,
                crate::icons::Icon::Camera { blocked: true },
                p.ink_secondary,
            ),
            SiteIndicator::Blocked { camera: false } => (
                ADDRESS_BLOCKED_ID,
                crate::icons::Icon::Microphone { blocked: true },
                p.ink_secondary,
            ),
        };
        site_press.clone()(
            div()
                .id(id)
                .debug_selector(move || id.to_string())
                .size(px(16.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(icon.canvas_with(theme, px(14.0), color)),
        )
    });
    let zoom_chip = model.zoom.map(|level| {
        div()
            .id(ADDRESS_ZOOM_ID)
            .debug_selector(|| ADDRESS_ZOOM_ID.to_string())
            // S12 (Paper): a readable chip — `ink` text at 500 on the
            // accessory fill, with room around the percentage.
            .px(Spacing::sp2())
            .py(px(1.0))
            .rounded(theme.radius.small())
            .bg(p.address_accessory_fill)
            .text_size(theme.type_.meta.size)
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(p.ink)
            .cursor_pointer()
            .child(SharedString::from(format!("{level}%")))
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                // The chip resets the zoom; the press never reaches the
                // field's overlay activation.
                cx.stop_propagation();
                on_zoom_reset(event, window, cx);
            })
    });
    div()
        .id("bbr-address")
        // The relative box is the containing block for the absolute
        // field anchor (§11.4 overlay anchor).
        .relative()
        .mx(Spacing::sp1())
        .my(Spacing::sp2())
        .px(Spacing::sp3())
        // §11.5: pill height = `density:row` (S 32 / N 36 / XL 44);
        // content centers inside the fixed-height pill.
        .h(theme.density.row())
        .rounded(theme.radius.large())
        // §11.5 fill: a white veil one step off the rail, following theme
        // and space tint through the material beneath, plus a
        // hairline-strength border; the hover steps the veil darker
        // (light) / brighter (dark) at `dur:fine` and never touches the
        // border.
        .bg(if model.hovered {
            p.address_fill_hover
        } else {
            p.address_fill
        })
        .border_1()
        .border_color(p.address_border)
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .cursor_pointer()
        // §10.2 production hover path: the wash follows the pointer.
        .on_hover(on_hover)
        // §11.5 activation: pointer press opens the §11.4 overlay with
        // the full URL selected (the production pointer entry, same
        // engine as Cmd-L). The caller's listener stops propagation so the
        // sidebar's click handler (which cancels the command surface)
        // never runs after it.
        .on_mouse_down(MouseButton::Left, on_open)
        .when(model.profile_hint.is_some(), |el| {
            let hint = model.profile_hint.as_ref().expect("checked above");
            el.child(
                div()
                    .id("bbr-command-profile-hint")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(Spacing::sp2())
                    .child(profile_chip(theme, hint.profile_index, &hint.spec))
                    .child(
                        div()
                            .text_size(theme.type_.label.size)
                            .font_weight(theme.type_.label.weight)
                            .text_color(p.ink_secondary)
                            .child(hint.label.clone()),
                    ),
            )
        })
        // Blueprint D03: the leading identity mark (14 pt glyph in a 16 pt
        // slot) — the not-secure triangle at `status.attention`, an
        // internal page's glyph, or the file mark, never color alone (the
        // shape carries it).
        .when_some(model.mark.filter(|_| indicator.is_none()), |el, mark| {
            el.child(site_press(
                div()
                    .id(ADDRESS_MARK_ID)
                    .debug_selector(|| ADDRESS_MARK_ID.to_string())
                    .size(px(16.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(mark.icon().canvas(theme, px(14.0))),
            ))
        })
        .children(indicator)
        .child(
            div()
                .id("bbr-address-domain")
                // §11.5: the domain text yields width by ellipsis
                // (mid-ellipsis at the char budget, GPUI `text_ellipsis`
                // as the pixel-exact backstop); it never pushes the
                // reserved accessory out of the pill.
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink)
                .child(model.domain.clone()),
        )
        // §11.5 trailing accessory slot: reserved geometry (one §14
        // glyph at `density:hit-min`), `flex-shrink-0`; it hosts the
        // §17.14 zoom chip only while zoom is not 100 %. The slot's width
        // never changes, so the chip causes no layout change.
        .child(
            div()
                .id("bbr-address-accessory")
                .w(accessory_slot_width(theme.density))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .children(zoom_chip),
        )
        .when_some(progress_line(theme, model.progress), |el, line| {
            el.child(line)
        })
        // The §11.4 field anchor overlays the pill (absolute, inset-0):
        // invisible, takes no layout space, records the pill rect.
        .child(field_anchor)
}

/// D5/§18.5: the 2 pt accent progress line on the field's inner bottom
/// edge, inset by the corner radius so it stays on the straight edge. The
/// sweep loops only while the tab loads; the finish is a one-shot; Reduce
/// Motion shows a static line while loading and nothing on finish.
fn progress_line(theme: &Theme, progress: AddressProgress) -> Option<gpui::AnyElement> {
    use crate::theme::motion::{progress_finish_sample, progress_sweep_sample, Moment};
    use gpui::{relative, AnimationExt};
    let accent = theme.palette.accent;
    let track = || {
        div()
            .id(ADDRESS_PROGRESS_ID)
            .absolute()
            .bottom(px(0.0))
            .left(theme.radius.large())
            .right(theme.radius.large())
            .h(px(2.0))
            .overflow_hidden()
    };
    let segment = move |left: f32, width: f32, opacity: f32| {
        div()
            .absolute()
            .top(px(0.0))
            .bottom(px(0.0))
            .left(relative(left))
            .w(relative(width))
            .bg(accent)
            .opacity(opacity)
    };
    match progress {
        AddressProgress::Idle => None,
        AddressProgress::Loading => Some(match Moment::AddressProgress.plan(theme.motion_mode) {
            Some(plan) => track()
                .with_animation(
                    "bbr-address-progress-sweep",
                    plan.animation(),
                    move |el, t| {
                        let (left, width) = progress_sweep_sample(t);
                        el.child(segment(left, width, 1.0))
                    },
                )
                .into_any_element(),
            None => track().child(segment(0.0, 1.0, 1.0)).into_any_element(),
        }),
        AddressProgress::Finishing => {
            let plan = Moment::AddressProgressFinish.plan(theme.motion_mode)?;
            Some(
                track()
                    .with_animation(
                        "bbr-address-progress-finish",
                        plan.animation(),
                        move |el, t| {
                            let (width, opacity) = progress_finish_sample(t, 0.3);
                            el.child(segment(0.0, width, opacity))
                        },
                    )
                    .into_any_element(),
            )
        }
    }
}

/// The §11.5 command **staging** surface (`bbr-command-bar`): the input
/// shown only while a non-address staged mode is active without a
/// floating presentation — New Space, Move Tab, and the §11.8 empty New
/// Tab surface. It collects a space name (never an address), keeps its
/// existing field styling, block cursor, and the §11.3 hints, and opens
/// the §11.4 overlay on pointer press like the legacy field. The address
/// control above is never this surface; the two are mutually exclusive by
/// the caller's `command_bar_mode` decision.
pub fn staging_surface(
    theme: &Theme,
    model: &StagingModel,
    field_anchor: gpui::AnyElement,
    on_open: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id("bbr-command-bar")
        .relative()
        .mx(Spacing::sp1())
        .my(Spacing::sp2())
        .px(Spacing::sp3())
        .h(theme.density.row())
        .rounded(theme.radius.large())
        .bg(if model.focused {
            p.surface_field
        } else {
            // At rest the staging field is the same §18.2 veil as the
            // address control it replaces, so it follows the space tint.
            p.address_fill
        })
        .on_mouse_down(MouseButton::Left, on_open)
        .border_2()
        .border_color(if model.focused { p.accent } else { p.hairline })
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .when(model.profile_hint.is_some(), |el| {
            let hint = model.profile_hint.as_ref().expect("checked above");
            el.child(
                div()
                    .id("bbr-command-profile-hint")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(Spacing::sp2())
                    .child(profile_chip(theme, hint.profile_index, &hint.spec))
                    .child(
                        div()
                            .text_size(theme.type_.label.size)
                            .font_weight(theme.type_.label.weight)
                            .text_color(p.ink_secondary)
                            .child(hint.label.clone()),
                    ),
            )
        })
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child("▶"),
        )
        .child(
            div()
                .id("bbr-command-text")
                .flex_1()
                .min_w_0()
                .h(theme.type_.mono.size * 1.5)
                .text_size(theme.type_.mono.size)
                .overflow_hidden()
                .child(super::edit_line(
                    &model.text,
                    model.edit,
                    SharedString::from("Type a URL…"),
                    super::EditLineColors {
                        text: p.ink,
                        placeholder: p.ink_faint,
                        selection: p.tint_accent,
                        caret: p.accent,
                    },
                    model.show_cursor,
                    model.line_sink.clone(),
                )),
        )
        .when(model.hint.is_some(), |el| {
            el.child(
                div()
                    .id("bbr-command-hint")
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_secondary)
                    .child(model.hint.clone().expect("hint is some")),
            )
        })
        .child(field_anchor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::EditState;

    #[test]
    fn the_zoom_chip_shows_only_off_one_hundred_percent() {
        assert_eq!(zoom_accessory(100), None);
        assert_eq!(zoom_accessory(110), Some(110));
        assert_eq!(zoom_accessory(90), Some(90));
    }

    #[test]
    fn address_models_round_trip_their_fields() {
        let hint = AddressProfileHint {
            profile_index: 1,
            spec: ProfileChipSpec {
                letter: "PR".into(),
                hue: bbr_domain::ProfileHue::Violet,
            },
            label: "Switch to Private".into(),
        };
        let model = AddressModel {
            domain: "example.com".into(),
            hovered: false,
            profile_hint: Some(hint.clone()),
            zoom: None,
            progress: AddressProgress::Idle,
            mark: None,
            site: AddressSite::default(),
        };
        assert_eq!(model.domain, "example.com");
        assert!(!model.hovered);
        let carried = model.profile_hint.clone().expect("hint set");
        assert_eq!(carried.profile_index, 1);
        assert_eq!(carried.spec.letter, "PR");
        assert_eq!(carried.label, "Switch to Private");
    }

    #[test]
    fn marks_follow_the_identity_and_secure_sites_show_none() {
        use crate::icons::Icon;
        use crate::text::address_identity;
        let mark = |url: &str| AddressMark::for_identity(&address_identity(url));
        assert_eq!(mark("https://x.com/"), None);
        assert_eq!(mark("http://127.0.0.1:9/"), None);
        assert_eq!(mark("about:blank"), None);
        assert_eq!(mark("http://router.lan/"), Some(AddressMark::NotSecure));
        assert_eq!(mark("http://router.lan/").unwrap().icon(), Icon::Warning);
        assert_eq!(mark("bbr://settings").unwrap().icon(), Icon::Gear);
        assert_eq!(mark("bbr://library").unwrap().icon(), Icon::Bookmark);
        assert_eq!(mark("file:///a/b.pdf"), Some(AddressMark::File));
    }

    #[test]
    fn staging_models_carry_the_presentation_fields() {
        let model = StagingModel {
            focused: true,
            text: "Reading".into(),
            hint: Some("New Space".into()),
            show_cursor: true,
            edit: EditState::end_of("Reading"),
            line_sink: None,
            profile_hint: None,
        };
        assert!(model.focused);
        assert_eq!(model.text, "Reading");
        assert!(model.show_cursor);
        assert_eq!(model.hint.as_deref(), Some("New Space"));
        assert!(model.profile_hint.is_none());
    }
}
