//! §18.5 favorite-added moment over the real app (task packet
//! `v2-motion-moments`). The toggle seam is the exact app method the
//! drag-into-grid drop commits through. Commit first: the new favorite and
//! its `bbr-fav-<n>` tile are in the same snapshot whose motion evidence
//! records the pop starting on that tile — full motion pops with specks
//! (`animated`), Reduce Motion flashes `tint.accent` (`reduced`).

use super::*;
use bbr_testkit::protocol::{MotionMomentSnapshot, Snapshot};

/// The persistent id of the active space's tab at `index`.
fn tab_id(snapshot: &Snapshot, index: usize) -> u64 {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.space == snapshot.active_space && tab.index == index)
        .expect("active-space tab")
        .id
}

/// The newest `favorite-pop` start, if any.
fn last_pop(snapshot: &Snapshot) -> Option<MotionMomentSnapshot> {
    snapshot
        .motion
        .started
        .iter()
        .rev()
        .find(|start| start.moment == "favorite-pop")
        .cloned()
}

#[test]
#[ignore]
fn a_new_favorite_pops_after_it_is_committed() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-v2-favorite-pop", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let initial = client.snapshot().expect("initial snapshot");
        let view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        let _second = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion");

        // Full motion: the pop starts on the committed tile.
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("favorite tab 0");
        let added = client.snapshot().expect("favorite added snapshot");
        assert_eq!(added.favorites.len(), 1, "the favorite is committed first");
        assert_eq!(added.favorites[0].tab, tab_id(&added, 0));
        assert_ids(&added, &["bbr-fav-0"]);
        let pop = last_pop(&added).expect("the pop started");
        assert_eq!(pop.target, "bbr-fav-0");
        assert_eq!(pop.variant, "animated");

        // Removing a favorite never pops.
        let starts = added.motion.started.len();
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("unfavorite tab 0");
        let removed = client.snapshot().expect("favorite removed snapshot");
        assert!(removed.favorites.is_empty());
        assert_eq!(
            removed.motion.started.len(),
            starts,
            "removal starts nothing"
        );

        // Reduce Motion: the `tint.accent` flash instead of pop + specks.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("reduce motion");
        client
            .send_ok(&dsl::toggle_favorite_tab(1))
            .expect("favorite tab 1");
        let reduced = client.snapshot().expect("reduced favorite snapshot");
        assert_eq!(reduced.favorites.len(), 1);
        assert_eq!(reduced.favorites[0].tab, tab_id(&reduced, 1));
        let flash = last_pop(&reduced).expect("the reduced start is recorded");
        assert_eq!(flash.target, "bbr-fav-0");
        assert_eq!(flash.variant, "reduced");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
