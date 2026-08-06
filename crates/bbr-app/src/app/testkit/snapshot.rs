//! The semantic E2E snapshot of the root view (testkit feature).
//!
//! Maps the controller snapshot, theme tokens, drag state, and shell
//! presentation onto the typed protocol snapshot the E2E suites assert.

use gpui::Window;

use crate::shell::TestSnapshot;

use super::BbrApp;

/// A snapshot for the E2E protocol (also used by unit tests).
impl BbrApp {
    pub fn snapshot(&self, window: &Window) -> TestSnapshot {
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
        let semantics = self.semantic_snapshot(&browser, &rows, window, sidebar_width, &budget);
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
            visible_ids: crate::shell::visible_ids(
                &browser,
                self.chrome_focused(window),
                self.command_bar_mode,
                self.sidebar_visible,
                self.command_bar_hint().as_deref(),
                self.command_profile_hint.as_deref(),
                drag_slot,
                self.dense_window.get(),
                pane_layout,
            ),
            semantics,
            command_bar_mode: self.command_bar_mode.into(),
            command_bar_text: self.command_bar_text.clone(),
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
        }
    }

    /// The typed semantic tree of the current render (DESIGN.md §15 /
    /// accessibility track). Mirrors the shell: only rows the render
    /// actually built (the pinned block + the bounded regular window), the
    /// space rows grouped as rendered, and the content-pane branch. The
    /// focus state comes from GPUI's real focus tree.
    fn semantic_snapshot(
        &self,
        browser: &bbr_browser::ControllerSnapshot,
        rows: &[crate::components::RowVisuals],
        window: &Window,
        sidebar_width: f32,
        budget: &usize,
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
            },
            command: crate::shell::semantics::SemanticCommand {
                mode: format!("{:?}", self.command_bar_mode).to_lowercase(),
                text: self.command_bar_text.clone(),
                hint: self.command_bar_hint(),
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
            rows: &rendered_rows,
            space_rows: &space_rows,
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
