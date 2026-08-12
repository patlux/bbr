//! Gallery-owned observation state for the G3 Sidebar composition story
//! (packet `component-gallery-compositions.md`).
//!
//! The Sidebar story renders the **exact production `bbr_ui::sidebar`
//! composition** — [`bbr_ui::sidebar::sidebar_surface`] with a plain
//! fixture-built [`bbr_ui::sidebar::SidebarModel`] and the one
//! [`bbr_ui::sidebar::Callbacks`] set — never a gallery-local wrapper,
//! reimplementation, restyle, or copy. The gallery's own contribution is
//! limited to this observation state: what the production callback slots
//! report back so the E2E can assert real pointer/keyboard interactions and
//! the canvas can reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority. Every one of the 28 `Callbacks` slots is represented; the
//! E2E asserts exact-once dispatch per family. The callback bundle (the
//! [`crate::sidebar_observer::SidebarObserver`] and its constructor that
//! owns the production propagation contract) lives in the focused
//! `sidebar_observer` module — a bounded split so this recording state
//! stays under the ~500-line module budget.

/// What a production sidebar `Callbacks` slot reports back to the gallery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarObservation {
    /// §11.4: a pointer press on the rail body (`on_rail_mouse_down`).
    RailMouseDown,
    /// §11.3: the rail's scroll-wheel listener (`on_rail_scroll_wheel`).
    RailScrollWheel,
    /// §11.14: the titlebar sidebar toggle (`titlebar.on_toggle_sidebar`;
    /// the §11.4 status-row control was removed in the cutover).
    ToggleSidebar,
    /// §11.14: the titlebar Back control (`titlebar.on_back`).
    TitlebarBack,
    /// §11.14: the titlebar Forward control (`titlebar.on_forward`).
    TitlebarForward,
    /// §11.14: the titlebar Reload control (`titlebar.on_reload`).
    TitlebarReload,
    /// §11.14: a press on the empty titlebar space
    /// (`titlebar.on_titlebar_mouse_down` — the native window-drag entry).
    TitlebarDrag,
    /// §11.14: pointer enter/leave on a titlebar control
    /// (`titlebar.on_control_hover`), payload = (control, hovered).
    TitlebarHover(String, bool),
    /// §11.14: primary-button down on a titlebar control
    /// (`titlebar.on_control_press`), payload = the control name.
    TitlebarPress(String),
    /// §11.14: primary-button up (`titlebar.on_control_release`).
    TitlebarRelease,
    /// §11.5: the address pill's hover wash (`on_command_hover`).
    CommandHover(bool),
    /// §11.4: the address pill / staging surface pointer press
    /// (`on_command_open`).
    CommandOpen,
    /// §11.1: row selection (`on_select_row`), payload = the row index.
    SelectRow(usize),
    /// §11.1: row close (`on_close_row`), payload = the row index.
    CloseRow(usize),
    /// §11.2: the pin control (`on_toggle_pin`), payload = the row index.
    TogglePin(usize),
    /// §10.9: pointer drag begin (`on_begin_drag`), payload = the row index.
    BeginDrag(usize),
    /// §10.3: primary-button down (`on_press_row`), payload = the row index.
    PressRow(usize),
    /// §10.3: primary-button up (`on_release_row`), payload = the row index.
    ReleaseRow(usize),
    /// §10.2: pointer enter/leave (`on_hover_row`), payload = (row, hovered).
    HoverRow(usize, bool),
    /// Visible New Tab row (`on_new_tab`) using the same staged authority.
    NewTab,
    /// §11.1/§11.6: the tab list's keyboard traversal (`on_tab_list_key_down`).
    TabListKeyDown(String),
    /// §11.10: a favorites tile activation (`on_activate_favorite`).
    FavoriteActivate(usize),
    /// §11.13: a secondary-click on a tab row (`on_context_menu_row`),
    /// payload = the row index — opens the Tab/Favorite context menu bound
    /// to that exact row's `TabId`.
    RowContextMenu(usize),
    /// §11.13: a secondary-click on a favorite tile
    /// (`on_context_menu_favorite`), payload = the tile index — opens the
    /// same menu bound to that tile's concrete `TabId`.
    FavoriteContextMenu(usize),
    /// §11.11: pointer entry into a folder row (`on_focus_folder`).
    FocusFolder(u64),
    /// §11.11: the disclosure toggle (`on_toggle_folder`).
    ToggleFolder(u64),
    /// §11.11: a nested member row activation (`on_select_folder_member`).
    FolderMemberSelect(u64, usize),
    /// §11.11: a nested member row close (`on_close_folder_member`).
    FolderMemberClose(u64, usize),
    /// §11.11: the folders region's keyboard traversal (`on_folders_key_down`).
    FoldersKeyDown(String),
    /// §11.4: the plus menu's keyboard operation (`on_plus_menu_key_down`).
    PlusMenuKeyDown(String),
    /// §11.4: a plus-menu item commit (`on_plus_menu_commit`), payload = the
    /// item index.
    PlusMenuCommit(usize),
    /// §13: a pointer press inside the open plus menu (`on_plus_menu_inside_press`).
    PlusMenuInsidePress,
    /// §11.4: the footer `[Downloads]` control (`on_downloads`).
    Downloads,
    /// §11.4: the footer `[+]` control (`on_plus`).
    Plus,
    /// §11.3: a space chip activation (`on_select_space`), payload = the
    /// persistent space id.
    SpaceSelect(u64),
    /// §11.3: the space strip's keyboard traversal (`on_space_strip_key_down`).
    SpaceStripKeyDown(String),
    /// §5/§11.4: the divider's pointer begin-drag (`on_divider_mouse_down`).
    DividerMouseDown,
    /// §9: the focused divider's keyboard equivalents (`on_divider_key_down`).
    DividerKeyDown(String),
}

/// The shared Sidebar recorder: plain fixture observation state, written
/// only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SidebarRecorder {
    /// `on_rail_mouse_down` dispatches.
    pub rail_mouse_downs: u32,
    /// `on_rail_scroll_wheel` dispatches.
    pub rail_scroll_wheels: u32,
    /// `titlebar.on_toggle_sidebar` dispatches.
    pub toggle_sidebars: u32,
    /// `titlebar.on_back` dispatches.
    pub titlebar_backs: u32,
    /// `titlebar.on_forward` dispatches.
    pub titlebar_forwards: u32,
    /// `titlebar.on_reload` dispatches.
    pub titlebar_reloads: u32,
    /// `titlebar.on_titlebar_mouse_down` dispatches.
    pub titlebar_drags: u32,
    /// `titlebar.on_control_hover` (control, hovered) pairs, in dispatch
    /// order.
    pub titlebar_hovers: Vec<(String, bool)>,
    /// `titlebar.on_control_press` control names, in dispatch order.
    pub titlebar_presses: Vec<String>,
    /// `titlebar.on_control_release` dispatches.
    pub titlebar_releases: u32,
    /// `on_command_hover` values, in dispatch order.
    pub command_hovers: Vec<bool>,
    /// `on_command_open` dispatches.
    pub command_opens: u32,
    /// `on_select_row` row indexes, in dispatch order.
    pub select_rows: Vec<usize>,
    /// `on_close_row` row indexes, in dispatch order.
    pub close_rows: Vec<usize>,
    /// `on_toggle_pin` row indexes, in dispatch order.
    pub toggle_pins: Vec<usize>,
    /// `on_begin_drag` row indexes, in dispatch order.
    pub drag_begins: Vec<usize>,
    /// `on_press_row` row indexes, in dispatch order.
    pub press_rows: Vec<usize>,
    /// `on_release_row` row indexes, in dispatch order.
    pub release_rows: Vec<usize>,
    /// `on_hover_row` (row, hovered) pairs, in dispatch order.
    pub hover_rows: Vec<(usize, bool)>,
    /// `on_new_tab` dispatches.
    pub new_tabs: u32,
    /// `on_tab_list_key_down` keystroke keys, in dispatch order.
    pub tab_list_keys: Vec<String>,
    /// `on_activate_favorite` tile indexes, in dispatch order.
    pub favorite_activations: Vec<usize>,
    /// `on_context_menu_row` row indexes, in dispatch order (§11.13).
    pub context_menu_rows: Vec<usize>,
    /// `on_context_menu_favorite` tile indexes, in dispatch order (§11.13).
    pub context_menu_favorites: Vec<usize>,
    /// `on_focus_folder` folder ids, in dispatch order.
    pub focus_folders: Vec<u64>,
    /// `on_toggle_folder` folder ids, in dispatch order.
    pub toggle_folders: Vec<u64>,
    /// `on_select_folder_member` (folder, member) pairs, in dispatch order.
    pub folder_member_selects: Vec<(u64, usize)>,
    /// `on_close_folder_member` (folder, member) pairs, in dispatch order.
    pub folder_member_closes: Vec<(u64, usize)>,
    /// `on_folders_key_down` keystroke keys, in dispatch order.
    pub folders_keys: Vec<String>,
    /// `on_plus_menu_key_down` keystroke keys, in dispatch order.
    pub plus_menu_keys: Vec<String>,
    /// `on_plus_menu_commit` item indexes, in dispatch order.
    pub plus_menu_commits: Vec<usize>,
    /// `on_plus_menu_inside_press` dispatches.
    pub plus_menu_inside_presses: u32,
    /// `on_downloads` dispatches.
    pub downloads: u32,
    /// `on_plus` dispatches.
    pub plus: u32,
    /// `on_select_space` space ids, in dispatch order.
    pub space_selects: Vec<u64>,
    /// `on_space_strip_key_down` keystroke keys, in dispatch order.
    pub space_strip_keys: Vec<String>,
    /// `on_divider_mouse_down` dispatches.
    pub divider_mouse_downs: u32,
    /// `on_divider_key_down` keystroke keys, in dispatch order.
    pub divider_keys: Vec<String>,
}

impl SidebarRecorder {
    /// Apply one observed sidebar interaction.
    pub fn observe(&mut self, event: SidebarObservation) {
        match event {
            SidebarObservation::RailMouseDown => self.rail_mouse_downs += 1,
            SidebarObservation::RailScrollWheel => self.rail_scroll_wheels += 1,
            SidebarObservation::ToggleSidebar => self.toggle_sidebars += 1,
            SidebarObservation::TitlebarBack => self.titlebar_backs += 1,
            SidebarObservation::TitlebarForward => self.titlebar_forwards += 1,
            SidebarObservation::TitlebarReload => self.titlebar_reloads += 1,
            SidebarObservation::TitlebarDrag => self.titlebar_drags += 1,
            SidebarObservation::TitlebarHover(control, hovered) => {
                self.titlebar_hovers.push((control, hovered))
            }
            SidebarObservation::TitlebarPress(control) => self.titlebar_presses.push(control),
            SidebarObservation::TitlebarRelease => self.titlebar_releases += 1,
            SidebarObservation::CommandHover(value) => self.command_hovers.push(value),
            SidebarObservation::CommandOpen => self.command_opens += 1,
            SidebarObservation::SelectRow(index) => self.select_rows.push(index),
            SidebarObservation::CloseRow(index) => self.close_rows.push(index),
            SidebarObservation::TogglePin(index) => self.toggle_pins.push(index),
            SidebarObservation::BeginDrag(index) => self.drag_begins.push(index),
            SidebarObservation::PressRow(index) => self.press_rows.push(index),
            SidebarObservation::ReleaseRow(index) => self.release_rows.push(index),
            SidebarObservation::HoverRow(index, hovered) => self.hover_rows.push((index, hovered)),
            SidebarObservation::NewTab => self.new_tabs += 1,
            SidebarObservation::TabListKeyDown(key) => self.tab_list_keys.push(key),
            SidebarObservation::FavoriteActivate(index) => self.favorite_activations.push(index),
            SidebarObservation::RowContextMenu(index) => self.context_menu_rows.push(index),
            SidebarObservation::FavoriteContextMenu(index) => {
                self.context_menu_favorites.push(index)
            }
            SidebarObservation::FocusFolder(folder) => self.focus_folders.push(folder),
            SidebarObservation::ToggleFolder(folder) => self.toggle_folders.push(folder),
            SidebarObservation::FolderMemberSelect(folder, member) => {
                self.folder_member_selects.push((folder, member))
            }
            SidebarObservation::FolderMemberClose(folder, member) => {
                self.folder_member_closes.push((folder, member))
            }
            SidebarObservation::FoldersKeyDown(key) => self.folders_keys.push(key),
            SidebarObservation::PlusMenuKeyDown(key) => self.plus_menu_keys.push(key),
            SidebarObservation::PlusMenuCommit(index) => self.plus_menu_commits.push(index),
            SidebarObservation::PlusMenuInsidePress => self.plus_menu_inside_presses += 1,
            SidebarObservation::Downloads => self.downloads += 1,
            SidebarObservation::Plus => self.plus += 1,
            SidebarObservation::SpaceSelect(space) => self.space_selects.push(space),
            SidebarObservation::SpaceStripKeyDown(key) => self.space_strip_keys.push(key),
            SidebarObservation::DividerMouseDown => self.divider_mouse_downs += 1,
            SidebarObservation::DividerKeyDown(key) => self.divider_keys.push(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_records_the_expected_field_only() {
        let mut recorder = SidebarRecorder::default();
        recorder.observe(SidebarObservation::RailMouseDown);
        recorder.observe(SidebarObservation::RailScrollWheel);
        recorder.observe(SidebarObservation::ToggleSidebar);
        recorder.observe(SidebarObservation::CommandHover(true));
        recorder.observe(SidebarObservation::CommandOpen);
        recorder.observe(SidebarObservation::SelectRow(3));
        recorder.observe(SidebarObservation::CloseRow(3));
        recorder.observe(SidebarObservation::TogglePin(3));
        recorder.observe(SidebarObservation::BeginDrag(3));
        recorder.observe(SidebarObservation::PressRow(3));
        recorder.observe(SidebarObservation::ReleaseRow(3));
        recorder.observe(SidebarObservation::HoverRow(3, true));
        recorder.observe(SidebarObservation::TabListKeyDown("down".into()));
        recorder.observe(SidebarObservation::FavoriteActivate(0));
        recorder.observe(SidebarObservation::FocusFolder(7));
        recorder.observe(SidebarObservation::ToggleFolder(7));
        recorder.observe(SidebarObservation::FolderMemberSelect(7, 0));
        recorder.observe(SidebarObservation::FolderMemberClose(7, 1));
        recorder.observe(SidebarObservation::FoldersKeyDown("enter".into()));
        recorder.observe(SidebarObservation::PlusMenuKeyDown("down".into()));
        recorder.observe(SidebarObservation::PlusMenuCommit(1));
        recorder.observe(SidebarObservation::PlusMenuInsidePress);
        recorder.observe(SidebarObservation::Downloads);
        recorder.observe(SidebarObservation::Plus);
        recorder.observe(SidebarObservation::SpaceSelect(9));
        recorder.observe(SidebarObservation::SpaceStripKeyDown("right".into()));
        recorder.observe(SidebarObservation::DividerMouseDown);
        recorder.observe(SidebarObservation::DividerKeyDown("right".into()));
        assert_eq!(recorder.rail_mouse_downs, 1);
        assert_eq!(recorder.rail_scroll_wheels, 1);
        assert_eq!(recorder.toggle_sidebars, 1);
        assert_eq!(recorder.command_hovers, vec![true]);
        assert_eq!(recorder.command_opens, 1);
        assert_eq!(recorder.select_rows, vec![3]);
        assert_eq!(recorder.close_rows, vec![3]);
        assert_eq!(recorder.toggle_pins, vec![3]);
        assert_eq!(recorder.drag_begins, vec![3]);
        assert_eq!(recorder.press_rows, vec![3]);
        assert_eq!(recorder.release_rows, vec![3]);
        assert_eq!(recorder.hover_rows, vec![(3, true)]);
        assert_eq!(recorder.tab_list_keys, vec!["down"]);
        assert_eq!(recorder.favorite_activations, vec![0]);
        assert_eq!(recorder.focus_folders, vec![7]);
        assert_eq!(recorder.toggle_folders, vec![7]);
        assert_eq!(recorder.folder_member_selects, vec![(7, 0)]);
        assert_eq!(recorder.folder_member_closes, vec![(7, 1)]);
        assert_eq!(recorder.folders_keys, vec!["enter"]);
        assert_eq!(recorder.plus_menu_keys, vec!["down"]);
        assert_eq!(recorder.plus_menu_commits, vec![1]);
        assert_eq!(recorder.plus_menu_inside_presses, 1);
        assert_eq!(recorder.downloads, 1);
        assert_eq!(recorder.plus, 1);
        assert_eq!(recorder.space_selects, vec![9]);
        assert_eq!(recorder.space_strip_keys, vec!["right"]);
        assert_eq!(recorder.divider_mouse_downs, 1);
        assert_eq!(recorder.divider_keys, vec!["right"]);
        // Observing one slot never disturbs the others.
        recorder.observe(SidebarObservation::Downloads);
        assert_eq!(recorder.downloads, 2);
        assert_eq!(recorder.plus, 1);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = SidebarRecorder::default();
        assert_eq!(recorder.rail_mouse_downs, 0);
        assert_eq!(recorder.rail_scroll_wheels, 0);
        assert_eq!(recorder.toggle_sidebars, 0);
        assert!(recorder.command_hovers.is_empty());
        assert_eq!(recorder.command_opens, 0);
        assert!(recorder.select_rows.is_empty());
        assert!(recorder.close_rows.is_empty());
        assert!(recorder.toggle_pins.is_empty());
        assert!(recorder.drag_begins.is_empty());
        assert!(recorder.press_rows.is_empty());
        assert!(recorder.release_rows.is_empty());
        assert!(recorder.hover_rows.is_empty());
        assert!(recorder.tab_list_keys.is_empty());
        assert!(recorder.favorite_activations.is_empty());
        assert!(recorder.focus_folders.is_empty());
        assert!(recorder.toggle_folders.is_empty());
        assert!(recorder.folder_member_selects.is_empty());
        assert!(recorder.folder_member_closes.is_empty());
        assert!(recorder.folders_keys.is_empty());
        assert!(recorder.plus_menu_keys.is_empty());
        assert!(recorder.plus_menu_commits.is_empty());
        assert_eq!(recorder.plus_menu_inside_presses, 0);
        assert_eq!(recorder.downloads, 0);
        assert_eq!(recorder.plus, 0);
        assert!(recorder.space_selects.is_empty());
        assert!(recorder.space_strip_keys.is_empty());
        assert_eq!(recorder.divider_mouse_downs, 0);
        assert!(recorder.divider_keys.is_empty());
    }
}
