//! BBR context menus over web content (task packet
//! `bbr-context-menus-over-web`, queue item 12, part B).
//!
//! The BBR-owned page menu: Back, Forward, Reload, Copy Link and
//! Open in New Tab, rendered inside the **one** `NativeAboveWeb` portal
//! window (never a second host, never an AppKit `NSMenu` over the web view,
//! never a screenshot composite). The portal window is non-activating and
//! never key by design, so keyboard input is handled by the main window's
//! GPUI focus ([`BbrApp::menu_handle`]) and the portal renders the same live
//! state.
//!
//! ## Ownership routing (the documented product/native decision)
//!
//! Right-clicks over a live WKWebView are observed by the native bridge in
//! `bbr-web-macos` ([`NativeContextMenuBridge`](bbr_web_macos::NativeContextMenuBridge));
//! WebKit always builds its own menu and the bridge classifies it at
//! `NSMenuWillOpenNotification` time. A page-level menu (all non-empty
//! titles in {Back, Forward, Reload, Inspect Element}) is cancelled and
//! re-requested as [`WebEvent::ContextMenuRequested`], which arrives here
//! through the exact engine event transport and presents this menu anchored
//! at the pointer. Any element menu (Open Link, Save Image, Copy, …) is
//! left entirely to WebKit — BBR never replaces or restyles it.
//!
//! ## Contract invariants (enforced by construction)
//!
//! - Opening, dismissing, and activating never create, destroy, move, or
//!   reselect a web session: the menu is bound to the **active** view at
//!   open (a request for a non-active view is a no-op) and its actions
//!   route through the same seams as the keyboard shortcuts
//!   ([`BbrApp::go_back`]/[`go_forward`]/[`reload`]) plus the engine's
//!   per-view `WebAction::CopyLink` and the production `open_tab` — only
//!   Open in New Tab intentionally opens exactly one new tab/session.
//! - Back/Forward enablement is genuine: it mirrors the session's real
//!   `CanGoBackChanged`/`CanGoForwardChanged` history availability, never a
//!   guess.
//! - The menu is dismissed by Escape, by an outside press (the portal
//!   backdrop's one-pointer-action), and by activating an entry; focus
//!   returns to the domain captured at open (`origin_chrome_focused`).
//! - Placement follows the pointer within the window and stays fully on
//!   screen: the anchored placement uses [`FitMode::SnapToWindowWithMargin`]
//!   (pure geometry, unit-tested).
//! - Exactly one active portal presentation per window: opening the menu
//!   discards an open find bar or command surface, and their opens discard
//!   the menu.

use bbr_web::{WebRect, WebViewId};

use crate::portal::geometry::{Edges, FitMode, PortalAnchor, Pt, Sz};
use crate::portal::state::PortalPlacement;
use crate::theme::Theme;

/// The menu's stable presentation id (allocated in the portal
/// presentation-id space next to the probe `0xB9`, the command surfaces
/// `0xC1`/`0xC2`, and the find bar `0xC3`).
pub const CONTEXT_MENU_PRESENTATION_ID: crate::portal::state::PresentationId =
    crate::portal::state::PresentationId(0xC4);
/// The menu's portal draw priority (same slot as the command surfaces and
/// the find bar).
pub const CONTEXT_MENU_PRIORITY: usize = 2;
/// The menu card width in points: the §11.4 command-surface family's
/// compact clamp minimum — settled chrome geometry, not an invented size.
pub const CONTEXT_MENU_WIDTH: f32 = 220.0;
/// The portal placement margin (DESIGN §5 spacing / §11.4 compact clamp).
pub const CONTEXT_MENU_MARGIN: Edges = Edges::all(8.0);

/// Stable §15 ids of the context menu (DESIGN.md §15; exposed through the
/// portal snapshot's typed `ids` field, registered in
/// [`crate::portal::probe::PortalProbeState::ids`]).
pub const CONTEXT_MENU_ID: &str = "bbr-context-menu";
pub const CONTEXT_MENU_BACKDROP_ID: &str = "bbr-context-menu-backdrop";
pub const CONTEXT_MENU_ROW_PREFIX: &str = "bbr-context-menu-row";

/// Stable item keys (the row id suffix and the snapshot's `key` field).
pub const MENU_BACK: &str = "back";
pub const MENU_FORWARD: &str = "forward";
pub const MENU_RELOAD: &str = "reload";
pub const MENU_COPY_LINK: &str = "copy-link";
pub const MENU_OPEN_NEW_TAB: &str = "open-new-tab";

/// One deterministic menu row: stable key, display title, and the genuine
/// enablement computed at open (Back/Forward from the session's real
/// history availability; Copy Link / Open in New Tab from a resolvable
/// page URL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuItem {
    pub key: &'static str,
    pub title: &'static str,
    pub enabled: bool,
}

impl ContextMenuItem {
    fn new(key: &'static str, title: &'static str, enabled: bool) -> Self {
        Self {
            key,
            title,
            enabled,
        }
    }
}

/// The app-side context-menu presentation state (mirrored on the wire as
/// `bbr_testkit::protocol::ContextMenuSnapshot`). Plain data: the portal
/// render and the testkit snapshot both derive from this same state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContextMenuState {
    /// The menu presentation is open for this window.
    pub open: bool,
    /// The menu owns keyboard focus (the main-window menu focus handle).
    pub focused: bool,
    /// The keyboard cursor / hovered row.
    pub selected: Option<usize>,
    /// The deterministic row model in render order.
    pub items: Vec<ContextMenuItem>,
    /// The active web view the menu was requested on.
    pub bound_view: Option<WebViewId>,
    /// The pointer anchor in window content coordinates (points, top-left
    /// origin) the placement resolved from.
    pub point: [f32; 2],
    /// The page URL the menu acts on (Copy Link / Open in New Tab).
    pub url: Option<String>,
    /// The focus domain captured at open; dismissal restores it exactly.
    pub origin_chrome_focused: bool,
}

/// The app-side context-menu controller: owns the presentation state and
/// the deterministic dismissal/activation rules. There is no session or
/// adapter to hold — the menu is pure chrome over the active view, so the
/// controller is exactly its state.
#[derive(Debug, Default)]
pub struct ContextMenuController {
    pub(crate) state: ContextMenuState,
}

impl ContextMenuController {
    /// A fresh controller with an empty (closed) menu state.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// The deterministic row model for the active view: Back/Forward mirror
    /// the session's real history availability, Reload is always enabled,
    /// and Copy Link / Open in New Tab need a resolvable URL.
    pub(crate) fn build_items(
        can_go_back: bool,
        can_go_forward: bool,
        url: Option<&str>,
    ) -> Vec<ContextMenuItem> {
        let link_enabled = url.is_some_and(|u| !u.is_empty());
        vec![
            ContextMenuItem::new(MENU_BACK, "Back", can_go_back),
            ContextMenuItem::new(MENU_FORWARD, "Forward", can_go_forward),
            ContextMenuItem::new(MENU_RELOAD, "Reload", true),
            ContextMenuItem::new(MENU_COPY_LINK, "Copy Link", link_enabled),
            ContextMenuItem::new(MENU_OPEN_NEW_TAB, "Open in New Tab", link_enabled),
        ]
    }

    /// The first enabled row (the keyboard cursor's landing spot at open).
    pub(crate) fn first_enabled(items: &[ContextMenuItem]) -> Option<usize> {
        items.iter().position(|item| item.enabled)
    }

    /// Move the keyboard cursor `delta` rows, skipping disabled entries
    /// (wrap-around within the enabled rows). The cursor always lands on an
    /// enabled row or stays put when none is enabled. Returns whether the
    /// selection moved.
    pub(crate) fn menu_move_selection(&mut self, delta: isize) -> bool {
        let enabled: Vec<usize> = (0..self.state.items.len())
            .filter(|&i| self.state.items[i].enabled)
            .collect();
        if enabled.is_empty() {
            return false;
        }
        let current = self
            .state
            .selected
            .and_then(|selected| enabled.iter().position(|&i| i == selected))
            .unwrap_or(0);
        let next = (current as isize + delta).rem_euclid(enabled.len() as isize) as usize;
        let selected = enabled[next];
        if self.state.selected == Some(selected) {
            return false;
        }
        self.state.selected = Some(selected);
        true
    }
}

/// The immutable menu data one portal-window render consumes.
#[derive(Clone, Debug)]
pub struct ContextMenuSurface {
    pub theme: Theme,
    pub focused: bool,
    pub selected: Option<usize>,
    pub items: Vec<ContextMenuItem>,
}

/// The menu's anchored placement: the pointer anchor with the top-left
/// corner attached, snapped to the window keeping the margin so the menu
/// always stays fully on screen. Width is the settled command-surface
/// compact clamp; the height derives from the row model (label size plus
/// the `sp:2` row padding) plus the `sp:1` card padding — settled tokens,
/// never local literals.
fn context_menu_placement(point: WebRect, theme: &Theme) -> PortalPlacement {
    let row_height =
        f32::from(theme.type_.label.size) + 2.0 * f32::from(crate::theme::Spacing::sp2());
    let card_padding = 2.0 * f32::from(crate::theme::Spacing::sp1());
    let size = Sz::new(CONTEXT_MENU_WIDTH, 5.0 * row_height + card_padding);
    PortalPlacement::Anchored(crate::portal::state::AnchoredPlacement {
        anchor_point: Pt::new(point.x, point.y),
        anchor: PortalAnchor::TopLeft,
        size,
        offset: Pt::new(0.0, 0.0),
        fit: FitMode::SnapToWindowWithMargin(CONTEXT_MENU_MARGIN),
    })
}

/// The app-side integration (open/select/activate/close/key/snapshot
/// methods) lives in the sibling `app` module, split by responsibility so
/// each file stays under the ~500-line budget.
mod app;

#[cfg(test)]
mod tests;
