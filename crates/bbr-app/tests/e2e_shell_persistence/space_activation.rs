//! Regression (task packet `v2-paper-alignment`): after a restore, the
//! non-default spaces are session-less. Activating one through the
//! production footer chip must hydrate exactly its visible tab and present
//! its native view — never the empty-space pane over a space that has tabs.

use bbr_persistence::{BrowserState, GlyphName, Space, SpaceGlyph, SpaceTint, Tab};

use crate::*;

fn two_space_state(page_one: &str, page_two: &str) -> BrowserState {
    let mut state = small_browser_state(page_one);
    state.profiles[0].spaces.push(Space {
        id: 20,
        title: "Second".into(),
        tint: SpaceTint::Pine,
        paper: Default::default(),
        glyph: SpaceGlyph::Glyph(GlyphName::Leaf),
        tabs: vec![Tab {
            id: 200,
            title: "Deferred".into(),
            url: page_two.into(),
            pinned: false,
            closed: false,
            pinned_page: None,
        }],
        active_tab: Some(200),
        recently_closed: Vec::new(),
        split: None,
        folders: Vec::new(),
    });
    state
}

#[test]
#[ignore]
fn activating_a_restored_space_presents_its_tab() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let profile = harness_root("persist-activate", pid).join("profile");
    seed_browser_state(&profile, &two_space_state(&page_one, &page_two));

    let (mut harness, socket, token) = spawn_harness("persist-activate", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(
            restored.engine_sessions, 1,
            "only the active space hydrates"
        );
        let second = restored
            .spaces
            .iter()
            .find(|space| space.title == "Second")
            .expect("second space restored")
            .space;

        client
            .send_ok(&dsl::click_space(second))
            .expect("activate the restored space");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after activation");
        let activated = client.snapshot().expect("activated snapshot");
        let view = activated
            .active_web_view
            .expect("the restored space's tab has a native view");
        assert_eq!(activated.engine_sessions, 2, "exactly the destination tab");
        assert!(
            !activated
                .visible_ids
                .iter()
                .any(|id| id == "bbr-empty-new-tab"),
            "no empty-space pane over a space with tabs"
        );
        assert_eq!(activated.motion.illustration, None);
        wait_title(&mut client, view, PAGE_TWO_TITLE, 0);
    }));
    finish(result, &mut harness, &socket, &token);
}
