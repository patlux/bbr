//! Blueprint D10 real-app E2E (boards M-6, M-7, PR-22): History and Clear
//! Browsing Data.
//!
//! A visit opened from the command bar lists under Today with its time; a
//! visit restored from a pre-D10 store lists under Earlier without one.
//! ⌘Y opens History over the content card (the native slot is hidden, not
//! advertised); typed keys search; ↓ and Delete remove a visit; Recently
//! closed reopens a closed tab with its URL; the command bar's Show All History
//! and History › Show All History open it too. Settings › Privacy and ⌘⇧⌫
//! open Clear Browsing Data; Escape closes the range menu, then the sheet,
//! then History; Return clears the last day's visits and the profile's
//! cookies, which the fixture's cookie echo proves on a real WKWebView.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_persistence::{CommandHistoryEntry, Repository};
use bbr_testkit::protocol::{HistoryInput, HistorySnapshot, SettingsInput, Snapshot};
use bbr_testkit::Request;
use e2e_common::*;

const TAG: &str = "history";

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("keystroke");
}

fn typed(client: &mut Client, text: &str) {
    for ch in text.chars() {
        key(client, &format!("{ch}->{ch}"));
    }
}

fn input(client: &mut Client, input: HistoryInput) {
    client
        .send_ok(&Request::HistoryInput { input })
        .expect("history input");
}

fn snapshot(client: &mut Client) -> Snapshot {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
    client.snapshot().expect("snapshot")
}

fn wait(client: &mut Client, what: &str, done: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = snapshot(client);
        if done(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "{what} never happened");
    }
}

fn history(snapshot: &Snapshot) -> &HistorySnapshot {
    snapshot.history.as_ref().expect("History is open")
}

fn titles(history: &HistorySnapshot) -> Vec<String> {
    history.groups.iter().map(|g| g.title.clone()).collect()
}

/// Seed a store whose one visit predates D10 (a recency counter, no time).
fn seed(profile: &std::path::Path, url: &str) {
    std::fs::create_dir_all(profile).unwrap();
    let repository = Repository::new(profile.join("bbr.sqlite"));
    repository.initialize_and_restore().unwrap();
    repository
        .store_command_history(&[CommandHistoryEntry {
            id: 1,
            title: "Seeded page".into(),
            url: url.into(),
            visit_count: 3,
            last_visited: 7,
        }])
        .unwrap();
}

#[test]
#[ignore]
fn history_lists_visits_reopens_closed_tabs_and_clears_data() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, page_two) = fixtures();
    let set_alpha = fixtures.url("/__fx/cookie/set-alpha");
    let read_cookie = fixtures.url("/__fx/cookie/read");
    seed(
        &harness_root(TAG, std::process::id()).join("profile"),
        &page_two,
    );
    let (mut harness, socket, token) = spawn_harness(TAG, &set_alpha);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let mut since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);
        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first view");
        if settled.title.as_deref() != Some("Cookie Set alpha") {
            since = wait_title(&mut client, first_view, "Cookie Set alpha", since);
        }
        assert_eq!(settled.store.history_count, 1, "the seeded visit");

        // A visit from the command bar: ⌘T, the URL, Return.
        key(&mut client, "cmd-t");
        client
            .send_ok(&Request::SetCommandBarText {
                text: read_cookie.clone(),
            })
            .expect("url");
        client
            .send_ok(&Request::FlushCommandRanking)
            .expect("flush ranking");
        key(&mut client, "enter");
        let opened = wait(&mut client, "the visited tab", |s| {
            s.tabs.len() == 2 && s.store.history_count == 2
        });
        let read_view = opened.active_web_view.expect("read view");
        since = wait_title(&mut client, read_view, "Cookie alpha", since);

        // Close it: Recently closed holds it.
        key(&mut client, "cmd-w");
        wait(&mut client, "the closed tab", |s| s.tabs.len() == 1);

        // ⌘Y: History covers the content card.
        key(&mut client, "cmd-y");
        let open = wait(&mut client, "History", |s| s.history.is_some());
        let page = history(&open);
        assert_eq!(page.view, "all");
        assert_eq!(page.range, "all");
        assert_eq!(page.closed_count, 1);
        assert!(page.focused, "History holds the keyboard");
        assert_eq!(titles(page), ["Today", "Earlier"]);
        let visit = &page.groups[0].rows[0];
        assert!(read_cookie.ends_with(&visit.url), "{visit:?}");
        assert!(visit.time.is_some(), "a D10 visit shows its time");
        let seeded = &page.groups[1].rows[0];
        assert_eq!(seeded.title, "Seeded page");
        assert_eq!(seeded.time, None, "a pre-D10 visit has no time");
        assert!(open.visible_ids.iter().any(|id| id == "bbr-history"));
        assert!(
            !open
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-native-web-slot")),
            "the native slot is hidden under History"
        );
        assert!(!open.library.open, "History is not the Saved Pages page");

        // Typed keys search; Backspace clears.
        typed(&mut client, "seeded");
        let searched = snapshot(&mut client);
        assert_eq!(history(&searched).search, "seeded");
        assert_eq!(titles(history(&searched)), ["Earlier"]);
        for _ in 0.."seeded".len() {
            key(&mut client, "backspace");
        }

        // The Today range hides the undated visit.
        input(&mut client, HistoryInput::Range { range: 0 });
        assert_eq!(titles(history(&snapshot(&mut client))), ["Today"]);
        input(&mut client, HistoryInput::Range { range: 2 });

        // ↓ ↓ selects the seeded visit; Delete removes it.
        key(&mut client, "down");
        key(&mut client, "down");
        let selected = snapshot(&mut client);
        assert_eq!(history(&selected).selected, Some(seeded.id));
        assert!(selected
            .visible_ids
            .iter()
            .any(|id| id == &format!("bbr-history-remove-{}", seeded.id)));
        key(&mut client, "delete");
        let removed = snapshot(&mut client);
        assert_eq!(removed.store.history_count, 1);
        assert_eq!(titles(history(&removed)), ["Today"]);

        // Recently closed reopens the tab with its URL.
        input(&mut client, HistoryInput::View { view: 1 });
        let closed = snapshot(&mut client);
        let closed_row = &history(&closed).groups[0].rows[0];
        assert_eq!(history(&closed).groups[0].title, "Recently closed");
        input(&mut client, HistoryInput::Open { id: closed_row.id });
        let reopened = wait(&mut client, "the reopened tab", |s| {
            s.history.is_none() && s.tabs.len() == 2
        });
        let reopened_view = reopened.active_web_view.expect("reopened view");
        assert_eq!(
            reopened.tabs[reopened.active_tab_index.unwrap()].url,
            read_cookie
        );
        since = wait_title(&mut client, reopened_view, "Cookie alpha", since);

        // Settings › Privacy › Clear… opens the sheet over History in the
        // browser window; Cancel and Escape leave everything as it was.
        client.send_ok(&Request::OpenSettings).expect("settings");
        client
            .send_ok(&Request::SettingsInput {
                input: SettingsInput::Pane {
                    pane: "privacy".into(),
                },
            })
            .expect("privacy");
        client
            .send_ok(&Request::SettingsInput {
                input: SettingsInput::Press {
                    row: "privacy-clear-data".into(),
                },
            })
            .expect("clear");
        let from_settings = wait(&mut client, "the sheet from Settings", |s| {
            s.history.as_ref().is_some_and(|h| h.clear.is_some())
        });
        assert_sheet_centered(&mut client, "bbr-clear-data", "bbr-clear-data-veil", 460.0);
        assert_eq!(
            history(&from_settings).clear.as_ref().unwrap().profile,
            "Default"
        );
        client
            .send_ok(&Request::CloseSettings)
            .expect("close settings");
        input(&mut client, HistoryInput::ClearCancel);
        key(&mut client, "escape");
        wait(&mut client, "History closed", |s| s.history.is_none());

        // The command bar's Show All History and the History menu open it too.
        key(&mut client, "cmd-t");
        client
            .send_ok(&Request::SetCommandBarText {
                text: "> show all history".into(),
            })
            .expect("query");
        client
            .send_ok(&Request::FlushCommandRanking)
            .expect("flush ranking");
        let ranked = snapshot(&mut client);
        assert_eq!(
            ranked.command_results.first().map(|row| row.key.as_str()),
            Some("action:17"),
            "{:?}",
            ranked.command_results
        );
        key(&mut client, "enter");
        wait(&mut client, "History from the command bar", |s| {
            s.history.is_some()
        });
        key(&mut client, "escape");
        wait(&mut client, "History closed", |s| s.history.is_none());
        client
            .send_ok(&Request::InvokeMenuItem {
                menu: "History".into(),
                item: "Show All History".into(),
            })
            .expect("menu item");
        wait(&mut client, "History from the menu", |s| {
            s.history.is_some()
        });
        key(&mut client, "escape");
        wait(&mut client, "History closed", |s| s.history.is_none());

        // ⌘⇧⌫ opens History with the sheet.
        key(&mut client, "cmd-shift-backspace");
        let sheet = wait(&mut client, "Clear Browsing Data", |s| {
            s.history.as_ref().is_some_and(|h| h.clear.is_some())
        });
        let clear = history(&sheet).clear.clone().unwrap();
        assert_eq!(clear.range, "Last 24 hours");
        assert_eq!(clear.options[0].label, "Browsing history · 1 page");
        assert!(clear.options[0].checked && clear.options[1].checked);
        assert!(!clear.options[3].enabled, "downloads need All time");
        assert!(sheet.visible_ids.iter().any(|id| id == "bbr-clear-data"));

        // Escape closes the range menu first, then keeps the sheet.
        input(&mut client, HistoryInput::ClearRangeMenu);
        assert!(
            history(&snapshot(&mut client))
                .clear
                .as_ref()
                .unwrap()
                .range_open
        );
        key(&mut client, "escape");
        let menu_closed = snapshot(&mut client);
        let clear = history(&menu_closed).clear.as_ref().unwrap();
        assert!(!clear.range_open);
        input(&mut client, HistoryInput::ClearRange { index: 4 });
        let all_time = snapshot(&mut client);
        let clear = history(&all_time).clear.as_ref().unwrap();
        assert_eq!(clear.range, "All time");
        assert!(clear.options[3].enabled);
        input(&mut client, HistoryInput::ClearRange { index: 1 });

        // Return clears: the visit goes, a toast says so.
        key(&mut client, "enter");
        let cleared = wait(&mut client, "the clear", |s| {
            s.history.as_ref().is_some_and(|h| h.clear.is_none())
        });
        assert_eq!(cleared.store.history_count, 0);
        assert!(history(&cleared).empty.is_some());
        assert_eq!(
            cleared.toast.as_ref().map(|t| t.message.as_str()),
            Some("Cleared 24 hours of data from Default")
        );
        key(&mut client, "escape");
        wait(&mut client, "History closed", |s| s.history.is_none());

        // The profile's cookie is gone: the echo reads none.
        let until = deadline();
        loop {
            client
                .send_ok(&dsl::navigate(read_cookie.clone()))
                .expect("read again");
            let event = client
                .wait_for_event_since(
                    |event| {
                        matches!(event, Event::Web(WebEvent::TitleChanged(id, title))
                            if *id == reopened_view && title.starts_with("Cookie "))
                    },
                    since,
                    deadline(),
                )
                .expect("cookie title");
            since = event.0;
            if matches!(&event.1, Event::Web(WebEvent::TitleChanged(_, title)) if title == "Cookie none")
            {
                break;
            }
            assert!(Instant::now() < until, "the cookie survived the clear");
        }
        let _ = page_one;

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
