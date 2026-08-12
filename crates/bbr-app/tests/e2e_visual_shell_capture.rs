//! Manual/CI-artifact visual evidence for the reference-faithful production
//! shell. The ignored tests launch an externally supplied *optimized* BBR
//! binary with the explicit `visual-capture` feature, populate only through
//! existing production
//! requests, capture the exact native parent-window id, and quit gracefully.
//!
//! This is not a pixel assertion and adds no production screenshot API. Run
//! under the repository GUI lock with `BBR_VISUAL_CAPTURE_BIN` and
//! `BBR_VISUAL_CAPTURE_DIR` set by the bounded capture recipe.

#![cfg(all(target_os = "macos", feature = "visual-capture"))]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

use bbr_testkit::protocol::{dsl, Request, Snapshot};
use e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, open_tab,
    wait_for_window_settled, AssertUnwindSafe, Client, FixtureServer, Harness, Instant,
    PAGE_ONE_TITLE, PAGE_TWO_TITLE,
};
use std::fs;
use std::path::{Path, PathBuf};

const CAPTURE_WIDTH_PT: f32 = 1080.0;
const CAPTURE_HEIGHT_PT: f32 = 1407.0;
const SIDEBAR_CROP_WIDTH_PT: f64 = 304.0;

#[test]
#[ignore = "visual evidence: requires optimized app binary and an unlocked macOS GUI"]
fn representative_production_path_capture() {
    capture(CaptureKind::Representative);
}

#[test]
#[ignore = "visual evidence: requires optimized app binary and an unlocked macOS GUI"]
fn ordinary_current_data_capture() {
    capture(CaptureKind::Ordinary);
}

#[derive(Clone, Copy)]
enum CaptureKind {
    Representative,
    Ordinary,
}

impl CaptureKind {
    fn name(self) -> &'static str {
        match self {
            Self::Representative => "representative",
            Self::Ordinary => "ordinary",
        }
    }
}

fn capture(kind: CaptureKind) {
    let _serial = e2e_common::serial_gui();
    let root_tag = format!("visual-{}", kind.name());
    let root = harness_root(&root_tag, std::process::id());
    let profile = root.join("profile");
    fs::create_dir_all(&profile).expect("create capture profile");
    let socket = harness_socket_path(&root_tag, std::process::id());
    let token = format!("visual-{}-{}", kind.name(), std::process::id());
    let output_dir = PathBuf::from(
        std::env::var_os("BBR_VISUAL_CAPTURE_DIR").expect("BBR_VISUAL_CAPTURE_DIR is required"),
    );
    fs::create_dir_all(&output_dir).expect("create visual capture output dir");
    let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").expect("BBR_VISUAL_CAPTURE_BIN is required");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");
    let separator = fixtures.url("/separator-title.html");
    let long_title = fixtures.url("/long-title.html");

    let mut harness = Harness::spawn(
        &bin,
        &[
            "--bbr-initial-url",
            &page_one,
            "--bbr-profile-dir",
            profile.to_str().expect("profile path"),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
            &format!("--bbr-window-size={CAPTURE_WIDTH_PT}x{CAPTURE_HEIGHT_PT}"),
        ],
        &root,
    );

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let mut since = client
            .attach(None)
            .expect("attach")
            .last()
            .map_or(0, |v| v.0);
        // The capture is the reference-faithful **light** shell evidence
        // (task packet `system-appearance-theme`): force the light rail
        // regardless of the host appearance so the captured state matches
        // the integrated VS-REFERENCE evidence exactly.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("force light capture");
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush forced light capture");
        let initial = client.snapshot().expect("initial snapshot");
        let initial_view = initial.active_web_view.expect("initial view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = e2e_common::wait_title(&mut client, initial_view, PAGE_ONE_TITLE, since);
        }

        if matches!(kind, CaptureKind::Representative) {
            populate_representative(&mut client, &mut since, &page_two, &separator, &long_title);
        }

        client
            .send_ok(&Request::SetWindowSize {
                width: CAPTURE_WIDTH_PT,
                height: CAPTURE_HEIGHT_PT,
            })
            .expect("set capture window size");
        client
            .send_ok(&Request::FocusWeb)
            .expect("settle capture in web-focus rest state");
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush capture frame");
        let snapshot = wait_for_window_settled(&mut client, deadline());
        assert_capture_state(kind, &snapshot);
        let window_number = snapshot.portal.parent_window_number;
        assert!(window_number > 0, "native parent window number is required");

        let metadata_path = output_dir.join(format!("{}.snapshot.json", kind.name()));
        fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&snapshot).expect("serialize snapshot"),
        )
        .expect("write capture snapshot");

        let direct_capture = PathBuf::from(format!(
            "/tmp/bbr-visual-{}-{}.png",
            kind.name(),
            std::process::id()
        ));
        client
            .send_ok(&Request::CaptureChrome {
                path: direct_capture.to_string_lossy().into_owned(),
            })
            .expect("capture production GPUI frame");
        assert!(direct_capture.metadata().is_ok_and(|meta| meta.len() > 0));
        let full_path = output_dir.join(format!("{}.full.png", kind.name()));
        fs::rename(&direct_capture, &full_path).expect("move direct capture into artifact dir");
        let rail_path = output_dir.join(format!("{}.rail.png", kind.name()));
        crop_left_rail(&full_path, &rail_path, snapshot.window_size[0]);

        client.quit().expect("graceful visual capture quit");
        wait_for_process_exit(&mut harness, deadline());
    }));

    match result {
        Ok(()) => {
            harness.success = true;
            eprintln!(
                "VISUAL CAPTURE {} PASSED: {}",
                kind.name(),
                output_dir.display()
            );
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
}

fn populate_representative(
    client: &mut Client,
    since: &mut u64,
    page_two: &str,
    separator: &str,
    long_title: &str,
) {
    open_tab(client, since, page_two, Some(PAGE_TWO_TITLE));
    open_tab(client, since, separator, Some(e2e_common::SEPARATOR_TITLE));
    open_tab(client, since, long_title, Some(e2e_common::LONG_TITLE));

    for index in 0..4 {
        client
            .send_ok(&dsl::toggle_favorite_tab(index))
            .expect("favorite representative tab");
    }
    client
        .send_ok(&Request::PinTab { index: 0 })
        .expect("pin representative primary tab");
    client
        .send_ok(&dsl::click_footer_plus())
        .expect("open plus menu");
    client
        .send_ok(&dsl::click_plus_menu_item(1))
        .expect("create production folder");
    client
        .send_ok(&Request::FlushFrame)
        .expect("flush folder creation");
    client
        .send_ok(&Request::SelectTab { index: 0 })
        .expect("select representative primary tab");
    client
        .send_ok(&Request::FlushFrame)
        .expect("flush representative frame");
}

fn assert_capture_state(kind: CaptureKind, snapshot: &Snapshot) {
    assert_eq!(snapshot.shell_modes.theme, "light");
    assert!(snapshot.sidebar_visible);
    assert!(
        (snapshot.sidebar_width - 300.0).abs() <= 0.5,
        "normal rail width"
    );
    assert!(snapshot
        .visible_ids
        .iter()
        .any(|id| id == "bbr-active-space-title"));
    assert!(snapshot.visible_ids.iter().any(|id| id == "bbr-new-tab"));
    match kind {
        CaptureKind::Representative => {
            assert!(snapshot.tabs.len() >= 4, "representative real tabs");
            assert_eq!(snapshot.favorites.len(), 4, "representative real favorites");
            assert_eq!(snapshot.pinned_count, 1, "representative real pinned tab");
            assert_eq!(snapshot.active_tab_index, Some(0));
            assert_eq!(snapshot.folders.len(), 1, "representative real folder");
            assert_eq!(snapshot.folders[0].tab_count, 0, "truthful empty folder");
            assert!(!snapshot.folders[0].expanded);
        }
        CaptureKind::Ordinary => {
            assert_eq!(snapshot.tabs.len(), 1, "ordinary fresh-profile tab count");
            assert!(snapshot.favorites.is_empty());
            assert!(snapshot.folders.is_empty());
        }
    }
}

fn crop_left_rail(full_path: &Path, rail_path: &Path, window_width_points: f32) {
    let image = image::open(full_path).expect("open direct renderer capture");
    let scale = f64::from(image.width()) / f64::from(window_width_points);
    let crop_width = (SIDEBAR_CROP_WIDTH_PT * scale).round() as u32;
    image
        .crop_imm(0, 0, crop_width.min(image.width()), image.height())
        .save(rail_path)
        .expect("save left rail crop");
}

fn wait_for_process_exit(harness: &mut Harness, until: Instant) {
    loop {
        if let Some(status) = harness.child.try_wait().expect("try wait capture app") {
            assert!(status.success(), "capture app exit status: {status}");
            return;
        }
        assert!(Instant::now() < until, "capture app did not exit");
        std::thread::yield_now();
    }
}
