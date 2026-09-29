//! Blueprint D24 real-app E2E (board S-8, DESIGN §11.15): the sidebar
//! media card.
//!
//! A real WKWebView page plays a silent 60-second track. The card appears
//! only once the engine reports the session (WebKit's playback state plus
//! the isolated client-world probe), shows the Media Session title, the
//! site and profile, the known length and only the advertised controls
//! (no Previous/Next). Play/Pause, Seek and Mute act on the page's element
//! and come back through the next report. A navigation ends the session:
//! a command for its old identity is refused (`Ended`, then `Stale` once
//! the page plays again under a new generation). Dismiss hides the
//! session until its identity changes. Several playing tabs show one card
//! with a `+N` switch; the switch cycles, commands leave the selection
//! alone, and the title block shows the exact tab — across spaces and
//! profiles.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{BrowserChange, MediaInput, Snapshot};
use bbr_testkit::Request;
use bbr_web::media::MediaCommand;
use e2e_common::*;

const TAG: &str = "media";

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
        assert!(
            Instant::now() < until,
            "{what} never happened: {:?}",
            snapshot.media
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn press(client: &mut Client, input: MediaInput) {
    client
        .send_ok(&Request::MediaInput { input })
        .expect("media input");
}

fn host_of(url: &str) -> String {
    let rest = url.split_once("://").unwrap().1;
    rest.split([':', '/']).next().unwrap().to_string()
}

#[test]
#[ignore]
fn media_card_controls_the_playing_page() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = fixtures();
    let media = fixtures.url("/media.html");
    let (mut harness, socket, token) = spawn_harness(TAG, &media);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("media view");

        // The card appears once the engine reports the session.
        let shown = wait(&mut client, "the card", |s| {
            s.media.card.as_ref().is_some_and(|c| c.playing)
        });
        let card = shown.media.card.clone().unwrap();
        assert_eq!(card.view, view);
        assert_eq!(card.title, "Fixture Tone");
        assert_eq!(card.source, format!("{} · Default", host_of(&media)));
        assert_eq!(card.duration, Some(60.0));
        assert!(card.seekable && !card.live);
        assert_eq!(card.controls, vec!["play-pause", "mute"]);
        assert_eq!(card.others, 0);
        assert!(shown.visible_ids.iter().any(|id| id == "bbr-media-card"));
        let generation = card.generation;
        let muted = card.muted;

        // Pause and play come back through the page's own report.
        press(&mut client, MediaInput::PlayPause);
        let paused = wait(&mut client, "pause", |s| {
            s.media.card.as_ref().is_some_and(|c| !c.playing)
        });
        assert_eq!(paused.media.sessions[0].state, "paused");
        assert_eq!(paused.media.last_outcome.as_deref(), Some("Applied"));
        press(&mut client, MediaInput::PlayPause);
        wait(&mut client, "play", |s| {
            s.media.card.as_ref().is_some_and(|c| c.playing)
        });

        // Seek to the middle; mute and unmute.
        press(&mut client, MediaInput::Seek { fraction: 0.5 });
        wait(&mut client, "the seek", |s| {
            s.media
                .card
                .as_ref()
                .is_some_and(|c| c.elapsed.is_some_and(|e| (30.0..40.0).contains(&e)))
        });
        press(&mut client, MediaInput::ToggleMute);
        wait(&mut client, "mute", |s| {
            s.media.card.as_ref().is_some_and(|c| c.muted != muted)
        });
        press(&mut client, MediaInput::ToggleMute);
        wait(&mut client, "unmute", |s| {
            s.media.card.as_ref().is_some_and(|c| c.muted == muted)
        });
        // Unadvertised controls do nothing.
        press(&mut client, MediaInput::Next);
        let refused = snapshot(&mut client);
        assert_eq!(refused.media.last_outcome.as_deref(), Some("Unsupported"));
        assert_eq!(refused.active_web_view, Some(view));

        // The probe's state never reached the page's own world.
        assert_eq!(refused.title.as_deref(), Some("BBR Media"));

        // Dismiss hides the session; playback continues.
        press(&mut client, MediaInput::Dismiss);
        let dismissed = snapshot(&mut client);
        assert!(dismissed.media.card.is_none());
        assert_eq!(dismissed.media.sessions.len(), 1);
        assert!(!dismissed
            .visible_ids
            .iter()
            .any(|id| id == "bbr-media-card"));

        // A navigation ends the session; its identity is refused.
        client.send_ok(&dsl::navigate(&page_one)).expect("navigate");
        wait(&mut client, "the session to end", |s| {
            s.media.sessions.is_empty()
        });
        let old = MediaInput::Command {
            view,
            generation,
            command: MediaCommand::Pause,
        };
        press(&mut client, old.clone());
        assert_eq!(
            snapshot(&mut client).media.last_outcome.as_deref(),
            Some("Ended")
        );

        // The page plays again: a new identity, so the card is back and
        // the old one is stale.
        client
            .send_ok(&dsl::navigate(&media))
            .expect("navigate back");
        let again = wait(&mut client, "the new session", |s| {
            s.media.card.as_ref().is_some_and(|c| c.playing)
        });
        assert!(again.media.card.as_ref().unwrap().generation > generation);
        press(&mut client, old);
        let stale = snapshot(&mut client);
        assert_eq!(stale.media.last_outcome.as_deref(), Some("Stale"));
        assert!(stale.media.card.as_ref().unwrap().playing, "still playing");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn several_playing_tabs_share_one_card() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, _page_one, _page_two) = fixtures();
    let one = fixtures.url("/media.html?title=One");
    let two = fixtures.url("/media.html?title=Two");
    let (mut harness, socket, token) = spawn_harness(TAG, &one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);
        wait_for_window_settled(&mut client, deadline());
        wait(&mut client, "the first card", |s| {
            s.media.card.as_ref().is_some_and(|c| c.title == "One")
        });
        let home_space = snapshot(&mut client).active_space;

        // A second playing tab: one card, the newest session, `+1`.
        client.send_ok(&dsl::open_tab(&two)).expect("open tab");
        let both = wait(&mut client, "two sessions", |s| {
            s.media.sessions.len() == 2 && s.media.card.as_ref().is_some_and(|c| c.title == "Two")
        });
        assert_eq!(both.media.card.as_ref().unwrap().others, 1);
        assert_eq!(
            both.visible_ids
                .iter()
                .filter(|id| *id == "bbr-media-card")
                .count(),
            1,
            "never a stack"
        );
        assert_eq!(both.active_tab_index, Some(1));

        // The switch cycles; a command leaves the selection alone.
        press(&mut client, MediaInput::Cycle);
        let cycled = snapshot(&mut client);
        assert_eq!(cycled.media.card.as_ref().unwrap().title, "One");
        press(&mut client, MediaInput::PlayPause);
        let paused = wait(&mut client, "One paused", |s| {
            s.media
                .card
                .as_ref()
                .is_some_and(|c| c.title == "One" && !c.playing)
        });
        assert_eq!(paused.active_tab_index, Some(1), "selection kept");
        let two_view = paused.tabs[1].web_view.expect("Two's view");
        assert!(
            paused
                .media
                .sessions
                .iter()
                .any(|s| s.view == two_view && s.state == "playing"),
            "only the bound session paused"
        );

        // In another profile's space, the card stays and its title block
        // shows the exact tab back in its own space and profile.
        client
            .send_ok(&dsl::create_profile("Work", "Work"))
            .expect("create profile");
        let (_, change) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        let BrowserChange::ProfileCreated { space: work, .. } = change else {
            unreachable!("profile created")
        };
        client
            .send_ok(&dsl::activate_space(work))
            .expect("activate work");
        let away = wait(&mut client, "the work space", |s| s.active_space == work);
        assert!(away.media.card.as_ref().is_some_and(|c| c.title == "One"));
        press(&mut client, MediaInput::Source);
        let back = wait(&mut client, "the source tab", |s| {
            s.active_space == home_space && s.active_tab_index == Some(0)
        });
        assert_eq!(back.tabs[0].url, one);

        // Closing a playing tab removes its session from the card.
        client.send_ok(&dsl::close_tab(1)).expect("close Two");
        let single = wait(&mut client, "one session", |s| {
            s.media.card.as_ref().is_some_and(|c| c.others == 0)
        });
        assert_eq!(single.media.card.unwrap().title, "One");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
