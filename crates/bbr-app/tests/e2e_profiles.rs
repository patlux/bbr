//! Real-app profile isolation and cross-profile native-session E2E.

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

fn wait_change(client: &mut Client, since: u64) -> (u64, BrowserChange) {
    let (seq, event) = client
        .wait_for_event_since(
            |event| matches!(event, Event::BrowserChanged { .. }),
            since,
            deadline(),
        )
        .expect("browser change");
    let Event::BrowserChanged { kind } = event else {
        unreachable!()
    };
    (seq, kind)
}

#[test]
#[ignore]
fn profiles_isolate_cookies_recreate_cross_profile_and_destroy_suspended_views() {
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(fixtures_dir).expect("fixture server");
    let set_alpha = fixtures.url("/__fx/cookie/set-alpha");
    let set_beta = fixtures.url("/__fx/cookie/set-beta");
    let read_cookie = fixtures.url("/__fx/cookie/read");

    let root = std::env::temp_dir().join(format!("bbr-e2e-profiles-{}", std::process::id()));
    let profile_dir = root.join("profile");
    std::fs::create_dir_all(&profile_dir).unwrap();
    let socket = profile_dir.join("bbr-test.sock");
    let token = format!("profiles-{}", std::process::id());
    let bin = env!("CARGO_BIN_EXE_bbr");
    let mut harness = Harness::spawn(
        bin,
        &[
            "--bbr-initial-url",
            &set_alpha,
            "--bbr-profile-dir",
            profile_dir.to_str().unwrap(),
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
        let default_profile = initial.active_profile;
        let default_space = initial.active_space;
        let default_view = initial.active_web_view.expect("default view");
        if initial.title.as_deref() != Some("Cookie Set alpha") {
            since = wait_title(&mut client, default_view, "Cookie Set alpha", since);
        }
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read alpha");
        since = wait_title(&mut client, default_view, "Cookie alpha", since);

        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_change(&mut client, since);
        since = seq;
        let (private_profile, private_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        assert_ne!(default_profile, private_profile);
        assert_ne!(default_space, private_space);

        client
            .send_ok(&dsl::activate_space(private_space))
            .expect("activate private space");
        let (seq, change) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(
            change,
            BrowserChange::SpaceActivated {
                space: private_space
            }
        );
        let empty_private = client.snapshot().expect("empty private space");
        assert_eq!(empty_private.active_profile, private_profile);
        assert!(empty_private.tabs.is_empty());
        // The §11.8 empty pane names the owning profile from the snapshot;
        // it is never a hardcoded label.
        assert_eq!(
            empty_private.active_profile_title.as_deref(),
            Some("Private")
        );
        assert!(empty_private
            .visible_ids
            .iter()
            .any(|id| id == "bbr-empty-new-tab"));

        client
            .send_ok(&dsl::open_tab(&set_beta))
            .expect("open private tab");
        let (seq, change) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(change, BrowserChange::TabOpened { index: 0 });
        let private_view = client
            .snapshot()
            .expect("private tab")
            .active_web_view
            .expect("private view");
        since = wait_title(&mut client, private_view, "Cookie Set beta", since);
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read beta");
        since = wait_title(&mut client, private_view, "Cookie beta", since);

        // Switching stores must recover each profile's own cookie and view id.
        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("activate default");
        let (seq, _) = wait_change(&mut client, since);
        since = seq;
        let default_again = client.snapshot().expect("default again");
        assert_eq!(default_again.active_web_view, Some(default_view));
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("reread alpha");
        since = wait_title(&mut client, default_view, "Cookie alpha", since);

        // A move between spaces of one profile preserves the native session.
        client
            .send_ok(&dsl::create_space(default_profile, "Second Default"))
            .expect("create second default space");
        let (seq, created) = wait_change(&mut client, since);
        since = seq;
        let second_default_space = match created {
            BrowserChange::SpaceCreated { profile, space } => {
                assert_eq!(profile, default_profile);
                space
            }
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("activate default for same-profile move");
        let (seq, _) = wait_change(&mut client, since);
        since = seq;
        client
            .send_ok(&dsl::move_tab(0, second_default_space))
            .expect("same-profile move");
        let (seq, moved) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(
            moved,
            BrowserChange::TabMoved {
                to_space: second_default_space,
                recreated: false,
            }
        );
        client
            .send_ok(&dsl::activate_space(second_default_space))
            .expect("activate same-profile destination");
        let (seq, _) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(
            client.snapshot().unwrap().active_web_view,
            Some(default_view)
        );
        client
            .send_ok(&dsl::move_tab(0, default_space))
            .expect("move default tab back");
        let (seq, moved_back) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(
            moved_back,
            BrowserChange::TabMoved {
                to_space: default_space,
                recreated: false,
            }
        );

        client
            .send_ok(&dsl::activate_space(private_space))
            .expect("activate private again");
        let (seq, _) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(
            client.snapshot().unwrap().active_web_view,
            Some(private_view)
        );
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("reread beta");
        since = wait_title(&mut client, private_view, "Cookie beta", since);

        // Moving the private tab into the default profile must destroy its old
        // session and create a fresh hidden session backed by the default store.
        let before_move = since;
        client
            .send_ok(&dsl::move_tab(0, default_space))
            .expect("move to default profile");
        let (seq, moved) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(
            moved,
            BrowserChange::TabMoved {
                to_space: default_space,
                recreated: true,
            }
        );
        let private_empty = client.snapshot().expect("private empty after move");
        assert!(private_empty.tabs.is_empty());
        assert_eq!(private_empty.engine_sessions, 2);

        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("activate move destination");
        let _ = wait_change(&mut client, since);
        let moved_snapshot = client.snapshot().expect("moved tab snapshot");
        assert_eq!(moved_snapshot.tabs.len(), 2);
        assert_eq!(moved_snapshot.active_tab_index, Some(1));
        let moved_view = moved_snapshot.active_web_view.expect("moved view");
        assert_ne!(moved_view, private_view);
        assert_ne!(moved_view, default_view);
        since = wait_title(&mut client, moved_view, "Cookie alpha", before_move);

        // Suspending the hidden original tab removes its native session. A
        // later selection recreates it in the same profile with a fresh id.
        client
            .send_ok(&dsl::suspend_tab(0))
            .expect("suspend warm tab");
        let (seq, suspended) = wait_change(&mut client, since);
        since = seq;
        assert_eq!(suspended, BrowserChange::TabSuspended { index: 0 });
        let suspended_snapshot = client.snapshot().expect("suspended snapshot");
        assert_eq!(suspended_snapshot.engine_sessions, 1);
        assert_eq!(suspended_snapshot.tabs[0].web_view, None);

        client
            .send_ok(&dsl::select_tab(0))
            .expect("reactivate suspended tab");
        let (seq, _) = wait_change(&mut client, since);
        let reactivated = client.snapshot().expect("reactivated snapshot");
        let recreated_view = reactivated.active_web_view.expect("recreated view");
        assert_ne!(recreated_view, default_view);
        assert_eq!(reactivated.engine_sessions, 2);
        let _ = wait_title(&mut client, recreated_view, "Cookie alpha", seq);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    match result {
        Ok(()) => {
            harness.success = true;
            eprintln!("PROFILES E2E PASSED");
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
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
