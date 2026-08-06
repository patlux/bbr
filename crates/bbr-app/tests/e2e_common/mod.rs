//! Shared real-app shell E2E harness: spawn the real `bbr` binary with the
//! testkit gateway, drive it over the typed protocol, and keep failure
//! artifacts (event ring + app stderr) in the profile dir.
//!
//! Used by the shell-tabs and shell-spaces suites. GUI suites must not run in
//! parallel: AppKit window geometry interferes through screen clamping, so
//! every test in a suite takes the serial lock. No sleeps: steps synchronize
//! on semantic events or snapshot state under deadlines.

#![cfg(target_os = "macos")]
#![allow(dead_code)] // shared helper module: each suite uses a subset

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

/// GUI suites must not run in parallel: AppKit window geometry interferes
/// through screen clamping, so every test in this binary takes the lock.
pub static SERIAL_GUI: Mutex<()> = Mutex::new(());

pub use bbr_fixtures::FixtureServer;
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
    // Keep the root name short: the unix socket path must stay below macOS's
    // 104-byte `sun_path` limit (`TMPDIR` can be long).
    let root = std::env::temp_dir().join(format!("bbr-st-{root_tag}-{}", std::process::id()));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = profile.join("bbr-test.sock");
    let token = format!("shell-{root_tag}-{}", std::process::id());
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
