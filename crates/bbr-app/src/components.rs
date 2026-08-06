//! Reusable typed GPUI Chassis components (DESIGN.md §11).
//!
//! Every shell surface composes these instead of inventing local layout:
//! tab rows (pinned/regular anatomy), favicon tiles (monogram fallback),
//! section labels, the close control, the empty-space pane, and the crash
//! panel. Visual state comes from the §10 vocabulary; listeners are passed
//! in so any view can wire these without reimplementing the anatomy.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, ParentElement, SharedString,
    Stateful, Styled, Window,
};

use crate::palette::{FaviconSlate, ProfileHueRamp};
use crate::theme::{Radius, Spacing, Theme, ThemeMode};

/// §10 row states the shell renders and E2E asserts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabVisualState {
    Default,
    Selected,
    Loading,
    Error,
}

/// The favicon tile: the deterministic monogram (§7.2) until a real favicon
/// exists; while loading the slot shows the accent spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaviconSpec {
    pub letter: char,
    pub hue: usize,
}

/// One row's rendered identity, derived from the controller snapshot.
#[derive(Debug, Clone)]
pub struct RowVisuals {
    pub index: usize,
    pub title: SharedString,
    pub state: TabVisualState,
    pub favicon: Option<FaviconSpec>,
    pub show_close: bool,
}

/// §11.2 section label ("Pinned"/"Tabs"), uppercase `type:badge`; the caller
/// hides the label when its section is empty.
pub fn section_label(theme: &Theme, text: &str) -> Div {
    div()
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(theme.palette.ink_faint)
        .child(text.to_uppercase())
}

/// The one space-row visual state the shell renders (DESIGN.md §10/§11.3):
/// the active space is `selected` (accent rail + tint), every other space is
/// `default`. The states vocabulary matches the tab rows so the rail is the
/// primary selection cue in both surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceVisualState {
    Default,
    Selected,
}

/// §12 profile chip identity: the 1–2 character monogram plus the stable
/// round-robin hue the domain assigned to the profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileChipSpec {
    pub letter: String,
    pub hue: bbr_browser::ProfileHue,
}

/// One rendered space row's identity, derived from the controller snapshot.
#[derive(Debug, Clone)]
pub struct SpaceRowVisuals {
    /// The space's stable creation-order index (`bbr-space-row-<n>`).
    pub index: usize,
    /// The owning profile's creation-order index — the stable
    /// `bbr-profile-chip-<n>` identity (DESIGN.md §12), never the space
    /// index, so the rendered chip matches the E2E contract even when space
    /// and profile indices diverge.
    pub profile_index: usize,
    pub title: SharedString,
    /// The §11.3 count column (`type:meta` tabular figures).
    pub count: usize,
    pub state: SpaceVisualState,
    pub chip: ProfileChipSpec,
}

/// The §15 semantic id of a profile chip, keyed to the *profile* index
/// (DESIGN.md §12) — never the space index. The sidebar renders one chip per
/// space row, so a profile's spaces repeat this id; the testkit snapshot
/// exposes each chip id once per profile. This function is the single keying
/// both the render and the snapshot share, so the chip a row renders can
/// never drift from the E2E contract when space and profile indices diverge.
pub fn profile_chip_id(profile_index: usize) -> SharedString {
    SharedString::from(format!("bbr-profile-chip-{profile_index}"))
}

/// §12 profile chip: 20 pt square (18 at S), `radius:sm`, filled with the
/// profile's hue tile and a 1–2 character monogram. The letter derives from
/// the profile title so identity is readable per row without a header.
pub fn profile_chip(theme: &Theme, profile_index: usize, spec: &ProfileChipSpec) -> Stateful<Div> {
    let size = if theme.density == crate::theme::Density::Small {
        px(18.0)
    } else {
        px(20.0)
    };
    let tile = ProfileHueRamp::tile(spec.hue, theme.mode);
    div()
        .id(profile_chip_id(profile_index))
        .size(size)
        .rounded(Radius.small())
        .bg(tile.tile)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(tile.letter)
        .child(spec.letter.clone())
}

/// §11.3 space row anatomy:
/// `[profile chip 20] · sp:2 · [name, trunc] · [count, type:meta tnum]`,
/// `density:space-row` tall. The active space shows the accent rail exactly
/// like a selected tab; `on_select` is the production view listener.
pub fn space_row(
    theme: &Theme,
    visuals: &SpaceRowVisuals,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let selected = visuals.state == SpaceVisualState::Selected;
    div()
        .id(("bbr-space-row", visuals.index))
        .h(theme.density.space_row())
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .bg(if selected {
            p.tint_selected
        } else {
            p.surface_sidebar
        })
        .cursor_pointer()
        .when(selected, |el| {
            el.child(
                div()
                    .w(px(3.0))
                    .h(theme.density.space_row() - px(6.0))
                    .rounded(Radius.small())
                    .bg(p.accent),
            )
        })
        .child(profile_chip(theme, visuals.profile_index, &visuals.chip))
        .child(
            div()
                .id(("bbr-space-title", visuals.index))
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(if selected { p.ink } else { p.ink_secondary })
                .child(visuals.title.clone()),
        )
        .child(
            div()
                .id(("bbr-space-count", visuals.index))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child(visuals.count.to_string()),
        )
        .on_mouse_down(MouseButton::Left, on_select)
}

/// §11.3/§11.5 New Space affordance at the foot of the space region: focuses
/// the command bar in NewSpace mode (the pointer half of `Cmd-Shift-N`).
pub fn space_new_row(
    theme: &Theme,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-space-new")
        .h(theme.density.space_row())
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_select)
        .child(
            div()
                .size(px(20.0))
                .rounded(Radius.small())
                .border_1()
                .border_color(theme.palette.hairline)
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.label.size)
                .text_color(theme.palette.ink_secondary)
                .child("+"),
        )
        .child(
            div()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(theme.palette.ink_secondary)
                .child("New Space"),
        )
}

/// §7.2 favicon tile: 16 pt (S: 14) square, `radius:sm`, monogram letter on
/// the FNV-slate ramp; loading swaps the letter for the accent spinner dot.
pub fn favicon_tile(
    theme: &Theme,
    index: usize,
    spec: Option<FaviconSpec>,
    state: TabVisualState,
) -> Stateful<Div> {
    let size = theme.density.icon();
    let (bg, fg) = match spec {
        Some(spec) => {
            let bg = match theme.mode {
                ThemeMode::Dark => FaviconSlate::dark(spec.hue),
                ThemeMode::Light => FaviconSlate::light(spec.hue),
            };
            (bg, theme.palette.surface_glass)
        }
        None => (theme.palette.surface_field, theme.palette.ink_faint),
    };
    let letter = spec.map(|s| s.letter).unwrap_or('?');
    div()
        .id(("bbr-tab-favicon", index))
        .size(size)
        .rounded(Radius.small())
        .bg(bg)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(fg)
        .child(if state == TabVisualState::Loading {
            SharedString::from("●")
        } else {
            SharedString::from(letter.to_string())
        })
}

/// §11.1 row title: one line, end-ellipsis, `type:title`; the §11.9 fallback
/// and the §4 separator-aware truncation have already been applied by the
/// caller. Loading dims to `ink.secondary`.
pub fn row_title(theme: &Theme, index: usize, text: &str, state: TabVisualState) -> Stateful<Div> {
    let color = if state == TabVisualState::Loading {
        theme.palette.ink_secondary
    } else {
        theme.palette.ink
    };
    div()
        .id(("bbr-tab-title", index))
        .flex_1()
        .overflow_hidden()
        .text_ellipsis()
        .text_size(theme.type_.title.size)
        .font_weight(theme.type_.title.weight)
        .text_color(color)
        .child(SharedString::from(text.to_string()))
}

/// §11.1 state slot: the fixed 8 pt trailing slot that later holds the
/// `attention` dot (§10.6). Present on every row so the `bbr-tab-state-<n>`
/// assert id (§15) exists; empty until an attention surface ships.
pub fn row_state_slot(theme: &Theme, index: usize) -> Stateful<Div> {
    div()
        .id(("bbr-tab-state", index))
        .size(px(8.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_faint)
}

/// §11.1 close control: 16 pt glyph in a ≥ 24 pt hit square. Always
/// interactive from the first slice so the semantic id and the keyboard
/// equivalent exist; the hover-reveal polish lands with the hover slice.
pub fn close_control(theme: &Theme, index: usize) -> Stateful<Div> {
    div()
        .id(("bbr-tab-close", index))
        .size(theme.density.icon())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_secondary)
        .child("×")
}

/// §11.1 vertical tab row anatomy:
/// `[favicon 16] · [title flex, trunc] · [state slot 8] · [close 16]`,
/// tile per §8 with the selected rail at the leading edge.
///
/// `on_select`/`on_close` are the production view listeners (built with
/// `cx.listener` in `render.rs`), so the pointer path stays in the app layer.
/// The close control is a frontmost child hitbox: in the bubble phase its
/// handler runs before the row's and stops propagation, so a click on the
/// close glyph closes without first selecting the row.
pub fn tab_row(
    theme: &Theme,
    visuals: &RowVisuals,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let bg = match visuals.state {
        TabVisualState::Selected => p.tint_selected,
        TabVisualState::Error => p.status_error_surface,
        _ => p.surface_sidebar,
    };
    let close = close_control(theme, visuals.index)
        .on_mouse_down(MouseButton::Left, on_close)
        .when(visuals.show_close, |el| el.cursor_pointer());
    div()
        .id(("bbr-tab-row", visuals.index))
        .h(theme.density.row())
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .bg(bg)
        .cursor_pointer()
        .when(visuals.state == TabVisualState::Selected, |el| {
            el.child(
                div()
                    .w(px(3.0))
                    .h(theme.density.row_tile())
                    .rounded(Radius.small())
                    .bg(p.accent),
            )
        })
        .child(favicon_tile(
            theme,
            visuals.index,
            visuals.favicon,
            visuals.state,
        ))
        .child(row_title(
            theme,
            visuals.index,
            &visuals.title,
            visuals.state,
        ))
        .child(row_state_slot(theme, visuals.index))
        .child(close)
        .on_mouse_down(MouseButton::Left, on_select)
}

/// §11.8 empty space pane: `material:canvas` mark, `type:display` headline,
/// profile line, and a primary "New Tab ⌘T" button that focuses the command
/// bar in NewTab mode. `profile_line` names the owning profile (§11.8).
pub fn empty_space_pane(
    theme: &Theme,
    profile_line: &str,
    on_new_tab: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-empty-new-tab")
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .bg(theme.palette.surface_canvas)
        .child(
            div()
                .size(px(64.0))
                .rounded(theme.radius.large())
                .bg(theme.palette.surface_sidebar)
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.display.size)
                .font_weight(theme.type_.display.weight)
                .text_color(theme.palette.ink_secondary)
                .child("B"),
        )
        .child(
            div()
                .text_size(theme.type_.display.size)
                .font_weight(theme.type_.display.weight)
                .text_color(theme.palette.ink)
                .child("No tabs in this space"),
        )
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_secondary)
                .child(profile_line.to_string()),
        )
        .child(
            div()
                .id("bbr-empty-new-tab-button")
                .mt(Spacing::sp3())
                .px(Spacing::sp4())
                .py(Spacing::sp2())
                .rounded(theme.radius.medium())
                .bg(theme.palette.accent)
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(theme.palette.surface_glass)
                .on_mouse_down(MouseButton::Left, on_new_tab)
                .child("New Tab  ⌘T"),
        )
}

/// §11.8 crash panel: replaces the glass area of a crashed tab with the
/// lifering glyph, copy, and Reload / Close tab actions. The crashed tab row
/// stays selected and listed; the native view is hidden.
pub fn crash_panel(
    theme: &Theme,
    on_reload: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-crash-panel")
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .bg(theme.palette.surface_field)
        .child(
            div()
                .size(px(32.0))
                .rounded(theme.radius.full())
                .border_2()
                .border_color(theme.palette.status_error)
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.label.size)
                .text_color(theme.palette.status_error)
                .child("!"),
        )
        .child(
            div()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(theme.palette.ink)
                .child("This page crashed"),
        )
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_secondary)
                .child("The tab stays open; reload to restore it."),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(Spacing::sp3())
                .mt(Spacing::sp2())
                .child(
                    div()
                        .id("bbr-crash-reload")
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .bg(theme.palette.accent)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.surface_glass)
                        .on_mouse_down(MouseButton::Left, on_reload)
                        .child("Reload"),
                )
                .child(
                    div()
                        .id("bbr-crash-close")
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .border_1()
                        .border_color(theme.palette.border_strong)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .on_mouse_down(MouseButton::Left, on_close)
                        .child("Close tab"),
                ),
        )
}
