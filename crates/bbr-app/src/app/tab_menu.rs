//! Production Tab/Favorite context-menu composition (task packet
//! `tab-favorite-context-menus`; DESIGN.md §11.13/§15) — the app-owned
//! authority half of the menu.
//!
//! `bbr-app` remains the sole menu-state authority: it resolves the exact
//! persistent `TabId` that invoked the menu (a sidebar tab row or a
//! Favorite tile — the Favorite is another presentation of the same concrete
//! tab), computes the availability matrix through the portable
//! `bbr-browser::TabMenuController`, derives the per-source folder/space
//! destinations and the B2 bookmark state, maps everything onto the
//! presentation-only `bbr-ui::surfaces::tab_menu` DTO with pre-bound
//! callbacks, resolves the clamped placement, and owns dismissal/focus
//! choreography. `bbr-ui` sees only a plain presentation model and
//! callbacks — never a `TabId`, space, profile, session, or controller
//! identifier.
//!
//! **No-open mutation.** Opening, traversing, hovering, and dismissing the
//! menu never activates, selects, moves, reloads, closes, recreates, or
//! changes the profile/space/session of the source tab. Every action row
//! dispatches **exactly one typed command** against the bound `TabId`
//! through an existing portable authority:
//!
//! - browser-owned rows (`Reload`, `TogglePin`, `Suspend`, `CopyLink`,
//!   `MoveToStart`, `MoveToEnd`, `Close`, `ReopenTab`, `MoveToSpace`) route
//!   through [`bbr_browser::TabMenuController::dispatch`], which re-resolves
//!   the live controller state at commit;
//! - organization-owned rows (`MoveToFolder`, `ToggleFavorite`) route
//!   through `OrganizationController` commands keyed by the `TabId`;
//! - B2 rows (`BookmarkSave` / `BookmarkRemove`) route through the one
//!   app-owned `SavedPageCommand` and never retain the `TabId` in the Saved
//!   Page.
//!
//! Rows whose typed command authority does not exist yet (Duplicate Tab,
//! native Share, split/media/rename/icon/bulk) are omitted entirely — never
//! decorative dead rows.

use bbr_browser::{FolderDestination, SpaceDestination, TabMenuAvailability, TabMenuSource};
use bbr_domain::{FolderId, SpaceId, TabId};
use bbr_ui::surfaces::tab_menu::{TabMenuModel, TabMenuRect, TabMenuRow as PresentRow};

/// The stable card width of the Tab/Favorite menu: the §11.4 command-surface
/// family's compact clamp minimum — settled chrome geometry, not an invented
/// size (the same width the page menu uses).
pub const TAB_MENU_WIDTH: f32 = 220.0;
/// The placement margin (DESIGN §5 spacing / §11.4 compact clamp).
pub const TAB_MENU_MARGIN: f32 = 8.0;
/// The §15 id of the Tab-row menu (stable presentation source key).
pub const TAB_MENU_SOURCE: &str = "bbr-tab-menu";
/// The §15 id of the Favorite menu (the same menu family; the app registers
/// the id it presents).
pub const FAVORITE_MENU_SOURCE: &str = "bbr-favorite-menu";

/// The typed action one menu row dispatches. Rows without an authority are
/// never built, so this enum is exactly the set of owned actions today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabMenuAction {
    Reload,
    TogglePin,
    Suspend,
    CopyLink,
    MoveToStart,
    MoveToEnd,
    Close,
    ReopenTab,
    MoveToSpace(SpaceId),
    MoveToFolder(FolderId),
    ToggleFavorite,
    BookmarkSave,
    BookmarkRemove,
}

/// One deterministic app-side menu row: the presentation fields (stable §15
/// id, display label, genuine enablement, danger flag) plus the optional
/// typed action to dispatch and the optional one-level submenu rows. The
/// row carries no domain identifier — the bound `TabId` lives on the menu
/// state. A submenu parent (`action: None`) is never dispatachable: its
/// activation opens the submenu card instead of committing a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabMenuRow {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub danger: bool,
    pub action: Option<TabMenuAction>,
    pub submenu: Option<Vec<TabMenuRow>>,
}

impl TabMenuRow {
    fn action(
        id: impl Into<String>,
        label: impl Into<String>,
        enabled: bool,
        action: TabMenuAction,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled,
            danger: false,
            action: Some(action),
            submenu: None,
        }
    }

    fn danger(id: impl Into<String>, label: impl Into<String>, action: TabMenuAction) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
            danger: true,
            action: Some(action),
            submenu: None,
        }
    }

    /// A submenu parent row. The parent carries **no** typed action —
    /// activating it opens its submenu card; the submenu rows own the
    /// commits. Never a placeholder command.
    fn submenu(id: impl Into<String>, label: impl Into<String>, submenu: Vec<TabMenuRow>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
            danger: false,
            action: None,
            submenu: Some(submenu),
        }
    }
}

/// The app-owned menu presentation state (mirrored on the wire as
/// `bbr_testkit::protocol::TabMenuSnapshot`). Plain data: the render and the
/// testkit snapshot both derive from this same state. The bound `TabId` is
/// app-internal by design — it never crosses into `bbr-ui` or the wire.
#[derive(Debug, Clone, PartialEq)]
pub struct TabMenuState {
    /// The menu presentation is open for this window.
    pub open: bool,
    /// The menu owns keyboard focus (the main-window menu focus handle).
    pub focused: bool,
    /// The exact persistent `TabId` the menu is bound to; `None` while
    /// closed.
    pub bound_tab: Option<TabId>,
    /// The stable presentation source key (`bbr-tab-menu` |
    /// `bbr-favorite-menu`).
    pub source_key: String,
    /// The keyboard cursor / hovered top-level row.
    pub selected: Option<usize>,
    /// The top-level row whose submenu card is currently open.
    pub open_submenu: Option<usize>,
    /// The keyboard cursor / hovered row inside the open submenu.
    pub submenu_selected: Option<usize>,
    /// The deterministic top-level row model in render order.
    pub rows: Vec<TabMenuRow>,
    /// The resolved card placement (window content coordinates, points).
    pub anchor: TabMenuRect,
    /// The resolved submenu card placement, when a submenu is open.
    pub submenu_rect: Option<TabMenuRect>,
    /// The focus domain captured at open; dismissal restores it exactly.
    pub origin_chrome_focused: bool,
    /// §13 one-pointer-action marker: a pointer press inside the open card
    /// sets it (its handler runs first in the bubble phase) so the root
    /// outside-press handler never dismisses a press inside the menu.
    pub press_inside: bool,
    /// The deferred submenu close: leaving the open submenu's parent row
    /// sets it (the pointer may be crossing into the submenu card); the
    /// next top-level row enter or submenu row enter resolves it. Keeps the
    /// submenu from flickering while the pointer crosses the card boundary.
    pub pending_submenu_close: bool,
}

impl Default for TabMenuState {
    fn default() -> Self {
        Self {
            open: false,
            focused: false,
            bound_tab: None,
            source_key: String::new(),
            selected: None,
            open_submenu: None,
            submenu_selected: None,
            rows: Vec::new(),
            anchor: TabMenuRect::new(0.0, 0.0, 0.0, 0.0),
            submenu_rect: None,
            origin_chrome_focused: false,
            press_inside: false,
            pending_submenu_close: false,
        }
    }
}

impl TabMenuState {
    /// The resolved `bbr-ui` presentation model for the current render.
    pub fn model(&self) -> TabMenuModel {
        TabMenuModel {
            focused: self.open && self.focused,
            selected: self.selected,
            open_submenu: self.open_submenu,
            submenu_selected: self.submenu_selected,
            rows: self.rows.iter().map(present_row).collect(),
            source_key: self.source_key.clone().into(),
        }
    }

    /// The top-level row at `index` (for a submenu commit, the open
    /// submenu's row at `index`).
    pub fn row_at(&self, index: usize) -> Option<&TabMenuRow> {
        self.rows.get(index)
    }

    /// The submenu row at `index` of the open submenu.
    pub fn submenu_row_at(&self, index: usize) -> Option<&TabMenuRow> {
        let parent = self.open_submenu?;
        self.rows.get(parent)?.submenu.as_ref()?.get(index)
    }

    /// The first enabled top-level row (the keyboard cursor's landing spot).
    pub fn first_enabled(&self) -> Option<usize> {
        self.rows.iter().position(|row| row.enabled)
    }

    /// Move the top-level keyboard cursor `delta` rows, skipping disabled
    /// entries (wrap-around). The cursor always lands on an enabled row or
    /// stays put when none is enabled. Returns whether the selection moved.
    pub fn move_selection(&mut self, delta: isize) -> bool {
        move_cursor(&self.rows, &mut self.selected, delta)
    }

    /// Move the open submenu's cursor `delta` rows (skipping disabled
    /// entries). Returns whether the selection moved.
    pub fn move_submenu_selection(&mut self, delta: isize) -> bool {
        let Some(parent) = self.open_submenu else {
            return false;
        };
        let Some(rows) = self.rows.get(parent).and_then(|row| row.submenu.as_ref()) else {
            return false;
        };
        move_cursor(rows, &mut self.submenu_selected, delta)
    }

    /// Open the top-level row at `index`'s submenu and land the cursor on
    /// its first enabled row. No-op when the row has no submenu.
    pub fn open_submenu_at(&mut self, index: usize) {
        let has_submenu = self
            .rows
            .get(index)
            .is_some_and(|row| row.submenu.is_some());
        if !has_submenu {
            return;
        }
        self.open_submenu = Some(index);
        self.submenu_selected = self.rows[index]
            .submenu
            .as_ref()
            .and_then(|rows| rows.iter().position(|row| row.enabled));
    }

    /// Close the open submenu (the keyboard cursor returns to the parent).
    pub fn close_submenu(&mut self) {
        self.open_submenu = None;
        self.submenu_selected = None;
    }

    /// Recompute the open submenu's resolved placement from the current
    /// anchor and rows (edge flip included), whenever the submenu opens or
    /// the relevant selection changes. `None` when no submenu is open —
    /// the destination card must visibly render and edge-flip against the
    /// live window size/theme the caller passes.
    pub fn recompute_submenu_rect(&mut self, window_size: [f32; 2], theme: &bbr_ui::theme::Theme) {
        self.submenu_rect = match self.open_submenu {
            Some(index) => self
                .rows
                .get(index)
                .and_then(|row| row.submenu.as_ref())
                .map(|submenu| submenu_rect(self.anchor, submenu, window_size, theme)),
            None => None,
        };
    }
}

/// The shared keyboard/pointer cursor move: skip disabled entries with
/// wrap-around (a cursor over a disabled row would be a dead end for both
/// pointer and keyboard, so a hovered disabled row keeps the previous
/// cursor).
fn move_cursor(rows: &[TabMenuRow], selected: &mut Option<usize>, delta: isize) -> bool {
    let enabled: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].enabled).collect();
    if enabled.is_empty() {
        return false;
    }
    let current = selected
        .and_then(|s| enabled.iter().position(|&i| i == s))
        .unwrap_or(0);
    let next = (current as isize + delta).rem_euclid(enabled.len() as isize) as usize;
    let target = enabled[next];
    if *selected == Some(target) {
        return false;
    }
    *selected = Some(target);
    true
}

/// Map one app-side row onto the presentation-only `bbr-ui` DTO (the id,
/// label, enablement, danger, and nested submenu rows — never an action or
/// domain identifier).
fn present_row(row: &TabMenuRow) -> PresentRow {
    PresentRow {
        id: row.id.clone().into(),
        label: row.label.clone().into(),
        enabled: row.enabled,
        danger: row.danger,
        submenu: row
            .submenu
            .as_ref()
            .map(|rows| rows.iter().map(present_row).collect()),
    }
}

/// The deterministic top-level row model for one resolved source, in render
/// order (the packet's group order: placement → page/session → lifecycle →
/// system/share → ordering → close). `bookmark_save` / `bookmark_remove`
/// mirror the B2 Saved Page state for the source's current URL (the B2
/// authority is app-owned; the row is omitted when the URL is not
/// navigable). A submenu group with no destinations is omitted whole —
/// never a decorative dead parent.
pub fn build_rows(
    _source: &TabMenuSource,
    availability: &TabMenuAvailability,
    bookmark_save: bool,
    bookmark_remove: bool,
) -> Vec<TabMenuRow> {
    let mut rows = Vec::new();
    // Placement: Move to Folder › (the source space's folders).
    if !availability.folders.is_empty() {
        rows.push(TabMenuRow::submenu(
            "bbr-tab-menu-move-folder",
            "Move to Folder",
            availability
                .folders
                .iter()
                .enumerate()
                .map(|(n, folder)| {
                    TabMenuRow::action(
                        format!("bbr-tab-menu-folder-{n}"),
                        &folder.title,
                        true,
                        TabMenuAction::MoveToFolder(folder.folder),
                    )
                })
                .collect(),
        ));
    }
    // Placement: Move to Space › (every other space; cross-profile
    // destinations carry the visible profile identity, DESIGN §11.3).
    if !availability.spaces.is_empty() {
        rows.push(TabMenuRow::submenu(
            "bbr-tab-menu-move-space",
            "Move to Space",
            availability
                .spaces
                .iter()
                .enumerate()
                .map(|(n, space)| {
                    TabMenuRow::action(
                        format!("bbr-tab-menu-space-{n}"),
                        space_label(space),
                        true,
                        TabMenuAction::MoveToSpace(space.space),
                    )
                })
                .collect(),
        ));
    }
    // Page/session: Reload Tab.
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-reload",
        "Reload Tab",
        availability.reload,
        TabMenuAction::Reload,
    ));
    // Page/session: the Favorite toggle (the label follows the source's
    // favorite state; a Favorite is another presentation of the same tab).
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-favorite",
        if availability.favorite {
            "Remove from Favorites"
        } else {
            "Add to Favorites"
        },
        true,
        TabMenuAction::ToggleFavorite,
    ));
    // Page/session: the B2 bookmark row (capability-gated; omitted when the
    // URL is not navigable for a durable Saved Page).
    if bookmark_save || bookmark_remove {
        rows.push(TabMenuRow::action(
            bbr_ui::surfaces::tab_menu::TAB_MENU_BOOKMARK_ID,
            if bookmark_remove {
                "Remove Bookmark"
            } else {
                "Bookmark Page"
            },
            true,
            if bookmark_remove {
                TabMenuAction::BookmarkRemove
            } else {
                TabMenuAction::BookmarkSave
            },
        ));
    }
    // Lifecycle: Pin / Unpin, Suspend.
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-pin",
        if availability.pinned {
            "Unpin Tab"
        } else {
            "Pin Tab"
        },
        true,
        TabMenuAction::TogglePin,
    ));
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-suspend",
        "Suspend Tab",
        availability.suspend,
        TabMenuAction::Suspend,
    ));
    // System/share: Copy Link (the engine owns the pasteboard).
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-copy-link",
        "Copy Link",
        availability.copy_link,
        TabMenuAction::CopyLink,
    ));
    // Ordering: Move to Start / Move to End within the source's region.
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-move-start",
        "Move to Start",
        availability.move_to_start,
        TabMenuAction::MoveToStart,
    ));
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-move-end",
        "Move to End",
        availability.move_to_end,
        TabMenuAction::MoveToEnd,
    ));
    // Close: Close Tab (danger) and Reopen Closed Tab where legal.
    rows.push(TabMenuRow::danger(
        "bbr-tab-menu-close-tab",
        "Close Tab",
        TabMenuAction::Close,
    ));
    rows.push(TabMenuRow::action(
        "bbr-tab-menu-reopen",
        "Reopen Closed Tab",
        availability.reopen,
        TabMenuAction::ReopenTab,
    ));
    rows
}

/// The `Move to Space` destination label: the space title, plus the visible
/// profile identity for cross-profile destinations (DESIGN.md §11.3 — the
/// isolation is never communicated by color alone).
fn space_label(space: &SpaceDestination) -> String {
    match space.profile_title.as_deref() {
        Some(profile) => format!("{} · {}", space.title, profile),
        None => space.title.clone(),
    }
}

/// The resolved clamped placement of the menu card and its open submenu, in
/// window content coordinates (points, top-left origin). The card anchors at
/// the invoking pointer, clamped to the window with the §11.4 margin; the
/// submenu card draws to the right of the parent (edge-flipped to the left
/// when the window's right edge would clip it) with its top clamped. Pure
/// geometry — the caller passes the current window size, the theme (the
/// settled row tokens), and the derived rows.
pub fn tab_menu_placement(
    point: [f32; 2],
    rows: &[TabMenuRow],
    open_submenu: Option<usize>,
    window_size: [f32; 2],
    theme: &bbr_ui::theme::Theme,
) -> (TabMenuRect, Option<TabMenuRect>) {
    let card = card_rect(point, rows, window_size, theme);
    let submenu_rect = open_submenu
        .and_then(|index| rows.get(index).and_then(|row| row.submenu.as_ref()))
        .map(|submenu| submenu_rect(card, submenu, window_size, theme));
    (card, submenu_rect)
}

/// The card rect anchored at the pointer, clamped fully on screen.
fn card_rect(
    point: [f32; 2],
    rows: &[TabMenuRow],
    window_size: [f32; 2],
    theme: &bbr_ui::theme::Theme,
) -> TabMenuRect {
    let width = TAB_MENU_WIDTH;
    let height = menu_height(rows.len(), theme);
    let max_x = (window_size[0] - (TAB_MENU_MARGIN + width)).max(0.0);
    let max_y = (window_size[1] - (TAB_MENU_MARGIN + height)).max(0.0);
    TabMenuRect::new(
        clamp(point[0], TAB_MENU_MARGIN, max_x),
        clamp(point[1], TAB_MENU_MARGIN, max_y),
        width,
        height,
    )
}

/// The submenu card: to the right of the parent card at the parent's top,
/// flipped to the left when the right edge would clip, top clamped.
fn submenu_rect(
    card: TabMenuRect,
    rows: &[TabMenuRow],
    window_size: [f32; 2],
    theme: &bbr_ui::theme::Theme,
) -> TabMenuRect {
    let width = TAB_MENU_WIDTH;
    let height = menu_height(rows.len(), theme);
    let right_edge = card.x + card.width + width;
    let x = if right_edge + TAB_MENU_MARGIN <= window_size[0] {
        card.x + card.width
    } else {
        (card.x - width).max(TAB_MENU_MARGIN)
    };
    let max_y = (window_size[1] - (TAB_MENU_MARGIN + height)).max(0.0);
    let y = clamp(card.y, TAB_MENU_MARGIN, max_y);
    TabMenuRect::new(x, y, width, height)
}

/// The card height from the row model: `density:row`-tall entries
/// (`label.size` + the `sp:2` row padding) plus the `sp:1` card padding —
/// settled tokens, never local literals.
fn menu_height(row_count: usize, theme: &bbr_ui::theme::Theme) -> f32 {
    let row_height =
        f32::from(theme.type_.label.size) + 2.0 * f32::from(bbr_ui::theme::Spacing::sp2());
    let card_padding = 2.0 * f32::from(bbr_ui::theme::Spacing::sp1());
    row_height * row_count as f32 + card_padding
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max.max(min))
}

/// The resolved destination folders for the source space (the organization
/// read model filtered to the source's owning space — membership is
/// per-space).
pub fn source_folders(
    organization: &mut bbr_browser::OrganizationController,
    controller: &bbr_browser::BrowserController,
    space: SpaceId,
) -> Vec<FolderDestination> {
    organization
        .organization_snapshot(controller)
        .folders
        .iter()
        .filter(|folder| folder.space == space)
        .map(|folder| FolderDestination {
            folder: folder.id,
            title: folder.title.clone(),
        })
        .collect()
}

/// The app-side tab-menu controller: owns the presentation state and the
/// deterministic dismissal/activation rules. There is no session or adapter
/// to hold — the menu is pure chrome over the sidebar, so the controller is
/// exactly its state.
#[derive(Debug, Default)]
pub struct TabMenuController {
    pub(crate) state: TabMenuState,
}

impl TabMenuController {
    /// A fresh controller with an empty (closed) menu state.
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

mod app;
#[cfg(test)]
mod tests;
