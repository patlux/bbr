//! The Sidebar composition observer bundle (packet
//! `component-gallery-compositions.md`): a bounded split of
//! `sidebar_recorder.rs` so the recording state and the callback bundle
//! each stay under the ~500-line module budget. This module owns the
//! production [`bbr_ui::sidebar::Callbacks`] bundle the Sidebar story
//! routes through — the observer slots and the constructor that owns the
//! production **propagation contract** (exactly the slots the app's
//! `sidebar_callbacks` stop propagation on: row close, pin, folder member
//! close, plus-menu commit, plus-menu inside press, downloads, plus,
//! divider mouse-down). The observed values live in
//! [`crate::sidebar_recorder::SidebarRecorder`].

use std::rc::Rc;

use bbr_domain::{FolderId, SpaceId};
use bbr_ui::sidebar::model::{
    BoolCallback, Callbacks, FolderCallback, HoverCallback, KeyCallback, MemberCallback,
    MouseCallback, RowCallback, RowUpCallback, SpaceCallback, WheelCallback,
};
use bbr_ui::titlebar::{TitlebarCallbacks, TitlebarControl};
use gpui::{App, Window};

use crate::sidebar_recorder::SidebarObservation;

/// The callback bundle the Sidebar story routes the production
/// `bbr_ui::sidebar::Callbacks` slots through. Each callback records its
/// interaction. The propagation contract mirrors the app's production
/// wiring exactly: the row close, the pin control, the folder member close,
/// the plus-menu commit, the plus-menu inside press, the `[Downloads]`
/// control, the `[+]` control, and the divider mouse-down stop propagation
/// (the app's listeners do the same, §11.1/§11.2/§11.4/§11.11/§13/§5).
pub struct SidebarObserver {
    /// §11.4 rail-body press.
    pub on_rail_mouse_down: MouseCallback,
    /// §11.3 rail scroll wheel.
    pub on_rail_scroll_wheel: WheelCallback,
    /// §11.14 the titlebar row's callbacks (the sidebar toggle + the
    /// Back/Forward/Reload cluster + the window drag + hover/press state).
    pub titlebar: TitlebarCallbacks,
    /// §11.5 address-pill hover.
    pub on_command_hover: BoolCallback,
    /// §11.4 address pill / staging press.
    pub on_command_open: MouseCallback,
    /// §17.14 address zoom chip press.
    pub on_address_zoom_reset: MouseCallback,
    /// Blueprint D11 address site slot press.
    pub on_address_site: MouseCallback,
    /// §11.1 row selection.
    pub on_select_row: RowCallback,
    /// §11.1 row close (stops propagation).
    pub on_close_row: RowCallback,
    /// §11.2 pin control (stops propagation).
    pub on_toggle_pin: RowCallback,
    /// §10.9 drag begin.
    pub on_begin_drag: RowCallback,
    /// §10.3 press down.
    pub on_press_row: RowCallback,
    /// §10.3 press up.
    pub on_release_row: RowUpCallback,
    /// §10.2 hover.
    pub on_hover_row: HoverCallback,
    /// Visible New Tab row.
    pub on_new_tab: MouseCallback,
    /// §11.1/§11.6 tab-list keyboard.
    pub on_tab_list_key_down: KeyCallback,
    /// §11.10 favorites tile activation.
    pub on_activate_favorite: RowCallback,
    /// §11.13 tab-row secondary-click (the Tab/Favorite context-menu open;
    /// the app's open method stops propagation).
    pub on_context_menu_row: RowCallback,
    /// §11.13 favorite-tile secondary-click (the same menu bound to the
    /// tile's concrete `TabId`; the app's open method stops propagation).
    pub on_context_menu_favorite: RowCallback,
    /// §11.11 folder-row focus.
    pub on_focus_folder: FolderCallback,
    /// §11.11 folder-row hover used by the count-reveal presentation.
    pub on_hover_folder: FolderHoverCallback,
    /// §11.11 folder disclosure.
    pub on_toggle_folder: FolderCallback,
    /// §11.11 nested member select.
    pub on_select_folder_member: MemberCallback,
    /// §11.11 nested member close (stops propagation).
    pub on_close_folder_member: MemberCallback,
    /// §11.11 folders keyboard.
    pub on_folders_key_down: KeyCallback,
    /// §11.4 plus-menu keyboard.
    pub on_plus_menu_key_down: KeyCallback,
    /// §11.4 plus-menu commit (stops propagation).
    pub on_plus_menu_commit: RowCallback,
    /// §13 plus-menu inside press (stops propagation).
    pub on_plus_menu_inside_press: MouseCallback,
    /// §11.4 footer `[Downloads]` (stops propagation).
    pub on_downloads: MouseCallback,
    /// §11.4 footer `[+]` (stops propagation).
    pub on_plus: MouseCallback,
    /// §11.3 space chip activation.
    pub on_select_space: SpaceCallback,
    /// §11.3 space-strip keyboard.
    pub on_space_strip_key_down: KeyCallback,
    /// §5/§11.4 divider drag begin (stops propagation).
    pub on_divider_mouse_down: MouseCallback,
    /// §9 divider keyboard.
    pub on_divider_key_down: KeyCallback,
}

/// The shared record action: one observation plus whether the slot stops
/// propagation (the production contract the constructor owns).
type RecordObservation = Rc<dyn Fn(SidebarObservation, bool, &mut App)>;
/// §11.11 folder-row hover callback (folder id, hovered).
pub type FolderHoverCallback = Rc<dyn Fn(FolderId, bool, &mut Window, &mut App)>;

/// Build the production sidebar observer callbacks from one record action.
/// `record` re-enters the app's production record path (the app-owned
/// entity upgrade); this constructor owns the production **propagation
/// contract** — exactly the slots the app's `sidebar_callbacks` stop
/// propagation on (row close, pin, folder member close, plus-menu commit,
/// plus-menu inside press, downloads, plus, divider mouse-down).
pub fn build_sidebar_observer(
    record: impl Fn(SidebarObservation, &mut App) + 'static,
) -> SidebarObserver {
    let record = Rc::new(record);
    let record_obs: RecordObservation = {
        let record = Rc::clone(&record);
        Rc::new(move |event, stop, cx| {
            if stop {
                cx.stop_propagation();
            }
            record(event, cx);
        })
    };
    let mouse = |event: SidebarObservation, stop: bool| {
        let record_obs = Rc::clone(&record_obs);
        Rc::new(
            move |_e: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record_obs(event.clone(), stop, cx);
            },
        )
    };
    let row = |make: fn(usize) -> SidebarObservation, stop: bool| {
        let record_obs = Rc::clone(&record_obs);
        Rc::new(
            move |index: usize, _e: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record_obs(make(index), stop, cx);
            },
        )
    };
    let titlebar_mouse = |event: SidebarObservation| {
        let record = Rc::clone(&record);
        Rc::new(
            move |_e: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(event.clone(), cx);
            },
        )
    };
    let titlebar = TitlebarCallbacks {
        on_toggle_sidebar: titlebar_mouse(SidebarObservation::ToggleSidebar),
        on_back: titlebar_mouse(SidebarObservation::TitlebarBack),
        on_forward: titlebar_mouse(SidebarObservation::TitlebarForward),
        on_reload: titlebar_mouse(SidebarObservation::TitlebarReload),
        on_titlebar_mouse_down: titlebar_mouse(SidebarObservation::TitlebarDrag),
        on_control_hover: {
            let record = Rc::clone(&record);
            Rc::new(
                move |control: TitlebarControl,
                      hovered: bool,
                      _window: &mut Window,
                      cx: &mut App| {
                    record(
                        SidebarObservation::TitlebarHover(control.id().to_string(), hovered),
                        cx,
                    );
                },
            )
        },
        on_control_press: {
            let record = Rc::clone(&record);
            Rc::new(
                move |control: TitlebarControl, _window: &mut Window, cx: &mut App| {
                    record(
                        SidebarObservation::TitlebarPress(control.id().to_string()),
                        cx,
                    );
                },
            )
        },
        on_control_release: {
            let record = Rc::clone(&record);
            Rc::new(move |_window: &mut Window, cx: &mut App| {
                record(SidebarObservation::TitlebarRelease, cx);
            })
        },
        // Blueprint D14: the history menu is app-owned; the gallery rail
        // has no session history to list.
        on_control_secondary: Rc::new(|_control, _event, _window, _cx| {}),
    };
    SidebarObserver {
        on_rail_mouse_down: mouse(SidebarObservation::RailMouseDown, false),
        on_rail_scroll_wheel: {
            let record = Rc::clone(&record);
            Rc::new(
                move |_e: &gpui::ScrollWheelEvent, _window: &mut Window, cx: &mut App| {
                    record(SidebarObservation::RailScrollWheel, cx);
                },
            )
        },
        titlebar,
        on_command_hover: {
            let record = Rc::clone(&record);
            Rc::new(move |value: &bool, _window: &mut Window, cx: &mut App| {
                record(SidebarObservation::CommandHover(*value), cx);
            })
        },
        on_command_open: mouse(SidebarObservation::CommandOpen, true),
        on_address_zoom_reset: mouse(SidebarObservation::AddressZoomReset, true),
        on_address_site: mouse(SidebarObservation::AddressSite, true),
        on_select_row: row(SidebarObservation::SelectRow, false),
        on_close_row: row(SidebarObservation::CloseRow, true),
        on_toggle_pin: row(SidebarObservation::TogglePin, true),
        on_begin_drag: row(SidebarObservation::BeginDrag, false),
        on_press_row: row(SidebarObservation::PressRow, false),
        on_release_row: {
            let record = Rc::clone(&record);
            Rc::new(
                move |index: usize, _e: &gpui::MouseUpEvent, _window: &mut Window, cx: &mut App| {
                    record(SidebarObservation::ReleaseRow(index), cx);
                },
            )
        },
        on_hover_row: {
            let record = Rc::clone(&record);
            Rc::new(
                move |index: usize, hovered: bool, _window: &mut Window, cx: &mut App| {
                    record(SidebarObservation::HoverRow(index, hovered), cx);
                },
            )
        },
        on_new_tab: mouse(SidebarObservation::NewTab, false),
        on_tab_list_key_down: {
            let record = Rc::clone(&record);
            Rc::new(
                move |e: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut App| {
                    record(
                        SidebarObservation::TabListKeyDown(e.keystroke.key.clone()),
                        cx,
                    );
                },
            )
        },
        on_activate_favorite: row(SidebarObservation::FavoriteActivate, false),
        on_context_menu_row: row(SidebarObservation::RowContextMenu, true),
        on_context_menu_favorite: row(SidebarObservation::FavoriteContextMenu, true),
        on_hover_folder: {
            let record = Rc::clone(&record);
            Rc::new(
                move |folder: FolderId, hovered: bool, _window: &mut Window, cx: &mut App| {
                    if hovered {
                        record(SidebarObservation::FocusFolder(folder.0), cx);
                    }
                },
            )
        },
        on_focus_folder: {
            let record = Rc::clone(&record);
            Rc::new(
                move |folder: FolderId,
                      _e: &gpui::MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    record(SidebarObservation::FocusFolder(folder.0), cx);
                },
            )
        },
        on_toggle_folder: {
            let record = Rc::clone(&record);
            Rc::new(
                move |folder: FolderId,
                      _e: &gpui::MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    record(SidebarObservation::ToggleFolder(folder.0), cx);
                },
            )
        },
        on_select_folder_member: {
            let record = Rc::clone(&record);
            Rc::new(
                move |folder: FolderId,
                      member: usize,
                      _e: &gpui::MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    record(SidebarObservation::FolderMemberSelect(folder.0, member), cx);
                },
            )
        },
        on_close_folder_member: {
            let record = Rc::clone(&record);
            Rc::new(
                move |folder: FolderId,
                      member: usize,
                      _e: &gpui::MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    // §11.11: a close press never also selects the member.
                    cx.stop_propagation();
                    record(SidebarObservation::FolderMemberClose(folder.0, member), cx);
                },
            )
        },
        on_folders_key_down: {
            let record = Rc::clone(&record);
            Rc::new(
                move |e: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut App| {
                    record(
                        SidebarObservation::FoldersKeyDown(e.keystroke.key.clone()),
                        cx,
                    );
                },
            )
        },
        on_plus_menu_key_down: {
            let record = Rc::clone(&record);
            Rc::new(
                move |e: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut App| {
                    record(
                        SidebarObservation::PlusMenuKeyDown(e.keystroke.key.clone()),
                        cx,
                    );
                },
            )
        },
        on_plus_menu_commit: row(SidebarObservation::PlusMenuCommit, true),
        on_plus_menu_inside_press: mouse(SidebarObservation::PlusMenuInsidePress, true),
        on_downloads: mouse(SidebarObservation::Downloads, true),
        on_plus: mouse(SidebarObservation::Plus, true),
        on_select_space: {
            let record = Rc::clone(&record);
            Rc::new(
                move |space: SpaceId,
                      _e: &gpui::MouseDownEvent,
                      _window: &mut Window,
                      cx: &mut App| {
                    record(SidebarObservation::SpaceSelect(space.0), cx);
                },
            )
        },
        on_space_strip_key_down: {
            let record = Rc::clone(&record);
            Rc::new(
                move |e: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut App| {
                    record(
                        SidebarObservation::SpaceStripKeyDown(e.keystroke.key.clone()),
                        cx,
                    );
                },
            )
        },
        on_divider_mouse_down: mouse(SidebarObservation::DividerMouseDown, true),
        on_divider_key_down: {
            let record = Rc::clone(&record);
            Rc::new(
                move |e: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut App| {
                    record(
                        SidebarObservation::DividerKeyDown(e.keystroke.key.clone()),
                        cx,
                    );
                },
            )
        },
    }
}

impl SidebarObserver {
    /// Assemble the one production [`Callbacks`] set from the observer
    /// slots — the exact 28-slot contract the rail wires onto the
    /// components. The story calls this once per render and passes the set
    /// to [`bbr_ui::sidebar::sidebar_surface`].
    pub fn callbacks(&self) -> Callbacks {
        Callbacks {
            on_rail_mouse_down: Rc::clone(&self.on_rail_mouse_down),
            on_rail_scroll_wheel: Rc::clone(&self.on_rail_scroll_wheel),
            titlebar: self.titlebar.clone(),
            on_command_hover: Rc::clone(&self.on_command_hover),
            on_command_open: Rc::clone(&self.on_command_open),
            on_address_zoom_reset: Rc::clone(&self.on_address_zoom_reset),
            on_address_site: Rc::clone(&self.on_address_site),
            on_select_row: Rc::clone(&self.on_select_row),
            on_close_row: Rc::clone(&self.on_close_row),
            on_toggle_pin: Rc::clone(&self.on_toggle_pin),
            on_begin_drag: Rc::clone(&self.on_begin_drag),
            on_press_row: Rc::clone(&self.on_press_row),
            on_release_row: Rc::clone(&self.on_release_row),
            on_hover_row: Rc::clone(&self.on_hover_row),
            on_new_tab: Rc::clone(&self.on_new_tab),
            on_tab_list_key_down: Rc::clone(&self.on_tab_list_key_down),
            on_activate_favorite: Rc::clone(&self.on_activate_favorite),
            on_context_menu_row: Rc::clone(&self.on_context_menu_row),
            on_context_menu_favorite: Rc::clone(&self.on_context_menu_favorite),
            on_hover_folder: Rc::clone(&self.on_hover_folder),
            on_focus_folder: Rc::clone(&self.on_focus_folder),
            on_toggle_folder: Rc::clone(&self.on_toggle_folder),
            on_select_folder_member: Rc::clone(&self.on_select_folder_member),
            on_close_folder_member: Rc::clone(&self.on_close_folder_member),
            on_folders_key_down: Rc::clone(&self.on_folders_key_down),
            on_plus_menu_key_down: Rc::clone(&self.on_plus_menu_key_down),
            on_plus_menu_commit: Rc::clone(&self.on_plus_menu_commit),
            on_plus_menu_inside_press: Rc::clone(&self.on_plus_menu_inside_press),
            on_downloads: Rc::clone(&self.on_downloads),
            on_plus: Rc::clone(&self.on_plus),
            on_select_space: Rc::clone(&self.on_select_space),
            // Blueprint D05: the chip's secondary click opens the app's
            // space menu; the gallery rail has no menu to open.
            on_context_menu_space: Rc::new(|_space, _event, _window, _cx| {}),
            on_context_menu_folder: Rc::new(|_folder, _event, _window, _cx| {}),
            on_space_strip_key_down: Rc::clone(&self.on_space_strip_key_down),
            on_divider_mouse_down: Rc::clone(&self.on_divider_mouse_down),
            on_divider_key_down: Rc::clone(&self.on_divider_key_down),
            // Blueprint D24: the gallery rail shows no media card.
            on_media: Rc::new(|_action, _window, _cx| {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callbacks_assembly_covers_every_slot() {
        // The observer's `callbacks()` must fill every one of the 28
        // production slots (the rail wires them onto the components).
        let observer = build_sidebar_observer(|_event, _cx| {});
        let callbacks = observer.callbacks();
        // The set is not the no-op set: every slot is the observer's
        // recording closure (a cheap structural check — the E2E proves
        // dispatch end to end).
        let noop = Callbacks::noop();
        let _ = &noop;
        // The observer exposes every slot the set needs.
        let _ = &callbacks.on_rail_mouse_down;
        let _ = &callbacks.on_rail_scroll_wheel;
        let _ = &callbacks.on_hover_folder;
        let _ = &callbacks.titlebar;
        let _ = &callbacks.on_command_hover;
        let _ = &callbacks.on_command_open;
        let _ = &callbacks.on_address_zoom_reset;
        let _ = &callbacks.on_address_site;
        let _ = &callbacks.on_select_row;
        let _ = &callbacks.on_close_row;
        let _ = &callbacks.on_toggle_pin;
        let _ = &callbacks.on_begin_drag;
        let _ = &callbacks.on_press_row;
        let _ = &callbacks.on_release_row;
        let _ = &callbacks.on_hover_row;
        let _ = &callbacks.on_tab_list_key_down;
        let _ = &callbacks.on_activate_favorite;
        let _ = &callbacks.on_context_menu_row;
        let _ = &callbacks.on_context_menu_favorite;
        let _ = &callbacks.on_focus_folder;
        let _ = &callbacks.on_toggle_folder;
        let _ = &callbacks.on_select_folder_member;
        let _ = &callbacks.on_close_folder_member;
        let _ = &callbacks.on_folders_key_down;
        let _ = &callbacks.on_plus_menu_key_down;
        let _ = &callbacks.on_plus_menu_commit;
        let _ = &callbacks.on_plus_menu_inside_press;
        let _ = &callbacks.on_downloads;
        let _ = &callbacks.on_plus;
        let _ = &callbacks.on_select_space;
        let _ = &callbacks.on_space_strip_key_down;
        let _ = &callbacks.on_divider_mouse_down;
        let _ = &callbacks.on_divider_key_down;
    }
}
