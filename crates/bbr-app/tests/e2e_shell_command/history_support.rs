//! Shared seeding/ranking support for the command-history provider E2E
//! (`history.rs` and `history_global.rs`): the production repository seed
//! path, the pre-seeded app spawn, and the deterministic history-row query.

use super::*;
use bbr_persistence::{CommandHistoryEntry, Repository};
use std::path::Path;

/// The persistence store file name the app opens at startup (mirrors
/// `crate::store::STORE_FILE_NAME`; the E2E crate cannot import the bin
/// crate's modules).
pub const STORE_FILE_NAME: &str = "bbr.sqlite";

/// Seed a schema-v2 store at `{profile_dir}/bbr.sqlite` with the given
/// command-history collection, exactly as the app's startup restore reads
/// it. The profile directory is created internally (SQLite cannot create
/// parent directories), so callers may seed a store that does not exist
/// yet.
pub fn seed_store(profile_dir: &Path, history: &[CommandHistoryEntry]) -> Repository {
    std::fs::create_dir_all(profile_dir).unwrap();
    let repository = Repository::new(profile_dir.join(STORE_FILE_NAME));
    repository.initialize_and_restore().unwrap();
    repository.store_command_history(history).unwrap();
    repository
}

/// A deterministic seeded entry with an explicit stable identity.
pub fn entry(
    id: u64,
    title: &str,
    url: &str,
    visit_count: u64,
    last_visited: u64,
) -> CommandHistoryEntry {
    CommandHistoryEntry {
        id,
        title: title.into(),
        url: url.into(),
        visit_count,
        last_visited,
    }
}

/// Spawn the real app with a pre-seeded profile directory (the seed runs
/// before the process starts, so the startup restore reads it).
pub fn spawn_seeded(root_tag: &str, page_one: &str, profile: &Path) -> (Harness, PathBuf, String) {
    std::fs::create_dir_all(profile).unwrap();
    let pid = std::process::id();
    let root = harness_root(root_tag, pid);
    let socket = harness_socket_path(root_tag, pid);
    let token = format!("shell-{root_tag}-{pid}");
    let bin = env!("CARGO_BIN_EXE_bbr");
    let harness = Harness::spawn(
        bin,
        &[
            "--bbr-initial-url",
            page_one,
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
        ],
        &root,
    );
    (harness, socket, token)
}

/// The seeded history row for `query` in the published results, asserting it
/// exists exactly once (the single-authority contract).
pub fn history_row(
    client: &mut Client,
    since: &mut u64,
    query: &str,
    expected_key: &str,
) -> bbr_testkit::protocol::CommandResultSnapshot {
    let _ = cmd_l(client, since);
    client
        .send_ok(&bbr_testkit::Request::SetCommandBarText { text: query.into() })
        .expect("set query");
    client
        .send_ok(&bbr_testkit::Request::FlushCommandRanking)
        .expect("flush ranking");
    let snapshot = client.snapshot().expect("history query snapshot");
    assert_eq!(snapshot.command_result_state, CommandResultState::Results);
    let matches = snapshot
        .command_results
        .iter()
        .filter(|row| row.key == expected_key)
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "{expected_key} ranks exactly once");
    assert_eq!(matches[0].group, "history");
    matches[0].clone()
}
