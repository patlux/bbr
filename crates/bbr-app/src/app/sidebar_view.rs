//! The GPUI sidebar assembler (DESIGN.md §11.1–§11.4).
//!
//! `bbr-app` is the G3 Sidebar composition's assembler: it builds the plain
//! [`SidebarModel`] from the controller snapshot and the app presentation
//! state, fills the one [`Callbacks`] set with the production app methods,
//! constructs the three app-owned child elements (the §11.4 command field
//! anchor, the §11.5 in-rail command results, the §11.12 downloads panel),
//! and delegates the rail composition to
//! [`bbr_ui::sidebar::sidebar_surface`]. All browser/session/profile/space/
//! ranking/persistence authority stays in `bbr-app`; the rail renders only.

use gpui::{prelude::*, Context, Div, SharedString, Stateful, Window};

use bbr_ui::components::RowVisuals;
use bbr_ui::sidebar::model::SidebarModel;
use bbr_ui::theme::Theme;

use super::BbrApp;

mod callbacks;
mod command_bar;
mod models;

impl BbrApp {
    /// The sidebar rail at the given effective width: the §11 column (brand,
    /// command slot, favorites grid, pinned rows, folders, regular tabs,
    /// status, toggle, plus menu, downloads panel, footer row) plus the §5
    /// 4 pt resize divider on the right edge. The assembler builds the
    /// plain [`SidebarModel`] and the one [`Callbacks`] set, constructs the
    /// three app-owned child elements (command field anchor, command
    /// results, downloads panel), and delegates the composition to
    /// [`bbr_ui::sidebar::sidebar_surface`].
    pub(crate) fn sidebar_surface(
        &mut self,
        theme: &Theme,
        sidebar_width: f32,
        chrome_focused: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let model = self.sidebar_model(theme, sidebar_width, chrome_focused, window);
        let callbacks = self.sidebar_callbacks(cx);
        let field_anchor = self.command_field_anchor_element();
        let favorites_band_anchor = self.favorites_band_anchor_element();
        let command_results = self
            .command_result_surface(theme)
            .map(|surface| surface.into_any());
        let downloads_panel = self
            .downloads_panel_surface(theme, window, cx)
            .map(|surface| surface.into_any());
        let space_config_panel = self
            .space_config_surface(theme, window, cx)
            .map(|surface| surface.into_any());
        bbr_ui::sidebar::sidebar_surface(
            theme,
            &model,
            &callbacks,
            bbr_ui::sidebar::SidebarChildren {
                command_field_anchor: field_anchor,
                favorites_band_anchor,
                command_results,
                downloads_panel,
                space_config_panel,
            },
        )
    }

    /// The §11.10 favorites-band anchor element: the invisible rect
    /// recorder overlaying the favorites region (the app-owned child the
    /// rail places absolutely over the region). It publishes the region's
    /// window-relative bounds into `favorites_band_rect` every frame, so
    /// the tab-drag resolver targets the real rendered drop band and the
    /// E2E snapshot exposes its typed geometry.
    fn favorites_band_anchor_element(&self) -> gpui::AnyElement {
        crate::app::favorites_band::favorites_band_anchor(self.favorites_band_rect.clone())
            .into_any()
    }

    /// Build the plain §11.4 rail model from the controller snapshot and the
    /// app presentation state — the single input contract of the sidebar
    /// composition. The dense window, row visuals, favorites/folders/
    /// spaces DTOs, status, command slot, and footer all derive here; the
    /// rail renders the model and never holds browser authority.
    fn sidebar_model(
        &mut self,
        theme: &Theme,
        sidebar_width: f32,
        chrome_focused: bool,
        window: &Window,
    ) -> SidebarModel {
        // §11.3: the active space's effective appearance tints the BBR-owned
        // sidebar material through the one central resolver (the base is
        // the shell's own `surface_sidebar` token), and a tracking swipe
        // interpolates that chrome toward the adjacent space's presentation
        // (chrome-only — text, hairlines, accent rails, icons, and the
        // native web content keep their own tokens).
        let chrome_tint = self.active_chrome_tint();
        // The U2 rail inputs (favorites tiles, folder rows/members) derived
        // once per frame and shared by the tab/favorites/folders surfaces.
        let u2 = self.u2_rail_data();
        let command = self.command_slot(theme, chrome_focused);
        let favorites = self.favorites_model(&u2.favorite_tiles);
        let folders = self.folders_model(&u2.folder_rows, &u2.folder_members, window);
        let tabs = self.tabs_model(theme, window);
        let browser = self.controller.snapshot();
        let status = crate::shell::status_model(&browser);
        let status_visible = crate::shell::status_visible(&browser);
        let active_space_title = browser
            .spaces
            .iter()
            .find(|space| space.id == browser.active_space)
            .map(|space| space.title.clone())
            .unwrap_or_else(|| "Default".to_string());
        let plus_menu = self.plus_menu_model();
        let footer = self.footer_model(window);
        let strip = self.strip_model();
        let divider_focused = self.divider_handle.is_focused(window);
        SidebarModel {
            sidebar_width,
            chrome_tint,
            active_space_title: SharedString::from(active_space_title),
            command,
            favorites,
            folders,
            tabs,
            status,
            status_visible,
            // §11.14: the titlebar row is the rail's top strip — the
            // safe-area geometry (native), the sidebar toggle, and the
            // Back/Forward/Reload cluster. The toggle uses the identical
            // sidebar-width/collapse authority as Cmd-B and the row
            // disappears with the rail on collapse.
            titlebar: self.titlebar_model(window),
            plus_menu,
            footer,
            strip,
            divider_focused,
            tab_list_handle: self.tab_list_handle.clone(),
            dense_scroll: self.dense_scroll.clone(),
            folders_handle: self.u2.folders_handle.clone(),
            plus_menu_handle: self.u2.plus_menu_handle.clone(),
            space_strip_handle: self.space_strip_handle.clone(),
            space_strip_scroll: self.space_strip_scroll.clone(),
            divider_handle: self.divider_handle.clone(),
        }
    }

    /// Map a tab slice to per-row visuals for the current theme. The §4
    /// one-line title truncation is applied here so the snapshot's row
    /// titles and the rendered rows share one source. `drag_source` marks
    /// the row currently being dragged (§10.9 `dragging` state); the row's
    /// `pinned` region identity drives the pin control. The pointer-state
    /// fields (hover/press) and the §10.2 close-reveal derive from the same
    /// app state the production listeners mutate, so the snapshot's
    /// `tab_states` always match the render.
    pub fn row_visuals(
        &self,
        tabs: &[bbr_browser::TabSnapshot],
        drag_source: Option<usize>,
    ) -> Vec<RowVisuals> {
        let budget = crate::shell::title_budget_chars(&self.theme());
        // The active-space tab rows belong to the active profile (the
        // controller's tab operations target it), so every row's favicon
        // image resolves from that profile's bounded host-keyed store.
        let profile = self.controller.snapshot().active_profile;
        tabs.iter()
            .map(|tab| {
                let host = bbr_ui::text::host_of(&tab.url);
                RowVisuals {
                    index: tab.index,
                    title: SharedString::from(bbr_ui::text::truncate_title(
                        &bbr_ui::text::display_title(&tab.url, &tab.title),
                        budget,
                    )),
                    state: if drag_source == Some(tab.index) {
                        bbr_ui::state::TabVisualState::Dragging
                    } else {
                        crate::shell::row_state(
                            tab.active,
                            tab.loading,
                            tab.crashed,
                            tab.unresponsive,
                            self.pressed_row == Some(tab.index),
                            self.hovered_row == Some(tab.index),
                        )
                    },
                    favicon: Some(bbr_ui::components::FaviconSpec {
                        letter: bbr_ui::components::favicon::monogram_letter(&tab.url),
                        hue: bbr_ui::components::favicon::monogram_hue(&tab.url),
                        // The resolved favicon image for the tab's host in
                        // the owning profile's bounded store, if acquired
                        // (task packet `real-favicon-pipeline.md`); the
                        // tile renders it instead of the monogram.
                        image: self.favicon_image(profile, &host),
                    }),
                    pinned: tab.pinned,
                    show_close: true,
                    reveal_close: self.hovered_row == Some(tab.index)
                        || self.pressed_row == Some(tab.index),
                }
            })
            .collect()
    }
}
