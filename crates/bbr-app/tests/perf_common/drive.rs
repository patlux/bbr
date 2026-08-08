//! The warm interaction matrix of a cold benchmark trial.
//!
//! After the startup and page-ready milestones, a driving trial exercises the
//! production interaction paths: warm tab selection, sidebar
//! toggle/resize/restoration, New-Tab command staging/open/close, pointer
//! selection, reopen, keyboard activation, warm space switching, tab
//! organization, and the dense-list scroll/selection of the bounded-window
//! render (Phase-4 dense slice). Each measured mutation is driven
//! through the typed benchmark drivers in [`super::benchmarks`], which assert
//! the semantic state of every accepted sample; raw samples accumulate in the
//! harness log and are returned for aggregation.

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BenchmarkKind, BrowserChange, Request};
use bbr_testkit::Client;

use super::benchmarks::{
    benchmark_click, benchmark_dense_scroll, benchmark_dense_select, benchmark_hover,
    benchmark_keystroke, benchmark_move_tab, benchmark_pin_tab, benchmark_press, benchmark_reopen,
    benchmark_reorder_tab, benchmark_resize_sidebar, benchmark_restore_sidebar, benchmark_select,
    benchmark_sidebar, benchmark_space_switch, benchmark_space_switch_cross_profile,
    wait_browser_change, wait_title,
};
use super::drive::u2::{drive_u2_sidebar, reveal_spaces_region};
use super::find_drive::drive_find_scenarios;
mod u2;

use super::command_drive::drive_command_scenarios;
use super::downloads_drive::drive_downloads_scenarios;
use super::errors_drive::drive_error_surface_scenarios;
use super::harness::AppHarness;
use super::scenarios::{TrialConfig, PAGE_ONE_TITLE, PAGE_TWO_TITLE};

/// Drive the full interaction matrix and return the raw samples per scenario.
/// The `since` cursor is advanced past the last consumed event.
#[allow(clippy::too_many_arguments)]
pub fn drive_interaction_matrix(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    page_one: &str,
    page_two: &str,
    page_find: &str,
) -> BTreeMap<String, Vec<u64>> {
    let mut cursor = *since;
    let fixtures_base = page_find
        .rsplit_once('/')
        .map(|(base, _)| base.to_string())
        .unwrap_or_default();
    let raw = drive_matrix_inner(
        client,
        harness,
        &mut cursor,
        config,
        page_one,
        page_two,
        page_find,
        &fixtures_base,
    );
    *since = cursor;
    raw
}

#[allow(clippy::too_many_arguments)]
fn drive_matrix_inner(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    page_one: &str,
    page_two: &str,
    page_find: &str,
    fixtures_base: &str,
) -> BTreeMap<String, Vec<u64>> {
    let cursor = since;
    let mut since = *cursor;
    let raw = {
        let initial_snapshot = client.snapshot().expect("initial snapshot");
        let first_view = initial_snapshot.active_web_view.expect("first native view");
        if initial_snapshot.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(client, first_view, PAGE_ONE_TITLE, since);
        }

        client
            .send_ok(&Request::OpenTab {
                url: page_two.to_string(),
            })
            .expect("open second tab");
        let second = client.snapshot().expect("second tab snapshot");
        let second_view = second.active_web_view.expect("second native view");
        if second.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(client, second_view, PAGE_TWO_TITLE, since);
        }

        let mut operation = 2_u64;
        // One discarded warm-up round.
        benchmark_select(client, &mut since, operation, 0, first_view);
        operation += 1;

        let mut tab_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (index, expected) = if sample % 2 == 0 {
                (0, first_view)
            } else {
                (1, second_view)
            };
            let measured = benchmark_select(client, &mut since, operation, index, expected);
            tab_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("warm_tab_switch".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        benchmark_sidebar(client, &mut since, operation, false);
        operation += 1;
        benchmark_sidebar(client, &mut since, operation, true);
        operation += 1;

        let mut sidebar_samples = Vec::with_capacity(config.interaction_samples);
        let mut expected_sidebar = true;
        for _ in 0..config.interaction_samples {
            expected_sidebar = !expected_sidebar;
            let measured = benchmark_sidebar(client, &mut since, operation, expected_sidebar);
            sidebar_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("sidebar_toggle".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        // Sidebar resize + restoration: the toggle loop above ends with the
        // rail visible (30 flips return it to the opened state), so the
        // resize path runs directly. Alternate widths across the adjustable
        // range and measure each presentation commit at the exact clamped
        // geometry. Every sample asserts the engine-received content rect
        // against the typed min/default/max/content floor; socket time is
        // excluded and a fast-but-wrong geometry is rejected in the app
        // before the sample even leaves it.
        let resize_widths = [260.0, 300.0, 340.0, 360.0];
        let mut resize_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let width = resize_widths[sample % resize_widths.len()];
            let measured = benchmark_resize_sidebar(client, &mut since, operation, width);
            resize_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("sidebar_resize".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        let mut restore_samples = Vec::with_capacity(config.interaction_samples);
        for _ in 0..config.interaction_samples {
            let measured = benchmark_restore_sidebar(client, &mut since, operation);
            restore_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("sidebar_restore".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        let mut new_tab_command_samples = Vec::with_capacity(config.new_tab_samples);
        let mut new_tab_open_samples = Vec::with_capacity(config.new_tab_samples);
        let mut active_close_samples = Vec::with_capacity(config.new_tab_samples);
        for _ in 0..config.new_tab_samples {
            let before = client.snapshot().expect("before Cmd-T benchmark");
            let before_tabs = before.tabs.len();
            let before_sessions = before.engine_sessions;
            let previous_view = before.active_web_view.expect("active view before Cmd-T");

            let staged = benchmark_keystroke(
                client,
                &mut since,
                operation,
                BenchmarkKind::NewTabCommandBar,
                "cmd-t",
            );
            operation += 1;
            let staged_snapshot = client.snapshot().expect("staged Cmd-T snapshot");
            assert_eq!(staged.active_web_view, Some(previous_view));
            assert_eq!(
                staged.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::NewTab
            );
            assert!(staged.chrome_focused);
            assert_eq!(staged_snapshot.tabs.len(), before_tabs);
            assert_eq!(staged_snapshot.engine_sessions, before_sessions);
            assert_eq!(
                staged_snapshot.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::NewTab
            );
            new_tab_command_samples.push(staged.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("new_tab_command_bar".into())
                .or_default()
                .push(staged.duration_ns);

            client
                .send_ok(&Request::SetCommandBarText {
                    text: page_one.to_string(),
                })
                .expect("set staged destination");
            let opened = benchmark_keystroke(
                client,
                &mut since,
                operation,
                BenchmarkKind::NewTabOpened,
                "enter",
            );
            operation += 1;
            let opened_snapshot = client.snapshot().expect("opened tab snapshot");
            assert_eq!(opened_snapshot.tabs.len(), before_tabs + 1);
            assert_eq!(opened_snapshot.engine_sessions, before_sessions + 1);
            assert_eq!(
                opened_snapshot.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::Inactive
            );
            assert!(!opened_snapshot.chrome_focused);
            assert_ne!(opened.active_web_view, Some(previous_view));
            new_tab_open_samples.push(opened.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("new_tab_open".into())
                .or_default()
                .push(opened.duration_ns);

            let closed = benchmark_keystroke(
                client,
                &mut since,
                operation,
                BenchmarkKind::ActiveTabClose,
                "cmd-w",
            );
            operation += 1;
            let closed_snapshot = client.snapshot().expect("closed tab snapshot");
            assert_eq!(closed_snapshot.tabs.len(), before_tabs);
            assert_eq!(closed_snapshot.engine_sessions, before_sessions);
            assert_eq!(closed_snapshot.active_web_view, Some(previous_view));
            assert_eq!(closed.active_web_view, Some(previous_view));
            assert!(!closed.chrome_focused);
            assert_eq!(
                closed.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::Inactive
            );
            active_close_samples.push(closed.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("active_tab_close".into())
                .or_default()
                .push(closed.duration_ns);
        }

        // The new-tab cycle always ends back on two tabs with the second
        // (fixture two) tab active in web focus. Pointer selection measures
        // the production row-listener path (chrome focus preserved), then a
        // reopen cycle closes the active tab via real Cmd-W and measures the
        // fresh-session presentation of the reopened tab.
        let mut pointer_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (index, expected) = if sample % 2 == 0 {
                (0, first_view)
            } else {
                (1, second_view)
            };
            let measured = benchmark_click(client, &mut since, operation, index, expected);
            pointer_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_select_pointer".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        let mut reopen_samples = Vec::with_capacity(config.new_tab_samples);
        for _ in 0..config.new_tab_samples {
            // Unmeasured setup: close the active tab through the production
            // Cmd-W path (active-close has its own scenario above).
            client
                .send_ok(&Request::DispatchKeystroke {
                    keystroke: "cmd-w".into(),
                })
                .expect("close active tab before reopen benchmark");
            let closed = client.snapshot().expect("closed before reopen benchmark");
            assert_eq!(closed.tabs.len(), 1);
            assert_eq!(closed.engine_sessions, 1);
            let neighbor = closed.active_web_view.expect("neighbor view");
            let expected_domain = closed.chrome_focused;
            let measured =
                benchmark_reopen(client, &mut since, operation, neighbor, expected_domain);
            operation += 1;
            reopen_samples.push(measured.0.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_reopen".into())
                .or_default()
                .push(measured.0.duration_ns);
            let _ = measured.1; // fresh view id retained for diagnostics
        }

        // Keyboard activation: Enter on the focused tab list activates the
        // selected tab — the chrome traversal commits and keyboard focus
        // moves into the page (interaction contract "Enter activates"). The
        // unmeasured production pointer path re-focuses the list between
        // samples; only the Enter transition itself is sampled. The reopen
        // cycle recreated tab 1, so capture the live view ids after it.
        let after_reopen = client.snapshot().expect("state after reopen cycle");
        let tab0_view = after_reopen.tabs[0].web_view.expect("tab 0 view");
        let tab1_view = after_reopen.tabs[1].web_view.expect("tab 1 view");
        let mut activate_samples = Vec::with_capacity(config.new_tab_samples);
        for sample in 0..config.new_tab_samples {
            let (index, expected) = if sample % 2 == 0 {
                (0, tab0_view)
            } else {
                (1, tab1_view)
            };
            client
                .send_ok(&Request::ClickTab { index })
                .expect("focus tab list before Enter");
            let focused = client.snapshot().expect("tab list focused");
            assert!(focused.chrome_focused, "pointer click focuses the list");
            assert_eq!(focused.active_web_view, Some(expected));
            let measured = benchmark_keystroke(
                client,
                &mut since,
                operation,
                BenchmarkKind::TabActivateKeyboard,
                "enter",
            );
            operation += 1;
            assert_eq!(
                measured.active_web_view,
                Some(expected),
                "Enter keeps the selected tab's view"
            );
            assert!(
                !measured.chrome_focused,
                "Enter activates into the page (web focus)"
            );
            assert_eq!(
                measured.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::Inactive
            );
            let after = client.snapshot().expect("after Enter snapshot");
            assert_eq!(after.active_web_view, Some(expected));
            assert!(!after.chrome_focused);
            activate_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_activate_keyboard".into())
                .or_default()
                .push(measured.duration_ns);
        }

        // Command-surface scenarios (DESIGN.md §11.5): the production
        // command-text path (Cmd-L selects the active URL, typing replaces
        // the selection, the real ranking debounce publishes the current
        // generation), the Enter activation of the selected exact-tab
        // row, and the Cmd-Enter new-tab activation (session-recreation
        // class). The corpus is the authoritative controller snapshot built
        // on demand, so the exact-URL match always ranks `tab:<id>` first;
        // the in-app guards reject a wrong first key/generation/mode/focus/
        // destination and the drivers assert the measured state. The block
        // lives in the focused `super::command_drive` module (command
        // additions stay under the ~500-line budget); it runs between the
        // keyboard-activation and pointer-state blocks and ends restored to
        // the two original warm tabs (2 sessions, web focus) the
        // pointer-state block requires.
        let command_samples = drive_command_scenarios(
            client,
            harness,
            &mut since,
            config,
            page_one,
            page_two,
            tab0_view,
            tab1_view,
            &mut operation,
        );

        // Pointer-state transitions through the production hover/press
        // listener methods (§10.2 hover wash + close reveal, §10.3 active
        // wash). Transient chrome-pointer indication: never a selection
        // mutation and never touches sessions. The pointer state is cleared
        // between samples (unmeasured setup) so every sample measures the
        // entry transition into the wash, not a no-op stay. Hover targets a
        // row that is NOT the selected tab: the composed §10 state lets the
        // more specific `selected` win over `hover`, so hovering the active
        // row legitimately renders `selected` and the in-app guard would
        // reject it. Always target the original fixture-ready row 0 while
        // row 1 is selected: reopen presents its fresh WKWebView before its
        // deferred background navigation necessarily starts, and `loading`
        // correctly outranks `hover`. Alternating onto that row made the
        // benchmark depend on when WebKit chose to begin the late load.
        let mut hover_samples = Vec::with_capacity(config.interaction_samples);
        let mut press_samples = Vec::with_capacity(config.interaction_samples);
        for _ in 0..config.interaction_samples {
            let pointer_index = 0;
            client
                .send_ok(&Request::ClickTab { index: 1 })
                .expect("select the non-hovered row");
            client
                .send_ok(&Request::UnhoverTabRow)
                .expect("unhover before hover benchmark");
            client
                .send_ok(&Request::ReleaseTabRow)
                .expect("release before hover benchmark");
            let measured = benchmark_hover(client, &mut since, operation, pointer_index);
            hover_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_hover".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;

            client
                .send_ok(&Request::UnhoverTabRow)
                .expect("unhover before press benchmark");
            client
                .send_ok(&Request::ReleaseTabRow)
                .expect("release before press benchmark");
            let measured = benchmark_press(client, &mut since, operation, pointer_index);
            press_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_press".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        // §11.4 compact rail: reveal the spaces region before the
        // space-switch samples (see `drive::u2`).
        reveal_spaces_region(client);

        // Warm space switching: create a second space in the default profile
        // with its own warm tab, then measure alternating activation between
        // the two spaces through the production activation path (DESIGN.md
        // §11.3 / interaction contract "each space remembers its own active
        // tab"). Every measured presentation must show the destination
        // space's warm selected tab with the same native session (no
        // recreation), keep the chrome focus domain, and expose the
        // destination space row as selected (§11.3/§15).
        let before_space = client.snapshot().expect("state before space setup");
        let default_profile = before_space.active_profile;
        let default_space = before_space.active_space;
        let default_space_view = before_space.active_web_view.expect("default space view");
        client
            .send_ok(&Request::CreateSpace {
                profile: default_profile,
                title: "Benchmark".into(),
            })
            .expect("create second space");
        let (seq, created) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let benchmark_space = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        // Activate the empty space and warm its own tab through the
        // production controller path (unmeasured setup).
        client
            .send_ok(&Request::ActivateSpace {
                space: benchmark_space,
            })
            .expect("activate benchmark space");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == benchmark_space),
            since,
        );
        since = seq;
        client
            .send_ok(&Request::OpenTab {
                url: page_two.to_string(),
            })
            .expect("open benchmark space tab");
        let second_space = client.snapshot().expect("benchmark space tab snapshot");
        let benchmark_space_view = second_space.active_web_view.expect("benchmark space view");
        if second_space.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(client, benchmark_space_view, PAGE_TWO_TITLE, since);
        }
        let expected_sessions = second_space.engine_sessions;
        client
            .send_ok(&Request::ActivateSpace {
                space: default_space,
            })
            .expect("back to default space");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == default_space),
            since,
        );
        since = seq;

        let mut space_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (space, expected) = if sample % 2 == 0 {
                (default_space, default_space_view)
            } else {
                (benchmark_space, benchmark_space_view)
            };
            let measured = benchmark_space_switch(
                client,
                &mut since,
                operation,
                space,
                expected,
                expected_sessions,
            );
            space_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("space_switch".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        // Cross-profile view-only space switching through the production
        // activation path (DESIGN.md §11.3 cross-context rule): the
        // destination profile appears at the top of the command field for
        // one interaction. The switch is still a warm view-only presentation
        // — the live-session count never changes — so it stays in the
        // 50/100 ms warm class; the in-app probe rejects any presentation
        // that misses or invents the hint, and the driver asserts the hint
        // copy in the accepted snapshot.
        client
            .send_ok(&Request::CreateProfile {
                profile_title: "Benchmark P".into(),
                space_title: "Benchmark P".into(),
            })
            .expect("create cross-profile benchmark space");
        let (seq, created) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let cross_space = match created {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&Request::ActivateSpace { space: cross_space })
            .expect("activate cross space");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == cross_space),
            since,
        );
        since = seq;
        client
            .send_ok(&Request::OpenTab {
                url: page_two.to_string(),
            })
            .expect("warm the cross space");
        let cross_seeded = client.snapshot().expect("cross space seeded");
        let cross_space_view = cross_seeded.active_web_view.expect("cross space view");
        if cross_seeded.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(client, cross_space_view, PAGE_TWO_TITLE, since);
        }
        let sessions_cross = cross_seeded.engine_sessions;
        client
            .send_ok(&Request::ActivateSpace {
                space: default_space,
            })
            .expect("back to default before cross switches");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == default_space),
            since,
        );
        since = seq;

        let mut cross_space_samples = Vec::with_capacity(config.interaction_samples);
        // The loop leaves the app on `default_space` before the first sample,
        // so the even samples start from it: sample 0 activates the cross
        // profile (Default → Benchmark P) and every following activation
        // alternates across the boundary, so each measured sample is a
        // genuine cross-profile activation that must present the destination
        // hint. (Starting on the same space would be a same-profile
        // re-activation that clears the hint and gets rejected by the
        // fast-but-wrong guard.)
        for sample in 0..config.interaction_samples {
            let (space, expected, hint) = if sample % 2 == 0 {
                (
                    cross_space,
                    cross_space_view,
                    Some("in Profile · Benchmark P"),
                )
            } else {
                (
                    default_space,
                    default_space_view,
                    Some("in Profile · Default"),
                )
            };
            let measured = benchmark_space_switch_cross_profile(
                client,
                &mut since,
                operation,
                space,
                expected,
                sessions_cross,
                hint,
            );
            cross_space_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("space_switch_cross_profile".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        // Tab organization through the production command paths: the pin
        // toggle (`Cmd-Shift-P`), region-bounded reorder (`Cmd-Shift-↑/↓`),
        // and moves between spaces (staged `Cmd-Shift-M` + Enter). The active
        // tab and its live session stay put through pin and reorder; a
        // same-profile move carries the session to the destination space;
        // a cross-profile move destroys and recreates it exactly once in the
        // destination store (the session-recreation class). The source space
        // stays active and its selection is repaired to the deterministic
        // neighbor, which is the presentation every move sample commits
        // against.
        client
            .send_ok(&Request::ActivateSpace {
                space: default_space,
            })
            .expect("activate default before organize");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == default_space),
            since,
        );
        since = seq;
        // A third default tab gives the active row a same-region neighbor in
        // both directions; the middle row is then selected through the
        // production pointer path (which also re-focuses the chrome list).
        client
            .send_ok(&Request::OpenTab {
                url: page_two.to_string(),
            })
            .expect("open third default tab");
        let third = client.snapshot().expect("third tab snapshot");
        let third_view = third.active_web_view.expect("third view");
        if third.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(client, third_view, PAGE_TWO_TITLE, since);
        }
        client
            .send_ok(&Request::ClickTab { index: 1 })
            .expect("select the middle row");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::TabSelected { index } if *index == 1),
            since,
        );
        since = seq;
        let organize_view = client.snapshot().expect("organize state").tabs[1]
            .web_view
            .expect("middle view");

        let mut pin_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let expected_pinned = if sample % 2 == 0 { 1 } else { 0 };
            let measured = benchmark_pin_tab(
                client,
                &mut since,
                operation,
                organize_view,
                expected_pinned,
            );
            pin_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_pin".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        let mut reorder_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let delta = if sample % 2 == 0 { -1 } else { 1 };
            let measured =
                benchmark_reorder_tab(client, &mut since, operation, delta, organize_view, 0);
            reorder_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_reorder".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        // Same-profile move: a dedicated space with its own warm tab; the
        // active tab moves into it and back, and the live WebViewId follows
        // it across the boundary. The destination activation after each
        // measured move asserts the session identity directly.
        client
            .send_ok(&Request::CreateSpace {
                profile: default_profile,
                title: "Organize A".into(),
            })
            .expect("create same-profile move space");
        let (seq, created) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let organize_space = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&Request::ActivateSpace {
                space: organize_space,
            })
            .expect("activate organize space");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == organize_space),
            since,
        );
        since = seq;
        client
            .send_ok(&Request::OpenTab {
                url: page_two.to_string(),
            })
            .expect("warm the organize space");
        let organize_seeded = client.snapshot().expect("organize space seeded");
        let organize_neighbor = organize_seeded
            .active_web_view
            .expect("organize neighbor view");
        if organize_seeded.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(client, organize_neighbor, PAGE_TWO_TITLE, since);
        }
        client
            .send_ok(&Request::ActivateSpace {
                space: default_space,
            })
            .expect("back to default before same-profile moves");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == default_space),
            since,
        );
        since = seq;

        let mut same_move_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (source, dest) = if sample % 2 == 0 {
                (default_space, organize_space)
            } else {
                (organize_space, default_space)
            };
            // Setup: the source space is active with the moved tab selected
            // (the destination activation from the previous sample restored
            // it there), so the repair view is the source's deterministic
            // neighbor.
            client
                .send_ok(&Request::ActivateSpace { space: source })
                .expect("activate move source");
            let (seq, _) = wait_browser_change(
                client,
                |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == source),
                since,
            );
            since = seq;
            let before = client.snapshot().expect("pre move snapshot");
            let active = before.active_tab_index.expect("active tab");
            let moved_view = before.active_web_view.expect("moved tab view");
            let repair = before
                .tabs
                .get(active + 1)
                .or_else(|| before.tabs.get(active.wrapping_sub(1)))
                .and_then(|t| t.web_view)
                .expect("source keeps a neighbor view");
            let measured = benchmark_move_tab(client, &mut since, operation, dest, repair, false);
            same_move_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_move_same_profile".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
            // Session identity: the moved tab is the destination's active tab
            // with the same WebViewId (same-profile moves keep the session).
            client
                .send_ok(&Request::ActivateSpace { space: dest })
                .expect("activate move destination");
            let (seq, _) = wait_browser_change(
                client,
                |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == dest),
                since,
            );
            since = seq;
            let after = client.snapshot().expect("post move destination");
            assert_eq!(
                after.active_web_view,
                Some(moved_view),
                "same-profile move keeps the WebViewId"
            );
        }

        // Cross-profile move: a private profile + space; each move destroys
        // the old session and recreates it exactly once in the destination
        // store (the wire event asserts `recreated: true`, the net session
        // count stays, and the destination activation proves the fresh view).
        client
            .send_ok(&Request::CreateProfile {
                profile_title: "Organize P".into(),
                space_title: "Organize P".into(),
            })
            .expect("create private move profile");
        let (seq, created) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let private_space = match created {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&Request::ActivateSpace {
                space: private_space,
            })
            .expect("activate private space");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == private_space),
            since,
        );
        since = seq;
        client
            .send_ok(&Request::OpenTab {
                url: page_two.to_string(),
            })
            .expect("warm the private space");
        let private_seeded = client.snapshot().expect("private space seeded");
        let private_neighbor = private_seeded
            .active_web_view
            .expect("private neighbor view");
        if private_seeded.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(client, private_neighbor, PAGE_TWO_TITLE, since);
        }
        client
            .send_ok(&Request::ActivateSpace {
                space: default_space,
            })
            .expect("back to default before cross moves");
        let (seq, _) = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == default_space),
            since,
        );
        since = seq;
        let sessions_before_cross = client
            .snapshot()
            .expect("cross session count")
            .engine_sessions;

        let mut cross_move_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (source, dest) = if sample % 2 == 0 {
                (default_space, private_space)
            } else {
                (private_space, default_space)
            };
            client
                .send_ok(&Request::ActivateSpace { space: source })
                .expect("activate cross source");
            let (seq, _) = wait_browser_change(
                client,
                |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == source),
                since,
            );
            since = seq;
            let before = client.snapshot().expect("pre cross move snapshot");
            let active = before.active_tab_index.expect("active tab");
            let moved_view = before.active_web_view.expect("moved tab view");
            let repair = before
                .tabs
                .get(active + 1)
                .or_else(|| before.tabs.get(active.wrapping_sub(1)))
                .and_then(|t| t.web_view)
                .expect("source keeps a neighbor view");
            let measured = benchmark_move_tab(client, &mut since, operation, dest, repair, true);
            cross_move_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_move_cross_profile".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
            assert_eq!(
                client
                    .snapshot()
                    .expect("cross session count after")
                    .engine_sessions,
                sessions_before_cross,
                "destroy + recreate exactly once keeps the session count"
            );
            // Session identity: the moved tab is the destination's active tab
            // with a fresh incarnation (the cross-profile recreation policy).
            client
                .send_ok(&Request::ActivateSpace { space: dest })
                .expect("activate cross destination");
            let (seq, _) = wait_browser_change(
                client,
                |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == dest),
                since,
            );
            since = seq;
            let after = client.snapshot().expect("post cross destination");
            assert_ne!(
                after.active_web_view,
                Some(moved_view),
                "cross-profile move recreates the view"
            );
        }

        // U2 sidebar surfaces: plus menu, favorites, folder disclosure
        // (see `drive::u2`) — each a warm next-frame presentation commit
        // through the production paths, asserted by the in-app probe guard.
        let (plus_menu_samples, favorites_samples, folder_toggle_samples) = drive_u2_sidebar(
            client,
            harness,
            &mut since,
            &mut operation,
            config.interaction_samples,
        );

        // Find-in-page (task packet `find-in-page-portal-bar`): the
        // production Cmd-F open, the query path (WebKit computes count /
        // ordinal through the portable contract), and warm next/previous
        // stepping — each measured at the exact WebKit-reported presentation
        // the in-app probe requires, on a deterministic multi-match fixture
        // loaded before any sample (fixture/network time excluded). The
        // block restores the matrix state before the dense seeding below.
        let find_samples = drive_find_scenarios(
            client,
            harness,
            &mut since,
            config,
            page_find,
            &mut operation,
        );

        // Downloads sidebar panel (task packet `downloads-sidebar-panel`):
        // the production footer `[Downloads]` open, the L6 progress commit,
        // and the retry row action — each measured at the render of the
        // anchored panel that carried the armed identity (the in-app probe
        // owns the fast-but-wrong guard). Fixture/network time is excluded:
        // the history is seeded with real deterministic fixture downloads
        // before any sample, and the progress/retry samples apply
        // deterministic L6 events in-process (never through a network
        // transfer). The block never touches tabs/sessions/views and
        // restores the matrix state (panel closed) before the dense
        // seeding below.
        let downloads_samples = drive_downloads_scenarios(
            client,
            harness,
            &mut since,
            config,
            fixtures_base,
            &mut operation,
        );

        // Web-error surface (task packet `error-surface-contract`): the
        // production incident-presentation through the exact seam the
        // native delegate's incident sink drives (`on_incident` → the
        // contract `present` lifecycle), committing only from a render of
        // the content area that drew the surface for the armed navigation
        // category over the unchanged active view (the in-app probe owns
        // the fast-but-wrong guard). Fixture/network time is excluded:
        // the classified incident is applied in-process, never through a
        // network transfer. The block never touches tabs/sessions/views
        // and restores the matrix state (surface dismissed) before the
        // dense seeding below.
        let error_surface_samples =
            drive_error_surface_scenarios(client, harness, &mut since, config, &mut operation);

        // Dense-list scroll + selection through the bounded-window render
        // (Phase-4 dense slice, performance contract "dense-list scrolling"):
        // seed an S-density collection (230 identities, 10 pinned, 20 live)
        // through the production controller open path with a deterministic
        // keep set, converge the session budget, then measure the production
        // scroll state (alternating deltas keep the anchor moving mid-list
        // so no sample hits the clamped edges) and warm selections (pinned
        // tab 0 plus one live recent tab — never a recreation). The 500-tab /
        // 50-session lifecycle contract stays with the functional dense E2E;
        // this scenario proves the bounded-window frame path stays inside the
        // warm budgets while the list is genuinely dense.
        let dense_recent_start = 230 - (20 - 10);
        client
            .send_ok(&Request::SeedDenseState {
                tab_count: 230,
                pinned_count: 10,
                recent_start: dense_recent_start,
                url: page_two.to_string(),
            })
            .expect("seed dense list");
        client
            .send_ok(&Request::EnforceSessionBudget { cap: 20 })
            .expect("enforce dense session budget");
        // Draw a frame so the render records the dense bounded window (the
        // seed/budget requests are model mutations; `dense` snapshot fields
        // mirror the last rendered frame, like the functional dense E2E).
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush dense seed frame");
        let dense_seeded = client.snapshot().expect("dense seeded snapshot");
        assert_eq!(
            dense_seeded.shell_modes.density, "small",
            "the dense collection flips to S density"
        );
        assert_eq!(dense_seeded.dense.item_count, dense_seeded.tabs.len() - 10);
        assert_eq!(dense_seeded.engine_sessions, 20);
        assert!(
            dense_seeded.dense.rendered_rows <= 64,
            "dense render stays bounded after seeding"
        );
        // A live warm regular row (the recency keep set) is the warm-select
        // target; pinned tab 0 is the always-on-screen target.
        let dense_warm_index = dense_seeded
            .tabs
            .iter()
            .position(|tab| tab.lifecycle == "warm")
            .expect("dense keep set owns warm sessions");
        let dense_warm_view = dense_seeded.tabs[dense_warm_index]
            .web_view
            .expect("warm dense view");
        let dense_pinned_view = dense_seeded.tabs[0].web_view.expect("pinned dense view");
        let dense_sessions = dense_seeded.engine_sessions;

        // Unmeasured setup: scroll to the top so the alternating deltas move
        // the anchor mid-list (a down-scroll from the bottom clamp would be a
        // no-op and cancel the first sample).
        client
            .send_ok(&Request::ScrollDenseTabs {
                delta_px: 1_000_000.0,
            })
            .expect("dense scroll to top");
        let mut dense_scroll_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let delta_px = if sample % 2 == 0 { -2500.0 } else { 2500.0 };
            let measured = benchmark_dense_scroll(client, &mut since, operation, delta_px);
            dense_scroll_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("dense_scroll".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        let mut dense_select_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (index, expected_view) = if sample % 2 == 0 {
                (0, dense_pinned_view)
            } else {
                (dense_warm_index, dense_warm_view)
            };
            let measured = benchmark_dense_select(
                client,
                &mut since,
                operation,
                index,
                expected_view,
                dense_sessions,
            );
            dense_select_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("dense_select".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        let mut raw = BTreeMap::new();
        raw.insert("warm_tab_switch".to_string(), tab_samples);
        raw.insert("tab_select_pointer".to_string(), pointer_samples);
        raw.insert("tab_activate_keyboard".to_string(), activate_samples);
        raw.insert("tab_reopen".to_string(), reopen_samples);
        raw.insert("sidebar_toggle".to_string(), sidebar_samples);
        raw.insert("sidebar_resize".to_string(), resize_samples);
        raw.insert("sidebar_restore".to_string(), restore_samples);
        raw.insert("new_tab_command_bar".to_string(), new_tab_command_samples);
        raw.insert("new_tab_open".to_string(), new_tab_open_samples);
        raw.insert("active_tab_close".to_string(), active_close_samples);
        raw.insert("space_switch".to_string(), space_samples);
        raw.insert(
            "space_switch_cross_profile".to_string(),
            cross_space_samples,
        );
        raw.insert("tab_pin".to_string(), pin_samples);
        raw.insert("tab_reorder".to_string(), reorder_samples);
        raw.insert("tab_hover".to_string(), hover_samples);
        raw.insert("tab_press".to_string(), press_samples);
        raw.extend(command_samples);
        raw.insert("tab_move_same_profile".to_string(), same_move_samples);
        raw.insert("tab_move_cross_profile".to_string(), cross_move_samples);
        raw.insert("sidebar_plus_menu".to_string(), plus_menu_samples);
        raw.insert("sidebar_favorites".to_string(), favorites_samples);
        raw.insert("sidebar_folder_toggle".to_string(), folder_toggle_samples);
        raw.extend(find_samples);
        raw.extend(downloads_samples);
        raw.extend(error_surface_samples);
        raw.insert("dense_scroll".to_string(), dense_scroll_samples);
        raw.insert("dense_select".to_string(), dense_select_samples);
        raw
    };
    *cursor = since;
    raw
}
