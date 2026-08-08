//! U2 harness helpers (DESIGN.md §11.3/§11.4): the fixed short `/tmp`
//! harness root and gateway socket path (macOS `sockaddr_un.sun_path` is
//! 104 bytes; `$TMPDIR` alone is 49+ bytes on standard macOS and grows
//! further under direnv), the always-present §11.3 space strip verification
//! driving the same production snapshot the migrated suites use, and the
//! SUN_LEN regression test guarding the socket bound. Split from `super` so
//! the shared module stays under the ~500-line budget.

use std::path::PathBuf;

use bbr_testkit::Client;

/// macOS `sockaddr_un.sun_path` is 104 bytes; the default `$TMPDIR`
/// (`/var/folders/…/T/`) alone is 49 bytes and can grow further under
/// direnv, so a harness root under `std::env::temp_dir()` cannot keep the
/// gateway socket (bound at `<root>/profile/bbr-test.sock`) below the limit
/// for the longer suite tags. Use the fixed short `/tmp` root instead
/// (macOS-only module) — the same convention as the testkit unit tests
/// (`/tmp/bbr-tc-{pid}-{n}`).
const HARNESS_ROOT_PREFIX: &str = "/tmp/bbr-st-";

/// Build the harness root for `root_tag` and `pid` (pid injected so the
/// SUN_LEN regression test can probe the worst plausible pid without
/// forking). The pid keeps suite runs isolated from each other.
pub fn harness_root(root_tag: &str, pid: u32) -> PathBuf {
    PathBuf::from(format!("{HARNESS_ROOT_PREFIX}{root_tag}-{pid}"))
}

/// The exact gateway socket path `spawn_harness` produces for `root_tag`:
/// `<root>/profile/bbr-test.sock`. Shared with the SUN_LEN regression test
/// so the worst-case assertion exercises the real builder logic.
pub fn harness_socket_path(root_tag: &str, pid: u32) -> PathBuf {
    harness_root(root_tag, pid)
        .join("profile")
        .join("bbr-test.sock")
}

/// §11.3/§11.4: the permanent space strip is always part of the rail — no
/// reveal step exists (the legacy `[Spaces]` toggle was removed). The
/// suites drive a snapshot and assert the always-present strip ids through
/// this seam so the migration keeps one assertion point.
pub fn reveal_spaces_region(client: &mut Client) -> bbr_testkit::protocol::Snapshot {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
    let snapshot = client.snapshot().expect("spaces strip snapshot");
    assert!(
        snapshot
            .visible_ids
            .iter()
            .any(|id| id == "bbr-space-strip"),
        "the permanent space strip always renders"
    );
    snapshot
}

/// macOS binds unix sockets into `sockaddr_un.sun_path` (104 bytes); longer
/// paths fail at bind time. The harness binds the gateway at
/// `<root>/profile/bbr-test.sock`, so the worst case — the longest suite tag
/// plus the largest plausible pid (10 decimal digits) — must stay strictly
/// below the limit under the fixed short `/tmp` root. This guards against
/// suite-tag growth or a move of the root back under `$TMPDIR`.
#[test]
fn harness_socket_stays_below_macos_sun_len() {
    const SUN_LEN: usize = 104;
    // Longest tag currently used across the shell suites.
    let longest_tag = "sidebar-u2-favorites";
    let socket = harness_socket_path(longest_tag, u32::MAX);
    let len = socket.as_os_str().len();
    assert!(
        len < SUN_LEN,
        "harness socket {socket:?} is {len} bytes; must stay strictly below macOS SUN_LEN ({SUN_LEN})"
    );
}
