//! Real-app visible-tab shell E2E (Phase-4 first vertical slice).
//!
//! Covers the controller-backed vertical tab surface over real WKWebViews:
//! pointer open/select/close/reopen through the production listener path,
//! keyboard traversal through real keystrokes, pinned/regular regions, the
//! per-row visual states (selected/loading/error) the shell renders, empty
//! space, long titles, missing favicons, warm-selection identity (no reload),
//! and crash recovery. No sleeps: every step synchronizes on semantic events
//! or snapshot state under a deadline.

#![cfg(target_os = "macos")]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// GUI suites must not run in parallel: AppKit window geometry interferes
/// through screen clamping, so every test in this binary takes the lock.
static SERIAL_GUI: Mutex<()> = Mutex::new(());

use bbr_fixtures::FixtureServer;
use bbr_testkit::protocol::{dsl, BrowserChange, CommandBarMode, Event, TabVisualState};
use bbr_testkit::Client;
use bbr_web::{WebEvent, WebViewId};

const PAGE_ONE_TITLE: &str = "BBR Fixture One";
const PAGE_TWO_TITLE: &str = "BBR Fixture Two";
const SLOW_TITLE: &str = "BBR Slow Page";
const LONG_TITLE: &str = "A title long enough that the vertical tab row must truncate it with an ellipsis — the shell's type:title rule keeps one line and the row never grows beyond its density height";

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

fn wait_chrome_focus(client: &mut Client, focused: bool, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::ChromeFocusChanged { chrome_focused } if *chrome_focused == focused),
            since,
            deadline(),
        )
        .expect("chrome focus event")
        .0
}

fn wait_web_event(client: &mut Client, predicate: impl Fn(&WebEvent) -> bool, since: u64) -> u64 {
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
fn open_tab(client: &mut Client, since: &mut u64, url: &str, title: Option<&str>) -> WebViewId {
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

fn spawn_harness(root_tag: &str, page_one: &str) -> (Harness, PathBuf, String) {
    // Keep the root name short: the unix socket path must stay below macOS's
    // 104-byte `sun_path` limit (`TMPDIR` can be long).
    let root = std::env::temp_dir().join(format!("bbr-st-{root_tag}-{}", std::process::id()));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = profile.join("bbr-test.sock");
    let token = format!("shell-tabs-{root_tag}-{}", std::process::id());
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

fn finish(result: std::thread::Result<()>, harness: &mut Harness, socket: &Path, token: &str) {
    match result {
        Ok(()) => {
            harness.success = true;
            eprintln!("SHELL-TABS E2E PASSED");
        }
        Err(panic) => {
            harness.diagnostics(socket, token);
            std::panic::resume_unwind(panic);
        }
    }
}

/// Pointer open/select/close/reopen through the production row listeners:
/// chrome focus domain preserved, warm selection reuses the session (no
/// reload), close destroys only its session, reopen restores identity with a
/// fresh session.
#[test]
#[ignore]
fn pointer_opens_selects_closes_reopens_visible_tabs() {
    let _serial = SERIAL_GUI.lock().expect("gui lock");
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

    let (mut harness, socket, token) = spawn_harness("pointer", &page_one);

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

        // Pointer open: a second visible row.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.tabs.len(), 2);
        assert_eq!(two.engine_sessions, 2);
        assert_eq!(two.active_tab_index, Some(1));
        assert_eq!(
            two.tab_states,
            vec![TabVisualState::Default, TabVisualState::Selected]
        );

        // Pointer select: activate row 0 through the production mouse-down
        // listener path. The pointer keeps the chrome focus domain.
        client.send_ok(&dsl::click_tab(0)).expect("click first row");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let selected = client.snapshot().expect("first selected");
        assert_eq!(selected.active_web_view, Some(first_view));
        assert_eq!(selected.active_tab_index, Some(0));
        assert_eq!(selected.engine_sessions, 2, "warm select keeps sessions");
        assert!(selected.chrome_focused, "pointer keeps chrome focus domain");
        assert_eq!(selected.tabs[0].web_view, Some(first_view));
        assert_eq!(
            selected.tab_states,
            vec![TabVisualState::Selected, TabVisualState::Default]
        );

        // Warm pointer selection must not reload the target: capture the
        // selection sequence, select back, and prove no engine load event
        // for the first view crossed the wire in between.
        client
            .send_ok(&dsl::click_tab(1))
            .expect("click second row");
        let (seq_second, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::BrowserChanged { kind } if *kind == BrowserChange::TabSelected { index: 1 }),
                since,
                deadline(),
            )
            .expect("second selected");
        let reselected = client.snapshot().expect("second reselected");
        assert_eq!(reselected.active_web_view, Some(second_view));
        assert_eq!(reselected.tabs[0].web_view, Some(first_view));
        assert_eq!(reselected.engine_sessions, 2);

        let reloads = client
            .attach(Some(since))
            .expect("replay selection window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < seq_second
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == first_view
                    )
            })
            .count();
        assert_eq!(reloads, 0, "warm selection must not reload the tab");
        since = seq_second;

        // Pointer close of the first row's close control: destroys exactly
        // its session, neighbor becomes active, chrome focus preserved.
        client
            .send_ok(&dsl::click_tab_close(0))
            .expect("click first close");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let closed = client.snapshot().expect("closed snapshot");
        assert_eq!(closed.tabs.len(), 1);
        assert_eq!(closed.engine_sessions, 1);
        assert_eq!(closed.active_web_view, Some(second_view));
        assert!(closed.chrome_focused);
        assert_eq!(closed.command_bar_mode, CommandBarMode::Inactive);

        // Reopen restores the closed tab's identity (title/URL) with a fresh
        // native session and keeps the invoking chrome focus domain.
        client.send_ok(&dsl::reopen_tab()).expect("reopen tab");
        since = wait_browser_change(&mut client, BrowserChange::TabReopened { index: 0 }, since);
        let reopened = client.snapshot().expect("reopened snapshot");
        assert_eq!(reopened.tabs.len(), 2);
        assert_eq!(reopened.active_tab_index, Some(0));
        assert_eq!(reopened.engine_sessions, 2);
        assert_eq!(reopened.tabs[0].url, page_one, "URL identity preserved");
        assert!(reopened.chrome_focused, "reopen preserves chrome domain");
        let fresh_view = reopened.active_web_view.expect("fresh view");
        assert_ne!(fresh_view, first_view, "fresh session incarnation");
        if reopened.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, fresh_view, PAGE_ONE_TITLE, since);
        }
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(
            restored.tabs[0].title, PAGE_ONE_TITLE,
            "title identity preserved"
        );

        // Empty space: closing the final tab through the pointer path leaves
        // the empty New Tab surface (no about:blank, no session).
        client
            .send_ok(&dsl::click_tab_close(0))
            .expect("close first row");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        client
            .send_ok(&dsl::click_tab_close(0))
            .expect("close last row");
        let _ = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let empty = client.snapshot().expect("empty space snapshot");
        assert!(empty.tabs.is_empty());
        assert_eq!(empty.engine_sessions, 0);
        assert_eq!(empty.active_web_view, None);
        assert_eq!(empty.command_bar_mode, CommandBarMode::NewTab);
        assert!(empty.chrome_focused);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Keyboard traversal of the visible tab list with real keystrokes: Cmd-T +
/// Enter opens, arrow keys move selection, backspace closes, Cmd-Shift-T
/// reopens. The tab list keeps chrome focus across the sequence.
#[test]
#[ignore]
fn keyboard_navigates_visible_tab_list() {
    let _serial = SERIAL_GUI.lock().expect("gui lock");
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

    let (mut harness, socket, token) = spawn_harness("keyboard", &page_one);

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

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Keyboard open: Cmd-T stages, real keystrokes commit the new tab.
        keystroke(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let staged = client.snapshot().expect("staged snapshot");
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("set destination");
        keystroke(&mut client, "enter");
        since = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        let opened = client.snapshot().expect("opened snapshot");
        assert_eq!(opened.tabs.len(), 2);
        assert_eq!(opened.engine_sessions, 2);
        let second_view = opened.active_web_view.expect("second view");
        if opened.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, second_view, PAGE_TWO_TITLE, since);
        }

        // Enter moves focus into the new page; a pointer click on a row then
        // brings the tab list into chrome focus (production path).
        assert!(!client.snapshot().expect("web focus").chrome_focused);
        client
            .send_ok(&dsl::click_tab(0))
            .expect("click first row to focus the list");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        // The pointer listener emits the focus change before the selection
        // event, so the chrome-domain assertion reads the authoritative
        // snapshot rather than racing the event cursor.
        assert!(
            client.snapshot().expect("tab list focused").chrome_focused,
            "pointer row click focuses the tab list (chrome domain)"
        );

        // Real arrow-key traversal: down selects the second row, up returns.
        keystroke(&mut client, "down");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let down = client.snapshot().expect("down snapshot");
        assert_eq!(down.active_tab_index, Some(1));
        assert_eq!(down.active_web_view, Some(second_view));
        assert_eq!(
            down.engine_sessions, 2,
            "keyboard warm select keeps sessions"
        );
        assert!(down.chrome_focused, "keyboard traversal stays in chrome");

        keystroke(&mut client, "up");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let up = client.snapshot().expect("up snapshot");
        assert_eq!(up.active_tab_index, Some(0));
        assert_eq!(up.active_web_view, Some(first_view));
        assert!(up.chrome_focused);

        // Backspace closes the active row; Cmd-Shift-T reopens it. Both
        // preserve the chrome focus domain.
        keystroke(&mut client, "backspace");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let after_close = client.snapshot().expect("closed snapshot");
        assert_eq!(after_close.tabs.len(), 1);
        assert_eq!(after_close.active_tab_index, Some(0));
        assert_eq!(after_close.active_web_view, Some(second_view));
        assert_eq!(after_close.engine_sessions, 1);
        assert!(after_close.chrome_focused);

        keystroke(&mut client, "cmd-shift-t");
        since = wait_browser_change(&mut client, BrowserChange::TabReopened { index: 0 }, since);
        let after_reopen = client.snapshot().expect("reopened snapshot");
        assert_eq!(after_reopen.tabs.len(), 2);
        assert_eq!(after_reopen.active_tab_index, Some(0));
        assert_eq!(after_reopen.engine_sessions, 2);
        assert!(after_reopen.chrome_focused);
        let fresh_view = after_reopen.active_web_view.expect("fresh view");
        assert_ne!(fresh_view, first_view);
        if after_reopen.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, fresh_view, PAGE_ONE_TITLE, since);
        }
        assert_eq!(
            client.snapshot().expect("restored title").tabs[0].title,
            PAGE_ONE_TITLE
        );

        // Enter activates the selected row: the chrome traversal commits and
        // keyboard focus moves into the page (DESIGN.md §11.1 / interaction
        // contract). Selection and session identity are untouched.
        keystroke(&mut client, "enter");
        since = wait_chrome_focus(&mut client, false, since);
        let activated = client.snapshot().expect("activated snapshot");
        assert_eq!(activated.active_tab_index, Some(0));
        assert_eq!(activated.active_web_view, Some(fresh_view));
        assert_eq!(activated.engine_sessions, 2, "activation never recreates");
        assert!(!activated.chrome_focused, "Enter moves focus into the page");
        assert_eq!(activated.command_bar_mode, CommandBarMode::Inactive);

        // The page keeps keyboard focus afterwards; a later pointer row click
        // returns to the tab list (chrome domain) without losing selection.
        assert!(!client.snapshot().expect("web focus stays").chrome_focused);
        client
            .send_ok(&dsl::click_tab(1))
            .expect("click second row after activation");
        let _ = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let back = client.snapshot().expect("back in chrome");
        assert!(back.chrome_focused);
        assert_eq!(back.active_tab_index, Some(1));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Keyboard close of the final tab must open the empty New Tab command
/// surface exactly like the pointer path (interaction contract: closing the
/// final tab leaves the space empty and offers an obvious way to open a new
/// tab; no about:blank is created). Regression for the keyboard path that
/// used to reset the staged surface back to the tab list.
#[test]
#[ignore]
fn keyboard_close_of_final_tab_opens_empty_new_tab_surface() {
    let _serial = SERIAL_GUI.lock().expect("gui lock");
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");

    let (mut harness, socket, token) = spawn_harness("empty-keyboard", &page_one);

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

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Seed a second tab so the final close is the unambiguous last one.
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_one.clone(),
            })
            .expect("open second tab");
        since = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.tabs.len(), 2);
        let second_view = two.active_web_view.expect("second view");
        if two.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, second_view, PAGE_ONE_TITLE, since);
        }

        // Focus the tab list through the production pointer path, then close
        // the active row. The first close leaves a tab behind, so the list
        // keeps focus for repeated keys.
        client
            .send_ok(&dsl::click_tab(1))
            .expect("focus list on row 1");
        let _ = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        assert!(client.snapshot().expect("list focused").chrome_focused);
        keystroke(&mut client, "backspace");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 1 }, since);
        let one = client.snapshot().expect("one-tab snapshot");
        assert_eq!(one.tabs.len(), 1);
        assert_eq!(one.active_web_view, Some(first_view));
        assert!(one.chrome_focused, "repeated keys stay in the tab list");
        assert_eq!(one.command_bar_mode, CommandBarMode::Inactive);

        // The final backspace must open the empty New Tab command surface and
        // keep it open: NewTab mode + chrome (command-bar) focus, no session.
        keystroke(&mut client, "backspace");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let empty = client.snapshot().expect("empty keyboard snapshot");
        assert!(empty.tabs.is_empty());
        assert_eq!(empty.engine_sessions, 0);
        assert_eq!(empty.active_web_view, None);
        assert_eq!(empty.command_bar_mode, CommandBarMode::NewTab);
        assert!(empty.chrome_focused, "command surface owns chrome focus");

        // Enter on the empty command surface with no destination creates
        // nothing; the staged surface stays open for the next destination.
        keystroke(&mut client, "enter");
        let still_empty = client.snapshot().expect("still empty after Enter");
        assert!(still_empty.tabs.is_empty());
        assert_eq!(still_empty.engine_sessions, 0);
        assert_eq!(still_empty.command_bar_mode, CommandBarMode::NewTab);

        // A committed destination re-opens the flow from the empty state.
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_one.clone(),
            })
            .expect("set destination");
        keystroke(&mut client, "enter");
        let _ = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 0 }, since);
        let reborn = client.snapshot().expect("reborn snapshot");
        assert_eq!(reborn.tabs.len(), 1);
        assert_eq!(reborn.engine_sessions, 1);
        assert!(
            !reborn.chrome_focused,
            "committed New Tab moves to the page"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The shell renders pinned/regular regions, per-row visual states
/// (loading/error) from controller snapshots, long-title and missing-favicon
/// fallbacks, and recovers a crashed tab with a fresh session.
#[test]
#[ignore]
fn shell_renders_regions_fallbacks_and_crash_recovery() {
    let _serial = SERIAL_GUI.lock().expect("gui lock");
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let long_title = fixtures.url("/long-title.html");
    let slow = fixtures.url("/__fx/slow");
    let blank = "about:blank";

    let (mut harness, socket, token) = spawn_harness("states", &page_one);

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

        // about:blank has no hostname — the §7.2 monogram falls back to the
        // scheme and the title fallback chain still yields a row label, so
        // the missing-favicon tab renders like any other row.
        open_tab(&mut client, &mut since, blank, None);
        open_tab(&mut client, &mut since, &long_title, Some(LONG_TITLE));
        let slow_view = open_tab(&mut client, &mut since, &slow, Some(SLOW_TITLE));

        let four = client.snapshot().expect("four-tab snapshot");
        assert_eq!(four.tabs.len(), 4);
        assert_eq!(four.engine_sessions, 4);
        assert_eq!(four.tabs[0].url, page_one);
        assert_eq!(four.tabs[1].url, blank);
        assert_eq!(four.tabs[2].url, long_title, "long-title tab holds its URL");
        assert_eq!(four.tabs[2].title, LONG_TITLE, "long title survives");
        assert_eq!(four.active_tab_index, Some(3));
        assert_eq!(four.active_web_view, Some(slow_view));

        // Pinned region: pinning the second row moves it to the pinned head
        // without touching sessions (testkit seam; pin commands are a later
        // slice). The shell renders the pinned region separately (§11.2).
        client.send_ok(&dsl::pin_tab(1)).expect("pin second tab");
        let _ = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        let pinned = client.snapshot().expect("pinned snapshot");
        assert_eq!(pinned.tabs.len(), 4);
        assert!(pinned.tabs[0].pinned, "pinned region leads the space");
        assert!(
            pinned.tabs[1..].iter().all(|tab| !tab.pinned),
            "regular region follows the pinned head"
        );
        assert_eq!(pinned.tabs[0].url, blank, "pinned row is the blank tab");
        assert_eq!(pinned.engine_sessions, 4, "pin never recreates sessions");
        assert_eq!(pinned.active_tab_index, Some(3));
        assert_eq!(
            pinned.tab_states[0],
            TabVisualState::Default,
            "pinned rows carry the same state vocabulary"
        );

        // Loading state: navigate the slow fixture and observe the §10.5
        // row state while the deterministic response is still pending.
        client
            .send_ok(&dsl::navigate(&slow))
            .expect("navigate slow page");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(view, true) if *view == slow_view),
            since,
        );
        let loading = client.snapshot().expect("loading snapshot");
        assert_eq!(loading.active_tab_index, Some(3));
        assert!(loading.tabs[3].loading, "session loading flag observable");
        assert_eq!(loading.tab_states[3], TabVisualState::Loading);
        since = wait_title(&mut client, slow_view, SLOW_TITLE, since);
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(view, false) if *view == slow_view),
            since,
        );
        let loaded = client.snapshot().expect("loaded snapshot");
        assert!(!loaded.tabs[3].loading);
        assert_eq!(loaded.tab_states[3], TabVisualState::Selected);

        // Crash state: the engine event rides the real event transport; the
        // shell must show the §10.7 error state for the row and the crash
        // panel surface, with the session destroyed.
        client
            .send_ok(&dsl::inject_web_event(WebEvent::PageCrashed(slow_view)))
            .expect("inject crash");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::PageCrashed(view) if *view == slow_view),
            since,
        );
        let crashed = client.snapshot().expect("crashed snapshot");
        assert!(
            crashed.tabs[3].crashed,
            "crash flag survives session teardown"
        );
        assert_eq!(crashed.tabs[3].web_view, None, "session destroyed");
        assert_eq!(crashed.active_web_view, None, "no native surface presented");
        assert_eq!(crashed.engine_sessions, 3);
        assert_eq!(crashed.tab_states[3], TabVisualState::Error);

        // Recovery: reload reactivates the tab with a fresh session and the
        // preserved URL; the crash state clears. The mutation is synchronous,
        // so the immediate snapshot observes the fresh incarnation.
        client.send_ok(&dsl::reload()).expect("reload crashed tab");
        let recovered = client.snapshot().expect("recovered snapshot");
        assert_eq!(recovered.tabs.len(), 4);
        assert_eq!(recovered.engine_sessions, 4);
        assert_eq!(recovered.active_tab_index, Some(3));
        assert!(
            !recovered.tabs[3].crashed,
            "fresh session clears crash state"
        );
        assert_eq!(
            recovered.tabs[3].url, slow,
            "preserved URL drives recreation"
        );
        let fresh_view = recovered.active_web_view.expect("fresh view");
        assert_ne!(fresh_view, slow_view, "fresh incarnation");
        since = wait_title(&mut client, fresh_view, SLOW_TITLE, since);

        // Back to the first tab: warm identity must still hold everywhere.
        client
            .send_ok(&dsl::click_tab(1))
            .expect("click page-one row");
        let _ = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let back = client.snapshot().expect("back snapshot");
        assert_eq!(back.active_web_view, Some(first_view));
        assert_eq!(back.tabs[1].web_view, Some(first_view));
        assert_eq!(back.engine_sessions, 4);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
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
