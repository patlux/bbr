//! The G3 Sidebar composition's single input contract (packet
//! `component-gallery-compositions.md`): the plain [`SidebarModel`] DTO and
//! the one [`Callbacks`] set of handler closures.
//!
//! `bbr-app` is the assembler: it builds the model from the controller
//! snapshot (dense window, row visuals, favorites/folders/spaces DTOs,
//! status, command slot, footer) and fills every callback slot with the
//! production app methods (re-entering the app entity). The gallery fills
//! recorder callbacks. The rail composition ([`super::render`]) renders the
//! model through the production `bbr-ui` components and never holds any
//! browser/session/profile/space/ranking/persistence authority.
//!
//! The model is plain data: no GPUI entity, no controller snapshot, no
//! engine, no portal host, and no pre-built element. GPUI handles
//! (focus/scroll) travel in the model; the app-owned child elements (the
//! command field anchor, the command-result ranking surface, the downloads
//! panel) are passed separately at the render/composition boundary so the
//! DTO stays genuinely `Clone` + `Debug`.

use bbr_domain::{FolderId, SpaceId};
use gpui::{
    App, FocusHandle, KeyDownEvent, MouseDownEvent, MouseUpEvent, ScrollHandle, ScrollWheelEvent,
    SharedString, Window,
};

use crate::components::address::{AddressModel, StagingModel};
use crate::components::{
    FavoriteTileSpec, FolderMemberVisual, FolderRowSpec, RowVisuals, StatusModel,
};
use crate::state::SpaceChipVisuals;
use crate::theme::SpaceTint;
use crate::titlebar::{TitlebarCallbacks, TitlebarModel};

/// One handler closure slot of the sidebar `Callbacks` set. The app fills
/// every slot with the production app methods (re-entering the app entity);
/// the gallery fills recorder callbacks. `Rc<dyn Fn>` so the rail can
/// clone a slot into per-row listeners. The event references are
/// higher-ranked so the rail can call a slot with any event lifetime.
pub type MouseCallback = std::rc::Rc<dyn for<'a> Fn(&'a MouseDownEvent, &mut Window, &mut App)>;
/// A `MouseUpEvent` callback slot.
pub type MouseUpCallback = std::rc::Rc<dyn for<'a> Fn(&'a MouseUpEvent, &mut Window, &mut App)>;
/// A `KeyDownEvent` callback slot.
pub type KeyCallback = std::rc::Rc<dyn for<'a> Fn(&'a KeyDownEvent, &mut Window, &mut App)>;
/// A `ScrollWheelEvent` callback slot.
pub type WheelCallback = std::rc::Rc<dyn for<'a> Fn(&'a ScrollWheelEvent, &mut Window, &mut App)>;
/// A `&bool` callback slot (hover entries).
pub type BoolCallback = std::rc::Rc<dyn for<'a> Fn(&'a bool, &mut Window, &mut App)>;
/// A row-indexed `MouseDownEvent` callback slot.
pub type RowCallback =
    std::rc::Rc<dyn for<'a> Fn(usize, &'a MouseDownEvent, &mut Window, &mut App)>;
/// A row-indexed `MouseUpEvent` callback slot.
pub type RowUpCallback =
    std::rc::Rc<dyn for<'a> Fn(usize, &'a MouseUpEvent, &mut Window, &mut App)>;
/// A row-indexed hover callback slot.
pub type HoverCallback = std::rc::Rc<dyn Fn(usize, bool, &mut Window, &mut App)>;
/// A folder-id `MouseDownEvent` callback slot.
pub type FolderCallback =
    std::rc::Rc<dyn for<'a> Fn(FolderId, &'a MouseDownEvent, &mut Window, &mut App)>;
/// A folder-id hover callback slot (§11.11 count reveal).
pub type FolderHoverCallback = std::rc::Rc<dyn Fn(FolderId, bool, &mut Window, &mut App)>;
/// A folder-id + member `MouseDownEvent` callback slot.
pub type MemberCallback =
    std::rc::Rc<dyn for<'a> Fn(FolderId, usize, &'a MouseDownEvent, &mut Window, &mut App)>;
/// A space-id `MouseDownEvent` callback slot.
pub type SpaceCallback =
    std::rc::Rc<dyn for<'a> Fn(SpaceId, &'a MouseDownEvent, &mut Window, &mut App)>;

/// The §11.5 command slot: the address pill at rest, or the non-address
/// command staging surface while a staged mode is active. The app resolves
/// its `CommandBarMode` authority into this presentation enum; the rail
/// renders the matching production `bbr-ui` component.
#[derive(Debug, Clone)]
pub enum CommandSlot {
    /// The §11.5 address pill (`bbr-address`).
    Address(AddressModel),
    /// The §11.5 command staging surface (`bbr-command-bar`).
    Staging(StagingModel),
}

/// The §11.10 favorites region data: the deterministic tile grid plus the
/// §10.9 drag-target wash while a tab drag targets the grid.
#[derive(Debug, Clone, Default)]
pub struct FavoritesModel {
    /// The tiles in deterministic insertion order (`bbr-fav-<n>`).
    pub tiles: Vec<FavoriteTileSpec>,
    /// §10.9: a tab drag currently targets the grid (a drop toggles the
    /// dragged tab as a favorite) — the band or the drop cell is the live
    /// target.
    pub targeted: bool,
    /// §11.10: a tab drag is active, so a populated grid offers the drop
    /// cell at the end of the wrap order (the zero-favorites band is its
    /// own target and needs no extra cell).
    pub offer: bool,
    /// §18.5 favorite pop: `(tab id, moment key)` of the favorite that was
    /// just committed; its tile plays the pop once.
    pub pop: Option<(u64, u64)>,
}

/// The §11.11 folders region data: the folder rows plus the nested member
/// rows of expanded folders, and the region's keyboard-focus state.
#[derive(Debug, Clone, Default)]
pub struct FoldersModel {
    /// The folder rows in creation order (`bbr-folder-row-<n>`).
    pub rows: Vec<FolderRowSpec>,
    /// The nested §11.1 member rows of expanded folders.
    pub members: Vec<FolderMemberVisual>,
    /// §9: whether the folders region owns chrome focus (the ring).
    pub focused: bool,
}

/// The §11.1/§11.2/§11.6 tab region data: the bounded dense window the app
/// derived from the production scroll anchor, plus the drag tick and the
/// list's keyboard-focus state. The rail renders only this window — a
/// 500-tab list never lays out 500 rows per frame.
#[derive(Debug, Clone, Default)]
pub struct TabsModel {
    /// The pinned region rows (a small, stable block above the scroll view).
    pub pinned: Vec<RowVisuals>,
    /// The tab-index slot at the end of the pinned region (the drop slot
    /// after the last pinned tab). Grouped pinned tabs render inside their
    /// folder, so this can exceed the loose rows' count.
    pub pinned_end: usize,
    /// The visible regular-region window rows (bounded, §11.6).
    pub window: Vec<RowVisuals>,
    /// The spacer height above the window (keeps the container's content
    /// height equal to the full collection).
    pub scroll_top: f32,
    /// The spacer height below the window.
    pub scroll_bottom: f32,
    /// §10.9: the drag insertion tick's target slot (`None` at rest).
    pub tick: Option<usize>,
    /// §10.9: whether the tick is legal (muted at a clamped region edge).
    pub tick_valid: bool,
    /// §9: whether the tab list owns chrome focus (the ring + brightened
    /// section labels).
    pub list_focused: bool,
    /// §11.2: an unpinned tab is being dragged, so the organized area
    /// appends the "Pin tab here" ghost row.
    pub pin_offer: bool,
    /// §10.9: the ghost row is the live target (the pointer is over it).
    pub pin_offer_targeted: bool,
}

/// The §10.9 drag proxy: the dragged row's visuals and its window rect
/// (`[x, y, width, height]`, from
/// [`crate::components::drag_offer::drag_proxy_rect`]).
#[derive(Debug, Clone)]
pub struct DragProxyModel {
    pub row: RowVisuals,
    pub rect: [f32; 4],
}

/// The §11.4 plus-menu data: the keyboard selection index into the fixed
/// two-item v1 menu ([`crate::components::menu::plus_menu_items`]).
#[derive(Debug, Clone, Copy, Default)]
pub struct PlusMenuModel {
    /// The selected item index (clamped on read by `PlusMenuState`).
    pub selected: usize,
}

/// The §11.4 footer data: the `[Downloads]` `tnum` count, the control's
/// keyboard-focus state, and the focus handle (dismissal returns focus
/// here). The permanent §11.3 space strip travels in the model's own
/// [`StripModel`] field; the rail builds the strip and places it between
/// the fixed controls.
#[derive(Debug, Clone)]
pub struct FooterModel {
    /// The §11.4 `type:meta` `tnum` count while any active attempt exists
    /// (§17.11: in-flight only).
    pub downloads_count: Option<usize>,
    /// §9: whether the `[Downloads]` control owns keyboard focus.
    pub downloads_focused: bool,
    /// The `[Downloads]` control's focus handle (the §11.12 dismissal
    /// return target).
    pub downloads_focus: FocusHandle,
}

/// The complete §11.4 rail model — the single input contract of the sidebar
/// composition. Every field is plain data (or a GPUI handle); the rail
/// renders it through the production `bbr-ui` components. The app-owned
/// child elements (command field anchor, command results, downloads panel)
/// are passed separately to [`super::render::sidebar_surface`] so the DTO
/// stays genuinely `Clone` + `Debug`.
#[derive(Debug, Clone)]
pub struct SidebarModel {
    /// The effective rail width (the requested width clamped to the window;
    /// 0 when collapsed — the app decides visibility).
    pub sidebar_width: f32,
    /// §11.3: the active space's resolved chrome tint (or the interpolated
    /// swipe tint while a swipe tracks).
    pub chrome_tint: gpui::Rgba,
    /// D4: `(source tint, moment key)` while a committed space switch
    /// cross-fades the chrome tint.
    pub tint_fade: Option<(gpui::Rgba, u64)>,
    /// v2 D6: whether the active space's paper carries the static grain.
    pub grain: bool,
    /// The active Space title shown as the rail's primary list heading.
    /// This is real controller data, not a brand label or fixture copy.
    pub active_space_title: SharedString,
    /// The §11.5 command slot (address pill or staging surface).
    pub command: CommandSlot,
    /// The §11.10 favorites region (always present in the app — the
    /// zero-favorites band teaches the gesture; `None` only in gallery
    /// compositions without the region).
    pub favorites: Option<FavoritesModel>,
    /// The §11.11 folders region (`None` while empty).
    pub folders: Option<FoldersModel>,
    /// The §11.1/§11.2/§11.6 tab region (always present; the pinned block
    /// and the bounded regular window).
    pub tabs: TabsModel,
    /// The §10.9 drag proxy while a tab drag is active (`None` at rest).
    pub drag_proxy: Option<DragProxyModel>,
    /// The §11.4 status line model.
    pub status: StatusModel,
    /// Whether the transient status line is visible. Ordinary settled page
    /// title/URL text is already represented by the address and tab row and
    /// stays hidden; loading, crash and unresponsive states remain visible.
    pub status_visible: bool,
    /// The §11.14 titlebar row (the top strip of the expanded full-height
    /// rail): the safe-area geometry, the sidebar toggle, and the
    /// Back/Forward/Reload cluster. The row disappears with the rail on
    /// collapse — no content toolbar replaces it.
    pub titlebar: TitlebarModel,
    /// The §11.4 plus menu (`None` while closed).
    pub plus_menu: Option<PlusMenuModel>,
    /// The §11.4 footer row data.
    pub footer: FooterModel,
    /// The §11.3 permanent space strip data (the rail builds the strip and
    /// places it in the footer between the fixed controls).
    pub strip: StripModel,
    /// §9: whether the resize divider owns keyboard focus (the accent
    /// hairline).
    pub divider_focused: bool,
    /// The tab list's focus handle (the §9 keyboard traversal domain).
    pub tab_list_handle: FocusHandle,
    /// The regular region's tracked scroll handle (the §11.6 dense anchor).
    pub dense_scroll: ScrollHandle,
    /// The folders region's focus handle.
    pub folders_handle: FocusHandle,
    /// The plus menu's focus handle (the menu owns keyboard focus while
    /// open).
    pub plus_menu_handle: FocusHandle,
    /// The space strip's focus handle (the §9 strip traversal domain).
    pub space_strip_handle: FocusHandle,
    /// The space strip's tracked scroll handle (the selected chip is
    /// scrolled into view through it).
    pub space_strip_scroll: ScrollHandle,
    /// The resize divider's focus handle.
    pub divider_handle: FocusHandle,
}

/// The one set of handler closures the rail wires onto the production
/// `bbr-ui` components. Every slot is a plain closure the caller fills; the
/// rail never invents behavior.
#[derive(Clone)]
pub struct Callbacks {
    /// §11.4: a pointer press on the rail body focuses the chrome domain.
    pub on_rail_mouse_down: MouseCallback,
    /// §11.3: the rail's scroll-wheel listener (the sidebar swipe entry).
    pub on_rail_scroll_wheel: WheelCallback,
    /// §11.14: the titlebar row's callbacks (the sidebar toggle + the
    /// Back/Forward/Reload cluster) — the one titlebar slot set.
    pub titlebar: TitlebarCallbacks,
    /// §11.5: the address pill's hover wash.
    pub on_command_hover: BoolCallback,
    /// §11.4: the address pill / staging surface pointer press opens the
    /// field-anchored overlay.
    pub on_command_open: MouseCallback,
    /// §17.14: the address zoom chip press resets the site zoom to 100 %
    /// (the chip stops propagation; the overlay never opens).
    pub on_address_zoom_reset: MouseCallback,
    /// §11.1: row selection through the production pointer path.
    pub on_select_row: RowCallback,
    /// §11.1: row close through the production pointer path (stops
    /// propagation).
    pub on_close_row: RowCallback,
    /// §11.2: the pin control toggles the row's pin state (stops
    /// propagation).
    pub on_toggle_pin: RowCallback,
    /// §10.9: pressing the row begins a pointer drag of that row.
    pub on_begin_drag: RowCallback,
    /// §10.3: the primary button down paints the pressed wash.
    pub on_press_row: RowCallback,
    /// §10.3: the primary button up clears the pressed wash.
    pub on_release_row: RowUpCallback,
    /// §10.2: the pointer enters/leaves a row (hover wash + close reveal).
    pub on_hover_row: HoverCallback,
    /// Visible New Tab row; routes to the existing staged NewTab authority.
    pub on_new_tab: MouseCallback,
    /// §11.1/§11.6: the tab list's keyboard traversal (`↑`/`↓`, `⌫`).
    pub on_tab_list_key_down: KeyCallback,
    /// §11.10: a favorites tile activation.
    pub on_activate_favorite: RowCallback,
    /// §11.13: a secondary-click on a tab row opens the Tab/Favorite
    /// context menu bound to that exact row's `TabId` (never selects).
    pub on_context_menu_row: RowCallback,
    /// §11.13: a secondary-click on a favorite tile opens the same menu
    /// bound to that tile's exact `TabId` (never activates).
    pub on_context_menu_favorite: RowCallback,
    /// §11.11: pointer entry into a folder row (focuses the region with
    /// that row as the keyboard anchor).
    pub on_focus_folder: FolderCallback,
    /// Blueprint D25 (§11.13): a secondary-click on a folder row opens the
    /// folder menu bound to that exact folder (never toggles it).
    pub on_context_menu_folder: FolderCallback,
    /// §11.11: the pointer enters/leaves a folder row (hover/focus-reveals
    /// the `type:meta` count).
    pub on_hover_folder: FolderHoverCallback,
    /// §11.11: the disclosure toggle (focuses the row and toggles it).
    pub on_toggle_folder: FolderCallback,
    /// §11.11: a nested member row activation.
    pub on_select_folder_member: MemberCallback,
    /// §11.11: a nested member row close (stops propagation).
    pub on_close_folder_member: MemberCallback,
    /// §11.11: the folders region's keyboard traversal (`↑`/`↓`, Enter).
    pub on_folders_key_down: KeyCallback,
    /// §11.4: the plus menu's keyboard operation (`↑`/`↓`/Enter/Escape).
    pub on_plus_menu_key_down: KeyCallback,
    /// §11.4: a plus-menu item commit at `index` (the pointer path).
    pub on_plus_menu_commit: RowCallback,
    /// §13: a pointer press inside the open plus menu marks itself so the
    /// root outside-press handler never dismisses it (stops propagation).
    pub on_plus_menu_inside_press: MouseCallback,
    /// §11.4: the footer `[Downloads]` control toggles the downloads panel.
    pub on_downloads: MouseCallback,
    /// §11.4: the footer `[+]` control toggles the plus menu.
    pub on_plus: MouseCallback,
    /// §11.3: a space chip activation through the production pointer path.
    pub on_select_space: SpaceCallback,
    /// Blueprint D05 (§11.13): a secondary-click on a space chip opens the
    /// space menu bound to that exact space (never activates it).
    pub on_context_menu_space: SpaceCallback,
    /// §11.3: the space strip's keyboard traversal (`↑`/`↓`, Enter).
    pub on_space_strip_key_down: KeyCallback,
    /// §5/§11.4: the divider's pointer begin-drag.
    pub on_divider_mouse_down: MouseCallback,
    /// §9: the focused divider's keyboard equivalents (`←`/`→`/`0`).
    pub on_divider_key_down: KeyCallback,
}

impl Callbacks {
    /// A no-op callback set (the gallery's recorder stories start from
    /// here; the app fills every slot).
    pub fn noop() -> Self {
        let noop_mouse: MouseCallback = std::rc::Rc::new(|_e, _window, _cx| {});
        let noop_key: KeyCallback = std::rc::Rc::new(|_e, _window, _cx| {});
        let noop_wheel: WheelCallback = std::rc::Rc::new(|_e, _window, _cx| {});
        let noop_bool: BoolCallback = std::rc::Rc::new(|_b, _window, _cx| {});
        let noop_row: RowCallback = std::rc::Rc::new(|_i, _e, _window, _cx| {});
        let noop_row_up: RowUpCallback = std::rc::Rc::new(|_i, _e, _window, _cx| {});
        let noop_hover: HoverCallback = std::rc::Rc::new(|_i, _b, _window, _cx| {});
        let noop_folder: FolderCallback = std::rc::Rc::new(|_f, _e, _window, _cx| {});
        let noop_folder_hover: FolderHoverCallback = std::rc::Rc::new(|_f, _b, _window, _cx| {});
        let noop_member: MemberCallback = std::rc::Rc::new(|_f, _m, _e, _window, _cx| {});
        let noop_space: SpaceCallback = std::rc::Rc::new(|_s, _e, _window, _cx| {});
        Callbacks {
            on_rail_mouse_down: noop_mouse.clone(),
            on_rail_scroll_wheel: noop_wheel,
            titlebar: TitlebarCallbacks::noop(),
            on_command_hover: noop_bool,
            on_command_open: noop_mouse.clone(),
            on_address_zoom_reset: noop_mouse.clone(),
            on_select_row: noop_row.clone(),
            on_close_row: noop_row.clone(),
            on_toggle_pin: noop_row.clone(),
            on_begin_drag: noop_row.clone(),
            on_press_row: noop_row.clone(),
            on_release_row: noop_row_up,
            on_hover_row: noop_hover,
            on_new_tab: noop_mouse.clone(),
            on_tab_list_key_down: noop_key.clone(),
            on_activate_favorite: noop_row.clone(),
            on_context_menu_row: noop_row.clone(),
            on_context_menu_favorite: noop_row.clone(),
            on_focus_folder: noop_folder.clone(),
            on_context_menu_folder: noop_folder.clone(),
            on_hover_folder: noop_folder_hover,
            on_toggle_folder: noop_folder.clone(),
            on_select_folder_member: noop_member.clone(),
            on_close_folder_member: noop_member.clone(),
            on_folders_key_down: noop_key.clone(),
            on_plus_menu_key_down: noop_key.clone(),
            on_plus_menu_commit: noop_row.clone(),
            on_plus_menu_inside_press: noop_mouse.clone(),
            on_downloads: noop_mouse.clone(),
            on_plus: noop_mouse.clone(),
            on_select_space: noop_space.clone(),
            on_context_menu_space: noop_space,
            on_space_strip_key_down: noop_key.clone(),
            on_divider_mouse_down: noop_mouse.clone(),
            on_divider_key_down: noop_key,
        }
    }
}

/// The §11.3 space-strip data the rail needs to build the strip: the chip
/// visuals in canonical order, the interpolated swipe tint, and the
/// scroll/focus handles. The app derives the visuals from the controller
/// snapshot; the rail renders the chips and the scroll viewport.
#[derive(Debug, Clone)]
pub struct StripModel {
    /// One chip per space in canonical grouped order.
    pub chips: Vec<SpaceChipVisuals>,
    /// The chrome-only interpolated presentation while a swipe tracks
    /// (`None` at rest).
    pub swipe_tint: Option<SpaceTint>,
    /// The strip's tracked scroll handle.
    pub scroll: ScrollHandle,
    /// The strip's focus handle.
    pub focus: FocusHandle,
}
