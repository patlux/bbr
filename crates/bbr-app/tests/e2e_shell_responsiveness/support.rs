use crate::e2e_common::*;
use bbr_testkit::protocol::{Request, Snapshot, TabVisualState, WebContentProbe};

pub const BUSY_PATH: &str = "/__fx/hang/busy";
pub const HEAVY_PATH: &str = "/__fx/hang/heavy";
pub const DIALOG_PATH: &str = "/__fx/hang/dialog";
pub const BEFOREUNLOAD_PATH: &str = "/__fx/hang/beforeunload";

pub fn start(tag: &str) -> (FixtureServer, Harness, PathBuf, String, Client, u64) {
    let (fixtures, page_one, _) = fixtures();
    let (harness, socket, token) = spawn_harness(tag, &page_one);
    let mut client = connect(&socket);
    client.hello(&token).expect("hello");
    let replay = client.attach(None).expect("attach");
    let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
    let initial = client.snapshot().expect("initial snapshot");
    let view = initial.active_web_view.expect("initial view");
    if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
        since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
    }
    (fixtures, harness, socket, token, client, since)
}

pub fn navigate(client: &mut Client, url: &str) {
    client
        .send_ok(&Request::Navigate { url: url.into() })
        .expect("navigate fixture");
}

pub fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

pub fn mark_unresponsive(client: &mut Client, view: WebViewId) {
    client
        .send_ok(&Request::InjectWebEvent {
            event: WebEvent::UnresponsiveChanged(view, true),
        })
        .expect("inject native-equivalent timeout event");
    let state = wait_snapshot(client, |snapshot| {
        snapshot
            .tabs
            .iter()
            .any(|tab| tab.web_view == Some(view) && tab.unresponsive)
    });
    let index = state
        .tabs
        .iter()
        .position(|tab| tab.web_view == Some(view))
        .expect("unresponsive tab index");
    assert_eq!(state.tab_states[index], TabVisualState::Unresponsive);
}

pub fn wait_real_unresponsive(client: &mut Client, view: WebViewId) -> Snapshot {
    client
        .send_ok(&Request::ProbeRendererResponsiveness { view })
        .expect("arm real renderer probe");
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("responsiveness snapshot");
        if snapshot
            .tabs
            .iter()
            .any(|tab| tab.web_view == Some(view) && tab.unresponsive)
        {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "real renderer probe never marked view {view:?} unresponsive"
        );
        std::thread::yield_now();
    }
}

pub fn wait_fixture_hit(fixtures: &FixtureServer, path: &str) {
    let until = deadline();
    while fixtures.hits(path) == 0 {
        assert!(Instant::now() < until, "fixture {path} was not requested");
        std::thread::yield_now();
    }
}

pub fn wait_snapshot(client: &mut Client, predicate: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("bounded snapshot poll");
        if predicate(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "snapshot predicate did not settle");
        std::thread::yield_now();
    }
}

pub fn probe_content(client: &mut Client, view: WebViewId) -> WebContentProbe {
    client
        .send_ok(&Request::ProbeWebContent { view })
        .expect("probe committed web content");
    wait_snapshot(client, |snapshot| {
        snapshot
            .web_content_probe
            .as_ref()
            .is_some_and(|probe| probe.view == view)
    })
    .web_content_probe
    .expect("web content probe")
}

pub fn semantic_focused(snapshot: &Snapshot, id: &str) -> bool {
    fn find(nodes: &[bbr_testkit::protocol::SemanticNode], id: &str) -> bool {
        nodes
            .iter()
            .any(|node| (node.id == id && node.focused) || find(&node.children, id))
    }
    find(&snapshot.semantics, id)
}

pub fn assert_unresponsive_surface(snapshot: &Snapshot, view: WebViewId) {
    assert_eq!(snapshot.active_web_view, Some(view));
    assert_ids(
        snapshot,
        &[
            "bbr-unresponsive-panel",
            "bbr-unresponsive-wait",
            "bbr-unresponsive-reload",
            "bbr-unresponsive-close",
        ],
    );
    fn contains(nodes: &[bbr_testkit::protocol::SemanticNode], id: &str) -> bool {
        nodes
            .iter()
            .any(|node| node.id == id || contains(&node.children, id))
    }
    assert!(
        contains(&snapshot.semantics, "bbr-unresponsive-panel"),
        "typed semantic tree exposes the unresponsive pane"
    );
    assert!(
        contains(&snapshot.semantics, "bbr-unresponsive-wait")
            && contains(&snapshot.semantics, "bbr-unresponsive-reload")
            && contains(&snapshot.semantics, "bbr-unresponsive-close"),
        "typed semantic tree exposes the closed recovery set"
    );
}

pub fn finish_case(
    result: std::thread::Result<()>,
    harness: &mut Harness,
    socket: &std::path::Path,
    token: &str,
) {
    finish(result, harness, socket, token);
}
