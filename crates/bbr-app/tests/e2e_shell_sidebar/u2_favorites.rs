use super::*;
use bbr_testkit::protocol::dsl;
/// §11.10 favorites over the real app: the grid is window-global — identical
/// tile order across same-profile and cross-profile space switches; each tile
/// carries the exact `TabId + SpaceId + ProfileId`; clicking a tile from
/// another profile switches inside the same window and selects the exact
/// existing tab/session (no copy, move, or recreation); closing the concrete
/// tab removes the tile; moving the tab makes the favorite follow its stable
/// `TabId` to the new space/profile.
#[test]
#[ignore]
fn favorites_grid_stays_global_and_activates_the_exact_tab_across_profiles() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-u2-favorites", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let default_profile = initial.active_profile;

        // Three tabs in the default space; favorite the middle one through
        // the production toggle path (the exact method the drag-into-grid
        // drop commits through).
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let _third_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        client
            .send_ok(&dsl::toggle_favorite_tab(1))
            .expect("favorite the middle tab");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after favorite");
        let with_favorite = client.snapshot().expect("favorite snapshot");
        assert_eq!(with_favorite.favorites.len(), 1, "one tile");
        let tile = &with_favorite.favorites[0];
        assert_eq!(tile.index, 0, "insertion order index");
        // The tile resolves the exact concrete tab + its canonical owner.
        let middle = with_favorite
            .all_tabs
            .iter()
            .find(|tab| tab.space == default_space && tab.index == 1)
            .expect("middle tab in global model");
        assert_eq!(tile.tab, middle.id, "the exact concrete TabId");
        assert_eq!(tile.space, default_space.0, "canonical owning space");
        assert_eq!(
            tile.profile, default_profile.0,
            "profile derived from the space"
        );
        assert!(!tile.letter.is_empty(), "§7.2 monogram fallback letter");
        assert_ids(
            &with_favorite,
            &[
                "bbr-section-favorites",
                "bbr-favorites",
                "bbr-fav-0",
                "bbr-fav-icon-0",
            ],
        );
        // §11.10 grid geometry: four columns at every rail width; the tile
        // width derives from the rail (no trailing slack), the height stays
        // fixed per density.
        let expected_tile = |rail: f32| (rail - 4.0 - 16.0 - 3.0 * 4.0) / 4.0;
        assert_eq!(with_favorite.favorites_columns, 4);
        assert_eq!(
            with_favorite.favorites_tile_size,
            [expected_tile(with_favorite.sidebar_width), 44.0]
        );
        for (delta, rail) in [(100.0, 400.0), (-200.0, 200.0), (100.0, 300.0)] {
            client
                .send_ok(&dsl::drag_sidebar_divider(delta))
                .expect("resize the rail");
            client
                .send_ok(&bbr_testkit::Request::FlushFrame)
                .expect("flush after resize");
            let resized = client.snapshot().expect("resized snapshot");
            assert_eq!(resized.sidebar_width, rail);
            assert_eq!(resized.favorites_columns, 4, "never a fifth column");
            assert_eq!(resized.favorites_tile_size, [expected_tile(rail), 44.0]);
        }
        // The favorited tab's identity is untouched.
        assert_eq!(with_favorite.tabs.len(), 3);
        assert_eq!(with_favorite.tabs[1].web_view, Some(second_view));
        assert_eq!(with_favorite.active_tab_index, Some(2));

        // Same-profile space switch: the grid keeps the exact same tile
        // order and owner.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create same-profile space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::click_space(reading))
            .expect("activate reading");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after same-profile switch");
        let same_profile = client.snapshot().expect("same-profile switch snapshot");
        assert_eq!(same_profile.favorites.len(), 1, "tiles never filtered");
        assert_eq!(
            same_profile.favorites[0], *tile,
            "identical tile order and owner across the same-profile switch"
        );

        // Cross-profile switch: a second profile's space keeps the same grid
        // unchanged (a profile is a container boundary, not a window).
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let (private_profile, private_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let _private_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let cross = client.snapshot().expect("cross-profile snapshot");
        assert_eq!(cross.active_profile, private_profile);
        assert_eq!(cross.favorites.len(), 1, "window-global grid unfiltered");
        assert_eq!(cross.favorites[0], *tile, "identical tile across profiles");

        // Click the tile from inside the private profile: the same window
        // switches to the owning space/container and selects the exact
        // existing tab/session — never a copy, move, or recreation.
        let sessions_before = cross.engine_sessions;
        let tabs_before = cross.all_tabs.len();
        client
            .send_ok(&dsl::click_favorite_tile(0))
            .expect("activate favorite tile");
        let activated = client.snapshot().expect("favorite activated snapshot");
        assert_eq!(activated.active_space, default_space, "same-window switch");
        assert_eq!(
            activated.active_profile, default_profile,
            "the owning profile's container is active"
        );
        assert_eq!(
            activated.active_tab_index,
            Some(1),
            "the exact favorited tab is selected"
        );
        assert_eq!(
            activated.active_web_view,
            Some(second_view),
            "the exact existing session is presented"
        );
        assert_eq!(
            activated.engine_sessions, sessions_before,
            "activation never recreates a session"
        );
        assert_eq!(
            activated.all_tabs.len(),
            tabs_before,
            "activation never copies or moves a tab"
        );
        assert!(activated.chrome_focused);
        assert_eq!(activated.favorites.len(), 1, "the tile stays");

        // Closing the concrete tab removes its tile.
        client
            .send_ok(&dsl::click_tab_close(1))
            .expect("close the favorited tab");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after closing favorite");
        let closed = client.snapshot().expect("favorite removed snapshot");
        assert!(
            closed.favorites.is_empty(),
            "closing the concrete tab removes the tile"
        );
        // Back to zero favorites: the grid goes, the drop band returns.
        assert!(!closed.visible_ids.iter().any(|id| id == "bbr-favorites"));
        assert_ids(
            &closed,
            &[
                "bbr-section-favorites",
                "bbr-favorites-empty",
                "bbr-favorites-empty-label",
            ],
        );

        // Re-favorite a tab, then move it across profiles: the favorite
        // follows the same stable `TabId` and resolves the new space/profile.
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("favorite the first tab");
        let refavorited = client.snapshot().expect("refavorited snapshot");
        let moved_tab_id = refavorited.favorites[0].tab;
        assert_eq!(refavorited.favorites[0].space, default_space.0);
        assert_eq!(refavorited.favorites[0].profile, default_profile.0);
        client
            .send_ok(&dsl::move_tab(0, private_space))
            .expect("move the favorited tab across profiles");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: private_space,
                recreated: true,
            },
            since,
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after cross-profile move");
        let moved = client.snapshot().expect("favorite followed the move");
        assert_eq!(moved.favorites.len(), 1, "the tile follows the moved tab");
        assert_eq!(
            moved.favorites[0].tab, moved_tab_id,
            "the same stable TabId"
        );
        assert_eq!(
            moved.favorites[0].space, private_space.0,
            "resolves the new space"
        );
        assert_eq!(
            moved.favorites[0].profile, private_profile.0,
            "resolves the new profile"
        );
        assert_ne!(moved.favorites[0].space, default_space.0);
        // The moved tab itself is intact (no duplicate was created).
        assert_eq!(
            moved
                .all_tabs
                .iter()
                .filter(|t| t.id == moved_tab_id)
                .count(),
            1,
            "exactly one incarnation of the moved tab"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
