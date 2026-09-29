//! Blueprint D19 (board L-13, DESIGN §11.13): the macOS menu bar and the
//! Dock menu. BBR supplies the structure and titles; macOS draws them
//! (Native OS). Every item dispatches an existing GPUI action — the same
//! action its keyboard shortcut dispatches — so a menu command and its
//! shortcut can never diverge, and GPUI shows the bound shortcut beside the
//! title. Titles match the ⌘T `>` action names (`bbr_command` registry).
//! The Spaces menu and the Dock menu list the spaces and are rebuilt when
//! the space list, a title or the active space changes.
//!
//! No Edit menu: text editing in pages stays with WebKit's own key
//! handling; claiming ⌘C/⌘V in the menu would take them from the page.

use gpui::{App, Menu, MenuItem};

use crate::actions::*;

/// Activate the space with this persistent id (Spaces menu, Dock menu).
#[derive(Clone, gpui::Action, PartialEq, Eq)]
#[action(namespace = bbr, no_json)]
pub struct ActivateSpace {
    pub id: u64,
}

gpui::actions!(
    bbr,
    [
        /// `Cmd-Q`: flush the browser state and quit (nothing is asked;
        /// everything restores).
        Quit,
        /// `Cmd-H`: hide BBR.
        HideApp,
        /// `Cmd-M`: minimize the browser window.
        MinimizeWindow,
        /// Blueprint D18: open the About BBR window.
        ShowAbout,
        /// Blueprint D16: `Cmd-,` opens the Settings window.
        ShowSettings,
    ]
);

/// One space as the menus list it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSpace {
    pub id: u64,
    pub title: String,
    pub active: bool,
}

/// A portable description of one menu item (what the testkit reports and
/// what [`build`] turns into GPUI menu items).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub enum ItemSpec {
    Separator,
    Action {
        name: String,
        /// The dispatched action's registered name (e.g. `bbr::CloseTab`).
        action: String,
        checked: bool,
    },
}

/// One menu of the menu bar.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub struct MenuSpec {
    pub name: String,
    pub items: Vec<ItemSpec>,
}

/// An item with its action (the builder's working form).
enum Entry {
    Separator,
    Action {
        name: String,
        action: Box<dyn gpui::Action>,
        checked: bool,
    },
}

fn item(name: &str, action: impl gpui::Action) -> Entry {
    Entry::Action {
        name: name.to_string(),
        action: Box::new(action),
        checked: false,
    }
}

fn space_items(spaces: &[MenuSpace]) -> Vec<Entry> {
    spaces
        .iter()
        .map(|space| Entry::Action {
            name: space.title.clone(),
            action: Box::new(ActivateSpace { id: space.id }),
            checked: space.active,
        })
        .collect()
}

/// The menu bar in order: BBR, File, View, History, Spaces, Tab, Window.
fn bar(spaces: &[MenuSpace]) -> Vec<(&'static str, Vec<Entry>)> {
    let mut spaces_menu = vec![item("New Space…", NewSpace)];
    if !spaces.is_empty() {
        spaces_menu.push(Entry::Separator);
        spaces_menu.extend(space_items(spaces));
    }
    vec![
        (
            "BBR",
            vec![
                item("About BBR", ShowAbout),
                item("Settings…", ShowSettings),
                Entry::Separator,
                item("Hide BBR", HideApp),
                Entry::Separator,
                item("Quit BBR", Quit),
            ],
        ),
        (
            "File",
            vec![
                item("New Tab", OpenPalette),
                item("Open Location…", OpenOverlay),
                Entry::Separator,
                item("Close Tab", CloseTab),
                item("Reopen Closed Tab", ReopenTab),
                Entry::Separator,
                item("Find in Page…", FindInPage),
            ],
        ),
        (
            "View",
            vec![
                item("Toggle Sidebar", ToggleSidebar),
                item("Focus Sidebar", FocusRail),
                Entry::Separator,
                item("Zoom In", ZoomIn),
                item("Zoom Out", ZoomOut),
                item("Actual Size", ZoomReset),
            ],
        ),
        (
            "History",
            vec![
                item("Back", Back),
                item("Forward", Forward),
                item("Reload", Reload),
                Entry::Separator,
                item("Save Page to Library", ToggleBookmark),
                item("Saved Pages Library", OpenSavedPagesLibrary),
                Entry::Separator,
                item("Show All History", ShowHistory),
                item("Clear Browsing Data…", ClearBrowsingData),
            ],
        ),
        ("Spaces", spaces_menu),
        (
            "Tab",
            vec![
                item("New Tab", OpenPalette),
                item("Close Tab", CloseTab),
                item("Reopen Closed Tab", ReopenTab),
                Entry::Separator,
                item("Pin or Unpin Tab", PinTab),
                item("Move Tab to Space…", MoveTab),
                Entry::Separator,
                item("Reorder Tab Up", ReorderTabUp),
                item("Reorder Tab Down", ReorderTabDown),
            ],
        ),
        ("Window", vec![item("Minimize", MinimizeWindow)]),
    ]
}

/// The Dock menu: the spaces, then New Tab.
fn dock(spaces: &[MenuSpace]) -> Vec<Entry> {
    let mut entries = space_items(spaces);
    if !entries.is_empty() {
        entries.push(Entry::Separator);
    }
    entries.push(item("New Tab", OpenPalette));
    entries
}

#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
fn spec_items(entries: &[Entry]) -> Vec<ItemSpec> {
    entries
        .iter()
        .map(|entry| match entry {
            Entry::Separator => ItemSpec::Separator,
            Entry::Action {
                name,
                action,
                checked,
            } => ItemSpec::Action {
                name: name.clone(),
                action: action.name().to_string(),
                checked: *checked,
            },
        })
        .collect()
}

fn gpui_items(entries: Vec<Entry>) -> Vec<MenuItem> {
    entries
        .into_iter()
        .map(|entry| match entry {
            Entry::Separator => MenuItem::separator(),
            Entry::Action {
                name,
                action,
                checked,
            } => MenuItem::Action {
                name: name.into(),
                action,
                os_action: None,
                checked,
                disabled: false,
            },
        })
        .collect()
}

/// The portable menu-bar description for `spaces`.
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub fn menu_bar_spec(spaces: &[MenuSpace]) -> Vec<MenuSpec> {
    bar(spaces)
        .iter()
        .map(|(name, entries)| MenuSpec {
            name: (*name).to_string(),
            items: spec_items(entries),
        })
        .collect()
}

/// The portable Dock-menu description for `spaces`.
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub fn dock_menu_spec(spaces: &[MenuSpace]) -> Vec<ItemSpec> {
    spec_items(&dock(spaces))
}

/// The action behind the menu-bar item `menu › name` (the testkit invokes
/// it through the same dispatch an `NSMenuItem` click takes).
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub fn menu_action(spaces: &[MenuSpace], menu: &str, name: &str) -> Option<Box<dyn gpui::Action>> {
    let entries = if menu == "Dock" {
        dock(spaces)
    } else {
        bar(spaces).into_iter().find(|(title, _)| *title == menu)?.1
    };
    entries.into_iter().find_map(|entry| match entry {
        Entry::Action {
            name: title,
            action,
            ..
        } if title == name => Some(action),
        _ => None,
    })
}

/// Install the menu bar and the Dock menu for `spaces` (idempotent; call
/// again when the spaces change).
pub fn install(cx: &mut App, spaces: &[MenuSpace]) {
    let menus: Vec<Menu> = bar(spaces)
        .into_iter()
        .map(|(name, entries)| Menu {
            name: name.into(),
            items: gpui_items(entries),
            disabled: false,
        })
        .collect();
    cx.set_menus(menus);
    cx.set_dock_menu(gpui_items(dock(spaces)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spaces() -> Vec<MenuSpace> {
        vec![
            MenuSpace {
                id: 2,
                title: "Default".into(),
                active: true,
            },
            MenuSpace {
                id: 4,
                title: "Work".into(),
                active: false,
            },
        ]
    }

    #[test]
    fn the_bar_follows_the_board_and_lists_the_spaces() {
        let bar = menu_bar_spec(&spaces());
        let names: Vec<&str> = bar.iter().map(|menu| menu.name.as_str()).collect();
        assert_eq!(
            names,
            ["BBR", "File", "View", "History", "Spaces", "Tab", "Window"]
        );
        let spaces_menu = &bar[4].items;
        assert_eq!(
            spaces_menu[2],
            ItemSpec::Action {
                name: "Default".into(),
                action: "bbr::ActivateSpace".into(),
                checked: true,
            }
        );
        // Titles match the ⌘T action names.
        let tab = &bar[5].items;
        assert!(tab.contains(&ItemSpec::Action {
            name: "Reopen Closed Tab".into(),
            action: "bbr::ReopenTab".into(),
            checked: false,
        }));
        // No Edit menu: page editing keys stay with WebKit.
        assert!(!names.contains(&"Edit"));
    }

    #[test]
    fn the_dock_lists_the_spaces_then_new_tab() {
        let dock = dock_menu_spec(&spaces());
        assert_eq!(dock.len(), 4);
        assert_eq!(
            dock[3],
            ItemSpec::Action {
                name: "New Tab".into(),
                action: "bbr::OpenPalette".into(),
                checked: false,
            }
        );
        assert!(menu_action(&spaces(), "Dock", "Work").is_some());
        assert!(menu_action(&spaces(), "Tab", "Close Tab").is_some());
        assert!(menu_action(&spaces(), "Tab", "Nope").is_none());
    }
}
