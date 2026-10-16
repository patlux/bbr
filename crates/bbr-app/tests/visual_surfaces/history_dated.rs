//! Dated History stills (formerly withheld, 2026-10-12): a fresh profile
//! seeded with fixed visits across five local days plus two pre-D10 visits,
//! and the display clock pinned to Tuesday 2026-09-29 12:00 UTC, so day
//! groups, visit times and the Today range never depend on the guest's clock
//! or zone. Real app and History controls; Clear Browsing Data runs for All
//! time last.
use super::{key, save, wait, Client, PAGE};
use crate::e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, AssertUnwindSafe, Harness,
};
use bbr_persistence::{CommandHistoryEntry, Repository};
use bbr_testkit::protocol::{dsl, HistoryInput, Request, ThemeModeOverride};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Tuesday 2026-09-29 12:00:00 UTC.
const NOW_MS: u64 = 1_790_683_200_000;
const MINUTE_MS: u64 = 60_000;
const DAY_MS: u64 = 24 * 60 * MINUTE_MS;
/// Clear Browsing Data's "All time" range.
const ALL_TIME: usize = 4;

/// Five visits a day for five days, newest first: (title, URL).
const VISITS: [[(&str, &str); 5]; 5] = [
    [
        ("GPUI layout guide", "https://docs.example.test/gpui/layout"),
        (
            "Release notes 0.9",
            "https://news.example.test/releases/0-9",
        ),
        (
            "Design review agenda",
            "https://team.example.test/design-review",
        ),
        (
            "Metal shading primer",
            "https://docs.example.test/metal/shading",
        ),
        ("Coffee order", "https://cafe.example.test/order"),
    ],
    [
        (
            "Browser architecture notes",
            "https://reading.example.test/browser",
        ),
        ("Tart image checklist", "https://ops.example.test/tart"),
        ("Weather in Hamburg", "https://weather.example.test/hamburg"),
        ("Type scale reference", "https://design.example.test/type"),
        ("Pull request 412", "https://code.example.test/pr/412"),
    ],
    [
        (
            "Weekend reading list",
            "https://weekend.example.test/reading",
        ),
        ("Train timetable", "https://rail.example.test/timetable"),
        (
            "Recipe: lentil soup",
            "https://food.example.test/lentil-soup",
        ),
        ("Photo backup status", "https://photos.example.test/backup"),
        ("Bike repair shops", "https://map.example.test/bike-repair"),
    ],
    [
        ("Accessibility audit", "https://a11y.example.test/audit"),
        (
            "Keyboard shortcuts sheet",
            "https://docs.example.test/shortcuts",
        ),
        (
            "Invoice 2026-091",
            "https://billing.example.test/invoice/2026-091",
        ),
        (
            "Conference schedule",
            "https://events.example.test/schedule",
        ),
        ("Plant care notes", "https://garden.example.test/care"),
    ],
    [
        ("Sprint retrospective", "https://team.example.test/retro"),
        ("Font licensing FAQ", "https://type.example.test/licensing"),
        (
            "Library opening hours",
            "https://library.example.test/hours",
        ),
        ("Rust release blog", "https://blog.example.test/rust"),
        ("Packing list", "https://travel.example.test/packing"),
    ],
];

/// Minutes before noon of each slot: 11:40, 10:15, 09:05, 08:30, 07:50.
const SLOTS: [u64; 5] = [20, 105, 175, 210, 250];

fn seed(profile: &Path) {
    let repository = Repository::new(profile.join("bbr.sqlite"));
    repository
        .initialize_and_restore()
        .expect("initialize dated fixture store");
    let mut entries = Vec::new();
    for (day, visits) in VISITS.iter().enumerate() {
        for (slot, (title, url)) in visits.iter().enumerate() {
            entries.push(CommandHistoryEntry {
                id: (day * 10 + slot + 1) as u64,
                title: (*title).into(),
                url: (*url).into(),
                visit_count: 1,
                last_visited: NOW_MS - day as u64 * DAY_MS - SLOTS[slot] * MINUTE_MS,
            });
        }
    }
    // Pre-D10 visits carry a small counter, no time: listed under Earlier.
    for (id, title, url) in [
        (
            91,
            "Archived handbook",
            "https://archive.example.test/handbook",
        ),
        (92, "Old team wiki", "https://wiki.example.test/team"),
    ] {
        entries.push(CommandHistoryEntry {
            id,
            title: title.into(),
            url: url.into(),
            visit_count: 1,
            last_visited: id,
        });
    }
    repository
        .store_command_history(&entries)
        .expect("seed fixed dated history without navigating external URLs");
}

pub fn capture() {
    let _serial = crate::e2e_common::serial_gui();
    for (appearance, mode) in [
        ("light", ThemeModeOverride::Light),
        ("dark", ThemeModeOverride::Dark),
    ] {
        let tag = format!("visual-history-{appearance}");
        let root = harness_root(&tag, std::process::id());
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        seed(&profile);
        let socket = harness_socket_path(&tag, std::process::id());
        let token = format!("visual-history-{}", std::process::id());
        let output = PathBuf::from(std::env::var_os("BBR_VISUAL_CAPTURE_DIR").unwrap());
        fs::create_dir_all(&output).unwrap();
        let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").unwrap();
        let mut harness = Harness::spawn(
            &bin,
            &[
                "--bbr-initial-url",
                "data:text/html,<title>BBR%20Visual%20History</title><p>History%20fixture</p>",
                "--bbr-profile-dir",
                profile.to_str().unwrap(),
                "--bbr-test-token",
                &token,
                "--bbr-ids=sequential",
                "--bbr-hidden",
                "--bbr-window-size=1080x1407",
            ],
            &root,
        );
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut client = connect(&socket);
            client.hello(&token).unwrap();
            client.attach(None).unwrap();
            client
                .send_ok(&dsl::set_appearance_override(Some(mode)))
                .unwrap();
            client
                .send_ok(&dsl::set_display_accessibility(
                    Some(false),
                    Some(true),
                    Some(false),
                ))
                .unwrap();
            client
                .send_ok(&Request::SetWindowSize {
                    width: 1080.0,
                    height: 1407.0,
                })
                .unwrap();
            crate::e2e_common::wait_for_window_settled(&mut client, deadline());
            client.send_ok(&Request::FocusWeb).unwrap();
            wait(&mut client, |s| {
                !s.loading
                    && s.title.as_deref() == Some("BBR Visual History")
                    && s.tabs.iter().all(|tab| !tab.unresponsive && !tab.crashed)
            });
            client
                .send_ok(&Request::SetDisplayClock {
                    now_ms: Some(NOW_MS),
                    offset_s: 0,
                })
                .expect("pin the display clock");
            dated(&mut client, &output, appearance);
            client.quit().unwrap();
            crate::wait_for_process_exit(&mut harness, deadline());
        }));
        match result {
            Ok(()) => harness.success = true,
            Err(panic) => {
                harness.diagnostics(&socket, &token);
                std::panic::resume_unwind(panic);
            }
        }
    }
}

fn dated(client: &mut Client, output: &Path, appearance: &str) {
    key(client, "cmd-y");
    let all = wait(client, |s| {
        s.history
            .as_ref()
            .is_some_and(|h| h.range == "all" && h.groups.len() == 6)
    });
    let page = all.history.as_ref().unwrap();
    let titles: Vec<_> = page.groups.iter().map(|g| g.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "Today",
            "Yesterday",
            "Sunday, 27 September",
            "Saturday, 26 September",
            "Friday, 25 September",
            "Earlier"
        ]
    );
    let times: Vec<_> = page.groups[0]
        .rows
        .iter()
        .map(|r| r.time.as_deref().unwrap_or_default())
        .collect();
    assert_eq!(times, ["11:40", "10:15", "09:05", "08:30", "07:50"]);
    assert!(page.groups[5].rows.iter().all(|r| r.time.is_none()));
    save(
        client,
        output,
        "history-dated",
        appearance,
        PAGE,
        all.clone(),
    );

    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::Range { range: 0 },
        })
        .unwrap();
    let today = wait(client, |s| {
        s.history
            .as_ref()
            .is_some_and(|h| h.range == "today" && h.groups.len() == 1)
    });
    assert_eq!(today.history.as_ref().unwrap().groups[0].rows.len(), 5);
    save(
        client,
        output,
        "history-range-today",
        appearance,
        PAGE,
        today,
    );
    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::Range { range: 2 },
        })
        .unwrap();
    wait(client, |s| {
        s.history.as_ref().is_some_and(|h| h.range == "all")
    });

    let second = page.groups[0].rows[1].id;
    key(client, "down");
    key(client, "down");
    let selected = wait(client, |s| {
        s.history
            .as_ref()
            .is_some_and(|h| h.selected == Some(second))
    });
    save(
        client,
        output,
        "history-selected",
        appearance,
        PAGE,
        selected,
    );

    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::ClearOpen,
        })
        .unwrap();
    wait(client, |s| {
        s.history.as_ref().is_some_and(|h| h.clear.is_some())
    });
    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::ClearRange { index: ALL_TIME },
        })
        .unwrap();
    wait(client, |s| {
        s.history
            .as_ref()
            .and_then(|h| h.clear.as_ref())
            .is_some_and(|c| c.range == "All time" && c.confirm_enabled)
    });
    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::ClearConfirm,
        })
        .unwrap();
    let cleared = wait(client, |s| {
        s.history
            .as_ref()
            .is_some_and(|h| h.clear.is_none() && h.groups.is_empty() && h.empty.is_some())
    });
    save(client, output, "history-cleared", appearance, PAGE, cleared);
    key(client, "escape");
    wait(client, |s| s.history.is_none());
}
