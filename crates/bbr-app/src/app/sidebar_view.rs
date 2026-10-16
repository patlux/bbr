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
        self.rail_surface(theme, sidebar_width, chrome_focused, None, window, cx)
    }

    /// The rail composition, docked (`floating: None`) or inside the
    /// floating panel (blueprint D02): then the rail root stays transparent
    /// (the panel paints the material) and every callback runs against the
    /// browser window, which keeps the keyboard, whatever window delivered
    /// the pointer event.
    pub(crate) fn rail_surface(
        &mut self,
        theme: &Theme,
        sidebar_width: f32,
        chrome_focused: bool,
        floating: Option<gpui::WindowHandle<gpui_component::Root>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut model = self.sidebar_model(theme, sidebar_width, chrome_focused, window);
        model.floating = floating.is_some();
        let mut callbacks = self.sidebar_callbacks(cx);
        let field_anchor = match floating {
            Some(browser) => {
                callbacks = super::rail_float_view::retarget(callbacks, browser);
                crate::app::palette::command_field_anchor_following(
                    self.command_field_rect.clone(),
                    cx.entity(),
                    browser,
                )
                .into_any()
            }
            None => self.command_field_anchor_element(),
        };
        let favorites_band_anchor = self.favorites_band_anchor_element();
        // §11.2/§10.9: the ghost-row and drag-proxy rect recorders (placed
        // by the rail only while those render).
        let pin_offer_anchor = Some(
            crate::app::favorites_band::rect_anchor(self.drag_offer_rects.pin_offer.clone())
                .into_any(),
        );
        // Task packet `drag-back-out-of-pins`: the regular region begins
        // below the New Tab row.
        let new_tab_anchor = Some(
            crate::app::favorites_band::rect_anchor(self.drag_offer_rects.new_tab.clone())
                .into_any(),
        );
        let drag_proxy_anchor = Some(
            crate::app::favorites_band::rect_anchor(self.drag_offer_rects.proxy.clone()).into_any(),
        );
        // Blueprint D02: one rect recorder per folder row (refilled every
        // frame the folders render), the rows' 2-D drop rects.
        self.drag_offer_rects.folders.borrow_mut().clear();
        let folder_anchors = model
            .folders
            .as_ref()
            .map(|folders| {
                folders
                    .rows
                    .iter()
                    .map(|row| (row.id, self.folder_rect_anchor(row.id)))
                    .collect()
            })
            .unwrap_or_default();
        // Blueprint D19: one rect recorder per rendered tab row (a drop from
        // another window resolves its slot against them).
        self.drag_offer_rects.rows.borrow_mut().clear();
        let row_anchors = model
            .tabs
            .pinned
            .iter()
            .chain(model.tabs.window.iter())
            .map(|row| (row.index, self.tab_row_anchor(row.index)))
            .collect();
        let command_results = self
            .command_result_surface(theme)
            .map(|surface| surface.into_any());
        let downloads_panel = self
            .downloads_panel_surface(theme, sidebar_width, window, cx)
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
                pin_offer_anchor,
                new_tab_anchor,
                drag_proxy_anchor,
                folder_anchors,
                row_anchors,
                command_results,
                downloads_panel,
                space_config_panel,
            },
        )
    }

    /// The rect recorder of tab row `index` (blueprint D19).
    fn tab_row_anchor(&self, index: usize) -> gpui::AnyElement {
        let rects = self.drag_offer_rects.rows.clone();
        crate::app::favorites_band::RectAnchor::new(move |rect| {
            let mut rects = rects.borrow_mut();
            match rects.iter_mut().find(|(row, _)| *row == index) {
                Some(entry) => entry.1 = rect,
                None => rects.push((index, rect)),
            }
        })
        .into_any()
    }

    /// The rect recorder of folder row `folder` (blueprint D02): publishes
    /// the row's window rect into `drag_offer_rects.folders`.
    fn folder_rect_anchor(&self, folder: bbr_domain::FolderId) -> gpui::AnyElement {
        let rects = self.drag_offer_rects.folders.clone();
        crate::app::favorites_band::RectAnchor::new(move |rect| {
            let mut rects = rects.borrow_mut();
            match rects.iter_mut().find(|(id, _)| *id == folder) {
                Some(entry) => entry.1 = rect,
                None => rects.push((folder, rect)),
            }
        })
        .into_any()
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
        if favorites.is_none() {
            // The favorites region does not render this frame: drop the
            // recorded band rect so the snapshot and the drag resolver
            // never consume stale geometry from an earlier drag's band.
            *self.favorites_band_rect.borrow_mut() = None;
        }
        let folders = self.folders_model(&u2.folder_rows, &u2.folder_members, window);
        let tabs = self.tabs_model(theme, window, &u2.grouped);
        if !tabs.pin_offer {
            // No ghost row this frame: drop its recorded rect so neither the
            // resolver nor the snapshot consumes a previous drag's geometry.
            *self.drag_offer_rects.pin_offer.borrow_mut() = None;
        }
        let drag_proxy = self.drag_proxy_model(theme, sidebar_width);
        if drag_proxy.is_none() {
            *self.drag_offer_rects.proxy.borrow_mut() = None;
        }
        let browser = self.controller.read().snapshot();
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
        let strip = self.strip_model(window);
        let divider_focused = self.divider_handle.is_focused(window);
        SidebarModel {
            sidebar_width,
            chrome_tint,
            tint_fade: self.tint_fade(),
            grain: self.active_space_appearance().paper.grain,
            floating: false,
            active_space_title: SharedString::from(active_space_title),
            command,
            favorites,
            folders,
            tabs,
            drag_proxy,
            drag_destination: self.drag_destination(),
            status,
            status_visible,
            media: self.media_card_model(),
            // §11.14: the titlebar row is the rail's top strip — the
            // safe-area geometry (native), the sidebar toggle, and the
            // Back/Forward/Reload cluster. The toggle uses the identical
            // sidebar-width/collapse authority as Cmd-S and the row
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
    /// the row currently being dragged (§10.9 `dragging` state). The pointer-state
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
        let profile = self.controller.read().snapshot().active_profile;
        // Blueprint D23: the rows of a split group form a bracketed block.
        let marks = self.split_marks(tabs.len());
        tabs.iter()
            .map(|tab| {
                let host = bbr_ui::text::host_of(&tab.url);
                let closed = tab.lifecycle == bbr_browser::TabLifecycle::Closed;
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
                    // §10.12: a closed pinned row greys out and offers no
                    // close control.
                    closed,
                    show_close: !closed,
                    reveal_close: !closed
                        && (self.hovered_row == Some(tab.index)
                            || self.pressed_row == Some(tab.index)),
                    split: marks.get(tab.index).copied().flatten(),
                }
            })
            .collect()
    }
}
