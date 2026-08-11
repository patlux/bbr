//! Gallery-owned observation state for the G2 Group 4 sidebar-component
//! stories (packet `component-gallery-browser-components.md`).
//!
//! The Group 4 stories render the **exact production `bbr-ui` symbols**
//! (`favorites_grid`/`favorite_tile`, `folder_row`/`folder_item_row`,
//! `footer_surface`, `profile_chip`, `plus_menu_items`/`PlusMenuState`)
//! with plain fixture models and recorder callbacks — never a
//! gallery-local wrapper, reimplementation, restyle, or copy. The
//! gallery's own contribution is limited to this observation state: what
//! the production callbacks report back so the E2E can assert real pointer
//! interactions and the canvas can reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority.

use std::rc::Rc;

use gpui::{App, Window};

/// What a production Group 4 sidebar-component callback reports back to
/// the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group4Observation {
    /// The §11.10 favorites `on_activate` fired for the tile at `index`
    /// (the deterministic `bbr-fav-<n>` tile).
    FavoriteActivate(usize),
    /// The §11.11 folder-row `on_toggle` fired for the row at `index`.
    FolderToggle(usize),
    /// The §11.11 nested-row `on_select` fired for `(folder row, member)`.
    FolderMemberSelect(usize, usize),
    /// The §11.11 nested-row `on_close` fired for `(folder row, member)`.
    FolderMemberClose(usize, usize),
    /// The §11.4 footer `[Downloads]` `on_downloads` fired.
    Downloads,
    /// The §11.4 footer `[+]` `on_plus` fired.
    Plus,
    /// The §11.4 plus-menu item commit fired for the item id.
    MenuItem(&'static str),
}

/// The shared Group 4 recorder: plain fixture observation state, written
/// only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Group4Recorder {
    /// The tile indices the production `on_activate` reported, in dispatch
    /// order.
    pub favorite_activations: Vec<usize>,
    /// The row indices the production folder `on_toggle` reported.
    pub folder_toggles: Vec<usize>,
    /// The `(folder row, member)` pairs the production nested `on_select`
    /// reported.
    pub folder_selects: Vec<(usize, usize)>,
    /// The `(folder row, member)` pairs the production nested `on_close`
    /// reported.
    pub folder_closes: Vec<(usize, usize)>,
    /// The number of `[Downloads]` presses that fired `on_downloads`.
    pub downloads: u32,
    /// The number of `[+]` presses that fired `on_plus`.
    pub plus: u32,
    /// The item ids the production plus-menu commit reported, in dispatch
    /// order.
    pub menu_items: Vec<&'static str>,
}

impl Group4Recorder {
    /// Apply one observed Group 4 interaction.
    pub fn observe(&mut self, event: Group4Observation) {
        match event {
            Group4Observation::FavoriteActivate(index) => self.favorite_activations.push(index),
            Group4Observation::FolderToggle(index) => self.folder_toggles.push(index),
            Group4Observation::FolderMemberSelect(row, member) => {
                self.folder_selects.push((row, member))
            }
            Group4Observation::FolderMemberClose(row, member) => {
                self.folder_closes.push((row, member))
            }
            Group4Observation::Downloads => self.downloads += 1,
            Group4Observation::Plus => self.plus += 1,
            Group4Observation::MenuItem(id) => self.menu_items.push(id),
        }
    }
}

/// A production `MouseDownEvent` callback carrying a payload the story
/// closure supplies (the tile/row/member/item the press targeted),
/// re-entering the app through the production record path.
pub type Group4PayloadFn<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

/// The callback bundle the Group 4 stories route the production
/// `bbr-ui` sidebar-component callbacks through. Each callback records its
/// interaction. Only the folder member `on_close` stops propagation,
/// exactly like the app's production listener (a close press on a nested
/// row never also selects the member, §11.11 — the app's
/// `close_folder_member` listener stops propagation; the member row's
/// `on_select` listener never runs after it).
pub struct Group4Observer {
    /// §11.10 tile activation (`favorite_tile` `on_activate`), payload =
    /// the tile index.
    pub on_favorite_activate: Group4PayloadFn<usize>,
    /// §11.11 folder-row disclosure (`folder_row` `on_toggle`), payload =
    /// the row index.
    pub on_folder_toggle: Group4PayloadFn<usize>,
    /// §11.11 nested-row activation (`folder_item_row` `on_select`),
    /// payload = `(folder row, member)`.
    pub on_folder_select: Group4PayloadFn<(usize, usize)>,
    /// §11.11 nested-row close (`folder_item_row` `on_close`), payload =
    /// `(folder row, member)`; stops propagation like the app.
    pub on_folder_close: Group4PayloadFn<(usize, usize)>,
    /// §11.4 footer `[Downloads]` (`footer_surface` `on_downloads`).
    pub on_downloads: Group4PayloadFn<()>,
    /// §11.4 footer `[+]` (`footer_surface` `on_plus`).
    pub on_plus: Group4PayloadFn<()>,
    /// §11.4 plus-menu item commit, payload = the production item id.
    pub on_menu_commit: Group4PayloadFn<&'static str>,
}

/// Build the production Group 4 observer callbacks from one record action.
/// `record` re-enters the app's production record path (the app-owned
/// entity upgrade); this constructor owns the production **propagation
/// contract**: only the folder member close stops propagation (a close
/// press never also selects the member, §11.11 — the exact wiring the
/// app's `folders_surface` uses).
pub fn build_group4_observer(
    record: impl Fn(Group4Observation, &mut App) + 'static,
) -> Group4Observer {
    let record = Rc::new(record);
    let make_payload = |event: Group4Observation, stop: bool| {
        let record = record.clone();
        Rc::new(move |_payload: (), _window: &mut Window, cx: &mut App| {
            if stop {
                cx.stop_propagation();
            }
            record(event, cx);
        })
    };
    Group4Observer {
        on_favorite_activate: {
            let record = record.clone();
            Rc::new(move |index: usize, _window: &mut Window, cx: &mut App| {
                record(Group4Observation::FavoriteActivate(index), cx);
            })
        },
        on_folder_toggle: {
            let record = record.clone();
            Rc::new(move |index: usize, _window: &mut Window, cx: &mut App| {
                record(Group4Observation::FolderToggle(index), cx);
            })
        },
        on_folder_select: {
            let record = record.clone();
            Rc::new(
                move |target: (usize, usize), _window: &mut Window, cx: &mut App| {
                    record(
                        Group4Observation::FolderMemberSelect(target.0, target.1),
                        cx,
                    );
                },
            )
        },
        on_folder_close: {
            let record = record.clone();
            Rc::new(
                move |target: (usize, usize), _window: &mut Window, cx: &mut App| {
                    // §11.11: a close press never also selects the member
                    // (the app's close listener stops propagation).
                    cx.stop_propagation();
                    record(Group4Observation::FolderMemberClose(target.0, target.1), cx);
                },
            )
        },
        on_downloads: make_payload(Group4Observation::Downloads, false),
        on_plus: make_payload(Group4Observation::Plus, false),
        on_menu_commit: {
            let record = record.clone();
            Rc::new(
                move |id: &'static str, _window: &mut Window, cx: &mut App| {
                    record(Group4Observation::MenuItem(id), cx);
                },
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_records_the_expected_field_only() {
        let mut recorder = Group4Recorder::default();
        recorder.observe(Group4Observation::FavoriteActivate(0));
        recorder.observe(Group4Observation::FavoriteActivate(10));
        recorder.observe(Group4Observation::FolderToggle(10));
        recorder.observe(Group4Observation::FolderMemberSelect(10, 0));
        recorder.observe(Group4Observation::FolderMemberClose(10, 1));
        recorder.observe(Group4Observation::Downloads);
        recorder.observe(Group4Observation::Downloads);
        recorder.observe(Group4Observation::Plus);
        recorder.observe(Group4Observation::MenuItem("bbr-plus-new-space"));
        recorder.observe(Group4Observation::MenuItem("bbr-plus-new-folder"));
        assert_eq!(recorder.favorite_activations, vec![0, 10]);
        assert_eq!(recorder.folder_toggles, vec![10]);
        assert_eq!(recorder.folder_selects, vec![(10, 0)]);
        assert_eq!(recorder.folder_closes, vec![(10, 1)]);
        assert_eq!(recorder.downloads, 2);
        assert_eq!(recorder.plus, 1);
        assert_eq!(
            recorder.menu_items,
            vec!["bbr-plus-new-space", "bbr-plus-new-folder"]
        );
        // Observing one surface never disturbs the others.
        recorder.observe(Group4Observation::Downloads);
        assert_eq!(recorder.favorite_activations, vec![0, 10]);
        assert_eq!(recorder.downloads, 3);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = Group4Recorder::default();
        assert!(recorder.favorite_activations.is_empty());
        assert!(recorder.folder_toggles.is_empty());
        assert!(recorder.folder_selects.is_empty());
        assert!(recorder.folder_closes.is_empty());
        assert_eq!(recorder.downloads, 0);
        assert_eq!(recorder.plus, 0);
        assert!(recorder.menu_items.is_empty());
    }
}
