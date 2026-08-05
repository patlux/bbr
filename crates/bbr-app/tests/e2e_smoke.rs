//! Phase-1 smoke E2E: spawns the real `bbr` binary (with the `testkit`
//! feature), attaches to its gateway, and drives a real WKWebView through
//! startup + initial load + back/forward/reload + resize + focus round-trip
//! + graceful shutdown — purely via events and snapshots.
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

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bbr_fixtures::FixtureServer;
use bbr_testkit::protocol::{dsl, Event, Request, Response, Snapshot};
use bbr_testkit::Client;

/// Fixture page titles (deterministic content).
const PAGE_ONE_TITLE: &str = "BBR Fixture One";
const PAGE_TWO_TITLE: &str = "BBR Fixture Two";

struct Harness {
    child: Child,
    profile_dir: PathBuf,
    artifacts_dir: PathBuf,
    /// Drained stderr (kept for post-mortem; also prevents a full pipe
    /// buffer from blocking the app on `write`).
    stderr_buf: Arc<Mutex<String>>,
    /// Becomes true only after the whole smoke passed; the profile dir is
    /// then removed on clean-up instead of being preserved for post-mortem.
    success: bool,
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

    /// Spawn the app with its stderr drained on a background thread.
    fn spawn(bin: &str, args: &[&str], profile_dir: &Path, artifacts_dir: &Path) -> Harness {
        let mut child = Command::new(bin)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn bbr");
        let stderr_buf = Arc::new(Mutex::new(String::new()));
        if let Some(mut stderr) = child.stderr.take() {
            let buf = stderr_buf.clone();
            std::thread::Builder::new()
                .name("bbr-e2e-stderr-drain".into())
                .spawn(move || {
                    use std::io::Read;
                    let mut chunk = [0u8; 4096];
                    loop {
                        match stderr.read(&mut chunk) {
                            Ok(0) => break,
                            Ok(n) => {
                                if let Ok(mut guard) = buf.lock() {
                                    guard.push_str(&String::from_utf8_lossy(&chunk[..n]));
                                    // Bound memory to the tail (diagnostics).
                                    if guard.len() > 131072 {
                                        let start = guard.len() - 131072;
                                        *guard = guard.split_off(start);
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                })
                .ok();
        }
        Harness {
            child,
            profile_dir: profile_dir.to_path_buf(),
            artifacts_dir: artifacts_dir.to_path_buf(),
            stderr_buf,
            success: false,
        }
    }

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }

    fn dump_stderr(&mut self) {
        let buf = self
            .stderr_buf
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        eprintln!("--- app stderr ---\n{buf}");
        let _ = std::fs::create_dir_all(&self.artifacts_dir);
        let _ = std::fs::write(self.artifacts_dir.join("app-stderr.txt"), buf);
    }

    /// Pull the gateway event ring into the artifacts dir for post-mortem.
    fn save_event_ring(&mut self, socket: &Path, token: &str) {
        let _ = std::fs::create_dir_all(&self.artifacts_dir);
        match Client::connect(socket) {
            Ok(mut client) => match client.hello(token) {
                Ok(_) => match client.attach(None) {
                    Ok(replay) => {
                        let mut out = String::new();
                        for (seq, event) in replay {
                            out.push_str(&format!("{seq}: {event:?}\n"));
                        }
                        let path = self.artifacts_dir.join("event-ring.txt");
                        if let Err(e) = std::fs::write(&path, out) {
                            eprintln!("could not write event ring to {}: {e}", path.display());
                        }
                    }
                    Err(e) => eprintln!("attach for event ring failed: {e}"),
                },
                Err(e) => eprintln!("hello for event ring failed: {e}"),
            },
            Err(e) => eprintln!("connect for event ring failed: {e}"),
        }
        eprintln!("artifacts kept at {}", self.artifacts_dir.display());
    }

    fn cleanup(&mut self) {
        self.terminate();
        if self.success {
            let _ = std::fs::remove_dir_all(&self.profile_dir);
        } else {
            eprintln!(
                "profile (post-mortem) kept at {}",
                self.profile_dir.display()
            );
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.cleanup();
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
    let mut harness = Harness::spawn(
        bin,
        &[
            "--bbr-initial-url",
            &page_one,
            "--bbr-profile-dir",
            profile_dir.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
            "--bbr-clock=system",
        ],
        &profile_dir,
        &artifacts_dir,
    );

    // 4. Connect and drive the smoke. `catch_unwind` lets the diagnostics
    // path (event ring + stderr) run even when a step panics on an assertion,
    // so a failure always leaves high-signal artifacts behind.
    let result = catch_unwind(AssertUnwindSafe(|| {
        run_smoke(&mut harness, &socket, &token, &page_one, &page_two)
    }));
    match result {
        Ok(Ok(())) => {
            harness.success = true;
            harness.cleanup();
            eprintln!("SMOKE PASSED");
        }
        Ok(Err(e)) => fail(&mut harness, &socket, &token, &fixtures, e),
        Err(panic) => {
            harness.save_event_ring(&socket, &token);
            harness.terminate();
            harness.dump_stderr();
            eprintln!(
                "fixture hits: page1={}, page2={}",
                fixtures.hits("/page1.html"),
                fixtures.hits("/page2.html")
            );
            eprintln!("SMOKE PANICKED");
            std::panic::resume_unwind(panic);
        }
    }
}

/// Unified failure path: preserve the event ring + stderr, then panic.
fn fail(
    harness: &mut Harness,
    socket: &Path,
    token: &str,
    fixtures: &FixtureServer,
    e: Box<dyn std::error::Error>,
) -> ! {
    harness.save_event_ring(socket, token);
    harness.terminate();
    harness.dump_stderr();
    eprintln!(
        "fixture hits: page1={}, page2={}",
        fixtures.hits("/page1.html"),
        fixtures.hits("/page2.html")
    );
    eprintln!("SMOKE FAILED: {e}");
    panic!("smoke test failed: {e}");
}

fn event_deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
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

    // A seq cursor: every event wait below is `wait_for_event_since`, so a
    // stale event queued from an earlier step (e.g. the previous page's title
    // re-emitted at the next navigation's `didStart`) can never satisfy a later
    // assertion. Baseline the cursor at the last event already replayed.
    let mut since = replay.iter().map(|(s, _)| *s).max().unwrap_or(0);

    // AppReady may already be in the attach replay because the gateway starts
    // before the harness connects. Only wait for a future event when replay
    // did not contain it.
    if !replay
        .iter()
        .any(|(_, event)| matches!(event, Event::AppReady { .. }))
    {
        let (seq, _) = client.wait_for_event_since(
            |e| matches!(e, Event::AppReady { .. }),
            since,
            event_deadline(),
        )?;
        since = since.max(seq);
    }

    let snap = client.snapshot()?;
    eprintln!("snapshot after attach: {snap:#?}");

    // 5. Initial load warm-up. The attach replay already contains the events
    // when WebKit finishes before the harness connects; otherwise wait. These
    // waits are deliberately cursor-free: there is no earlier step whose
    // leftovers could pollute them, and within the initial load `LoadingChanged(false)`
    // (seq N+13) legitimately precedes the KVO title (seq N+19). A seq cursor
    // here would reject the valid `LoadingChanged(false)` as stale.
    if !replay.iter().any(|(_, event)| {
        matches!(event, Event::Web(bbr_web::WebEvent::TitleChanged(_, title)) if title == PAGE_ONE_TITLE)
    }) {
        let (seq, _) =
            client.wait_for_event(|e| {
                matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_ONE_TITLE)
            }, event_deadline())?;
        since = since.max(seq);
    }
    if !replay.iter().any(|(_, event)| {
        matches!(
            event,
            Event::Web(bbr_web::WebEvent::LoadingChanged(_, false))
        )
    }) {
        let (seq, _) = client.wait_for_event(
            |e| matches!(e, Event::Web(bbr_web::WebEvent::LoadingChanged(_, false))),
            event_deadline(),
        )?;
        since = since.max(seq);
    }

    // Snapshot reflects the loaded page.
    let snap = client.snapshot()?;
    assert_eq!(snap.title.as_deref(), Some(PAGE_ONE_TITLE));
    assert_eq!(snap.url.as_deref(), Some(page_one));
    assert!(snap.web_view_hosted, "native WKWebView must be hosted");
    assert_eq!(snap.engine_kind, "native-wkwebview");

    // 6. Navigate to page_two via the engine action. The title change and the
    // `can_go_back` flip are emitted by the same state push; wait for the
    // state event *after* the title so the snapshot cannot race ahead of the
    // back-history update.
    client.send_ok(&dsl::navigate(page_two))?;
    let (title_seq, _) = client.wait_for_event_since(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_TWO_TITLE),
        since,
        event_deadline(),
    )?;
    since = since.max(title_seq);
    let (back_seq, _) = client.wait_for_event_since(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::CanGoBackChanged(_, true))),
        since,
        event_deadline(),
    )?;
    since = since.max(back_seq);
    let snap = client.snapshot()?;
    assert_eq!(snap.title.as_deref(), Some(PAGE_TWO_TITLE));
    assert!(
        snap.can_go_back,
        "after navigating forward, back must be enabled"
    );

    // 7. Back → page_one.
    client.send_ok(&dsl::back())?;
    let (one_seq, _) = client.wait_for_event_since(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_ONE_TITLE),
        since,
        event_deadline(),
    )?;
    since = since.max(one_seq);
    let (fwd_seq, _) = client.wait_for_event_since(
        |e| {
            matches!(
                e,
                Event::Web(bbr_web::WebEvent::CanGoForwardChanged(_, true))
            )
        },
        since,
        event_deadline(),
    )?;
    since = since.max(fwd_seq);
    let snap = client.snapshot()?;
    assert_eq!(snap.title.as_deref(), Some(PAGE_ONE_TITLE));
    assert!(
        snap.can_go_forward,
        "after going back, forward must be enabled"
    );

    // 8. Forward → page_two again.
    client.send_ok(&dsl::forward())?;
    let (forward_title_seq, _) = client.wait_for_event_since(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::TitleChanged(_, t)) if t == PAGE_TWO_TITLE),
        since,
        event_deadline(),
    )?;
    since = since.max(forward_title_seq);

    // 9. Reload → observe a *fresh* load cycle. The forward step already
    // produced LoadingChanged events; the seq cursor forbids the reload
    // assertion from matching those leftovers.
    client.send_ok(&dsl::reload())?;
    let (load_seq, _) = client.wait_for_event_since(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::LoadingChanged(_, true))),
        since,
        event_deadline(),
    )?;
    since = since.max(load_seq);
    client.wait_for_event_since(
        |e| matches!(e, Event::Web(bbr_web::WebEvent::LoadingChanged(_, false))),
        since,
        event_deadline(),
    )?;

    // 10. Resize the window; the native slot re-frames on the next layout.
    // Choose a target relative to the current, screen-clamped geometry. A
    // fixed large target can be clamped by macOS to the existing size on a
    // small display and would not prove that resize works.
    let before = client.snapshot()?;
    // When AppKit already clamped a narrow window to BBR's minimum width,
    // shrinking again is a no-op. Grow small windows; shrink larger ones.
    let target_width = if before.window_size[0] < 700.0 {
        before.window_size[0] + 120.0
    } else {
        before.window_size[0] - 140.0
    };
    let target_height = if before.window_size[1] < 460.0 {
        before.window_size[1] + 80.0
    } else {
        before.window_size[1] - 100.0
    };
    client.send_ok(&Request::SetWindowSize {
        width: target_width,
        height: target_height,
    })?;
    let after = wait_for_window_resize(&mut client, &before, event_deadline())?;
    assert!(
        (after.window_size[0] - before.window_size[0]).abs() > 50.0,
        "width must actually change, got {before:?} -> {after:?}"
    );

    // 11. Focus round-trip via synthetic keystrokes (permission-free). The
    // focus events are emitted from the app's real focus paths (`Cmd-L` →
    // command bar, `Escape` → web view).
    client.send_ok(&Request::DispatchKeystroke {
        keystroke: "cmd-l".into(),
    })?;
    let (focus_seq, _) = client.wait_for_event_since(
        |e| {
            matches!(
                e,
                Event::ChromeFocusChanged {
                    chrome_focused: true
                }
            )
        },
        since,
        event_deadline(),
    )?;
    since = since.max(focus_seq);
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
    let (_web_seq, _) = client.wait_for_event_since(
        |e| {
            matches!(
                e,
                Event::ChromeFocusChanged {
                    chrome_focused: false
                }
            )
        },
        since,
        event_deadline(),
    )?;
    let snap = client.snapshot()?;
    assert!(
        !snap.chrome_focused,
        "web view should be focused after Escape"
    );

    // 12. Graceful quit.
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

fn wait_for_window_resize(
    client: &mut Client,
    before: &Snapshot,
    deadline: Instant,
) -> Result<Snapshot, Box<dyn std::error::Error>> {
    loop {
        let snapshot = client.snapshot()?;
        if (snapshot.window_size[0] - before.window_size[0]).abs() > 50.0 {
            return Ok(snapshot);
        }
        if Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!(
                    "window did not resize before deadline: {:?} -> {:?}",
                    before.window_size, snapshot.window_size
                ),
            )
            .into());
        }
        std::thread::yield_now();
    }
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

// Keep the `Response` import referenced (used indirectly by the client API).
#[allow(dead_code)]
fn _uses_response(_: &Response) {}
