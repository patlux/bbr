//! Native evidence for open presentations: hidden-parent fixtures exercise
//! logical chrome, while shown fixtures must prove actual child visibility.

use super::*;
use bbr_testkit::protocol::PortalSnapshot;

pub fn portal_visibility_settled(portal: &PortalSnapshot) -> bool {
    portal.native_visible == portal.parent_visible
}

pub fn assert_open_portal_visibility(portal: &PortalSnapshot) {
    assert!(
        portal.owner_retained,
        "the reusable host retains its browser owner"
    );
    assert_eq!(
        portal.native_visible, portal.parent_visible,
        "a hidden parent must not expose a standalone portal; a shown fixture must show it"
    );
    if portal.parent_visible {
        assert!(
            portal.app_active && !portal.app_hidden && portal.parent_is_key,
            "shown-window evidence requires the active browser owner"
        );
        assert!(
            portal.child_attached,
            "measured AppKit parentWindow relation"
        );
    } else {
        assert!(
            !portal.portal_owns_probe_point,
            "hidden chrome cannot intercept input"
        );
    }
}

/// Explicit shown fixture for tests asserting native interaction/visibility.
/// Kept separate from the deterministic hidden geometry harness.
pub fn spawn_visible_harness(tag: &str, page: &str) -> (Harness, PathBuf, String) {
    let pid = std::process::id();
    let root = harness_root(tag, pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = harness_socket_path(tag, pid);
    let token = format!("shell-{tag}-{pid}");
    let harness = Harness::spawn(
        env!("CARGO_BIN_EXE_bbr"),
        &[
            "--bbr-initial-url",
            page,
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
        ],
        &root,
    );
    (harness, socket, token)
}
