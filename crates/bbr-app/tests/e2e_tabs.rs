//! Real-app multi-tab E2E: controller-backed tabs over multiple native WKWebViews.

#![cfg(target_os = "macos")]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bbr_fixtures::FixtureServer;
use bbr_testkit::protocol::{dsl, BrowserChange, Event};
use bbr_testkit::Client;
use bbr_web::{WebEvent, WebViewId};

const PAGE_ONE_TITLE: &str = "BBR Fixture One";
const PAGE_TWO_TITLE: &str = "BBR Fixture Two";

struct Harness {
    child: Child,
    root: PathBuf,
    stderr: Arc<Mutex<String>>,
    success: bool,
}

impl Harness {
    fn spawn(bin: &str, args: &[&str], root: &Path) -> Self {
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

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }

    fn diagnostics(&mut self, socket: &Path, token: &str) {
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

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

fn connect(socket: &Path) -> Client {
    let until = Instant::now() + Duration::from_secs(60);
    loop {
        match Client::connect(socket) {
            Ok(client) => return client,
            Err(_) if Instant::now() < until => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => panic!("could not connect to gateway: {error}"),
        }
    }
}

fn wait_title(client: &mut Client, view: WebViewId, title: &str, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::Web(WebEvent::TitleChanged(id, value)) if *id == view && value == title),
            since,
            deadline(),
        )
        .expect("title event")
        .0
}

fn wait_browser_change(client: &mut Client, expected: BrowserChange, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::BrowserChanged { kind } if *kind == expected),
            since,
            deadline(),
        )
        .expect("browser change")
        .0
}

#[test]
#[ignore]
fn real_tabs_preserve_sessions_route_navigation_and_destroy_on_close() {
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");

    let root = std::env::temp_dir().join(format!("bbr-e2e-tabs-{}", std::process::id()));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = profile.join("bbr-test.sock");
    let token = format!("tabs-{}", std::process::id());
    let bin = env!("CARGO_BIN_EXE_bbr");
    let mut harness = Harness::spawn(
        bin,
        &[
            "--bbr-initial-url",
            &page_one,
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
        ],
        &root,
    );

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        assert_eq!(initial.tabs.len(), 1);
        assert_eq!(initial.engine_sessions, 1);
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-t".into(),
            })
            .expect("Cmd-T");
        since = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        let opened = client.snapshot().expect("opened blank tab");
        assert_eq!(opened.tabs.len(), 2);
        assert_eq!(opened.engine_sessions, 2);
        assert_eq!(opened.active_tab_index, Some(1));
        assert_eq!(opened.tabs[0].web_view, Some(first_view));
        let second_view = opened.active_web_view.expect("second view");
        assert_ne!(first_view, second_view);

        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("set second URL");
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "enter".into(),
            })
            .expect("navigate new tab");
        since = wait_title(&mut client, second_view, PAGE_TWO_TITLE, since);
        let two_tabs = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two_tabs.tabs.len(), 2);
        assert_eq!(two_tabs.engine_sessions, 2);
        assert_eq!(two_tabs.active_web_view, Some(second_view));
        assert_eq!(two_tabs.tabs[0].web_view, Some(first_view));

        // Build history only in tab 2, then prove navigation targets its id.
        client
            .send_ok(&dsl::navigate(&page_one))
            .expect("navigate second tab");
        since = wait_title(&mut client, second_view, PAGE_ONE_TITLE, since);
        let (back_state_seq, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::Web(WebEvent::CanGoBackChanged(id, true)) if *id == second_view),
                since,
                deadline(),
            )
            .expect("second tab history");
        since = back_state_seq;

        client.send_ok(&dsl::back()).expect("back in second tab");
        since = wait_title(&mut client, second_view, PAGE_TWO_TITLE, since);
        client
            .send_ok(&dsl::forward())
            .expect("forward in second tab");
        since = wait_title(&mut client, second_view, PAGE_ONE_TITLE, since);
        client.send_ok(&dsl::reload()).expect("reload second tab");
        let (loading_seq, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::Web(WebEvent::LoadingChanged(id, true)) if *id == second_view),
                since,
                deadline(),
            )
            .expect("fresh reload start");
        since = client
            .wait_for_event_since(
                |event| matches!(event, Event::Web(WebEvent::LoadingChanged(id, false)) if *id == second_view),
                loading_seq,
                deadline(),
            )
            .expect("reload finish")
            .0;

        // Switch away and back: both native identities must remain unchanged.
        client
            .send_ok(&dsl::select_tab(0))
            .expect("select first tab");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let selected_first = client.snapshot().expect("first selected");
        assert_eq!(selected_first.active_web_view, Some(first_view));
        assert_eq!(selected_first.engine_sessions, 2);
        assert_eq!(selected_first.tabs[1].web_view, Some(second_view));

        client
            .send_ok(&dsl::select_tab(1))
            .expect("reselect second tab");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let reselected = client.snapshot().expect("second reselected");
        assert_eq!(reselected.active_web_view, Some(second_view));
        assert_eq!(reselected.tabs[0].web_view, Some(first_view));

        // Cmd-W closes the active second tab and destroys exactly its session.
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-w".into(),
            })
            .expect("Cmd-W");
        let _ = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 1 }, since);
        let closed = client.snapshot().expect("closed snapshot");
        assert_eq!(closed.tabs.len(), 1);
        assert_eq!(closed.engine_sessions, 1);
        assert_eq!(closed.active_web_view, Some(first_view));
        assert_eq!(closed.tabs[0].web_view, Some(first_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    match result {
        Ok(()) => {
            harness.success = true;
            eprintln!("TABS E2E PASSED");
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            eprintln!(
                "fixture hits: page1={}, page2={}",
                fixtures.hits("/page1.html"),
                fixtures.hits("/page2.html")
            );
            std::panic::resume_unwind(panic);
        }
    }
}

fn wait_for_exit(child: &mut Child, deadline: Instant) -> std::process::ExitStatus {
    loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return status;
        }
        assert!(Instant::now() < deadline, "app did not exit");
        std::thread::sleep(Duration::from_millis(50));
    }
}
