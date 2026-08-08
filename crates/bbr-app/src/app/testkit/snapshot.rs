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
        let drag_space = self
            .tab_drag
            .space_target()
            .map(|space| bbr_testkit::protocol::TestSpaceId(space.0));
        let budget = crate::shell::space_title_budget_chars(&theme);
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
        let favorite_tiles = crate::components::favorite_tiles(&organization.favorites);
        let folder_rows = crate::components::folder_row_specs(
            &organization.folders,
            &self.u2.folder_presentation,
        );
        let folder_members = crate::components::folder_member_visuals(
            &folder_rows,
            &organization.folders,
            &browser.all_tabs,
        );
        let download_rows = self.download_rows_snapshot();
        let downloads = self.downloads_panel_snapshot();
        let semantics = self.semantic_snapshot(
            &browser,
            &rows,
            window,
            sidebar_width,
            &budget,
            &favorite_tiles,
            &folder_rows,
            &folder_members,
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
        TestSnapshot {
            url: browser.nav.url.clone(),
            title: browser.nav.title.clone(),
            loading: browser.nav.loading,
            can_go_back: browser.nav.can_go_back,
            can_go_forward: browser.nav.can_go_forward,
            web_view_hosted: browser.web_view_hosted,
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
                    crate::theme::ThemeMode::Light => "light".into(),
                    crate::theme::ThemeMode::Dark => "dark".into(),
                },
                contrast: match theme.contrast {
                    crate::theme::ContrastMode::Standard => "standard".into(),
                    crate::theme::ContrastMode::Increased => "increased".into(),
                },
                density: match theme.density {
                    crate::theme::Density::Small => "small".into(),
                    crate::theme::Density::Normal => "normal".into(),
                    crate::theme::Density::ExtraLarge => "extralarge".into(),
                },
            },
            tab_states: rows
                .iter()
                .map(|row| match row.state {
                    crate::components::TabVisualState::Default => {
                        bbr_testkit::protocol::TabVisualState::Default
                    }
                    crate::components::TabVisualState::Hover => {
                        bbr_testkit::protocol::TabVisualState::Hover
                    }
                    crate::components::TabVisualState::Active => {
                        bbr_testkit::protocol::TabVisualState::Active
                    }
                    crate::components::TabVisualState::Selected => {
                        bbr_testkit::protocol::TabVisualState::Selected
                    }
                    crate::components::TabVisualState::Loading => {
                        bbr_testkit::protocol::TabVisualState::Loading
                    }
                    crate::components::TabVisualState::Attention => {
                        bbr_testkit::protocol::TabVisualState::Attention
                    }
                    crate::components::TabVisualState::Error => {
                        bbr_testkit::protocol::TabVisualState::Error
                    }
                    crate::components::TabVisualState::Disabled => {
                        bbr_testkit::protocol::TabVisualState::Disabled
                    }
                    crate::components::TabVisualState::Dragging => {
                        bbr_testkit::protocol::TabVisualState::Dragging
                    }
                    crate::components::TabVisualState::DragTarget => {
                        bbr_testkit::protocol::TabVisualState::DragTarget
                    }
                    crate::components::TabVisualState::Focused => {
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
            drag_space,
            drag_valid: self.tab_drag.valid(),
            space_states: crate::shell::space_states(
                &browser,
                drag_space.map(|s| bbr_browser::SpaceId(s.0)),
            )
            .into_iter()
            .map(|state| match state {
                crate::components::SpaceVisualState::Default => {
                    bbr_testkit::protocol::SpaceVisualState::Default
                }
                crate::components::SpaceVisualState::Selected => {
                    bbr_testkit::protocol::SpaceVisualState::Selected
                }
                crate::components::SpaceVisualState::DragTarget => {
                    bbr_testkit::protocol::SpaceVisualState::DragTarget
                }
            })
            .collect(),
            space_titles: crate::shell::space_titles(&browser, budget),
            visible_ids: crate::shell::snapshot::visible_ids_u2(
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
                self.dense_window.get(),
                pane_layout,
                &crate::shell::snapshot::U2IdsInput {
                    spaces_visible: self.u2.spaces_visible,
                    favorites: &organization.favorites,
                    folders: &organization.folders,
                    expanded: self.u2.folder_presentation.clone(),
                    plus_menu: self.u2.plus_menu,
                    downloads_open: self.downloads.open,
                    downloads_window: self
                        .downloads
                        .window_for(self.downloads.render_state.scroll_anchor),
                    downloads_layout: self.downloads_layout(),
                },
            ),
            semantics,
            command_bar_mode: self.command_bar_mode.into(),
            command_bar_text: self.command_text.text.clone(),
            command_surface: self.command_surface.into(),
            command_result_state: command_result_state.into(),
            command_results,
            command_cursor: self.command_text.cursor,
            command_selection: self.command_text.selection.map(|(start, end)| [start, end]),
            sidebar_visible: self.sidebar_visible,
            sidebar_width,
            sidebar_min: crate::sidebar::SIDEBAR_MIN,
            sidebar_default: crate::sidebar::SIDEBAR_DEFAULT,
            sidebar_max: crate::sidebar::SIDEBAR_MAX,
            content_min: crate::sidebar::CONTENT_MIN,
            divider_dragging: self.sidebar.is_dragging(),
            divider_label: crate::sidebar::SIDEBAR_DIVIDER_LABEL.to_string(),
            content_rect: self.performance.content_rect().unwrap_or([0.0; 4]),
            pane_layout,
            pane_rects: pane_rect_snapshots,
            window_size: [window_width, window_height],
            portal: self.portal_snapshot(),
            find: self.find_snapshot(window),
            auth: self.auth_snapshot_protocol(),
            spaces_visible: self.u2.spaces_visible,
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
                    id: row.id,
                    title: row.title.clone(),
                    tab_count: row.count,
                    expanded: row.expanded,
                })
                .collect(),
            download_rows,
            downloads,
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
                    crate::app::downloads::DownloadMachine::Waiting => {
                        bbr_testkit::protocol::DownloadMachineState::Waiting
                    }
                    crate::app::downloads::DownloadMachine::Running => {
                        bbr_testkit::protocol::DownloadMachineState::Running
                    }
                    crate::app::downloads::DownloadMachine::Succeeded => {
                        bbr_testkit::protocol::DownloadMachineState::Succeeded
                    }
                    crate::app::downloads::DownloadMachine::Failed => {
                        bbr_testkit::protocol::DownloadMachineState::Failed
                    }
                    crate::app::downloads::DownloadMachine::Cancelled => {
                        bbr_testkit::protocol::DownloadMachineState::Cancelled
                    }
                },
                group: match row.group {
                    crate::app::downloads::DownloadGroupKind::Active => {
                        bbr_testkit::protocol::DownloadGroup::Active
                    }
                    crate::app::downloads::DownloadGroupKind::History => {
                        bbr_testkit::protocol::DownloadGroup::History
                    }
                },
                filename: row.filename.clone(),
                progress_fraction: row.progress_fraction,
                visual_state: match row.visual {
                    crate::app::downloads::DownloadVisual::Default => {
                        bbr_testkit::protocol::TabVisualState::Default
                    }
                    crate::app::downloads::DownloadVisual::Hover => {
                        bbr_testkit::protocol::TabVisualState::Hover
                    }
                    crate::app::downloads::DownloadVisual::Pressed => {
                        bbr_testkit::protocol::TabVisualState::Active
                    }
                    crate::app::downloads::DownloadVisual::Focused => {
                        bbr_testkit::protocol::TabVisualState::Focused
                    }
                    crate::app::downloads::DownloadVisual::Disabled => {
                        bbr_testkit::protocol::TabVisualState::Disabled
                    }
                },
                action: match row.action {
                    crate::app::downloads::DownloadActionKind::Cancel => {
                        bbr_testkit::protocol::DownloadRowAction::Cancel
                    }
                    crate::app::downloads::DownloadActionKind::Retry => {
                        bbr_testkit::protocol::DownloadRowAction::Retry
                    }
                    crate::app::downloads::DownloadActionKind::Reveal => {
                        bbr_testkit::protocol::DownloadRowAction::Reveal
                    }
                    crate::app::downloads::DownloadActionKind::None => {
                        bbr_testkit::protocol::DownloadRowAction::None
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
            })
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
            max_visible: crate::app::downloads::DOWNLOADS_MAX_VISIBLE,
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
    /// space rows grouped as rendered (gated by the §11.4 spaces toggle),
    /// the U2 surfaces (footer, favorites grid, folders, plus menu), and
    /// the content-pane branch. The focus state comes from GPUI's real
    /// focus tree.
    #[allow(clippy::too_many_arguments)]
    fn semantic_snapshot(
        &self,
        browser: &bbr_browser::ControllerSnapshot,
        rows: &[crate::components::RowVisuals],
        window: &Window,
        sidebar_width: f32,
        budget: &usize,
        favorite_tiles: &[crate::components::FavoriteTileSpec],
        folder_rows: &[crate::components::FolderRowSpec],
        folder_members: &[crate::components::FolderMemberVisual],
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
        let rendered_rows: Vec<crate::components::RowVisuals> = rendered_indices
            .into_iter()
            .filter_map(|index| rows.get(index).cloned())
            .collect();
        let space_rows: Vec<crate::components::SpaceRowVisuals> =
            crate::shell::space_groups(browser, *budget)
                .into_iter()
                .flat_map(|group| group.rows.into_iter())
                .collect();
        let content = if browser.active_view.is_some() {
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
                spaces: self.space_list_handle.is_focused(window),
                tabs: self.tab_list_handle.is_focused(window),
                divider: self.divider_handle.is_focused(window),
                folders: self.u2.folders_handle.is_focused(window),
                plus_menu: self.u2.plus_menu_handle.is_focused(window),
            },
            command: crate::shell::semantics::SemanticCommand {
                mode: format!("{:?}", self.command_bar_mode).to_lowercase(),
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
            divider_label: crate::sidebar::SIDEBAR_DIVIDER_LABEL.to_string(),
            command_surface: self.command_surface,
            rows: &rendered_rows,
            space_rows: &space_rows,
            spaces_visible: self.u2.spaces_visible,
            favorites: favorite_tiles,
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
