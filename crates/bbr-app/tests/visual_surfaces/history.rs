//! History stills use keyboard filtering and the real tab-close authority.
use super::{key, save, wait};
use bbr_testkit::{protocol::HistoryInput, Client, Request};
use std::path::Path;

pub fn filters(client: &mut Client, output: &Path, appearance: &str) {
    for ch in "architecture".chars() {
        key(client, &format!("{ch}->{ch}"));
    }
    let filtered = wait(client, |s| {
        s.history
            .as_ref()
            .is_some_and(|h| h.search == "architecture")
    });
    let groups = &filtered.history.as_ref().unwrap().groups;
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].rows.len(), 1);
    assert_eq!(groups[0].rows[0].title, "Browser architecture notes");
    save(
        client,
        output,
        "history-search",
        appearance,
        false,
        filtered,
    );
    for _ in "architecture".chars() {
        key(client, "backspace");
    }
    for ch in "no-match".chars() {
        key(client, &format!("{ch}->{ch}"));
    }
    let empty = wait(client, |s| {
        s.history
            .as_ref()
            .is_some_and(|h| h.search == "no-match" && h.empty.is_some())
    });
    assert!(empty.history.as_ref().unwrap().groups.is_empty());
    save(
        client,
        output,
        "history-search-empty",
        appearance,
        false,
        empty,
    );
    for _ in "no-match".chars() {
        key(client, "backspace");
    }
    wait(client, |s| {
        s.history.as_ref().is_some_and(|h| h.search.is_empty())
    });
}

pub fn closed(client: &mut Client, output: &Path, appearance: &str) {
    let before = client.snapshot().unwrap();
    // Fixed data documents avoid exposing the ephemeral fixture server port in
    // History. Real WKWebViews load them; no host/network or title injection.
    for (url, title) in [
        (
            "data:text/html,%3Ctitle%3EReading%20notes%3C/title%3E",
            "Reading notes",
        ),
        (
            "data:text/html,%3Ctitle%3EDesign%20reference%3C/title%3E",
            "Design reference",
        ),
    ] {
        client
            .send_ok(&Request::OpenTab { url: url.into() })
            .unwrap();
        wait(client, |s| {
            !s.loading && s.title.as_deref() == Some(title) && s.tabs.len() == 5
        });
        key(client, "cmd-w");
        wait(client, |s| s.tabs.len() == 4);
    }
    client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
    key(client, "cmd-y");
    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::View { view: 1 },
        })
        .unwrap();
    let closed = wait(client, |s| {
        s.history
            .as_ref()
            .is_some_and(|h| h.view == "closed" && h.closed_count == 2)
    });
    let page = closed.history.as_ref().unwrap();
    assert_eq!(page.groups.len(), 1);
    assert_eq!(page.groups[0].title, "Recently closed");
    assert_eq!(
        page.groups[0]
            .rows
            .iter()
            .map(|r| r.title.as_str())
            .collect::<Vec<_>>(),
        ["Design reference", "Reading notes"]
    );
    assert!(page.groups[0].rows.iter().all(|r| r.time.is_none()));
    let mut expected_tabs = before.tabs;
    // Each close reactivates the primary tab; the final explicit selection
    // also advances the production recency counter. All other fields stay exact.
    expected_tabs[0].last_used += 3;
    assert_eq!(closed.tabs, expected_tabs);
    assert_eq!(closed.engine_sessions, before.engine_sessions);
    save(
        client,
        output,
        "history-recently-closed",
        appearance,
        false,
        closed,
    );
    key(client, "escape");
    wait(client, |s| s.history.is_none());
}
