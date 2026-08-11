//! The gallery dependency-boundary contract (packet
//! `component-gallery-mvp.md`): the crate's direct manifest `[dependencies]`
//! contains exactly `bbr-ui`, `bbr-domain`, `gpui`, `gpui_platform`, and
//! `gpui-component` (the standard library is implicit; dev-only test support
//! stays on those same allowed crates). `bbr-domain` supplies the fixture
//! DTOs/identity authority (`ProfileHue`) the gallery fixtures and the §12
//! profile-chip story need — a portable identity crate, not a runtime
//! authority; `gpui-component` is the vendored 0.5.1 production control
//! stack whose real core controls the G1b Core Controls stories render (the
//! same source bbr-app consumes). The source imports no forbidden
//! first-party/runtime authority and declares no native bridge, WebKit,
//! persistence, or network crate. The allowlist is re-derived from
//! `CARGO_MANIFEST_DIR`'s manifest — never a copied list.

use std::collections::BTreeSet;

/// The exact allowed direct normal dependencies (plus the implicit std).
/// `bbr-domain` is the single hue/identity authority the §12 profile-chip
/// story needs (bbr-ui does not re-export `ProfileHue`); it is the portable
/// identity crate, not a runtime authority. `gpui-component` is the
/// authorized established vendored 0.5.1 production control stack the G1b
/// Core Controls stories render (the same source bbr-app consumes).
const ALLOWED_DIRECT: [&str; 5] = [
    "bbr-ui",
    "bbr-domain",
    "gpui",
    "gpui_platform",
    "gpui-component",
];

/// First-party/runtime authority the gallery source must never import.
const FORBIDDEN_SOURCE: [&str; 8] = [
    "bbr_app",
    "bbr_browser",
    "bbr_web",
    "bbr_web_macos",
    "bbr_testkit",
    "bbr_fixtures",
    "bbr_persistence",
    "bbr_command",
];

fn manifest() -> toml::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&path).expect("read gallery manifest");
    text.parse::<toml::Value>().expect("parse gallery manifest")
}

#[test]
fn direct_dependencies_are_exactly_the_allowlist() {
    let manifest = manifest();
    let deps = manifest
        .get("dependencies")
        .and_then(|d| d.as_table())
        .expect("gallery [dependencies] table");
    let direct: BTreeSet<&str> = deps.keys().map(String::as_str).collect();
    let allowed: BTreeSet<&str> = ALLOWED_DIRECT.iter().copied().collect();
    assert_eq!(
        direct, allowed,
        "gallery direct dependencies must be exactly {allowed:?}, got {direct:?}"
    );
    // No target-specific dependency may add a forbidden authority either.
    if let Some(targets) = manifest.get("target").and_then(|t| t.as_table()) {
        for (_, table) in targets {
            if let Some(deps) = table.get("dependencies").and_then(|d| d.as_table()) {
                for name in deps.keys() {
                    assert!(
                        ALLOWED_DIRECT.contains(&name.as_str()),
                        "target dependency {name} is not in the allowlist"
                    );
                }
            }
        }
    }
}

#[test]
fn source_imports_no_forbidden_authority() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs(&src, &mut files);
    assert!(!files.is_empty(), "gallery has source files");
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read source file");
        for forbidden in FORBIDDEN_SOURCE {
            assert!(
                !text.contains(&format!("{forbidden}::"))
                    && !text.contains(&format!("use {forbidden}")),
                "{} imports forbidden authority {forbidden}",
                file.display()
            );
        }
    }
}

fn collect_rs(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read source dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The post-cutover `rg` guard (packet
/// `component-gallery-browser-components.md`): `bbr-app` defines **no**
/// duplicate of the G2 Tab Row / Address/Input / Find production symbols
/// and **no** `pub use bbr_ui` compatibility alias for them. The one
/// `tab_row`/`RowVisuals`/`drag_tick`, `address_control`/`staging_surface`/
/// `AddressModel`/`StagingModel`/`AddressProfileHint`, and
/// `find_bar`/`FindSurface`/`FIND_BAR_ID` authority is `bbr-ui`; the app
/// imports them and keeps only its controller-snapshot composition.
/// Implemented with a pure-std source scan (hermetic — no external `rg`
/// binary required in the evaluation shell) over `crates/bbr-app/src`.
#[test]
fn bbr_app_defines_no_tab_row_address_or_find_duplicate_or_reexport_alias() {
    let app_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-app")
        .join("src");
    let mut files = Vec::new();
    collect_rs(&app_src, &mut files);
    assert!(!files.is_empty(), "bbr-app source exists");
    // The migrated tab-row, address, and find symbols must exist exactly
    // once in the workspace: in `bbr-ui`. The app may not redefine any of
    // them (the paren keeps `fn tab_row(` from matching the app-owned
    // `fn tab_row_h` geometry helper; the address components keep their
    // app-owned **composition methods** (`fn address_control(&self, …)`,
    // `fn staging_surface(&self, …)` — the DTO-building dispatchers that
    // call the production symbols) — so the guard pins the production
    // `pub fn` definitions and the DTO/id constants, never those methods).
    let migrated = [
        "fn tab_row(",
        "struct RowVisuals",
        "fn drag_tick(",
        "fn row_title(",
        "fn row_state_slot(",
        "fn close_control(",
        "fn pin_control(",
        "fn row_wash(",
        "pub fn address_control(",
        "pub fn staging_surface(",
        "struct AddressModel",
        "struct StagingModel",
        "struct AddressProfileHint",
        "pub fn find_bar(",
        "struct FindSurface",
        "const FIND_BAR_ID",
    ];
    for file in &files {
        let text = std::fs::read_to_string(file).expect("read app source");
        assert!(
            !text.contains("pub use bbr_ui::"),
            "{} re-exports bbr_ui (no compatibility alias)",
            file.display()
        );
        for symbol in migrated {
            assert!(
                !text.contains(symbol),
                "{} defines a duplicate {symbol}",
                file.display()
            );
        }
    }
    // The app still consumes the production symbols through the one path.
    let sidebar = std::fs::read_to_string(app_src.join("app").join("sidebar_view.rs"))
        .expect("read sidebar_view.rs");
    assert!(
        sidebar.contains("components::tab_row"),
        "sidebar renders through bbr_ui::components::tab_row"
    );
    // The portal root renders the find bar through the one bbr-ui path.
    let probe =
        std::fs::read_to_string(app_src.join("portal").join("probe.rs")).expect("read probe.rs");
    assert!(
        probe.contains("bbr_ui::surfaces::find::find_bar"),
        "the portal root renders through bbr_ui::surfaces::find::find_bar"
    );
}
