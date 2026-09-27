//! App-owned snapshot → bbr-ui DTO adapters (the G1a cutover's app-side
//! derivations).
//!
//! The packet's model/callback contract keeps controller-read-model
//! derivation in `bbr-app`: the browser snapshot types ([`bbr_browser`])
//! are app/testkit authority, so the functions that turn them into `bbr-ui`
//! presentation DTOs live here and nowhere else. `bbr-ui` owns the DTO
//! types and the pure rendering; this module owns "what the controller
//! snapshot renders". The adapters were extracted verbatim from the
//! pre-cutover `components::favorites` / `components::folders` modules
//! (the derivation semantics are unchanged — only the home moved).

use bbr_browser::{ControllerSnapshot, FavoriteSnapshot, FolderSnapshot};
use bbr_domain::FolderId;

use bbr_ui::components::favicon::{monogram_hue, monogram_letter};
use bbr_ui::components::favorites::FavoriteTileSpec;
use bbr_ui::components::folders::{FolderMemberVisual, FolderPresentation, FolderRowSpec};
use bbr_ui::text::{display_title, host_of};

use super::favicon_presentation::FaviconImageMirror;

/// Derive the rendered tile specs from the session-local favorites in
/// deterministic insertion order (the same source the grid renders). Each
/// tile's favicon image resolves from its exact owning profile's bounded
/// host-keyed store through the decoded mirror — the same record the tab
/// row derives from, one acquisition, every presentation.
pub fn favorite_tiles(
    favorites: &[FavoriteSnapshot],
    mirror: &FaviconImageMirror,
) -> Vec<FavoriteTileSpec> {
    favorites
        .iter()
        .map(|favorite| {
            let host = host_of(&favorite.url);
            FavoriteTileSpec {
                index: favorite.index,
                tab: favorite.tab.0,
                space: favorite.space.0,
                profile: favorite.profile.0,
                title: favorite.title.clone(),
                url: favorite.url.clone(),
                letter: monogram_letter(&favorite.url),
                hue: monogram_hue(&favorite.url),
                image: mirror.get(&(favorite.profile, host)).cloned(),
            }
        })
        .collect()
}

/// Derive the rendered folder rows in creation order (the same source the
/// folders list renders). Member counts come from the L5 snapshot; the
/// expanded/collapsed value from the app-local presentation state, keyed by
/// the stable domain folder id. `revealed` is the folder whose `type:meta`
/// count the hover/focus state reveals (`None` at rest — every count stays
/// quiet).
pub fn folder_row_specs(
    folders: &[FolderSnapshot],
    presentation: &FolderPresentation,
    revealed: Option<FolderId>,
) -> Vec<FolderRowSpec> {
    folders
        .iter()
        .map(|folder| FolderRowSpec {
            index: folder.index,
            id: folder.id,
            title: folder.title.clone(),
            count: folder.members.len(),
            expanded: presentation.is_expanded(folder.id),
            reveal_count: revealed == Some(folder.id),
        })
        .collect()
}

/// Resolve the expanded folders' member rows from the L5 snapshot: each
/// member `TabId` maps through the controller's global read model to its
/// display title and monogram letter, in folder order, for expanded folders
/// only (collapsed folders render no nested rows). Deterministic: the
/// member's owning folder is resolved by its stable domain id; the folder
/// creation-order position and the member's folder position are the ids.
pub fn folder_member_visuals(
    rows: &[FolderRowSpec],
    folders: &[FolderSnapshot],
    browser: &ControllerSnapshot,
    mirror: &FaviconImageMirror,
) -> Vec<FolderMemberVisual> {
    let tabs = &browser.all_tabs;
    let mut visuals = Vec::new();
    for spec in rows {
        if !spec.expanded {
            continue;
        }
        let folder = folders
            .iter()
            .find(|folder| folder.id == spec.id)
            .expect("folder row derives from the snapshot");
        for (member, tab_id) in folder.members.iter().enumerate() {
            let tab = tabs
                .iter()
                .find(|tab| tab.id == *tab_id)
                .expect("reconciled folder member exists in the controller");
            let host = host_of(&tab.url);
            visuals.push(FolderMemberVisual {
                folder: folder.id,
                folder_index: spec.index,
                member,
                title: display_title(&tab.url, &tab.title),
                letter: monogram_letter(&tab.url),
                hue: monogram_hue(&tab.url),
                // The member's exact owning profile's bounded store record
                // (the same record its tab row derives from).
                image: mirror.get(&(tab.profile, host)).cloned(),
                // §10.4: the member is the window's selected tab (the active
                // space's active index) — the nested row then lifts.
                selected: tab.space == browser.active_space
                    && Some(tab.index) == browser.active_index,
            });
        }
    }
    visuals
}
