//! §18.5 download-finished moment over a real WKWebView download (task
//! packet `v2-motion-moments`). Commit first: the row is `Succeeded` in the
//! same snapshot whose motion evidence records the check starting on that
//! row's status slot — full motion draws the stroke and ring (`animated`),
//! Reduce Motion only shows the check (`static`).

use bbr_testkit::protocol::{DownloadMachineState, MotionMomentSnapshot};

use crate::e2e_common::*;
use crate::support::*;

/// The newest `download-check` start, if any.
fn last_check(snapshot: &bbr_testkit::protocol::Snapshot) -> Option<MotionMomentSnapshot> {
    snapshot
        .motion
        .started
        .iter()
        .rev()
        .find(|start| start.moment == "download-check")
        .cloned()
}

#[test]
#[ignore]
fn a_finished_download_draws_its_check_after_the_success_commits() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-v2-check", &page_one);
    let zero_url = fixtures.url("/__fx/dl/zero.bin");
    let known_url = fixtures.url("/__fx/dl/known.bin");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion");
        open_downloads(&mut client);

        // Full motion.
        let id = start_download(&mut client, &fixtures, &zero_url, "zero.bin");
        let row = wait_row_terminal(&mut client, id, DownloadMachineState::Succeeded);
        let done = client.snapshot().expect("succeeded snapshot");
        // Commit first: the success is in the model.
        assert_eq!(
            row_by_id(&done, id).machine_state,
            DownloadMachineState::Succeeded
        );
        let check = last_check(&done).expect("the check moment started");
        assert_eq!(check.target, format!("bbr-download-status-{}", row.row));
        assert_eq!(check.variant, "animated");

        // Reduce Motion: the check simply appears.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("reduce motion");
        let id = start_download(&mut client, &fixtures, &known_url, "known.bin");
        let row = wait_row_terminal(&mut client, id, DownloadMachineState::Succeeded);
        let done = client.snapshot().expect("reduced succeeded snapshot");
        assert_eq!(
            row_by_id(&done, id).machine_state,
            DownloadMachineState::Succeeded
        );
        let check = last_check(&done).expect("the reduced start is recorded");
        assert_eq!(check.target, format!("bbr-download-status-{}", row.row));
        assert_eq!(check.variant, "static");
        assert!(
            !done
                .motion
                .moments
                .iter()
                .any(|moment| moment.moment == "download-check"),
            "Reduce Motion runs no check animation"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
