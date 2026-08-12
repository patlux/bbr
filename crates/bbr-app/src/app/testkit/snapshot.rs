//! The semantic E2E snapshot of the root view (testkit feature).
//!
//! Maps the controller snapshot, theme tokens, drag state, and shell
//! presentation onto the typed protocol snapshot the E2E suites assert.

use gpui::Window;

use crate::shell::TestSnapshot;

use super::BbrApp;
use crate::app::command_rank::CommandResultRow;

/// Map one app-local ranked row onto the typed wire snapshot (the typed
/// activation stays app-local; the wire exposes the stable key/group/title,
/// the selection state the palette renders, and the destructive flag).
pub fn command_result_protocol(
    row: &CommandResultRow,
    selected: bool,
) -> bbr_testkit::protocol::CommandResultSnapshot {
    bbr_testkit::protocol::CommandResultSnapshot {
        key: row.key.clone(),
        group: row.group.clone(),
        title: row.title.clone(),
        subtitle: row.subtitle.clone(),
        selected,
        destructive: row.destructive,
    }
}

/// A snapshot for the E2E protocol (also used by unit tests).
impl BbrApp {
    pub fn snapshot(&mut self, window: &Window) -> TestSnapshot {
        let browser = self.controller.snapshot();
        let theme = self.theme();
        let drag_source = self.tab_drag.source();
        let rows = self.row_visuals(&browser.tabs, drag_source);
        let drag_slot = self.tab_drag.tab_slot();
        let drag_chip = self.tab_drag.chip_target();
        let space_chips: Vec<bbr_testkit::protocol::SpaceChipSnapshot> = self
            .space_chip_visuals()
            .into_iter()
            .map(|chip| bbr_testkit::protocol::SpaceChipSnapshot {
                space: bbr_testkit::protocol::TestSpaceId(chip.space.0),
                title: chip.title,
                identity: chip.identity,
                theme: chip.theme.name().to_string(),
                selected: chip.selected,
                drag_target: chip.drag_target,
                companion: chip.companion.map(|companion| {
                    bbr_testkit::protocol::ChipCompanionSnapshot {
                        profile: companion.profile_index,
                        monogram: companion.letter,
                        hue: companion.hue.name().to_string(),
                    }
                }),
            })
            .collect();
        let window_width = f32::from(window.bounds().size.width);
        let window_height = f32::from(window.bounds().size.height);
        let sidebar_width = if self.sidebar_visible {
            self.sidebar.effective_width(window_width)
        } else {
            0.0
        };
        // The authoritative pane layout and its keyed per-view geometry,
        // ordered by pane layout (primary, then secondary) — never map
        // iteration or creation order. The keyed observation is pruned to
        // the current panes each render, so destroyed/hidden panes leave no
        // orphan geometry evidence.
        let pane_layout = self.controller.pane_layout();
        let pane_rects = self.performance.pane_rects();
        let pane_rect_snapshots = match pane_layout {
            Some(layout) => {
                let mut rects = Vec::with_capacity(2);
                let rect_of = |view: bbr_web::WebViewId| {
                    pane_rects
                        .get(&view)
                        .map_or([0.0; 4], |r| [r.x, r.y, r.width, r.height])
                };
                rects.push(bbr_testkit::protocol::PaneRectSnapshot {
                    view: layout.primary,
                    rect: rect_of(layout.primary),
                });
                if let Some(secondary) = layout.secondary {
                    rects.push(bbr_testkit::protocol::PaneRectSnapshot {
                        view: secondary,
                        rect: rect_of(secondary),
                    });
                }
                rects
            }
            None => Vec::new(),
        };
        // The L5 organization read model (reconciled on read) plus the
        // app-local presentation state: the typed U2 snapshot fields. The
        // folder member rows resolve through the controller's global read
        // model (deterministic nested positions within expanded folders).
        let organization = self.organization.organization_snapshot(&self.controller);
        let favorite_tiles = crate::app::ui_adapters::favorite_tiles(&organization.favorites);
        let folder_rows = crate::app::ui_adapters::folder_row_specs(
            &organization.folders,
            &self.u2.folder_presentation,
        );
        let folder_members = crate::app::ui_adapters::folder_member_visuals(
            &folder_rows,
            &organization.folders,
            &browser.all_tabs,
        );
        let download_rows = self.download_rows_snapshot();
        let downloads = self.downloads_panel_snapshot();
        let error_surface = self.error_surface_snapshot();
        // The §11.16 Library presentation model — one model source shared
        // by the typed snapshot, the visible-id mirror, and the semantic
        // tree (the same model the render draws).
        let library_model = self.library_model();
        let semantics = self.semantic_snapshot(
            &browser,
            &rows,
            window,
            sidebar_width,
            &favorite_tiles,
            &folder_rows,
            &folder_members,
            &library_model,
        );
        let command_result_state = self.command_ranking.state;
        let command_results = self
            .command_ranking
            .results
            .iter()
            .enumerate()
            .map(|(index, row)| {
                command_result_protocol(row, index == self.command_ranking.selected)
            })
            .collect::<Vec<_>>();
        // The §11.5 in-rail result ids mirror the rail render; while a §11.4
        // palette/overlay presentation is open the floating surface renders
        // the same engine's results itself, so the in-rail list (and its
        // ids) is hidden — never duplicated.
        let in_rail_surface_ids = if self.command_surface.is_open() {
            Vec::new()
        } else {
            crate::app::command_surface::surface_ids(
                self.command_ranking.state,
                &self.command_ranking.results,
            )
        };
        // The error-surface id branch mirrors the render (task packet
        // `error-surface-contract`): the surface replaces the active view's
        // native slot, so the slot ids are not advertised while it shows.
        let error_surface_visible = error_surface.visible;
        let error_surface_origin = error_surface.origin.is_some();

        // §11.3 resolved appearance evidence: the active space's effective
        // appearance (the one controller authority) resolved through the
        // one central tint model. The snapshot carries the resolved
        // chrome/chip tint hex — the exact state the render paints — so E2E
        // can prove activation changes the appearance without reading raw
        // pixels.
        let active_appearance = {
            let appearance = self.active_space_appearance();
            let tint = bbr_ui::theme::space_tint(appearance.theme, &theme);
            Some(bbr_testkit::protocol::SpaceAppearanceSnapshot {
                space: bbr_testkit::protocol::TestSpaceId(appearance.space.0),
                theme: appearance.theme.name().to_string(),
                identity: appearance.identity.name(),
                chrome_tint: bbr_ui::theme::hex(tint.chrome),
                chip_tint: bbr_ui::theme::hex(tint.chip_active),
            })
        };
        let navigation_cover_visible = browser.active_view.is_some_and(|view| {
            self.find_engine
                .as_ref()
                .is_some_and(|engine| engine.borrow().navigation_cover_visible(view))
        });

        TestSnapshot {
            url: browser.nav.url.clone(),
            title: browser.nav.title.clone(),
            loading: browser.nav.loading,
            can_go_back: browser.nav.can_go_back,
            can_go_forward: browser.nav.can_go_forward,
            web_view_hosted: browser.web_view_hosted,
            navigation_cover_visible,
            active_profile: browser.active_profile,
            active_profile_title: browser.active_profile_title.clone(),
            active_profile_hue: browser.active_profile_hue.map(|hue| hue.name().to_string()),
            active_space: bbr_testkit::protocol::TestSpaceId(browser.active_space.0),
            tabs: browser.tabs.clone(),
            all_tabs: browser.all_tabs.clone(),
            profiles: browser.profiles.clone(),
            spaces: browser.spaces.clone(),
            active_tab_index: browser.active_index,
            active_web_view: browser.active_view,
            engine_sessions: browser.engine_sessions,
            engine_kind: self.engine_kind,
            chrome_focused: self.chrome_focused(window),
            lifecycle_counts: bbr_testkit::protocol::LifecycleCounts {
                active: browser.lifecycle_counts.active,
                warm: browser.lifecycle_counts.warm,
                suspended: browser.lifecycle_counts.suspended,
                discarded: browser.lifecycle_counts.discarded,
            },
            dense: self.dense_snapshot(&browser),
            shell_modes: bbr_testkit::protocol::ShellModes {
                theme: match theme.mode {
                    bbr_ui::theme::ThemeMode::Light => "light".into(),
                    bbr_ui::theme::ThemeMode::Dark => "dark".into(),
                },
                contrast: match theme.contrast {
                    bbr_ui::theme::ContrastMode::Standard => "standard".into(),
                    bbr_ui::theme::ContrastMode::Increased => "increased".into(),
                },
                density: match theme.density {
                    bbr_ui::theme::Density::Small => "small".into(),
                    bbr_ui::theme::Density::Normal => "normal".into(),
                    bbr_ui::theme::Density::ExtraLarge => "extralarge".into(),
                },
            },
            tab_states: rows
                .iter()
                .map(|row| match row.state {
                    bbr_ui::state::TabVisualState::Default => {
                        bbr_testkit::protocol::TabVisualState::Default
                    }
                    bbr_ui::state::TabVisualState::Hover => {
                        bbr_testkit::protocol::TabVisualState::Hover
                    }
                    bbr_ui::state::TabVisualState::Active => {
                        bbr_testkit::protocol::TabVisualState::Active
                    }
                    bbr_ui::state::TabVisualState::Selected => {
                        bbr_testkit::protocol::TabVisualState::Selected
                    }
                    bbr_ui::state::TabVisualState::Loading => {
                        bbr_testkit::protocol::TabVisualState::Loading
                    }
                    bbr_ui::state::TabVisualState::Attention => {
                        bbr_testkit::protocol::TabVisualState::Attention
                    }
                    bbr_ui::state::TabVisualState::Error => {
                        bbr_testkit::protocol::TabVisualState::Error
                    }
                    bbr_ui::state::TabVisualState::Unresponsive => {
                        bbr_testkit::protocol::TabVisualState::Unresponsive
                    }
                    bbr_ui::state::TabVisualState::Disabled => {
                        bbr_testkit::protocol::TabVisualState::Disabled
                    }
                    bbr_ui::state::TabVisualState::Dragging => {
                        bbr_testkit::protocol::TabVisualState::Dragging
                    }
                    bbr_ui::state::TabVisualState::DragTarget => {
                        bbr_testkit::protocol::TabVisualState::DragTarget
                    }
                    bbr_ui::state::TabVisualState::Focused => {
                        bbr_testkit::protocol::TabVisualState::Focused
                    }
                })
                .collect(),
            row_titles: rows.iter().map(|row| row.title.to_string()).collect(),
            pinned_count: crate::shell::pinned_count(&browser.tabs),
            command_bar_hint: self.command_bar_hint(),
            command_profile_hint: self.command_profile_hint.clone(),
            drag_source,
            drag_slot,
            drag_chip: drag_chip.map(|space| bbr_testkit::protocol::TestSpaceId(space.0)),
            drag_favorites: self.tab_drag.target()
                == Some(crate::app::drag_target::DragTarget::Favorites),
            favorites_rect: self
                .favorites_band_rect
                .borrow()
                .map(|rect| [rect.x, rect.y, rect.width, rect.height]),
            drag_valid: self.tab_drag.valid(),
            space_chips,
            active_appearance,
            space_strip_scroll: self.space_strip_scroll_snapshot(),
            space_config: self.space_config_snapshot(window),
            swipe: self.swipe_snapshot(),
            visible_ids: {
                let mut ids = crate::shell::snapshot::visible_ids_u2(
                    &browser,
                    self.chrome_focused(window),
                    self.command_bar_mode,
                    self.sidebar_visible,
                    self.command_bar_hint().as_deref(),
                    self.command_profile_hint.as_deref(),
                    self.command_surface,
                    command_result_state,
                    &self.command_ranking.results,
                    &in_rail_surface_ids,
                    drag_slot,
                    self.tab_drag.chip_target(),
                    self.dense_window.get(),
                    pane_layout,
                    &crate::shell::snapshot::U2IdsInput {
                        favorites: &organization.favorites,
                        favorites_band_visible: self.tab_drag.is_active()
                            && organization.favorites.is_empty(),
                        folders: &organization.folders,
                        expanded: self.u2.folder_presentation.clone(),
                        plus_menu: self.u2.plus_menu,
                        downloads_open: self.downloads.open,
                        downloads_window: self
                            .downloads
                            .window_for(self.downloads.render_state.scroll_anchor),
                        downloads_layout: self.downloads_layout(),
                        downloads_delete_rows: self.downloads_delete_rows_in_window(),
                        error_surface_visible,
                        error_surface_origin,
                        library_open: self.library.open,
                        library_ids: crate::app::library::model::library_visible_ids(
                            &library_model,
                        ),
                        space_config_open: self.space_config.open,
                    },
                );
                ids.extend(self.tab_menu_visible_ids());
                ids
            },
            semantics,
            command_bar_mode: self.command_bar_mode.into(),
            command_bar_text: self.command_text.text.clone(),
            address_domain: bbr_ui::text::address_domain(&self.active_url()),
            address_hovered: self.address_hovered,
            address_rect: self
                .command_field_rect
                .borrow()
                .as_ref()
                .copied()
                .map(|r| [r.x, r.y, r.width, r.height]),
            command_surface: self.command_surface.into(),
            command_result_state: command_result_state.into(),
            command_results,
            command_result_total: self.command_ranking.total,
            command_cursor: self.command_text.cursor,
            command_selection: self.command_text.selection.map(|(start, end)| [start, end]),
            sidebar_visible: self.sidebar_visible,
            titlebar: {
                let safe = self.safe_area(window);
                bbr_testkit::protocol::TitlebarSnapshot {
                    safe_area_height: safe.height,
                    traffic_lights: safe.traffic_lights,
                    sidebar_collapsed: !self.sidebar_visible,
                    can_go_back: browser.nav.can_go_back,
                    can_go_forward: browser.nav.can_go_forward,
                    can_reload: browser.active_view.is_some(),
                }
            },
            sidebar_width,
            sidebar_min: bbr_ui::sidebar::geometry::SIDEBAR_MIN,
            sidebar_default: bbr_ui::sidebar::geometry::SIDEBAR_DEFAULT,
            sidebar_max: bbr_ui::sidebar::geometry::SIDEBAR_MAX,
            content_min: bbr_ui::sidebar::geometry::CONTENT_MIN,
            divider_dragging: self.sidebar.is_dragging(),
            divider_label: bbr_ui::sidebar::geometry::SIDEBAR_DIVIDER_LABEL.to_string(),
            content_rect: self.performance.content_rect().unwrap_or([0.0; 4]),
            pane_layout,
            pane_rects: pane_rect_snapshots,
            window_size: [window_width, window_height],
            window_content_size: self
                .window_geometry
                .borrow()
                .map_or([0.0, 0.0], |[_, _, width, height]| {
                    [width as f32, height as f32]
                }),
            portal: self.portal_snapshot(),
            find: self.find_snapshot(window),
            zoom: {
                let zoom = self.zoom_snapshot();
                bbr_testkit::protocol::ZoomSnapshot {
                    level: zoom.level,
                    site: zoom.site,
                    view: zoom.view,
                    applied: zoom.applied,
                    pending: zoom.pending,
                }
            },
            context_menu: self.context_menu_snapshot(),
            tab_menu: self.tab_menu_snapshot(),
            auth: self.auth_snapshot_protocol(),
            plus_menu_open: self.u2.plus_menu.open,
            plus_menu_selection: self.u2.plus_menu.selection,
            favorites: favorite_tiles
                .iter()
                .map(|tile| bbr_testkit::protocol::FavoriteTileSnapshot {
                    index: tile.index,
                    tab: tile.tab,
                    space: tile.space,
                    profile: tile.profile,
                    title: tile.title.clone(),
                    url: tile.url.clone(),
                    letter: tile.letter.to_string(),
                    hue: tile.hue,
                })
                .collect(),
            folders: folder_rows
                .iter()
                .map(|row| bbr_testkit::protocol::FolderRowSnapshot {
                    index: row.index,
                    id: row.id.0,
                    title: row.title.clone(),
                    tab_count: row.count,
                    expanded: row.expanded,
                })
                .collect(),
            download_rows,
            downloads,
            #[cfg(all(target_os = "macos", feature = "testkit"))]
            web_content_probe: self
                .web_content_probe
                .borrow()
                .as_ref()
                .map(|(view, probe)| bbr_testkit::protocol::WebContentProbe {
                    view: *view,
                    ready_state: probe.ready_state.clone(),
                    dom_url: probe.dom_url.clone(),
                    dom_title: probe.dom_title.clone(),
                    has_body: probe.has_body,
                    root_background_color: probe.root_background_color.clone(),
                    body_background_color: probe.body_background_color.clone(),
                    native_url: probe.native_url.clone(),
                    native_title: probe.native_title.clone(),
                }),
            #[cfg(not(all(target_os = "macos", feature = "testkit")))]
            web_content_probe: None,
            error_surface,
            store: self.store_snapshot(),
            restore: match self.restore_evidence() {
                crate::app::RestoreEvidence::Fresh => {
                    bbr_testkit::protocol::RestoreDisposition::Fresh
                }
                crate::app::RestoreEvidence::Restored { deferred_tabs } => {
                    bbr_testkit::protocol::RestoreDisposition::Restored { deferred_tabs }
                }
                crate::app::RestoreEvidence::Failed => {
                    bbr_testkit::protocol::RestoreDisposition::Failed
                }
            },
            new_window: {
                let state = self.new_window_state();
                bbr_testkit::protocol::NewWindowSnapshot {
                    requests: state.requests,
                    created_tabs: state.created_tabs,
                    created_background_tabs: state.created_background_tabs,
                    blocked_reported: state.blocked_reported,
                }
            },
            content_gesture: {
                let state = self.content_gesture_state();
                let relations = self.controller.opener_pairs();
                bbr_testkit::protocol::ContentGestureSnapshot {
                    commits: state.commits,
                    history_routed: state.history_routed,
                    opener_returns: state.opener_returns,
                    no_ops: state.no_ops,
                    opener_relations: relations
                        .into_iter()
                        .map(|(child, opener)| (child.0, opener.0))
                        .collect(),
                }
            },
            preview: self.controller.snapshot().preview.map(|preview| {
                bbr_testkit::protocol::PreviewSnapshot {
                    id: preview.id,
                    view: preview.view,
                    source: preview.source,
                    profile: preview.profile,
                    url: preview.url,
                    title: preview.title.clone(),
                    loading: preview.loading,
                    can_go_back: preview.can_go_back,
                    can_go_forward: preview.can_go_forward,
                    crashed: preview.crashed,
                }
            }),
            peek_presented: self.peek_presented(),
            #[cfg(target_os = "macos")]
            preview_z_order: {
                // Truthful native z-order evidence (Link Preview / Peek
                // packet): while a preview is open the preview WKWebView's
                // subview index must exceed its covered origin's — the
                // preview is raised above the warm origin by
                // `raise_web_view`. `None` at rest or when the native
                // engine is unavailable.
                let snapshot = self.controller.snapshot();
                match (snapshot.preview, self.find_engine.as_ref()) {
                    (Some(preview), Some(engine)) => {
                        let engine = engine.borrow();
                        match (
                            engine.subview_index(preview.view),
                            engine.subview_index(preview.source),
                        ) {
                            (Some(preview_index), Some(source_index)) => {
                                Some(bbr_testkit::protocol::PreviewZOrder {
                                    preview_index,
                                    source_index,
                                })
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            },
            #[cfg(not(target_os = "macos"))]
            preview_z_order: None,
            library: self.library_snapshot(&library_model),
        }
    }

    /// The Phase-7 store composition snapshot: the truthful startup
    /// disposition (never silently converted), the effective global search
    /// engine feeding the command-bar fallback, and the in-memory history
    /// provider population ranking consumes.
    fn store_snapshot(&self) -> bbr_testkit::protocol::StoreSnapshot {
        use bbr_testkit::protocol::{StoreDiscardReasonSnapshot, StoreLoadSnapshot};
        let store_load = match self.store.store_load.as_ref() {
            Some(bbr_persistence::StoreLoad::Loaded) => StoreLoadSnapshot::Loaded,
            Some(bbr_persistence::StoreLoad::Discarded { reason }) => {
                let reason = match reason {
                    bbr_persistence::StoreDiscardReason::Unstamped => {
                        StoreDiscardReasonSnapshot::Unstamped
                    }
                    bbr_persistence::StoreDiscardReason::VersionMismatch { found } => {
                        StoreDiscardReasonSnapshot::VersionMismatch { found: *found }
                    }
                    bbr_persistence::StoreDiscardReason::Corrupt { detail } => {
                        StoreDiscardReasonSnapshot::Corrupt {
                            detail: detail.clone(),
                        }
                    }
                };
                StoreLoadSnapshot::Discarded { reason }
            }
            None => StoreLoadSnapshot::Unavailable,
        };
        let history_count = self.store.with_history(|history| history.len());
        let saved_pages_count = self.store.saved_pages_count();
        bbr_testkit::protocol::StoreSnapshot {
            store_load,
            search_engine: self.store.effective_settings().engine().id().to_string(),
            history_count,
            saved_pages_count,
        }
    }

    /// The §11.16 Saved Pages Library presentation snapshot (B3 packet
    /// `saved-pages-library`): the typed state the E2E suites assert,
    /// derived from the same presentation model the render draws plus the
    /// app-local state and the store's provider count — the empty default
    /// when the Library is closed (the surface is not drawn).
    fn library_snapshot(
        &self,
        model: &bbr_ui::components::SavedPagesLibraryModel,
    ) -> bbr_testkit::protocol::SavedPagesLibrarySnapshot {
        use bbr_testkit::protocol::{
            SavedPagesCollectionSnapshot, SavedPagesLibraryRowSnapshot, SavedPagesLibrarySnapshot,
        };
        if !self.library.open {
            return SavedPagesLibrarySnapshot::default();
        }
        let native_view_hidden = self
            .controller
            .snapshot()
            .active_view
            .is_some_and(|view| self.library.is_hidden(view));
        SavedPagesLibrarySnapshot {
            open: true,
            stage: match model.stage {
                bbr_ui::components::LibraryStage::Loading => "loading".into(),
                bbr_ui::components::LibraryStage::Ready => "ready".into(),
            },
            search: model.search.clone(),
            sort: model.sort.as_str().to_string(),
            total: model.total,
            rows: model
                .rows
                .iter()
                .map(|row| SavedPagesLibraryRowSnapshot {
                    bookmark: row.bookmark,
                    title: row.title.clone(),
                    url: row.url.clone(),
                    collection: row.collection.clone(),
                    selected: row.selected,
                    favicon_letter: row.favicon.letter.to_string(),
                })
                .collect(),
            window_first: model.window.first,
            window_last: model.window.last,
            window_anchor: model.window.anchor,
            collections: model
                .collections
                .iter()
                .map(|collection| SavedPagesCollectionSnapshot {
                    id: collection.id,
                    title: collection.title.clone(),
                    member_count: collection.member_count,
                })
                .collect(),
            selected: self.library.selected.clone(),
            remove_confirm: self.library.remove_confirm,
            remove_targets: self.library.remove_targets.clone(),
            collection_action: self
                .library
                .collection_action
                .map(|(id, action)| (id, action.as_str().to_string())),
            collection_draft: self.library.collection_draft.clone(),
            collection_create_staged: self.library.collection_create_staged,
            move_sheet_open: self.library.move_sheet_open,
            scroll_offset: self.library.scroll_offset,
            editing_row: self.library.editing_row,
            rename_draft: self.library.rename_draft.clone(),
            native_view_hidden,
            saved_pages_count: self.store.saved_pages_count(),
        }
    }

    /// The §11.8-style error-surface presentation (task packet
    /// `error-surface-contract`): the classified incident the content area
    /// renders for the active view, derived from the same presentation
    /// state the render consumes (the render excludes the process-crash
    /// class — the DESIGN §11.8 crash panel is its surface). The empty
    /// default means no error surface is shown.
    fn error_surface_snapshot(&self) -> bbr_testkit::protocol::ErrorSurfaceSnapshot {
        let Some(view) = self.controller.snapshot().active_view else {
            return bbr_testkit::protocol::ErrorSurfaceSnapshot::default();
        };
        let Some(spec) = self.error_surface.spec_for(view) else {
            return bbr_testkit::protocol::ErrorSurfaceSnapshot::default();
        };
        if spec.incident.reason.category() == bbr_web::web_errors::IncidentCategory::ProcessCrash {
            return bbr_testkit::protocol::ErrorSurfaceSnapshot::default();
        }
        use bbr_testkit::protocol::ErrorSurfaceCategory;
        bbr_testkit::protocol::ErrorSurfaceSnapshot {
            visible: true,
            view: Some(view),
            category: Some(match spec.incident.reason.category() {
                bbr_web::web_errors::IncidentCategory::Tls => ErrorSurfaceCategory::Tls,
                bbr_web::web_errors::IncidentCategory::Navigation => {
                    ErrorSurfaceCategory::Navigation
                }
                bbr_web::web_errors::IncidentCategory::Offline => ErrorSurfaceCategory::Offline,
                bbr_web::web_errors::IncidentCategory::ProcessCrash => {
                    unreachable!("the crash class is presented by the §11.8 crash panel")
                }
            }),
            reason: Some(spec.incident.reason.to_string()),
            origin: spec.origin.clone(),
            title: Some(spec.title.to_string()),
            action: Some(spec.action_label.to_string()),
            lifecycle: Some(spec.state.to_string()),
        }
    }

    /// The §11.12 download rows in deterministic panel order (Active by
    /// ascending `DownloadId`, then History most-recent-terminal first) —
    /// row index `<n>` is stable for E2E. Derived from the same row model
    /// the render builds, so the snapshot cannot drift from the shell.
    fn download_rows_snapshot(&self) -> Vec<bbr_testkit::protocol::DownloadRowSnapshot> {
        crate::app::downloads::row_visuals(&self.downloads, Some(self.downloads.focus))
            .into_iter()
            .map(|row| bbr_testkit::protocol::DownloadRowSnapshot {
                row: row.index,
                id: row.id,
                machine_state: match row.machine {
                    bbr_ui::surfaces::downloads::DownloadMachine::Waiting => {
                        bbr_testkit::protocol::DownloadMachineState::Waiting
                    }
                    bbr_ui::surfaces::downloads::DownloadMachine::Running => {
                        bbr_testkit::protocol::DownloadMachineState::Running
                    }
                    bbr_ui::surfaces::downloads::DownloadMachine::Succeeded => {
                        bbr_testkit::protocol::DownloadMachineState::Succeeded
                    }
                    bbr_ui::surfaces::downloads::DownloadMachine::Failed => {
                        bbr_testkit::protocol::DownloadMachineState::Failed
                    }
                    bbr_ui::surfaces::downloads::DownloadMachine::Cancelled => {
                        bbr_testkit::protocol::DownloadMachineState::Cancelled
                    }
                },
                group: match row.group {
                    bbr_ui::surfaces::downloads::DownloadGroupKind::Active => {
                        bbr_testkit::protocol::DownloadGroup::Active
                    }
                    bbr_ui::surfaces::downloads::DownloadGroupKind::History => {
                        bbr_testkit::protocol::DownloadGroup::History
                    }
                },
                filename: row.filename.clone(),
                progress_fraction: row.progress_fraction,
                visual_state: match row.visual {
                    bbr_ui::state::DownloadVisual::Default => {
                        bbr_testkit::protocol::TabVisualState::Default
                    }
                    bbr_ui::state::DownloadVisual::Hover => {
                        bbr_testkit::protocol::TabVisualState::Hover
                    }
                    bbr_ui::state::DownloadVisual::Pressed => {
                        bbr_testkit::protocol::TabVisualState::Active
                    }
                    bbr_ui::state::DownloadVisual::Focused => {
                        bbr_testkit::protocol::TabVisualState::Focused
                    }
                    bbr_ui::state::DownloadVisual::Disabled => {
                        bbr_testkit::protocol::TabVisualState::Disabled
                    }
                },
                action: match row.action {
                    bbr_ui::surfaces::downloads::DownloadActionKind::Cancel => {
                        bbr_testkit::protocol::DownloadRowAction::Cancel
                    }
                    bbr_ui::surfaces::downloads::DownloadActionKind::Retry => {
                        bbr_testkit::protocol::DownloadRowAction::Retry
                    }
                    bbr_ui::surfaces::downloads::DownloadActionKind::Restart => {
                        bbr_testkit::protocol::DownloadRowAction::Restart
                    }
                    bbr_ui::surfaces::downloads::DownloadActionKind::Reveal => {
                        bbr_testkit::protocol::DownloadRowAction::Reveal
                    }
                },
                action_enabled: row.action_enabled,
                action_reason: row
                    .action_reason
                    .as_deref()
                    .map(|reason| crate::app::downloads::disabled_reason(Some(reason)).to_string()),
                meta: row.meta.clone(),
                resolved_path: row.resolved_path.clone(),
                retry_of: row.retry_of,
                attempt: row.attempt,
                delete_available: row.delete_available,
            })
            .collect()
    }

    /// The window row indices that render the §17.8 Delete control
    /// (cancelled rows inside the bounded render window) — the same row
    /// model the render and the visible-ids derivation consume, so the ids
    /// mirror the shell's gating exactly.
    fn downloads_delete_rows_in_window(&self) -> Vec<usize> {
        let window = self
            .downloads
            .window_for(self.downloads.render_state.scroll_anchor);
        crate::app::downloads::row_visuals(&self.downloads, Some(self.downloads.focus))
            .into_iter()
            .filter(|row| row.delete_available && window.contains(&row.index))
            .map(|row| row.index)
            .collect()
    }

    /// The §11.12 downloads panel render contract (open, focus, bounded
    /// window): exactly what the last render built. Closed panels report the
    /// empty default so a dismissed surface never exposes stale window
    /// evidence.
    fn downloads_panel_snapshot(&self) -> bbr_testkit::protocol::DownloadsPanelSnapshot {
        let state = &self.downloads;
        if !state.open {
            return bbr_testkit::protocol::DownloadsPanelSnapshot::default();
        }
        bbr_testkit::protocol::DownloadsPanelSnapshot {
            open: true,
            focused_row: Some(state.focus),
            anchor: state.render_state.scroll_anchor,
            rendered_rows: state.render_state.rendered_rows,
            total_rows: state.render_state.item_count,
            viewport_rows: state.render_state.viewport_rows,
            max_visible: bbr_ui::surfaces::downloads::DOWNLOADS_MAX_VISIBLE,
            footer_count: self.downloads_controller_active_count().unwrap_or(0),
        }
    }

    /// Typed diagnostics of the one native portal host (task packet
    /// `native-portal-host`, DESIGN.md §15): the active presentation's
    /// logical state from the portal state + probe surface, and the native
    /// attachment/frame/visibility evidence from the bridge. Exposes only
    /// the active presentation; the closed host reports the empty default.
    fn portal_snapshot(&self) -> bbr_testkit::protocol::PortalSnapshot {
        let probe = self.portal_probe.borrow();
        let mut portal = bbr_testkit::protocol::PortalSnapshot {
            presentation: probe.presentation().map(|id| id.0),
            host: probe.host().map(|host| host.label().to_string()),
            visible: probe.is_visible(),
            rect: probe.resolved_rect().map(|r| [r.x, r.y, r.width, r.height]),
            ids: probe.ids(),
            ..Default::default()
        };
        #[cfg(target_os = "macos")]
        if let Some(native) = self.native_portal_diagnostics() {
            portal.child_attached = native.child_attached;
            portal.native_visible = native.native_visible;
            portal.frame_synced = native.synchronized;
            portal.nonactivating_panel = native.nonactivating_panel;
            portal.parent_window_number = native.parent_window_number;
            portal.portal_window_number = native.portal_window_number;
            portal.frontmost_at_probe_point = native.frontmost_at_probe_point;
            portal.frontmost_at_probe_point_is_this_app =
                native.frontmost_at_probe_point_is_this_app;
            portal.below_portal_at_probe_point = native.below_portal_at_probe_point;
            portal.portal_owns_probe_point = native.portal_owns_probe_point;
            portal.portal_is_key = native.portal_is_key;
            portal.portal_is_main = native.portal_is_main;
            portal.parent_fullscreen = native.parent_fullscreen;
            portal.parent_minimized = native.parent_minimized;
            portal.parent_on_active_space = native.parent_on_active_space;
            portal.portal_on_active_space = native.portal_on_active_space;
            portal.portal_scale_factor = native.portal_scale_factor;
        }
        portal
    }

    /// Typed find-in-page presentation state (task packet
    /// `find-in-page-portal-bar`): the open flag, the find field's real
    /// GPUI focus, the live query, the WebKit-reported count/ordinal, a
    /// pending request, and the stale marker after a document change.
    fn find_snapshot(&self, window: &Window) -> bbr_testkit::protocol::FindSnapshot {
        let state = &self.find.state;
        bbr_testkit::protocol::FindSnapshot {
            open: state.open,
            focused: state.open && self.find_handle.is_focused(window),
            query: state.query.clone(),
            match_count: state.match_count,
            current_match: state.current_match,
            pending: state.pending,
            stale: state.stale,
        }
    }

    /// Typed credential-sheet presentation state (queue item 10): the
    /// presented challenge's identity/metadata, lifecycle state, redacted
    /// outcome, and masked edit flags. Mirrors the coordinator's redacted
    /// `AuthSnapshotState` 1:1; by construction no field can hold a
    /// credential (the typed values never leave the sheet).
    fn auth_snapshot_protocol(&self) -> bbr_testkit::protocol::AuthSnapshot {
        let snap = self.auth_snapshot();
        bbr_testkit::protocol::AuthSnapshot {
            open: snap.open,
            challenge: snap.challenge,
            web_view: snap.web_view,
            host: snap.host,
            realm: snap.realm,
            scheme: snap.scheme,
            authority: snap.authority,
            state: snap.state,
            outcome: snap.outcome,
            focused_field: snap.focused_field,
            username_entered: snap.username_entered,
            password_entered: snap.password_entered,
            awaiting: snap.awaiting,
        }
    }

    /// The typed semantic tree of the current render (DESIGN.md §15 /
    /// accessibility track). Mirrors the shell: only rows the render
    /// actually built (the pinned block + the bounded regular window), the
    /// strip chips as rendered (the permanent §11.3 footer strip), the U2
    /// surfaces (footer, favorites grid, folders, plus menu), and the
    /// content-pane branch. The focus state comes from GPUI's real focus
    /// tree.
    #[allow(clippy::too_many_arguments)]
    fn semantic_snapshot(
        &self,
        browser: &bbr_browser::ControllerSnapshot,
        rows: &[bbr_ui::components::RowVisuals],
        window: &Window,
        sidebar_width: f32,
        favorite_tiles: &[bbr_ui::components::FavoriteTileSpec],
        folder_rows: &[bbr_ui::components::FolderRowSpec],
        folder_members: &[bbr_ui::components::FolderMemberVisual],
        library: &bbr_ui::components::SavedPagesLibraryModel,
    ) -> Vec<bbr_testkit::protocol::SemanticNode> {
        // Intersect the row visuals with the render's bounded window, like
        // `visible_ids`: a dense collection never exposes rows it did not
        // render.
        let total = browser.tabs.len();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let dense = self.dense_window.get();
        let mut rendered_indices: Vec<usize> = (0..pinned.min(total)).collect();
        rendered_indices.extend(
            (dense.first_rendered..dense.first_rendered + dense.rendered_rows)
                .filter(|&index| index < total),
        );
        let rendered_rows: Vec<bbr_ui::components::RowVisuals> = rendered_indices
            .into_iter()
            .filter_map(|index| rows.get(index).cloned())
            .collect();
        // The content branch mirrors the render exactly: the §11.16 Saved
        // Pages Library (B3 packet `saved-pages-library`) wins while open
        // (the internal chrome surface covers the content area and the
        // active view's native slot is hidden, never destroyed), then the
        // §11.8-style error surface (task packet
        // `error-surface-contract`) wins over the native slot, and the
        // crash panel wins over the empty pane.
        let content = if self.library.open {
            crate::shell::semantics::SemanticContent::Library {
                model: library.clone(),
                focused: self.library_handle.is_focused(window),
            }
        } else if let Some(spec) = self.active_error_surface_spec() {
            crate::shell::semantics::SemanticContent::Error {
                title: spec.title.to_string(),
                origin: spec.origin.clone(),
                copy: spec.copy.to_string(),
                action: spec.action_label.to_string(),
                dismiss: spec.dismiss_label.to_string(),
            }
        } else if browser.preview.is_none()
            && browser
                .active_index
                .and_then(|index| browser.tabs.get(index))
                .is_some_and(|tab| tab.unresponsive)
        {
            // §11.8a unresponsive pane (renderer-isolation lane): mirrors
            // `render.rs` — the pane replaces the active view's native slot
            // while the renderer is unresponsive (the session stays alive).
            // The preview guard mirrors the pane predicate's slot check: a
            // Peek preview covers the origin view with its own slot, so the
            // pane is never drawn while a preview is open.
            crate::shell::semantics::SemanticContent::Unresponsive
        } else if browser.active_view.is_some() {
            crate::shell::semantics::SemanticContent::Native
        } else if browser
            .active_index
            .and_then(|index| browser.tabs.get(index))
            .is_some_and(|tab| tab.crashed)
        {
            crate::shell::semantics::SemanticContent::Crash
        } else {
            crate::shell::semantics::SemanticContent::Empty {
                profile_line: crate::shell::profile_line(
                    browser.active_profile_title.as_deref().unwrap_or("Default"),
                ),
            }
        };
        let input = crate::shell::semantics::SemanticInput {
            browser,
            focus: crate::shell::semantics::SemanticFocus {
                command: self.chrome_handle.is_focused(window),
                spaces: self.space_strip_handle.is_focused(window),
                tabs: self.tab_list_handle.is_focused(window),
                divider: self.divider_handle.is_focused(window),
                folders: self.u2.folders_handle.is_focused(window),
                plus_menu: self.u2.plus_menu_handle.is_focused(window),
            },
            command: crate::shell::semantics::SemanticCommand {
                mode: format!("{:?}", self.command_bar_mode).to_lowercase(),
                // The §11.5 pill's derived domain (display-only; the pill
                // never holds the editable text).
                address: bbr_ui::text::address_domain(&self.active_url()),
                text: self.command_text.text.clone(),
                hint: self.command_bar_hint(),
                results: self
                    .command_ranking
                    .results
                    .iter()
                    .enumerate()
                    .map(|(index, row)| crate::shell::semantics::SemanticResultRow {
                        key: row.key.clone(),
                        title: row.title.clone(),
                        subtitle: row.subtitle.clone(),
                        selected: index == self.command_ranking.selected,
                        destructive: row.destructive,
                    })
                    .collect(),
                profile_hint: self.command_profile_hint.as_ref().and_then(|_| {
                    // The hint is only ever set to the active profile (any
                    // context mutation clears it), so the chip identity and
                    // title derive from the controller snapshot.
                    let title = browser.active_profile_title.clone().unwrap_or_default();
                    crate::shell::space::active_profile_chip(browser).map(
                        |(profile_index, spec)| crate::shell::semantics::SemanticProfileHint {
                            title,
                            monogram: spec.letter,
                            profile_index,
                        },
                    )
                }),
            },
            sidebar_visible: self.sidebar_visible,
            sidebar_width,
            divider_label: bbr_ui::sidebar::geometry::SIDEBAR_DIVIDER_LABEL.to_string(),
            command_surface: self.command_surface,
            rows: &rendered_rows,
            space_chips: &self.space_chip_visuals(),
            favorites: favorite_tiles,
            favorites_band_targeted: self.tab_drag.target()
                == Some(crate::app::drag_target::DragTarget::Favorites),
            folder_rows,
            folder_members,
            plus_menu: self.u2.plus_menu,
            content,
        };
        crate::shell::semantics::semantic_tree(&input)
    }

    /// The dense-list render contract from the last frame's render state and
    /// the active tab: bounded rendered window, deterministic scroll anchor,
    /// and whether the active row is inside the rendered window.
    fn dense_snapshot(
        &self,
        browser: &bbr_browser::ControllerSnapshot,
    ) -> bbr_testkit::protocol::DenseListSnapshot {
        let dense = self.dense_window.get();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let active_row = browser
            .active_index
            .map(|index| index.saturating_sub(pinned));
        let rendered_range = dense.first_rendered..dense.first_rendered + dense.rendered_rows;
        // A pinned active row is always on screen: the pinned block renders
        // fully above the scroll view (§11.2), so visibility never depends on
        // the scroll position. A regular-row selection is visible when it
        // lies inside the bounded window the render actually built.
        let active_visible = match browser.active_index {
            Some(index) if index < pinned => true,
            Some(_) => active_row
                .map(|row| rendered_range.contains(&(pinned + row)))
                .unwrap_or(false),
            None => false,
        };
        bbr_testkit::protocol::DenseListSnapshot {
            item_count: dense.item_count,
            pinned_rows: pinned,
            first_rendered: dense.first_rendered,
            rendered_rows: dense.rendered_rows,
            window_rows: crate::dense::DENSE_WINDOW_ROWS,
            scroll_anchor: dense.scroll_anchor,
            active_visible,
            active_row,
        }
    }
}
