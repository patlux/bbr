//! The §11.4 command-surface presentation state and render (DESIGN.md
//! §11.4/§13/§15).
//!
//! The sidebar command field owns two presentations of the same §11.5
//! engine: the **centered Cmd-T palette** (`bbr-palette`) in New Tab staging
//! mode and the **field-anchored Cmd-L / field-click overlay**
//! (`bbr-command-overlay`) in Location mode. This module owns the
//! presentation state — which surface is open — the stable §15 ids of both
//! surfaces, the routing seams the production entry paths (Cmd-T, Cmd-L,
//! field pointer-click) call, and the `elev:2` render both surfaces share.
//! The in-rail command bar (DESIGN.md §11.5) stays the baseline surface;
//! the palette and overlay are the only chrome allowed to overlap the glass
//! (§13).
//!
//! Contract invariants (enforced by construction):
//!
//! - At most one command surface is open at a time (a single enum field).
//! - A §11.5 mode entry keeps a presentation open only when it re-enters
//!   the coupled mode (palette ↔ New Tab staging, overlay ↔ Location);
//!   any other mode closes it back to the in-rail bar
//!   ([`CommandSurface::coupled_surface`]).
//! - Opening either surface clears the §11.3 cross-context profile hint
//!   (the interaction-contract lifetime boundary).
//! - Escape/commit/outside press (one pointer action, §13) close through
//!   the existing `cancel_command_bar` / `focus_web` exits, which restore
//!   the origin field text and focus domain unchanged.
//!
//! The overlay is anchored to the **measured** command-field rect (recorded
//! every frame by [`anchor::CommandFieldAnchor`]), so its placement never duplicates
//! the chrome layout math; the native content slot geometry is untouched by
//! either surface (the glass stays exactly where it is, §13).

use bbr_web::WebRect;
use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Window,
};

use crate::command_bar::CommandBarMode;
use crate::theme::{Density, Spacing, Theme, ThemeMode};

use super::command_rank::{CommandResultRow, CommandResultState};
use super::command_surface::{STATE_ID_EMPTY, STATE_ID_ERROR, STATE_ID_LOADING};
use super::BbrApp;

/// The §11.4 field-anchor element and the command-surface unit tests live in
/// submodules of this presentation module.
mod anchor;
pub(crate) use anchor::command_field_anchor;
#[cfg(all(test, feature = "testkit"))]
mod tests;

/// The open command-surface presentation (DESIGN.md §11.4). At most one is
/// open at a time; `None` when the in-rail §11.5 command bar is the active
/// surface (or nothing is).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandSurface {
    /// No palette/overlay presentation is open.
    #[default]
    None,
    /// The centered Cmd-T palette (`bbr-palette`) in New Tab staging mode.
    Palette,
    /// The field-anchored Cmd-L / field-click overlay (`bbr-command-overlay`)
    /// in Location mode.
    Overlay,
}

impl CommandSurface {
    /// True while a palette/overlay presentation is open.
    pub fn is_open(self) -> bool {
        !matches!(self, Self::None)
    }

    /// The §11.5 command-bar mode the open surface couples to: New Tab
    /// staging for the centered palette, Location for the field-anchored
    /// overlay. `Inactive` for the closed state (the in-rail surface owns
    /// its own mode).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn mode(self) -> CommandBarMode {
        match self {
            CommandSurface::None => CommandBarMode::Inactive,
            CommandSurface::Palette => CommandBarMode::NewTab,
            CommandSurface::Overlay => CommandBarMode::Location,
        }
    }

    /// The surface that survives a §11.5 mode entry: only the open
    /// presentation whose coupled mode is re-entered stays (palette ↔ New
    /// Tab staging, overlay ↔ Location); any other combination closes the
    /// presentation back to the in-rail command bar. This keeps the
    /// invariant "a surface is open only in its coupled mode" by
    /// construction (DESIGN.md §11.4 mutual exclusion).
    pub fn coupled_surface(self, mode: CommandBarMode) -> CommandSurface {
        match (self, mode) {
            (CommandSurface::Palette, CommandBarMode::NewTab) => CommandSurface::Palette,
            (CommandSurface::Overlay, CommandBarMode::Location) => CommandSurface::Overlay,
            _ => CommandSurface::None,
        }
    }
}

#[cfg(feature = "testkit")]
impl From<CommandSurface> for bbr_testkit::protocol::CommandSurface {
    fn from(surface: CommandSurface) -> Self {
        match surface {
            CommandSurface::None => Self::None,
            CommandSurface::Palette => Self::Palette,
            CommandSurface::Overlay => Self::Overlay,
        }
    }
}

/// Stable §15 ids of the centered palette (DESIGN.md §11.4/§15).
pub const PALETTE_ID: &str = "bbr-palette";
pub const PALETTE_FIELD_ID: &str = "bbr-palette-field";
pub const PALETTE_TEXT_ID: &str = "bbr-palette-text";
pub const PALETTE_MODE_ID: &str = "bbr-palette-mode";
pub const PALETTE_CURSOR_ID: &str = "bbr-palette-cursor";
/// Result rows of the centered palette keep the §11.5 row anatomy under
/// `bbr-palette-result-<k>`.
pub const PALETTE_RESULT_PREFIX: &str = "bbr-palette-result";
/// Typed state rows of the centered palette (loading / empty / error).
pub const PALETTE_STATE_LOADING: &str = "bbr-palette-loading";
pub const PALETTE_STATE_EMPTY: &str = "bbr-palette-empty";
pub const PALETTE_STATE_ERROR: &str = "bbr-palette-error";
/// The field-anchored overlay container (DESIGN.md §11.4); its results keep
/// the §11.5 `bbr-command-result-<k>` ids of the in-rail surface.
pub const OVERLAY_ID: &str = "bbr-command-overlay";

/// The stable ids the open surface exposes (DESIGN.md §15): the centered
/// palette with its field row (`-field`/`-text`/`-mode`/`-cursor`) and its
/// published result rows (`bbr-palette-result-<k>`, or the typed state row),
/// or the field-anchored overlay container with its `bbr-command-result-<k>`
/// rows. Shared by the render and the `visible_ids` derivation so E2E ids
/// can never drift from the shell. `None` when no surface is open.
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub fn surface_ids(
    surface: CommandSurface,
    state: CommandResultState,
    results: &[CommandResultRow],
) -> Vec<String> {
    match surface {
        CommandSurface::None => Vec::new(),
        CommandSurface::Palette => {
            let mut ids = [
                PALETTE_ID,
                PALETTE_FIELD_ID,
                PALETTE_TEXT_ID,
                PALETTE_MODE_ID,
                PALETTE_CURSOR_ID,
            ]
            .map(String::from)
            .to_vec();
            match state {
                CommandResultState::Results => ids.extend(
                    results
                        .iter()
                        .map(|row| format!("{PALETTE_RESULT_PREFIX}-{}", row.key)),
                ),
                CommandResultState::Loading => ids.push(PALETTE_STATE_LOADING.into()),
                CommandResultState::Empty => ids.push(PALETTE_STATE_EMPTY.into()),
                CommandResultState::ActivationError => ids.push(PALETTE_STATE_ERROR.into()),
                CommandResultState::Inactive => {}
            }
            ids
        }
        CommandSurface::Overlay => {
            let mut ids = vec![OVERLAY_ID.into()];
            match state {
                CommandResultState::Results => ids.extend(
                    results
                        .iter()
                        .map(|row| format!("bbr-command-result-{}", row.key)),
                ),
                CommandResultState::Loading => ids.push(STATE_ID_LOADING.into()),
                CommandResultState::Empty => ids.push(STATE_ID_EMPTY.into()),
                CommandResultState::ActivationError => ids.push(STATE_ID_ERROR.into()),
                CommandResultState::Inactive => {}
            }
            ids
        }
    }
}

impl BbrApp {
    /// Open the centered Cmd-T palette (`bbr-palette`) in New Tab staging
    /// mode: the same §11.5 engine, ranking, and staged semantics as the
    /// in-rail command bar (no tab or session before Enter; Escape cancels).
    /// The production `Cmd-T` binding routes here. Opening replaces any open
    /// surface presentation (single-state field → mutual exclusion) and
    /// clears the §11.3 profile hint through the shared command-bar entry.
    pub fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_surface = CommandSurface::Palette;
        self.activate_new_tab(window, cx);
    }

    /// Open the field-anchored overlay (`bbr-command-overlay`) in Location
    /// mode with the full URL selected in the field: the production Cmd-L /
    /// field pointer-click path (DESIGN.md §11.4). Same §11.5 engine and
    /// activation as the in-rail command bar. `Cmd-L` and the field
    /// pointer-click route here.
    pub fn open_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_surface = CommandSurface::Overlay;
        self.activate_command_bar(window, cx);
    }

    /// Close the open palette/overlay through the production Escape path:
    /// restores the origin field text and focus domain captured at open and
    /// returns the surface state to `None` (DESIGN.md §11.4). No-op when no
    /// surface presentation is open. Also the §13 one-pointer-action
    /// dismissal (the surface layer's mouse-down routes here).
    pub fn close_command_surface(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.command_surface.is_open() {
            return;
        }
        self.command_surface = CommandSurface::None;
        self.cancel_command_bar(window, cx);
    }

    /// The §11.4 open command-surface presentation: the centered Cmd-T
    /// palette or the field-anchored Cmd-L/click overlay, rendered as the
    /// root's topmost child (`elev:2` — the only chrome allowed to overlap
    /// the glass, §13). `None` when the in-rail §11.5 command bar is the
    /// active surface. The native content slot keeps its exact geometry
    /// either way.
    pub(crate) fn command_surface_presentation(
        &self,
        theme: &Theme,
        chrome_focused: bool,
        sidebar_width: f32,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        match self.command_surface {
            CommandSurface::None => None,
            CommandSurface::Palette => {
                Some(self.palette_presentation(theme, chrome_focused, sidebar_width, cx))
            }
            CommandSurface::Overlay => Some(self.overlay_presentation(theme, sidebar_width, cx)),
        }
    }

    /// The centered palette (DESIGN.md §11.4): `elev:2` `material:overlay`
    /// card, `win:palette` wide, `radius:lg` at S/N and `radius:xl` at XL,
    /// centered over the window content on a §13 dismissal layer (one
    /// pointer press outside the card closes it; the rail stays
    /// interactive).
    fn palette_presentation(
        &self,
        theme: &Theme,
        chrome_focused: bool,
        sidebar_width: f32,
        cx: &mut Context<Self>,
    ) -> Div {
        let p = theme.palette;
        let radius = match theme.density {
            Density::ExtraLarge => theme.radius.xlarge(),
            _ => theme.radius.large(),
        };
        let hold = cx.listener(|_this, _e: &MouseDownEvent, _window, cx| cx.stop_propagation());
        let card = div()
            .id(PALETTE_ID)
            .w(theme.density.palette_width())
            .rounded(radius)
            .bg(p.surface_overlay)
            .border_1()
            .border_color(p.hairline)
            .shadow(elev2_shadows(theme))
            .flex()
            .flex_col()
            .on_mouse_down(MouseButton::Left, hold)
            .child(self.palette_field(theme, chrome_focused))
            .children(self.surface_results(theme, PALETTE_RESULT_PREFIX, "bbr-palette"));
        self.surface_layer(cx, sidebar_width, card)
            .flex()
            .items_center()
            .justify_center()
    }

    /// The palette's field row (`bbr-palette-field`): the New Tab prefix
    /// glyph, the `type:mono` text (`bbr-palette-text`), the staged New Tab
    /// mode chip (`bbr-palette-mode`, `accent.ink`), and the block cursor
    /// (`bbr-palette-cursor`) while chrome focus is in the surface.
    fn palette_field(&self, theme: &Theme, chrome_focused: bool) -> Stateful<Div> {
        let p = theme.palette;
        let text = if self.command_text.is_empty() {
            SharedString::from("Type a URL…")
        } else {
            SharedString::from(self.command_text.text.clone())
        };
        div()
            .id(PALETTE_FIELD_ID)
            .px(Spacing::sp3())
            .py(Spacing::sp2())
            .gap(Spacing::sp2())
            .flex()
            .flex_row()
            .items_center()
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child("+"),
            )
            .child(
                div()
                    .id(PALETTE_TEXT_ID)
                    .flex_1()
                    .text_size(theme.type_.mono.size)
                    .text_color(p.ink)
                    .child(text),
            )
            .child(
                div()
                    .id(PALETTE_MODE_ID)
                    .text_size(theme.type_.badge.size)
                    .font_weight(theme.type_.badge.weight)
                    .text_color(p.accent_ink)
                    .child("New Tab"),
            )
            .when(chrome_focused && self.command_bar_mode.is_active(), |el| {
                el.child(div().id(PALETTE_CURSOR_ID).w(px(2.0)).h_full().bg(p.accent))
            })
    }

    /// The field-anchored overlay (DESIGN.md §11.4): `elev:2`
    /// `material:overlay` card anchored to the **measured** command field —
    /// top edge flush with the field's bottom edge, leading edge flush with
    /// its leading edge, width = field width clamped to 320–`win:palette`.
    /// Same result engine as the in-rail bar; results keep the §11.5
    /// `bbr-command-result-<k>` ids. The §13 dismissal layer covers the
    /// content area right of the rail.
    fn overlay_presentation(
        &self,
        theme: &Theme,
        sidebar_width: f32,
        cx: &mut Context<Self>,
    ) -> Div {
        let p = theme.palette;
        // The exact anchor is the field rect recorded by CommandFieldAnchor
        // on the last frame (the field is stationary while the surface is
        // open). Before the field rendered — or while the rail is collapsed
        // — a deterministic top-left fallback keeps the surface present.
        let rect = self
            .command_field_rect
            .borrow()
            .as_ref()
            .copied()
            .unwrap_or(WebRect::new(12.0, 48.0, 320.0, 34.0));
        let width = rect
            .width
            .clamp(320.0, f32::from(theme.density.palette_width()));
        let hold = cx.listener(|_this, _e: &MouseDownEvent, _window, cx| cx.stop_propagation());
        let card = div()
            .id(OVERLAY_ID)
            .absolute()
            // `rect` is window-absolute (CommandFieldAnchor records
            // `window.layout_bounds`), but the card's containing block is the
            // §13 surface layer, whose left edge starts at `x = sidebar_width`
            // (the rail width). Subtracting the rail width keeps the card's
            // leading edge flush with the field's leading edge (DESIGN.md
            // §11.4); the vertical inset is already correct because the layer
            // spans the full window height (top_0/bottom_0).
            .left(px(overlay_left_inset(rect.x, sidebar_width)))
            .top(px(rect.y + rect.height))
            .w(px(width))
            .rounded(theme.radius.large())
            .bg(p.surface_overlay)
            .border_1()
            .border_color(p.hairline)
            .shadow(elev2_shadows(theme))
            .py(Spacing::sp1())
            .on_mouse_down(MouseButton::Left, hold)
            .children(self.surface_results(theme, "bbr-command-result", "bbr-command"));
        self.surface_layer(cx, sidebar_width, card)
    }

    /// The §13 dismissal layer: an absolute pane over the window content
    /// right of the rail; one pointer press anywhere on it closes the open
    /// surface through the production Escape path (origin text and focus
    /// domain restored). The rail itself stays interactive — the field
    /// re-focuses the overlay, and other rail clicks dismiss through
    /// `focus_chrome`.
    fn surface_layer(
        &self,
        cx: &mut Context<Self>,
        sidebar_width: f32,
        card: Stateful<Div>,
    ) -> Div {
        let close = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.close_command_surface(window, cx);
        });
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .left(px(sidebar_width))
            .right_0()
            .on_mouse_down(MouseButton::Left, close)
            .child(card)
    }
}

/// The §2 `elev:2` shadow pair (DESIGN.md §2): a 1 pt light spread for the
/// hairline edge on dark material plus the ambient drop; the light theme
/// uses the hairline offset + the softer drop. The shadows are the only
/// paint the floating surface adds over the glass (§13 allows `elev:2`).
fn elev2_shadows(theme: &Theme) -> Vec<gpui::BoxShadow> {
    use gpui::{hsla, point, BoxShadow};
    match theme.mode {
        ThemeMode::Dark => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 1.0, 0.05),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(0.0),
                spread_radius: px(1.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.45),
                offset: point(px(0.0), px(16.0)),
                blur_radius: px(32.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
        ThemeMode::Light => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.04),
                offset: point(px(0.0), px(1.0)),
                blur_radius: px(0.0),
                spread_radius: px(0.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.09, 0.10),
                offset: point(px(0.0), px(12.0)),
                blur_radius: px(24.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
    }
}

/// The §11.4 overlay's horizontal inset inside the §13 surface layer
/// (DESIGN.md §11.4): the measured field rect is window-absolute
/// (CommandFieldAnchor records `window.layout_bounds`), but the overlay
/// card's containing block is the surface layer, whose left edge starts at
/// `x = sidebar_width` (the rail width). The card's window-x is therefore
/// `sidebar_width + inset`, so subtracting the rail width keeps the leading
/// edge flush with the field's leading edge.
fn overlay_left_inset(field_rect_x: f32, sidebar_width: f32) -> f32 {
    field_rect_x - sidebar_width
}
