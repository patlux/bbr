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
use gpui::{Context, Window};

use crate::command_bar::CommandBarMode;
use crate::portal::command::CommandPortalModel;
use crate::portal::geometry::{Edges, FitMode, PortalAnchor, Pt, Sz};
use crate::portal::state::{
    AnchoredPlacement, DismissalPolicy, PortalPlacement, PortalRequest, PresentationId,
};

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
/// The overlay's own field row (DESIGN.md §11.4): the full current URL is
/// preselected here, so one keystroke replaces it.
pub const OVERLAY_FIELD_ID: &str = "bbr-command-overlay-field";
pub const OVERLAY_TEXT_ID: &str = "bbr-command-overlay-text";
pub const OVERLAY_CURSOR_ID: &str = "bbr-command-overlay-cursor";

pub const PALETTE_PRESENTATION_ID: PresentationId = PresentationId(0xC1);
pub const OVERLAY_PRESENTATION_ID: PresentationId = PresentationId(0xC2);
pub const COMMAND_PORTAL_PRIORITY: usize = 2;

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
            let mut ids = vec![
                OVERLAY_ID.into(),
                OVERLAY_FIELD_ID.into(),
                OVERLAY_TEXT_ID.into(),
                OVERLAY_CURSOR_ID.into(),
            ];
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
        // Command surfaces and the Library are mutually exclusive in both
        // directions: Cmd-T replaces the open internal chrome surface before
        // staging New Tab mode and presenting the native command portal.
        if self.library.open {
            self.close_saved_pages_library(window, cx);
        }
        // Opening a command surface replaces any open find or context-menu
        // presentation (exactly one active portal surface per window).
        self.find_discard(cx);
        self.menu_discard(cx);
        self.dismiss_command_portal(cx);
        self.command_surface = CommandSurface::Palette;
        self.activate_new_tab(window, cx);
        self.present_command_portal(CommandSurface::Palette, window, cx);
    }

    /// Open the field-anchored overlay (`bbr-command-overlay`) in Location
    /// mode with the full URL selected in the field: the production Cmd-L /
    /// field pointer-click path (DESIGN.md §11.4). Same §11.5 engine and
    /// activation as the in-rail command bar. `Cmd-L` and the field
    /// pointer-click route here.
    pub fn open_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Cmd-L/pointer entry has the same symmetric Library replacement
        // contract as Cmd-T; no hidden native view or trapped Library focus
        // may survive under the command portal.
        if self.library.open {
            self.close_saved_pages_library(window, cx);
        }
        self.find_discard(cx);
        self.menu_discard(cx);
        self.dismiss_command_portal(cx);
        self.command_surface = CommandSurface::Overlay;
        self.activate_command_bar(window, cx);
        self.present_command_portal(CommandSurface::Overlay, window, cx);
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
        self.cancel_command_bar(window, cx);
    }

    fn command_portal_request(&self, surface: CommandSurface) -> PortalRequest {
        let theme = self.theme();
        match surface {
            CommandSurface::None => unreachable!("closed command surface has no request"),
            CommandSurface::Palette => PortalRequest::native(
                PALETTE_PRESENTATION_ID,
                PortalPlacement::Centered {
                    size: Sz::new(f32::from(theme.density.palette_width()), 420.0),
                    margin: Edges::all(8.0),
                },
                COMMAND_PORTAL_PRIORITY,
                DismissalPolicy::any(),
            ),
            CommandSurface::Overlay => {
                let rect = self
                    .command_field_rect
                    .borrow()
                    .as_ref()
                    .copied()
                    .unwrap_or(WebRect::new(
                        12.0,
                        48.0,
                        360.0,
                        f32::from(theme.density.row()),
                    ));
                // §11.4: the anchored overlay's width is the field width
                // clamped to 360–`win:palette`.
                let width = rect
                    .width
                    .clamp(360.0, f32::from(theme.density.palette_width()));
                PortalRequest::native(
                    OVERLAY_PRESENTATION_ID,
                    overlay_placement(rect, width),
                    COMMAND_PORTAL_PRIORITY,
                    DismissalPolicy::any(),
                )
            }
        }
    }

    pub(crate) fn present_command_portal(
        &mut self,
        surface: CommandSurface,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let request = self.command_portal_request(surface);
        let viewport = Sz::new(
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        );
        self.portal_probe
            .borrow_mut()
            .present_command(request, viewport, surface);
        self.activate_portal_request(request, cx);
    }

    pub(crate) fn dismiss_command_portal(&mut self, cx: &mut Context<Self>) {
        if self.command_surface.is_open() {
            self.portal_dismiss(cx);
        }
        self.command_surface = CommandSurface::None;
    }

    pub(crate) fn command_portal_model(&self, surface: CommandSurface) -> CommandPortalModel {
        CommandPortalModel {
            surface,
            theme: self.theme(),
            chrome_focused: surface.is_open(),
            text: self.command_text.text.clone(),
            state: self.command_ranking.state,
            results: self.command_ranking.results.clone(),
            selected: self.command_ranking.selected,
        }
    }

    #[cfg(feature = "testkit")]
    pub(crate) fn observe_portal_command_render(&self, surface: CommandSurface) {
        self.performance.observe_rendered_surface(surface);
    }
}

/// §11.4: the anchored overlay **covers** the §11.5 address control rather
/// than dropping below it — top edge flush with the control's top edge,
/// leading edge flush with the control's leading edge (DESIGN.md §11.4).
/// The address control is not visible while the overlay is open: the
/// overlay takes its place, so the address never appears twice.
fn overlay_placement(field: WebRect, width: f32) -> PortalPlacement {
    PortalPlacement::Anchored(AnchoredPlacement {
        anchor_point: Pt::new(field.x, field.y),
        anchor: PortalAnchor::TopLeft,
        size: Sz::new(width, 320.0),
        offset: Pt::new(0.0, 0.0),
        fit: FitMode::SnapToWindowWithMargin(Edges::all(8.0)),
    })
}
