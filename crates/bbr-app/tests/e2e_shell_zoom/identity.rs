//! Blueprint D03 (DESIGN §11.5): the address control's identity states over
//! a real WKWebView. The loopback fixture is a secure context (no mark); a
//! plain-http site shows the not-secure mark with its domain and is
//! announced as "Not secure"; a local file shows its file name — never the
//! path — behind the document mark. Internal-page naming (`bbr://…`) is
//! proven by the unit and gallery evidence until the first internal page
//! is served (D10/D15). Every step synchronizes on typed snapshot state
//! under a deadline — no sleeps.

use crate::e2e_common::*;
use crate::support::*;

const MARK_ID: &str = "bbr-address-mark";

/// Navigate the active tab and wait until the address reports `identity`.
fn navigate_until(
    client: &mut Client,
    url: &str,
    identity: &str,
) -> bbr_testkit::protocol::Snapshot {
    client
        .send_ok(&dsl::navigate(url.to_string()))
        .expect("navigate");
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush");
        let snapshot = client.snapshot().expect("identity snapshot");
        if snapshot.url.as_deref() == Some(url) && snapshot.address_identity == identity {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "address never reported {identity} for {url}: {} / {}",
            snapshot.address_identity,
            snapshot.address_domain
        );
    }
}

fn address_value(snapshot: &bbr_testkit::protocol::Snapshot) -> Option<String> {
    fn find(
        nodes: &[bbr_testkit::protocol::SemanticNode],
    ) -> Option<&bbr_testkit::protocol::SemanticNode> {
        for node in nodes {
            if node.id == "bbr-address" {
                return Some(node);
            }
            if let Some(found) = find(&node.children) {
                return Some(found);
            }
        }
        None
    }
    find(&snapshot.semantics)
        .expect("address node")
        .value
        .clone()
}

#[test]
#[ignore]
fn address_marks_not_secure_sites_and_names_local_files() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let fixtures = zoom_fixtures();
    let (mut harness, socket, token) = spawn_harness("address-identity", &fixtures.a1);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let (_baseline, view) = start(&mut client, PAGE_ONE_TITLE, since);

        // The loopback fixture is a secure context: domain only, no mark.
        let secure = client.snapshot().expect("secure snapshot");
        assert_eq!(secure.address_identity, "secure");
        assert_eq!(secure.address_domain, "127.0.0.1");
        assert!(!secure.visible_ids.iter().any(|id| id == MARK_ID));
        assert_eq!(address_value(&secure), None);

        // Plain http off loopback: the not-secure mark, spoken too.
        let plain = navigate_until(&mut client, "http://bbr-not-secure.invalid/", "not-secure");
        assert_eq!(plain.address_domain, "bbr-not-secure.invalid");
        assert!(plain.visible_ids.iter().any(|id| id == MARK_ID));
        assert_eq!(address_value(&plain).as_deref(), Some("Not secure"));

        // A local file: the file name only, behind the document mark.
        let file_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("bbr-fixtures")
            .join("html")
            .join("page2.html")
            .canonicalize()
            .expect("fixture path");
        let file_url = format!("file://{}", file_path.display());
        let file = navigate_until(&mut client, &file_url, "file");
        assert_eq!(file.address_domain, "page2.html", "never the path");
        assert!(file.visible_ids.iter().any(|id| id == MARK_ID));
        assert_eq!(address_value(&file).as_deref(), Some("Local file"));
        assert_eq!(file.active_web_view, Some(view), "same session throughout");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
