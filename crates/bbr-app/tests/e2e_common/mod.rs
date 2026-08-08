//! Shared real-app shell E2E harness: spawn the real `bbr` binary with the
//! testkit gateway, drive it over the typed protocol, and keep failure
//! artifacts (event ring + app stderr) in the profile dir.
//!
//! Used by the shell-tabs and shell-spaces suites (including the
//! shell-spaces-hint and shell-spaces-keyboard binaries). GUI suites must not run in
//! parallel: AppKit window geometry interferes through screen clamping, so
//! every test in a suite takes the serial lock. No sleeps: steps synchronize
//! on semantic events or snapshot state under deadlines.

#![cfg(target_os = "macos")]
#![allow(dead_code)] // shared helper module: each suite uses a subset

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

mod u2;

#[allow(unused_imports)] // each suite uses a subset of the shared helpers
pub use u2::{harness_root, harness_socket_path, strip_presence};

/// GUI suites must not run in parallel: AppKit window geometry interferes
/// through screen clamping, so every test in this binary takes the lock.
pub static SERIAL_GUI: Mutex<()> = Mutex::new(());

/// Take the serial GUI lock, recovering from a poisoned mutex: a panicked
/// sibling test must not cascade into the remaining tests of the binary —
/// the failure is diagnosed once, in the panicking test itself.
pub fn serial_gui() -> std::sync::MutexGuard<'static, ()> {
    SERIAL_GUI
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub use bbr_fixtures::FixtureServer;
#[allow(unused_imports)] // shared helper module: each suite uses a subset
pub use bbr_testkit::protocol::{dsl, BrowserChange, Event, TabVisualState};
pub use bbr_testkit::Client;
pub use bbr_web::{WebEvent, WebViewId};
pub use std::panic::{catch_unwind, AssertUnwindSafe};
pub use std::path::PathBuf;
pub use std::time::{Duration, Instant};

pub const PAGE_ONE_TITLE: &str = "BBR Fixture One";
pub const PAGE_TWO_TITLE: &str = "BBR Fixture Two";
pub const SLOW_TITLE: &str = "BBR Slow Page";
pub const LONG_TITLE: &str = "A title long enough that the vertical tab row must truncate it with an ellipsis — the shell's type:title rule keeps one line and the row never grows beyond its density height";
/// Fixture title whose first separator is early, so §4 truncation cuts at
/// the separator and the site name survives.
pub const SEPARATOR_TITLE: &str =
    "Site Name – A long descriptive suffix that surely pushes the row past its one-line budget";
/// The §4 separator-aware cut for the fixture above.
pub const SEPARATOR_ROW_TITLE: &str = "Site Name…";

pub struct Harness {
    pub child: Child,
    pub root: PathBuf,
    stderr: Arc<Mutex<String>>,
    pub success: bool,
}

impl Harness {
    pub fn spawn(bin: &str, args: &[&str], root: &Path) -> Self {
        let mut child = Command::new(bin)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn bbr");
        let stderr_buf = Arc::new(Mutex::new(String::new()));
        if let Some(mut stderr) = child.stderr.take() {
            let buf = stderr_buf.clone();
            std::thread::spawn(move || {
                use std::io::Read;
                let mut chunk = [0u8; 4096];
                while let Ok(n) = stderr.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    if let Ok(mut text) = buf.lock() {
                        text.push_str(&String::from_utf8_lossy(&chunk[..n]));
                        if text.len() > 131_072 {
                            let keep = text.len() - 131_072;
                            *text = text.split_off(keep);
                        }
                    }
                }
            });
        }
        Self {
            child,
            root: root.to_path_buf(),
            stderr: stderr_buf,
            success: false,
        }
    }

    pub fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }

    pub fn diagnostics(&mut self, socket: &Path, token: &str) {
        if let Ok(mut client) = Client::connect(socket) {
            if client.hello(token).is_ok() {
                if let Ok(events) = client.attach(None) {
                    let text = events
                        .into_iter()
                        .map(|(seq, event)| format!("{seq}: {event:?}\n"))
                        .collect::<String>();
                    let _ = std::fs::write(self.root.join("event-ring.txt"), text);
                }
            }
        }
        self.terminate();
        let stderr = self.stderr.lock().map(|s| s.clone()).unwrap_or_default();
        let _ = std::fs::write(self.root.join("app-stderr.txt"), &stderr);
        eprintln!("--- app stderr ---\n{stderr}");
        eprintln!("artifacts kept at {}", self.root.display());
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.terminate();
        if self.success {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

pub fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

pub fn connect(socket: &Path) -> Client {
    let until = Instant::now() + Duration::from_secs(60);
    loop {
        match Client::connect(socket) {
            Ok(client) => return client,
            Err(_) if Instant::now() < until => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => panic!("could not connect to gateway: {error}"),
        }
    }
}

pub fn wait_title(client: &mut Client, view: WebViewId, title: &str, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::Web(WebEvent::TitleChanged(id, value)) if *id == view && value == title),
            since,
            deadline(),
        )
        .expect("title event")
        .0
}

pub fn wait_browser_change(client: &mut Client, expected: BrowserChange, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::BrowserChanged { kind } if *kind == expected),
            since,
            deadline(),
        )
        .expect("browser change")
        .0
}

/// Wait for the next [`Event::BrowserChanged`] matching `matches` and return
/// its sequence cursor plus the concrete [`BrowserChange`] (for changes whose
/// payload, like the fresh space id, is only known after the event fires).
pub fn wait_browser_change_matching(
    client: &mut Client,
    matches: impl Fn(&BrowserChange) -> bool,
    since: u64,
) -> (u64, BrowserChange) {
    let (seq, event) = client
        .wait_for_event_since(
            |event| matches!(event, Event::BrowserChanged { kind } if matches(kind)),
            since,
            deadline(),
        )
        .expect("browser change");
    let Event::BrowserChanged { kind } = event else {
        unreachable!()
    };
    (seq, kind)
}

pub fn wait_chrome_focus(client: &mut Client, focused: bool, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::ChromeFocusChanged { chrome_focused } if *chrome_focused == focused),
            since,
            deadline(),
        )
        .expect("chrome focus event")
        .0
}

pub fn wait_web_event(
    client: &mut Client,
    predicate: impl Fn(&WebEvent) -> bool,
    since: u64,
) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::Web(web) if predicate(web)),
            since,
            deadline(),
        )
        .expect("web event")
        .0
}

/// Open a tab and wait for its deterministic fixture title (when known).
pub fn open_tab(client: &mut Client, since: &mut u64, url: &str, title: Option<&str>) -> WebViewId {
    client
        .send_ok(&bbr_testkit::Request::OpenTab { url: url.into() })
        .expect("open tab");
    let opened = client.snapshot().expect("opened tab snapshot");
    let view = opened.active_web_view.expect("new native view");
    if let Some(title) = title {
        if opened.title.as_deref() != Some(title) {
            *since = wait_title(client, view, title, *since);
        }
    }
    // Force the production draw so the render-derived snapshot fields
    // (visible_ids / dense window) reflect the new tab: the harness window
    // runs in the background, where the display link is throttled and a
    // mutation alone does not guarantee a render before the next snapshot.
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush after open tab");
    view
}

/// Start a fixture server on the standard deterministic pages.
pub fn fixtures() -> (FixtureServer, String, String) {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");
    (fixtures, page_one, page_two)
}

pub fn spawn_harness(root_tag: &str, page_one: &str) -> (Harness, PathBuf, String) {
    // Keep the root under the fixed short `/tmp` prefix: the gateway socket
    // must stay strictly below macOS's 104-byte `sun_path` limit, and
    // `$TMPDIR` alone is 49+ bytes on standard macOS (longer under direnv).
    let pid = std::process::id();
    let root = harness_root(root_tag, pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = harness_socket_path(root_tag, pid);
    let token = format!("shell-{root_tag}-{pid}");
    let bin = env!("CARGO_BIN_EXE_bbr");
    let harness = Harness::spawn(
        bin,
        &[
            "--bbr-initial-url",
            page_one,
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
        ],
        &root,
    );
    (harness, socket, token)
}

pub fn finish(result: std::thread::Result<()>, harness: &mut Harness, socket: &Path, token: &str) {
    match result {
        Ok(()) => {
            harness.success = true;
            eprintln!("SHELL E2E PASSED");
        }
        Err(panic) => {
            harness.diagnostics(socket, token);
            std::panic::resume_unwind(panic);
        }
    }
}

pub fn wait_for_exit(child: &mut Child, deadline: Instant) -> std::process::ExitStatus {
    loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return status;
        }
        assert!(Instant::now() < deadline, "app did not exit");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Poll the semantic snapshot until the engine-received native content rect
/// reaches `expected` (tolerance 0.5 pt). The rect is paint-driven — the
/// native slot reports the WKWebView frame on the next frame after a layout
/// mutation — so state fields assert immediately and geometry waits for the
/// paint under an explicit deadline. No sleeps: deadline-bounded snapshot
/// polling.
pub fn wait_for_content_rect(
    client: &mut Client,
    expected: [f32; 4],
    deadline: Instant,
) -> bbr_testkit::protocol::Snapshot {
    let close = |a: [f32; 4], b: [f32; 4]| {
        (a[0] - b[0]).abs() <= 0.5
            && (a[1] - b[1]).abs() <= 0.5
            && (a[2] - b[2]).abs() <= 0.5
            && (a[3] - b[3]).abs() <= 0.5
    };
    loop {
        let snapshot = client.snapshot().expect("content-rect snapshot");
        if close(snapshot.content_rect, expected) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "content rect did not reach {expected:?}; last {:?}",
            snapshot.content_rect
        );
        std::thread::yield_now();
    }
}

/// Poll until the slot's native content rect reaches the expected x/width
/// (height is not asserted here — the window bounds include the macOS
/// titlebar, so the slot's own height is the stable baseline captured once
/// and asserted for drift). Returns the snapshot with the reached rect.
pub fn wait_for_content_rect_x(
    client: &mut Client,
    expected_x: f32,
    expected_width: f32,
    deadline: Instant,
) -> bbr_testkit::protocol::Snapshot {
    loop {
        let snapshot = client.snapshot().expect("content-rect snapshot");
        let rect = snapshot.content_rect;
        if (rect[0] - expected_x).abs() <= 0.5 && (rect[2] - expected_width).abs() <= 0.5 {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "content rect x/width did not reach [{expected_x}, .., {expected_width}, ..]; last {rect:?}"
        );
        std::thread::yield_now();
    }
}

/// Consecutive frame-driven snapshots that must all show the same window
/// bounds and native content geometry before [`wait_for_window_settled`]
/// reports the window as settled. The harness window is never shown
/// (`--bbr-hidden` → `show: false`), but AppKit can still adjust a spawned
/// window frame shortly after creation (the adjustment lands via
/// `windowDidResize`), so a single — or even two consecutive — matching
/// observations can still precede a late geometry change. Requiring a few
/// fully-unchanged width + height + content observations outlives any late
/// adjustment while staying frame-driven, deadline-bounded, and sleep-free.
const WINDOW_SETTLE_FRAMES: usize = 4;

/// Poll until the window stops changing (the harness window is created with
/// `show: false`, and any late AppKit frame adjustment lands via
/// `windowDidResize`; GPUI reports the requested bounds first, then the
/// adjusted frame). Suites that assert absolute geometry (default sidebar
/// width, native content rect) must wait for this settle first; snapshots
/// taken mid-adjustment observe transient widths and flake. Returns the
/// settled snapshot.
pub fn wait_for_window_settled(
    client: &mut Client,
    deadline: Instant,
) -> bbr_testkit::protocol::Snapshot {
    let mut previous: Option<bbr_testkit::protocol::Snapshot> = None;
    let mut stable = 0usize;
    loop {
        let snapshot = client.snapshot().expect("window-settle snapshot");
        assert!(
            Instant::now() < deadline,
            "window did not settle after {WINDOW_SETTLE_FRAMES} unchanged frames; last {:?}",
            snapshot.window_size
        );
        match &previous {
            Some(prev) if window_geometry_close(prev, &snapshot) => {
                stable += 1;
                if stable >= WINDOW_SETTLE_FRAMES {
                    return snapshot;
                }
            }
            Some(_) => stable = 0,
            None => {}
        }
        previous = Some(snapshot);
        // Ask GPUI to complete another production draw instead of sleeping:
        // each frame-derived snapshot advances the settle contract, while a
        // later AppKit frame adjustment stays observable and resets the count.
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush while waiting for window settle");
    }
}

/// Two snapshots count as the same window geometry only when the window
/// bounds (width AND height) and the native content rect are all unchanged
/// within the half-point tolerance. A late AppKit frame adjustment changes
/// both the bounds and the content rect, so an adjustment between frames is
/// never mistaken for a settled window.
fn window_geometry_close(
    a: &bbr_testkit::protocol::Snapshot,
    b: &bbr_testkit::protocol::Snapshot,
) -> bool {
    let close = |x: f32, y: f32| (x - y).abs() <= 0.5;
    close(a.window_size[0], b.window_size[0])
        && close(a.window_size[1], b.window_size[1])
        && close(a.content_rect[0], b.content_rect[0])
        && close(a.content_rect[1], b.content_rect[1])
        && close(a.content_rect[2], b.content_rect[2])
        && close(a.content_rect[3], b.content_rect[3])
}

/// Poll until a SINGLE snapshot shows the window at `target_width` with the
/// native content rect at `content_width` — both fields from the same frame,
/// so a mid-transition snapshot can never satisfy the geometry contract.
/// The BBR platform adapter disables AppKit's grow-from-corner appearance
/// animation (see `bbr-web-macos/src/host.rs`), so a shown window keeps its
/// requested frame; a mismatch here is a real geometry regression and fails
/// loudly.
pub fn wait_for_window_and_content(
    client: &mut Client,
    target_width: f32,
    content_width: f32,
    deadline: Instant,
) -> bbr_testkit::protocol::Snapshot {
    loop {
        let snapshot = client.snapshot().expect("window+content snapshot");
        let close = |a: f32, b: f32| (a - b).abs() <= 0.5;
        if close(snapshot.window_size[0], target_width)
            && close(snapshot.content_rect[2], content_width)
        {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "window never reached width {target_width} with {content_width} pt native content (last window {:?}, rect {:?}); see E2E_ARCHITECTURE risks",
            snapshot.window_size,
            snapshot.content_rect
        );
        std::thread::yield_now();
    }
}

/// Assert the §15 semantic `bbr-*` ids the current render exposes. This is
/// the id-level evidence the design contract requires: every §11 assert id
/// must be present for the state the shell actually renders.
pub fn assert_ids(snapshot: &bbr_testkit::protocol::Snapshot, required: &[&str]) {
    for id in required {
        assert!(
            snapshot.visible_ids.iter().any(|present| present == id),
            "missing semantic id {id}; present: {:?}",
            snapshot.visible_ids
        );
    }
}

/// Skip GUI tests on headless CI (matches the other real-app suites).
pub fn allow_gui() -> bool {
    !std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false")
}
