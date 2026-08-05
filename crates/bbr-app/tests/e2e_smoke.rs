//! Phase-1 smoke E2E: spawns the real `bbr` binary (with the `testkit`
//! feature), attaches to its gateway, and drives a real WKWebView through
//! startup + initial load + back/forward/reload + focus round-trip.
//!
//! Run with:
//!
//! ```text
//! cargo test -p bbr --features testkit --test e2e_smoke -- --ignored --nocapture
//! ```
//!
//! It is `#[ignore]`d by default so a plain `cargo test --workspace` does
//! not open a GUI window or require a window server. Set `BBR_SKIP_GUI_TESTS=1`
//! to skip it entirely on headless CI.

#![cfg(target_os = "macos")]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use bbr_fixtures::FixtureServer;
use bbr_testkit::protocol::{dsl, Event, Request, Response};
use bbr_testkit::Client;

/// Fixture page titles (deterministic content).
const PAGE_ONE_TITLE: &str = "BBR Fixture One";
const PAGE_TWO_TITLE: &str = "BBR Fixture Two";

struct Harness {
    child: Child,
    profile_dir: PathBuf,
    artifacts_dir: PathBuf,
}

impl Harness {
    fn token() -> String {
        // 16 random hex chars.
        let mut s = String::new();
        for _ in 0..4 {
            s.push_str(&format!("{:04x}", rand16()));
        }
        s
    }
}

fn rand16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    (nanos & 0xffff) as u16 ^ (nanos >> 32) as u16
}

fn skip_if_requested() -> bool {
    std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|v| v != "0" && v.to_lowercase() != "false")
}

#[test]
#[ignore]
fn smoke_startup_and_real_navigation() {
    if skip_if_requested() {
        eprintln!("BBR_SKIP_GUI_TESTS set; skipping GUI smoke test");
        return;
    }

    // 1. Deterministic local fixture server (in-process).
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("start fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");

    // 2. Temp profile + artifacts dirs.
    let root = std::env::temp_dir().join(format!("bbr-e2e-smoke-{}", std::process::id()));
    let profile_dir = root.join("profile");
    let artifacts_dir = root.join("artifacts");
    std::fs::create_dir_all(&profile_dir).unwrap();
    std::fs::create_dir_all(&artifacts_dir).unwrap();

    let token = Harness::token();
    let socket = profile_dir.join("bbr-test.sock");

    // 3. Spawn the app. The binary is built by cargo (testkit feature).
    let bin = env!("CARGO_BIN_EXE_bbr");
    let child = Command::new(bin)
        .args([
            "--bbr-initial-url",
            &page_one,
            "--bbr-profile-dir",
            profile_dir.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
            "--bbr-clock=system",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn bbr");

    let mut harness = Harness {
        child,
        profile_dir,
        artifacts_dir,
    };

    // 4. Connect and wait for AppReady (event-driven, no sleeps).
    let result = run_smoke(&mut harness, &socket, &token, &page_one, &page_two);
    if let Err(e) = result {
        harness.terminate();
        harness.dump_stderr();
        eprintln!(
            "fixture hits: page1={}, page2={}",
            fixtures.hits("/page1.html"),
            fixtures.hits("/page2.html")
        );
        eprintln!("SMOKE FAILED: {e}");
        panic!("smoke test failed");
    }
    harness.cleanup();
    eprintln!("SMOKE PASSED");
}

fn run_smoke(
    harness: &mut Harness,
    socket: &Path,
    token: &str,
    page_one: &str,
    page_two: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut client = loop {
        match Client::connect(socket) {
            Ok(c) => break c,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(format!("could not connect to gateway: {e}").into()),
        }
    };

    let welcome = client.hello(token)?;
    assert_eq!(welcome.app_version, env!("CARGO_PKG_VERSION"));
    let replay = client.attach(None)?;
    eprintln!("attach replay: {replay:#?}");

    // AppReady may already be in the attach replay because the gateway starts
    // before the harness connects. Only wait for a future event when replay
    // did not contain it.
    if !replay
        .iter()
        .any(|(_, event)| matches!(event, Event::AppReady { .. }))
    {
        client.wait_for_event(
            |e| matches!(e, Event::AppReady { .. }),
            Instant::now() + Duration::from_secs(20),
        )?;
    }

    eprintln!("snapshot after attach: {:#?}", client.snapshot()?);

    // 5. Initial load: events may already be in the attach replay when WebKit
    // finishes before the harness connects. Otherwise wait event-driven.
    if !replay.iter().any(|(_, event)| {
        matches!(event, Event::Web(bbr_web::WebEvent::TitleChanged(_, title)) if title == PAGE_ONE_TITLE)
    }) {
        client.wait_for_event(
            |e| matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_ONE_TITLE),
            Instant::now() + Duration::from_secs(20),
        )?;
    }
    if !replay.iter().any(|(_, event)| {
        matches!(
            event,
            Event::Web(bbr_web::WebEvent::LoadingChanged(_, false))
        )
    }) {
        client.wait_for_event(
            |e| matches!(e, Event::Web(bbr_web::WebEvent::LoadingChanged(_, false))),
            Instant::now() + Duration::from_secs(20),
        )?;
    }

    // Snapshot reflects the loaded page.
    let snap = client.snapshot()?;
    assert_eq!(snap.title.as_deref(), Some(PAGE_ONE_TITLE));
    assert_eq!(snap.url.as_deref(), Some(page_one));
    assert!(snap.web_view_hosted, "native WKWebView must be hosted");
    assert_eq!(snap.engine_kind, "native-wkwebview");

    // 6. Navigate to page_two via the engine action.
    client.send_ok(&dsl::navigate(page_two))?;
    client.wait_for_event(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_TWO_TITLE),
        Instant::now() + Duration::from_secs(20),
    )?;
    let snap = client.snapshot()?;
    assert_eq!(snap.title.as_deref(), Some(PAGE_TWO_TITLE));
    assert!(
        snap.can_go_back,
        "after navigating forward, back must be enabled"
    );

    // 7. Back → page_one.
    client.send_ok(&dsl::back())?;
    client.wait_for_event(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_ONE_TITLE),
        Instant::now() + Duration::from_secs(20),
    )?;
    let snap = client.snapshot()?;
    assert_eq!(snap.title.as_deref(), Some(PAGE_ONE_TITLE));
    assert!(
        snap.can_go_forward,
        "after going back, forward must be enabled"
    );

    // 8. Forward → page_two again.
    client.send_ok(&dsl::forward())?;
    client.wait_for_event(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_TWO_TITLE),
        Instant::now() + Duration::from_secs(20),
    )?;

    // 9. Reload → observe a fresh load cycle.
    client.send_ok(&dsl::reload())?;
    client.wait_for_event(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::LoadingChanged(_, true))),
        Instant::now() + Duration::from_secs(20),
    )?;
    let _ = client.wait_for_event(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::LoadingChanged(_, false))),
        Instant::now() + Duration::from_secs(20),
    )?;

    // 10. Focus round-trip via synthetic keystrokes (permission-free).
    client.send_ok(&Request::DispatchKeystroke {
        keystroke: "cmd-l".into(),
    })?;
    client.wait_for_event(
        |e| {
            matches!(
                e,
                Event::ChromeFocusChanged {
                    chrome_focused: true
                }
            )
        },
        Instant::now() + Duration::from_secs(10),
    )?;
    let snap = client.snapshot()?;
    assert!(
        snap.chrome_focused,
        "command bar should be focused after Cmd-L"
    );

    // Type into the command bar via the direct command (avoids per-char
    // keystroke round trips) and confirm the text is visible.
    client.send_ok(&Request::SetCommandBarText {
        text: page_two.to_string(),
    })?;
    let snap = client.snapshot()?;
    assert_eq!(snap.command_bar_text, page_two);

    // Escape returns focus to the web view.
    client.send_ok(&Request::DispatchKeystroke {
        keystroke: "escape".into(),
    })?;
    client.wait_for_event(
        |e| {
            matches!(
                e,
                Event::ChromeFocusChanged {
                    chrome_focused: false
                }
            )
        },
        Instant::now() + Duration::from_secs(10),
    )?;
    let snap = client.snapshot()?;
    assert!(
        !snap.chrome_focused,
        "web view should be focused after Escape"
    );

    // 11. Graceful quit.
    client.quit()?;

    // The process should exit cleanly.
    let status = wait_timeout(&mut harness.child, Duration::from_secs(10));
    assert!(status.is_some(), "app did not exit after Quit");
    assert!(harness
        .child
        .try_wait()
        .unwrap()
        .map(|s| s.success())
        .unwrap_or(false));

    Ok(())
}

fn wait_timeout(child: &mut Child, dur: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + dur;
    loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

impl Harness {
    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }

    fn dump_stderr(&mut self) {
        if let Some(mut stderr) = self.child.stderr.take() {
            use std::io::Read;
            let mut buf = String::new();
            let _ = stderr.read_to_string(&mut buf);
            eprintln!("--- app stderr ---\n{buf}");
        }
        let _ = std::fs::create_dir_all(&self.artifacts_dir);
        eprintln!("artifacts kept at {}", self.artifacts_dir.display());
    }

    fn cleanup(&mut self) {
        self.terminate();
        let _ = std::fs::remove_dir_all(&self.profile_dir);
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.terminate();
        let _ = std::fs::remove_dir_all(&self.profile_dir);
    }
}

// Keep the `Response` import referenced (used indirectly by the client API).
#[allow(dead_code)]
fn _uses_response(_: &Response) {}
