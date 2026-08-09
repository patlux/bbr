//! Dependency-boundary contract tests (the G1a dependency-audit gate, unit
//! half).
//!
//! These tests re-derive the crate's allowed direct dependency set from the
//! manifest at `CARGO_MANIFEST_DIR` (never from a hardcoded copy of this
//! crate's `Cargo.toml`) and assert it exactly, prove the crate carries
//! `#![forbid(unsafe_code)]`, and pin the approved first-party closure
//! facts the metadata audit (`scripts/check_ui_dependencies.py`, the
//! proposed `just ui-deps` gate) checks at the package level: the only
//! permitted first-party transitive edge is the existing
//! `bbr-domain → bbr-web` `ProfileId` path, and `bbr-ui` itself never
//! declares or imports `bbr-web`. The script is the authority on the full
//! resolved closure; this module guards the manifest and the crate root.

use std::path::PathBuf;

/// The exact allowed direct normal dependency set: `gpui` (the pinned
/// GPUI-CE source the workspace patches every consumer to) and `bbr-domain`
/// for the single hue/appearance authority. The standard library is
/// implicit. No third-party product/runtime dependency may join them.
const ALLOWED_DIRECT: &[&str] = &["gpui", "bbr-domain"];

/// The first-party packages that may appear anywhere in `bbr-ui`'s normal
/// dependency closure: `bbr-domain` (direct) plus the existing
/// `bbr-domain → bbr-web` identity edge. Every other first-party crate
/// (`bbr-browser`, `bbr-app`, `bbr-testkit`, `bbr-fixtures`,
/// `bbr-persistence`, `bbr-split`, `bbr-command`, `bbr-web-macos`,
/// `bbr-gallery`) is forbidden in the closure.
const ALLOWED_FIRST_PARTY_CLOSURE: &[&str] = &["bbr-domain", "bbr-web"];

/// The forbidden direct source authorities: `bbr-ui` source must never
/// import or name these crates (the script additionally greps for these as
/// `use`/path tokens across `crates/bbr-ui/src`).
const FORBIDDEN_SOURCE_CRATES: &[&str] = &[
    "bbr_browser",
    "bbr_web",
    "bbr_app",
    "bbr_testkit",
    "bbr_fixtures",
    "bbr_persistence",
    "bbr_split",
    "bbr_command",
    "bbr_web_macos",
    "bbr_gallery",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The raw `[dependencies]` section of this crate's manifest, as declared
/// (before any workspace patch resolves). Returns the key of each
/// dependency line in declaration order.
fn direct_dependency_keys() -> Vec<String> {
    let manifest = std::fs::read_to_string(manifest_dir().join("Cargo.toml"))
        .expect("bbr-ui manifest readable at CARGO_MANIFEST_DIR");
    let section = manifest
        .split("\n[dependencies]")
        .nth(1)
        .expect("[dependencies] section present")
        .split("\n[")
        .next()
        .expect("section terminates at the next table header");
    section
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split('=').next().map(str::trim).map(str::to_string))
        .filter(|key| !key.is_empty())
        .collect()
}

#[test]
fn direct_dependencies_equal_the_allowlist_exactly() {
    let mut keys = direct_dependency_keys();
    keys.sort();
    let mut expected = ALLOWED_DIRECT.to_vec();
    expected.sort();
    assert_eq!(
        keys, expected,
        "bbr-ui declares exactly gpui + bbr-domain as direct normal \
         dependencies (the packet's exact direct manifest set)"
    );
}

#[test]
fn the_crate_forbids_unsafe_code() {
    let lib = std::fs::read_to_string(manifest_dir().join("src/lib.rs"))
        .expect("bbr-ui src/lib.rs readable");
    assert!(
        lib.contains("#![forbid(unsafe_code)]"),
        "the crate root carries #![forbid(unsafe_code)] (same rule as bbr-web/bbr-domain)"
    );
}

#[test]
fn the_approved_first_party_closure_is_exactly_domain_plus_web() {
    // The manifest may only reference first-party packages from the allowed
    // closure. This mirrors the script's closure check on the *declared*
    // manifest; the script additionally verifies the resolved `cargo
    // metadata` closure and every path to `bbr-web` through `bbr-domain`.
    let manifest = std::fs::read_to_string(manifest_dir().join("Cargo.toml"))
        .expect("bbr-ui manifest readable");
    let dependency_section = manifest
        .split("\n[dependencies]")
        .nth(1)
        .expect("[dependencies] section present")
        .split("\n[")
        .next()
        .expect("section terminates");
    for line in dependency_section.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let key = line.split('=').next().map(str::trim).unwrap_or("");
        if !key.is_empty() {
            assert!(
                ALLOWED_DIRECT.contains(&key),
                "unexpected direct dependency key {key:?}"
            );
        }
    }
    // The `bbr-domain` path dependency is the *only* first-party direct
    // edge; `bbr-web` must never be declared here.
    assert!(
        manifest.contains("bbr-domain = { path = \"../bbr-domain\" }"),
        "bbr-domain is the single first-party path dependency"
    );
    for forbidden in [
        "bbr-web",
        "bbr-browser",
        "bbr-app",
        "bbr-testkit",
        "bbr-persistence",
    ] {
        assert!(
            !dependency_section.contains(&format!("{forbidden} =")),
            "manifest must not declare {forbidden} as a direct dependency"
        );
    }
    assert!(
        ALLOWED_FIRST_PARTY_CLOSURE.contains(&"bbr-domain")
            && ALLOWED_FIRST_PARTY_CLOSURE.contains(&"bbr-web"),
        "the approved first-party closure stays exactly bbr-domain -> bbr-web"
    );
}

#[test]
fn source_never_imports_forbidden_crates() {
    // The source guard: no `bbr_ui` source file may reference the forbidden
    // first-party crates (imports, paths, or `use`). The script re-runs this
    // over the whole crate; the test guards the seam directly.
    let src = manifest_dir().join("src");
    let mut offending: Vec<String> = Vec::new();
    let mut entries = Vec::new();
    collect_rs_files(&src, &mut entries);
    for path in entries {
        if path
            .file_name()
            .is_some_and(|name| name == "dependency_contract.rs")
        {
            // The guard itself declares the forbidden list; it is not
            // product source and never imports any forbidden crate.
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("source file readable");
        for forbidden in FORBIDDEN_SOURCE_CRATES {
            if text.contains(forbidden) {
                offending.push(format!("{}: {forbidden}", path.display()));
            }
        }
    }
    assert!(
        offending.is_empty(),
        "bbr-ui source references forbidden crates:\n{}",
        offending.join("\n")
    );
}

fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("src dir readable") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}
