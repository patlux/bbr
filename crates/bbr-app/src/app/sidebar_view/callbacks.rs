//! The G3 Sidebar `Callbacks` construction (DESIGN.md §11.1–§11.4): the
//! app fills every slot of the one [`Callbacks`] set with the production
//! app methods. Each slot is an `Rc<dyn Fn>` that re-enters the app entity
//! on the real pointer/keyboard event (the established Library/downloads
//! wrapper pattern); the `bbr-ui` rail composition wires them onto the
//! components. No behavior is invented here — every slot routes to the
//! exact production method the pre-cutover render listeners called.

use std::rc::Rc;

use bbr_ui::sidebar::model::{
    BoolCallback, Callbacks, FolderCallback, HoverCallback, KeyCallback, MemberCallback,
    MouseCallback, RowCallback, RowUpCallback, SpaceCallback, WheelCallback,
};
use gpui::Context;

use super::BbrApp;

impl BbrApp {
    /// Fill the one sidebar [`Callbacks`] set with the production app
    /// methods. Every slot re-enters the app entity on the real event; the
    /// rail never invents behavior.
    pub(super) fn sidebar_callbacks(&mut self, cx: &mut Context<Self>) -> Callbacks {
        let entity = cx.entity();

        // §11.4: a pointer press on the rail body focuses the chrome domain.
        let on_rail_mouse_down: MouseCallback = Rc::new(move |_e, window, cx| {
            entity.update(cx, |view, cx| view.focus_chrome(window, cx));
        });

        // §11.3: the rail's scroll-wheel listener (the sidebar swipe entry).
        let entity = cx.entity();
        let on_rail_scroll_wheel: WheelCallback = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| {
                view.on_sidebar_scroll_wheel(event, window, cx)
            });
        });

        // §11.4: the collapse control (`Cmd-B` pointer entry).
        let entity = cx.entity();
        let on_toggle_sidebar: MouseCallback = Rc::new(move |_e, window, cx| {
            entity.update(cx, |view, cx| view.toggle_sidebar(window, cx));
        });

        // §11.5: the address pill's hover wash.
        let entity = cx.entity();
        let on_command_hover: BoolCallback = Rc::new(move |hovered, _window, cx| {
            entity.update(cx, |view, cx| view.set_address_hover(*hovered, cx));
        });

        // §11.4: the address pill / staging surface pointer press opens the
        // field-anchored overlay (and stops propagation so the rail's
        // chrome-focus handler never fights it).
        let entity = cx.entity();
        let on_command_open: MouseCallback = Rc::new(move |_e, window, cx| {
            entity.update(cx, |view, cx| view.open_overlay(window, cx));
            cx.stop_propagation();
        });

        // §11.1: row selection through the production pointer path.
        let entity = cx.entity();
        let on_select_row: RowCallback = Rc::new(move |index, _e, window, cx| {
            entity.update(cx, |view, cx| {
                view.select_row_via_pointer(index, window, cx)
            });
        });

        // §11.1: row close through the production pointer path (stops
        // propagation so the press never selects the row or starts a drag).
        let entity = cx.entity();
        let on_close_row: RowCallback = Rc::new(move |index, _e, window, cx| {
            cx.stop_propagation();
            entity.update(cx, |view, cx| view.close_row_via_pointer(index, window, cx));
        });

        // §11.2: the pin control toggles the row's pin state (stops
        // propagation).
        let entity = cx.entity();
        let on_toggle_pin: RowCallback = Rc::new(move |index, _e, window, cx| {
            cx.stop_propagation();
            entity.update(cx, |view, cx| view.toggle_row_pin(index, window, cx));
        });

        // §10.9: pressing the row begins a pointer drag of that row.
        let entity = cx.entity();
        let on_begin_drag: RowCallback = Rc::new(move |index, event, window, cx| {
            entity.update(cx, |view, cx| view.begin_tab_drag(event, index, window, cx));
        });

        // §10.3: the primary button down paints the pressed wash.
        let entity = cx.entity();
        let on_press_row: RowCallback = Rc::new(move |index, _e, _window, cx| {
            entity.update(cx, |view, cx| view.set_row_pressed(index, cx));
        });

        // §10.3: the primary button up clears the pressed wash.
        let entity = cx.entity();
        let on_release_row: RowUpCallback = Rc::new(move |_index, _e, _window, cx| {
            entity.update(cx, |view, cx| view.clear_row_pressed(cx));
        });

        // §10.2: the pointer enters/leaves a row (hover wash + close reveal).
        let entity = cx.entity();
        let on_hover_row: HoverCallback = Rc::new(move |index, hovered, _window, cx| {
            entity.update(cx, |view, cx| view.set_row_hover(index, hovered, cx));
        });

        // §11.1/§11.6: the tab list's keyboard traversal (`↑`/`↓`, `⌫`).
        let entity = cx.entity();
        let on_tab_list_key_down: KeyCallback = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| view.on_tab_list_key_down(event, window, cx));
        });

        // §11.10: a favorites tile activation.
        let entity = cx.entity();
        let on_activate_favorite: RowCallback = Rc::new(move |index, _e, window, cx| {
            entity.update(cx, |view, cx| view.activate_favorite(index, window, cx));
        });

        // §11.11: pointer entry into a folder row (focuses the region with
        // that row as the keyboard anchor).
        let entity = cx.entity();
        let on_focus_folder: FolderCallback = Rc::new(move |folder, _e, window, cx| {
            entity.update(cx, |view, cx| view.focus_folder_row(folder, window, cx));
        });

        // §11.11: the disclosure toggle (focuses the row and toggles it).
        let entity = cx.entity();
        let on_toggle_folder: FolderCallback = Rc::new(move |folder, _e, window, cx| {
            entity.update(cx, |view, cx| {
                view.focus_folder_row(folder, window, cx);
                view.toggle_folder(folder, cx);
            });
        });

        // §11.11: a nested member row activation.
        let entity = cx.entity();
        let on_select_folder_member: MemberCallback =
            Rc::new(move |folder, member, _e, window, cx| {
                entity.update(cx, |view, cx| {
                    view.select_folder_member(folder, member, window, cx)
                });
            });

        // §11.11: a nested member row close (stops propagation).
        let entity = cx.entity();
        let on_close_folder_member: MemberCallback =
            Rc::new(move |folder, member, _e, window, cx| {
                cx.stop_propagation();
                entity.update(cx, |view, cx| {
                    view.close_folder_member(folder, member, window, cx)
                });
            });

        // §11.11: the folders region's keyboard traversal (`↑`/`↓`, Enter).
        let entity = cx.entity();
        let on_folders_key_down: KeyCallback = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| view.on_folders_key_down(event, window, cx));
        });

        // §11.4: the plus menu's keyboard operation (`↑`/`↓`/Enter/Escape).
        let entity = cx.entity();
        let on_plus_menu_key_down: KeyCallback = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| view.on_plus_menu_key_down(event, window, cx));
        });

        // §11.4: a plus-menu item commit at `index` (the pointer path; stops
        // propagation so the rail's chrome-focus handler never fights it).
        let entity = cx.entity();
        let on_plus_menu_commit: RowCallback = Rc::new(move |index, _e, window, cx| {
            cx.stop_propagation();
            entity.update(cx, |view, cx| view.commit_plus_menu_item(index, window, cx));
        });

        // §13: a pointer press inside the open plus menu marks itself so
        // the root outside-press handler never dismisses it (stops
        // propagation).
        let entity = cx.entity();
        let on_plus_menu_inside_press: MouseCallback = Rc::new(move |_e, _window, cx| {
            entity.update(cx, |view, _cx| view.u2.plus_menu_press_inside = true);
            cx.stop_propagation();
        });

        // §11.4: the footer `[Downloads]` control toggles the downloads
        // panel (marks the inside press and stops propagation so the root
        // outside-press handler never dismisses the freshly opened panel).
        let entity = cx.entity();
        let on_downloads: MouseCallback = Rc::new(move |_e, window, cx| {
            entity.update(cx, |view, cx| {
                view.mark_downloads_press_inside();
                crate::app::downloads_behavior::toggle_downloads(view, window, cx);
            });
            cx.stop_propagation();
        });

        // §11.4: the footer `[+]` control toggles the plus menu (marks the
        // inside press and stops propagation).
        let entity = cx.entity();
        let on_plus: MouseCallback = Rc::new(move |_e, window, cx| {
            entity.update(cx, |view, cx| {
                view.u2.plus_menu_press_inside = true;
                view.toggle_plus_menu(window, cx);
            });
            cx.stop_propagation();
        });

        // §11.3: a space chip activation through the production pointer path.
        let entity = cx.entity();
        let on_select_space: SpaceCallback = Rc::new(move |space, _e, window, cx| {
            entity.update(cx, |view, cx| {
                view.select_space_via_pointer(space, window, cx)
            });
        });

        // §11.3: the space strip's keyboard traversal (`↑`/`↓`, Enter).
        let entity = cx.entity();
        let on_space_strip_key_down: KeyCallback = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| {
                view.on_space_strip_key_down(event, window, cx)
            });
        });

        // §5/§11.4: the divider's pointer begin-drag (stops propagation so
        // the rail's chrome-focus handler never overwrites the divider
        // focus).
        let entity = cx.entity();
        let on_divider_mouse_down: MouseCallback = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| view.begin_sidebar_drag(event, window, cx));
            cx.stop_propagation();
        });

        // §9: the focused divider's keyboard equivalents (`←`/`→`/`0`).
        let entity = cx.entity();
        let on_divider_key_down: KeyCallback = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| {
                view.on_sidebar_divider_key_down(event, window, cx)
            });
        });

        Callbacks {
            on_rail_mouse_down,
            on_rail_scroll_wheel,
            on_toggle_sidebar,
            on_command_hover,
            on_command_open,
            on_select_row,
            on_close_row,
            on_toggle_pin,
            on_begin_drag,
            on_press_row,
            on_release_row,
            on_hover_row,
            on_tab_list_key_down,
            on_activate_favorite,
            on_focus_folder,
            on_toggle_folder,
            on_select_folder_member,
            on_close_folder_member,
            on_folders_key_down,
            on_plus_menu_key_down,
            on_plus_menu_commit,
            on_plus_menu_inside_press,
            on_downloads,
            on_plus,
            on_select_space,
            on_space_strip_key_down,
            on_divider_mouse_down,
            on_divider_key_down,
        }
    }
}
