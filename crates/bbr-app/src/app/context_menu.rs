//! BBR context menus over web content (task packets
//! `bbr-context-menus-over-web` and `page-background-context-menu`).
//!
//! The BBR-owned page-background menu: Back, Forward, Reload, the
//! capability-gated Bookmark Page / Remove Bookmark row (B2 state), and
//! Copy Page Link, rendered inside the **one** `NativeAboveWeb` portal
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
//! ## Owned commands (the enrichment)
//!
//! Only currently owned commands render. `Select All` has no portable
//! `WebAction` yet, `Share…` has no native adapter yet, and Save Page As /
//! View Page Source / Inspect have no integrated capability — all are
//! omitted rather than invented. `Open in New Tab` is no longer a page-menu
//! command (the enriched command set is exactly Back / Forward / Reload /
//! Bookmark / Copy Page Link). The bookmark row is capability-gated by B2's
//! saved state: `Bookmark Page` when the canonical URL has no active
//! record, `Remove Bookmark` when it has one, omitted for a missing or
//! non-navigable URL — and it invokes the same `SavedPageCommand::toggle`
//! authority as Cmd-D.
//!
//! ## Contract invariants (enforced by construction)
//!
//! - Opening, dismissing, and activating never create, destroy, move, or
//!   reselect a web session: the menu is bound to the **active** view at
//!   open (a request for a non-active view is a no-op) and its actions
//!   route through the same seams as the keyboard shortcuts
//!   ([`BbrApp::go_back`]/[`go_forward`]/[`reload`]), the engine's per-view
//!   `WebAction::CopyLink`, and B2's one app-owned Saved Page command.
//! - Back/Forward enablement is genuine: it mirrors the session's real
//!   `CanGoBackChanged`/`CanGoForwardChanged` history availability, never a
//!   guess. Disabled rows carry a deterministic reason (DESIGN §11.13).
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
use bbr_ui::theme::Theme;

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

/// Stable §15 id of the page-menu backdrop (the full-window surface that
/// owns the §13 one-pointer-action dismissal; exposed through the portal
/// snapshot's typed `ids` field, registered in
/// [`crate::portal::probe::PortalProbeState::ids`]). The card id
/// (`bbr-page-menu`) and the per-action row ids moved to
/// `bbr_ui::surfaces::menu` with the card render / the row model.
pub const CONTEXT_MENU_BACKDROP_ID: &str = "bbr-page-menu-backdrop";

/// Stable item keys (the per-action row id suffix and the snapshot's `key`
/// field). The row's stable §15 id is `bbr-page-menu-<key>` (the
/// page-menu packet's per-action ids: `bbr-page-menu-back`, …,
/// `bbr-page-menu-bookmark`, `bbr-page-menu-copy-link`).
pub const MENU_BACK: &str = "back";
pub const MENU_FORWARD: &str = "forward";
pub const MENU_RELOAD: &str = "reload";
pub const MENU_BOOKMARK: &str = "bookmark";
pub const MENU_COPY_LINK: &str = "copy-link";

/// The B2 bookmark row's availability at open (the capability gate). Only
/// currently owned commands render; a missing capability omits its row
/// rather than inventing behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkRow {
    /// The canonical URL has no active Saved Page record: the enabled
    /// `Bookmark Page` row renders.
    Save,
    /// The canonical URL has an active record: the enabled
    /// `Remove Bookmark` row renders.
    Remove,
    /// The URL is missing or not navigable through the canonical seam: no
    /// bookmark row renders (nothing to bookmark truthfully).
    Unavailable,
}

/// One deterministic menu row: stable key, stable per-action §15 id,
/// display title, the genuine enablement computed at open (Back/Forward
/// from the session's real history availability; Copy Page Link from a
/// resolvable page URL; the bookmark row from B2 state), and the
/// deterministic disabled reason (DESIGN §11.13 — never a guess).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuItem {
    pub key: &'static str,
    /// The stable §15 id (`bbr-page-menu-<key>`).
    pub id: &'static str,
    pub title: &'static str,
    pub enabled: bool,
    /// `Some(reason)` for disabled rows (DESIGN §11.13: disabled rows stay
    /// visible when their group is useful and expose a deterministic
    /// reason); `None` for enabled rows.
    pub disabled_reason: Option<&'static str>,
}

impl ContextMenuItem {
    fn new(
        key: &'static str,
        id: &'static str,
        title: &'static str,
        enabled: bool,
        disabled_reason: Option<&'static str>,
    ) -> Self {
        Self {
            key,
            id,
            title,
            enabled,
            disabled_reason,
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
    /// The page URL the menu acts on (Copy Page Link).
    pub url: Option<String>,
    /// The captured page title at open (B2's bookmark row invokes the one
    /// app-owned command against the focused page's URL/title; the display
    /// ladder falls back to the URL when empty).
    pub title: Option<String>,
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
    /// the bookmark row is capability-gated by B2's saved state (omitted
    /// when the URL is missing or not navigable), and Copy Page Link needs
    /// a resolvable URL. Only currently owned commands render: Select All /
    /// Share / Save Page As / View Page Source / Inspect have no integrated
    /// capability and are never invented.
    pub(crate) fn build_items(
        can_go_back: bool,
        can_go_forward: bool,
        url: Option<&str>,
        bookmark: BookmarkRow,
    ) -> Vec<ContextMenuItem> {
        let link_enabled = url.is_some_and(|u| !u.is_empty());
        let mut items = vec![
            ContextMenuItem::new(
                MENU_BACK,
                "bbr-page-menu-back",
                "Back",
                can_go_back,
                (!can_go_back).then_some("No back history"),
            ),
            ContextMenuItem::new(
                MENU_FORWARD,
                "bbr-page-menu-forward",
                "Forward",
                can_go_forward,
                (!can_go_forward).then_some("No forward history"),
            ),
            ContextMenuItem::new(MENU_RELOAD, "bbr-page-menu-reload", "Reload", true, None),
        ];
        match bookmark {
            BookmarkRow::Save => items.push(ContextMenuItem::new(
                MENU_BOOKMARK,
                "bbr-page-menu-bookmark",
                "Bookmark Page",
                true,
                None,
            )),
            BookmarkRow::Remove => items.push(ContextMenuItem::new(
                MENU_BOOKMARK,
                "bbr-page-menu-bookmark",
                "Remove Bookmark",
                true,
                None,
            )),
            BookmarkRow::Unavailable => {}
        }
        items.push(ContextMenuItem::new(
            MENU_COPY_LINK,
            "bbr-page-menu-copy-link",
            "Copy Page Link",
            link_enabled,
            (!link_enabled).then_some("No page URL to copy"),
        ));
        items
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

/// The immutable menu data one portal-window render consumes: the resolved
/// theme plus the `bbr-ui` presentation model the card renders. The app
/// builds the model from the controller state (the anchor's DTO-building
/// role); the portal render reads the same live surface per render, so
/// selection and enablement can never drift from the app.
#[derive(Clone, Debug)]
pub struct ContextMenuSurface {
    pub theme: Theme,
    pub model: bbr_ui::surfaces::menu::ContextMenuModel,
}

/// The menu's anchored placement: the pointer anchor with the top-left
/// corner attached, snapped to the window keeping the margin so the menu
/// always stays fully on screen. Width is the settled command-surface
/// compact clamp; the height derives from the **actual** row model (label
/// size plus the `sp:2` row padding, `sp:1` card padding — settled tokens,
/// never local literals), so the card always fits its rows exactly.
fn context_menu_placement(point: WebRect, items: usize, theme: &Theme) -> PortalPlacement {
    let row_height =
        f32::from(theme.type_.label.size) + 2.0 * f32::from(bbr_ui::theme::Spacing::sp2());
    let card_padding = 2.0 * f32::from(bbr_ui::theme::Spacing::sp1());
    let size = Sz::new(CONTEXT_MENU_WIDTH, items as f32 * row_height + card_padding);
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
