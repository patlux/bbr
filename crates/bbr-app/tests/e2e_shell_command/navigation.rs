//! Real-WebKit content-arrival E2E for Cmd-T destinations (the
//! schemeless-navigation regression).
//!
//! The product owner reproduced on the packaged app: Cmd-T → "https://…"
//! → Enter works, while Cmd-T → "golem.de" (no scheme) → Enter leaves a
//! blank white content area. The shell's geometry, state, and visibility
//! bookkeeping were all green while the page was blank — no test ever
//! asserted that web content actually arrived.
//!
//! One scenario drives the EXACT production activation (Cmd-T palette,
//! typed text, Enter) with both inputs from the bug report: the same
//! contract must hold for both — real WebKit content evidence, never our
//! own flags:
//!
//! - a client-content-world probe (`document.readyState` / `document.URL` /
//!   `document.title` + the native `WKWebView` url/title) for committed
//!   content;
//! - the engine's `UrlChanged` (emitted from WebKit's
//!   `didStartProvisionalNavigation` with the provisional URL) for an
//!   accepted navigation — network-independent.
//!
//! Phase A is the regression: "golem.de" (no scheme) must arrive like its
//! scheme'd twin. The command-activation path normalizes the scheme-less
//! input to its absolute `https://` destination before the controller
//! navigates, so the engine accepts a real navigation and the visit is
//! recorded in the app-owned history provider (Phase 7). Phase B ("https://
//! golem.de") is the scheme'd twin — and, deterministically, the typed
//! destination may now rank the *recorded history row* first (`history:`
//! with an `OpenHistory` activation) instead of the navigate fallback; both
//! rows must still open the intended golem destination through production.
//! The fresh destination tab is closed between the phases so the tab corpus
//! stays clean; the recorded history visit deliberately stays, which is
//! exactly what Phase B's history-row path proves. No sleeps: every step is
//! deadline-bounded snapshot/event synchronization.

use super::*;

/// Open the Cmd-T palette, type `text` through the production text path,
/// flush the ranking debounce deterministically, and assert the staged
/// NewTab surface published its results.
fn cmd_t_type(client: &mut Client, since: &mut u64, text: &str) {
    key(client, "cmd-t");
    *since = wait_chrome_focus(client, true, *since);
    client
        .send_ok(&bbr_testkit::Request::SetCommandBarText { text: text.into() })
        .expect("set command text");
    client
        .send_ok(&bbr_testkit::Request::FlushCommandRanking)
        .expect("flush ranking");
    let snapshot = client.snapshot().expect("cmd-t typed snapshot");
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::NewTab);
    assert_eq!(
        snapshot.command_surface,
        bbr_testkit::protocol::CommandSurface::Palette
    );
    assert_eq!(snapshot.command_result_state, CommandResultState::Results);
}

/// Press Enter in the Cmd-T surface and return the post-activation snapshot
/// (a frame flush observes render-derived fields like the native content
/// rect in the hidden harness window).
fn enter_cmd_t(client: &mut Client) -> bbr_testkit::protocol::Snapshot {
    key(client, "enter");
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush after enter");
    client.snapshot().expect("post-enter snapshot")
}

/// Poll a real WebKit content probe for `view` until `satisfied`, re-issuing
/// the probe each iteration (so a mid-load document advances) under an
/// explicit deadline. Panics with the last observed probe state — the blank
/// `about:blank`/nil evidence that is exactly the product bug.
fn wait_content_probe(
    client: &mut Client,
    view: WebViewId,
    satisfied: impl Fn(&bbr_testkit::protocol::WebContentProbe) -> bool,
    what: &str,
) -> bbr_testkit::protocol::WebContentProbe {
    let deadline = deadline();
    loop {
        client
            .send_ok(&bbr_testkit::Request::ProbeWebContent { view })
            .expect("issue content probe");
        let snapshot = client.snapshot().expect("probe snapshot");
        if let Some(probe) = snapshot.web_content_probe.as_ref() {
            if probe.view == view && satisfied(probe) {
                return probe.clone();
            }
        }
        assert!(
            Instant::now() < deadline,
            "{what} — content probe for view {view:?} never satisfied; \
             last probe: {:?}",
            snapshot.web_content_probe
        );
        std::thread::yield_now();
    }
}

/// One Cmd-T destination activation: stage the Cmd-T palette, type `input`,
/// assert the ranked first row satisfies `expect_first` (the navigate
/// fallback for the scheme-less regression input, or the recorded history
/// row for a re-typed destination — both activate the same intended
/// destination), press Enter, and assert the activation aftermath (new tab,
/// fresh active view, palette closed, web focus, non-zero native geometry).
/// Returns the fresh view.
fn activate_cmd_t_destination(
    client: &mut Client,
    since: &mut u64,
    input: &str,
    tabs_before: usize,
    expect_first: impl Fn(&bbr_testkit::protocol::CommandResultSnapshot, &str) -> bool,
    what: &str,
) -> (WebViewId, bbr_testkit::protocol::Snapshot) {
    cmd_t_type(client, since, input);
    let rows = published_results(client);
    let first = &rows.command_results[0];
    assert!(
        expect_first(first, input),
        "{what}: the typed destination ranked {} ({}) — expected the navigate \
         fallback and/or its recorded history row",
        first.key,
        first.group
    );
    let after = enter_cmd_t(client);
    let view = after.active_web_view.expect("fresh destination view");
    assert_eq!(after.tabs.len(), tabs_before + 1, "exactly one new tab");
    assert_eq!(after.command_bar_mode, CommandBarMode::Inactive);
    assert!(!after.chrome_focused, "the fresh view owns focus");
    // The shell bookkeeping IS green for the fresh view — it is really the
    // active tab with real geometry. This is the gap the product owner hit:
    // everything the old suites assert passes while the user sees white.
    assert!(
        after.content_rect[2] > 0.0 && after.content_rect[3] > 0.0,
        "the fresh view has non-zero native geometry"
    );
    (view, after)
}

#[test]
#[ignore]
fn cmd_t_schemeless_and_schemed_destinations_both_arrive() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-nav", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // Baseline: the initial tab really has content — the client-world
        // probe reports the committed fixture document (never our own
        // visibility flags).
        let baseline = wait_content_probe(
            &mut client,
            first_view,
            |probe| probe.ready_state == "complete" && probe.dom_url == page_one,
            "initial tab content did not arrive",
        );
        assert_eq!(
            baseline.native_url.as_deref(),
            Some(page_one.as_str()),
            "the native WKWebView reports the committed fixture URL"
        );

        // ---- Phase A (the regression): the scheme-less input from the bug
        // report. The corpus is clean (only the initial fixture tab, and no
        // recorded visits yet), so "golem.de" ranks the navigate fallback
        // and Enter must deliver a real navigation/content — exactly like
        // the scheme'd twin. The activation normalizes the input to its
        // absolute `https://golem.de` destination (the fix under test), so
        // the engine accepts a real navigation and the visit is recorded in
        // the app-owned history provider.
        let tabs_before = client.snapshot().expect("pre-A snapshot").tabs.len();
        let (phase_a_view, after_a) = activate_cmd_t_destination(
            &mut client,
            &mut since,
            "golem.de",
            tabs_before,
            |row, input| row.key == format!("navigate:{input}") && row.group == "fallback",
            "the scheme-less input ranks the navigate fallback first (history is empty)",
        );
        assert_ne!(phase_a_view, first_view, "the fresh view is a new session");
        // The regression: the scheme-less destination must arrive — a
        // committed golem document (client-world probe) or at least an
        // accepted navigation (native URL). The scheme-less URL must never
        // leave the fresh view blank.
        let probe_a = wait_content_probe(
            &mut client,
            phase_a_view,
            |probe| {
                probe.dom_url.contains("golem.de")
                    || probe
                        .native_url
                        .as_deref()
                        .is_some_and(|url| url.contains("golem.de"))
            },
            "schemeless Cmd-T destination (golem.de) stayed blank",
        );
        // The acceptance evidence references the golem host: the committed
        // client-world document OR the accepted native URL (provisional).
        // The native-URL leg fires from `didStartProvisionalNavigation`
        // before the response commits, so the committed document may
        // legitimately still be `about:blank` at that instant — the
        // scheme-less URL is accepted, which is exactly what the pre-fix
        // build could never do (rejection with no provisional navigation).
        assert!(
            probe_a.dom_url.contains("golem.de")
                || probe_a
                    .native_url
                    .as_deref()
                    .is_some_and(|url| url.contains("golem.de")),
            "acceptance evidence must reference the golem host: {probe_a:?}"
        );
        // The engine accepted a navigation to the golem host
        // (didStartProvisionalNavigation emits the provisional URL).
        since = wait_web_event(
            &mut client,
            |ev| {
                matches!(
                    ev,
                    WebEvent::UrlChanged(id, url) if *id == phase_a_view && url.contains("golem.de")
                )
            },
            since,
        );
        // Keep the tab corpus clean for phase B: close the fresh destination
        // tab so the next input cannot rank an existing tab. The recorded
        // history visit deliberately stays (Phase 7 records every accepted
        // activation) — Phase B proves the history-row activation path that
        // re-typing the same destination deterministically ranks.
        let index = after_a.active_tab_index.expect("fresh tab index");
        client
            .send_ok(&bbr_testkit::Request::CloseTab { index })
            .expect("close destination tab");
        let closed = client.snapshot().expect("closed snapshot");
        assert_eq!(
            closed.active_web_view,
            Some(first_view),
            "closing returns to the warm initial tab"
        );

        // ---- Phase B (the scheme'd twin): same production activation, same
        // expectations — and real content arrives. The scheme'd input
        // matches the recorded history entry from Phase A exactly, so the
        // ranked first row is deterministically the history row
        // (`history:<id>`, `OpenHistory` activation) — or, if ranking ever
        // favors the fallback, the navigate row; both activate the same
        // intended golem destination through production, and the content
        // contract below must hold for either route.
        let tabs_before = client.snapshot().expect("pre-B snapshot").tabs.len();
        let (phase_b_view, _after_b) = activate_cmd_t_destination(
            &mut client,
            &mut since,
            "https://golem.de",
            tabs_before,
            |row, input| {
                (row.group == "history" && row.key.starts_with("history:"))
                    || (row.key == format!("navigate:{input}") && row.group == "fallback")
            },
            "the scheme'd twin ranks its recorded history row or the navigate fallback",
        );
        assert_ne!(phase_b_view, first_view, "the fresh view is a new session");
        // The engine must accept a real navigation to the scheme'd URL:
        // WebKit emits `UrlChanged` from `didStartProvisionalNavigation`
        // carrying the provisional URL (network-independent — it fires
        // before any DNS outcome).
        wait_web_event(
            &mut client,
            |ev| {
                matches!(
                    ev,
                    WebEvent::UrlChanged(id, url) if *id == phase_b_view && url.contains("golem.de")
                )
            },
            since,
        );
        // Same no-blank-page contract as Phase A: the history-row activation
        // must still navigate the fresh view to the intended destination — a
        // committed golem document or an accepted native URL.
        let probe_b = wait_content_probe(
            &mut client,
            phase_b_view,
            |probe| {
                probe.dom_url.contains("golem.de")
                    || probe
                        .native_url
                        .as_deref()
                        .is_some_and(|url| url.contains("golem.de"))
            },
            "scheme'd Cmd-T destination (https://golem.de) stayed blank",
        );
        assert!(
            probe_b.dom_url.contains("golem.de")
                || probe_b
                    .native_url
                    .as_deref()
                    .is_some_and(|url| url.contains("golem.de")),
            "acceptance evidence must reference the golem host: {probe_b:?}"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
