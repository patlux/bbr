//! `bbr-fixtures` binary: serves a fixture root for suites that want the
//! fixture server as a separate process (mirrors the future `run_e2e.py`
//! topology). Prints the ephemeral port on stdout, one line, then serves
//! until killed.
//!
//! Usage: `bbr-fixtures <root-dir>`

use std::path::PathBuf;

fn main() {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("crates/bbr-fixtures/html"));
    match bbr_fixtures::FixtureServer::start(&root) {
        Ok(server) => {
            println!("{}", server.port());
            println!("{}", server.base_url());
            // Keep the process alive; drop would stop accepting.
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
        Err(e) => {
            eprintln!("bbr-fixtures: failed to start: {e}");
            std::process::exit(1);
        }
    }
}
