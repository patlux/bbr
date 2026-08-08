//! WebKit page-menu ownership classification (task packet
//! `bbr-context-menus-over-web`).
//!
//! The pure, deterministic decision of the ownership routing: given the item
//! titles of the menu WebKit constructed at `NSMenuWillOpenNotification`
//! time, is it the generic page menu (BBR-owned) or an element menu
//! (WebKit-owned, left untouched)? Deliberately conservative: an empty,
//! unclassifiable, or unrecognized (e.g. localized) item set never cancels
//! WebKit's menu — the native menu wins, which is the safe direction.
//!
//! Split from `menu_adapter.rs` by responsibility (pure classification vs
//! the AppKit bridge machinery) so both files stay under the ~500-line
//! budget.

use objc2_app_kit::NSMenu;

/// The page-level item titles of WebKit's generic page menu (macOS). A menu
/// whose non-empty titles are all in this set is BBR-owned; anything else is
/// WebKit-owned. `Inspect Element` is shared with element menus, but element
/// menus always carry additional element-specific items, so the subset test
/// stays deterministic.
const PAGE_MENU_TITLES: &[&str] = &["Back", "Forward", "Reload", "Inspect Element"];

/// The outcome of the ownership routing for one WebKit menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuOwnership {
    /// The generic page menu: BBR owns it and replaces it with the portal
    /// menu at the pointer.
    PageOnly,
    /// The menu carries element-specific actions: WebKit owns it and shows
    /// its native menu untouched.
    ElementOwned,
}

/// Classify a WebKit-constructed menu by its non-empty item titles.
///
/// Pure and deterministic: strips empty/separator titles, then asks whether
/// every remaining title is a page-level item. An empty item set is
/// unclassifiable and conservatively reports [`MenuOwnership::ElementOwned`]
/// (never cancel a menu we cannot classify).
pub fn classify_menu(item_titles: &[String]) -> MenuOwnership {
    let actionable: Vec<&String> = item_titles.iter().filter(|t| !t.is_empty()).collect();
    if actionable.is_empty() {
        return MenuOwnership::ElementOwned;
    }
    if actionable
        .iter()
        .all(|title| PAGE_MENU_TITLES.contains(&title.as_str()))
    {
        MenuOwnership::PageOnly
    } else {
        MenuOwnership::ElementOwned
    }
}

/// The item titles of a menu (empty/separator titles preserved; the
/// classifier filters them).
pub fn menu_titles(menu: &NSMenu) -> Vec<String> {
    let items = menu.itemArray();
    let mut titles = Vec::with_capacity(items.count());
    for index in 0..items.count() {
        let item = items.objectAtIndex(index);
        titles.push(item.title().to_string());
    }
    titles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn page_background_menu_is_bbr_owned() {
        // The observed WebKit page menu on macOS 26 (history empty): Reload
        // plus an empty separator item.
        assert_eq!(
            classify_menu(&titles(&["Reload", ""])),
            MenuOwnership::PageOnly
        );
        // With history available the page menu adds Back/Forward.
        assert_eq!(
            classify_menu(&titles(&["Back", "Forward", "Reload"])),
            MenuOwnership::PageOnly
        );
        // Inspect Element is page-level (shared with element menus).
        assert_eq!(
            classify_menu(&titles(&["Back", "Forward", "Reload", "Inspect Element"])),
            MenuOwnership::PageOnly
        );
    }

    #[test]
    fn link_menu_is_webkit_owned() {
        // The observed WebKit link menu on macOS 26.
        let menu = classify_menu(&titles(&[
            "Open Link",
            "Open Link in New Window",
            "Download Linked File",
            "Copy Link",
            "",
            "Share…",
        ]));
        assert_eq!(menu, MenuOwnership::ElementOwned);
    }

    #[test]
    fn image_and_selection_menus_are_webkit_owned() {
        assert_eq!(
            classify_menu(&titles(&["Open Image in New Window", "Save Image As…", ""])),
            MenuOwnership::ElementOwned
        );
        assert_eq!(
            classify_menu(&titles(&["Copy", "Look Up", "Search with Google"])),
            MenuOwnership::ElementOwned
        );
    }

    #[test]
    fn unclassifiable_or_unknown_menus_stay_webkit_owned() {
        // An empty menu can never be cancelled.
        assert_eq!(classify_menu(&titles(&[])), MenuOwnership::ElementOwned);
        assert_eq!(classify_menu(&titles(&[""])), MenuOwnership::ElementOwned);
        // An unrecognized (e.g. localized) item is conservative: native wins.
        assert_eq!(
            classify_menu(&titles(&["Reload", "Something New"])),
            MenuOwnership::ElementOwned
        );
    }
}
