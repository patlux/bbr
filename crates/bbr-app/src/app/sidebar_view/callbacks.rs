//! The G3 Sidebar `Callbacks` construction (DESIGN.md §11.1–§11.4): the
//! app fills every slot of the one [`Callbacks`] set with the production
//! app methods. Each slot is an `Rc<dyn Fn>` that re-enters the app entity
//! on the real pointer/keyboard event (the established Library/downloads
//! wrapper pattern); the `bbr-ui` rail composition wires them onto the
//! components. No behavior is invented here — every slot routes to the
//! exact production method the pre-cutover render listeners called.

use std::rc::Rc;

use super::BbrApp;
use bbr_ui::sidebar::model::{
    BoolCallback, Callbacks, FolderCallback, FolderHoverCallback, HoverCallback, KeyCallback,
    MemberCallback, MouseCallback, RowCallback, RowUpCallback, SpaceCallback, WheelCallback,
};
use bbr_ui::titlebar::TitlebarCallbacks;
use gpui::Context;

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

        // §11.14: the titlebar row's callbacks — the sidebar toggle (the
        // `Cmd-B` pointer entry through the same sidebar-width/collapse
        // authority as the shortcut), Back/Forward/Reload through the
        // existing navigation authorities (the same dispatch as
        // Cmd-[ / Cmd-] / Cmd-R), the empty-titlebar window drag, and the
        // §10.2/§10.3 hover/pressed presentation state.
        let entity = cx.entity();
        let titlebar = TitlebarCallbacks {
            on_toggle_sidebar: Rc::new(move |_e, window, cx| {
                entity.update(cx, |view, cx| view.toggle_sidebar(window, cx));
            }),
            on_back: {
                let entity = cx.entity();
                Rc::new(move |_e, _window, cx| {
                    entity.update(cx, |view, _cx| view.go_back());
                })
            },
            on_forward: {
                let entity = cx.entity();
                Rc::new(move |_e, _window, cx| {
                    entity.update(cx, |view, _cx| view.go_forward());
                })
            },
            on_reload: {
                let entity = cx.entity();
                Rc::new(move |_e, _window, cx| {
                    entity.update(cx, |view, _cx| view.reload());
                })
            },
            on_titlebar_mouse_down: {
                let entity = cx.entity();
                Rc::new(move |event, window, cx| {
                    entity.update(cx, |view, cx| {
                        view.on_titlebar_mouse_down(event, window, cx)
                    });
                })
            },
            on_control_hover: {
                let entity = cx.entity();
                Rc::new(move |control, hovered, _window, cx| {
                    entity.update(cx, |view, cx| view.set_titlebar_hover(control, hovered, cx));
                })
            },
            on_control_press: {
                let entity = cx.entity();
                Rc::new(move |control, _window, cx| {
                    entity.update(cx, |view, cx| view.set_titlebar_pressed(control, cx));
                })
            },
            on_control_release: {
                let entity = cx.entity();
                Rc::new(move |_window, cx| {
                    entity.update(cx, |view, cx| view.clear_titlebar_pressed(cx));
                })
            },
        };

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

        // §17.14: the address zoom chip resets the active site to 100 %
        // through the production zoom authority (`Cmd-0`'s path).
        let entity = cx.entity();
        let on_address_zoom_reset: MouseCallback = Rc::new(move |_e, _window, cx| {
            entity.update(cx, |view, cx| view.zoom_reset(cx));
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

        // Reference-authoritative visible New Tab row: the exact Cmd-T
        // path — the centered palette in staged NewTab mode, no direct
        // tab/session creation before commit. The press stops propagating:
        // the rail's own mouse-down handler (`focus_chrome`) would
        // otherwise run later in the same bubble and immediately reset the
        // freshly staged NewTab mode to Inactive — the row would visibly
        // do nothing.
        let entity = cx.entity();
        let on_new_tab: MouseCallback = Rc::new(move |_event, window, cx| {
            cx.stop_propagation();
            entity.update(cx, |view, cx| view.open_palette(window, cx));
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

        // §11.13: a secondary-click on a tab row opens the Tab/Favorite
        // context menu bound to that exact row's `TabId` (never selects the
        // source).
        let entity = cx.entity();
        let on_context_menu_row: RowCallback = Rc::new(move |index, event, window, cx| {
            entity.update(cx, |view, cx| {
                view.open_tab_menu_from_row(index, event, window, cx)
            });
        });

        // §11.13: a secondary-click on a favorite tile opens the same menu
        // bound to that tile's exact `TabId` (never activates it).
        let entity = cx.entity();
        let on_context_menu_favorite: RowCallback = Rc::new(move |index, event, window, cx| {
            entity.update(cx, |view, cx| {
                view.open_tab_menu_from_favorite(index, event, window, cx)
            });
        });

        // §11.11: pointer entry into a folder row (focuses the region with
        // that row as the keyboard anchor).
        let entity = cx.entity();
        let on_focus_folder: FolderCallback = Rc::new(move |folder, _e, window, cx| {
            entity.update(cx, |view, cx| view.focus_folder_row(folder, window, cx));
        });

        // §11.11: the pointer enters/leaves a folder row — hover/focus
        // reveals the `type:meta` count (the same production hover path as
        // the row wash).
        let entity = cx.entity();
        let on_hover_folder: FolderHoverCallback = Rc::new(move |folder, hovered, _window, cx| {
            entity.update(cx, |view, cx| view.set_folder_hover(folder, hovered, cx));
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
            titlebar,
            on_command_hover,
            on_command_open,
            on_address_zoom_reset,
            on_select_row,
            on_close_row,
            on_toggle_pin,
            on_begin_drag,
            on_press_row,
            on_release_row,
            on_hover_row,
            on_new_tab,
            on_tab_list_key_down,
            on_activate_favorite,
            on_context_menu_row,
            on_context_menu_favorite,
            on_focus_folder,
            on_hover_folder,
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
