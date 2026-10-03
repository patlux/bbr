use bbr_visual::{
    browser_cases,
    gallery::{self, Case, Plan},
    provenance, runner, Suite,
};
use cargo_gpui_visual::{
    collection::catalog::{Catalog, State},
    store,
};
use std::{fs, path::Path};

fn fixture() -> (tempfile::TempDir, Plan, String) {
    let root = tempfile::tempdir().unwrap();
    let plan = Plan {
        schema: 1,
        cases: vec![Case {
            base: "gallery-00-state-00-standard-normal".into(),
            label: "Tokens".into(),
            state: "default".into(),
            group: "Foundations".into(),
            ranges: vec![[0, 1]],
            dark_ranges: None,
            endpoint: None,
        }],
    };
    let mut rows = Vec::new();
    let mut inventory = String::new();
    for mode in ["light", "dark"] {
        let case = format!("{}--{mode}", plan.cases[0].base);
        inventory += &format!("{case}\tTokens\tdefault\n");
        rows.push(format!("{case}-page-00\t{case}\tTokens\tdefault\t0\t1\t1"));
        image::RgbaImage::new(2400, 2000)
            .save(root.path().join(format!("{case}-page-00.png")))
            .unwrap();
    }
    fs::write(root.path().join("coverage.tsv"), rows.join("\n") + "\n").unwrap();
    (root, plan, inventory)
}

#[test]
fn valid_output_renames_only_identity_and_preserves_png_bytes() {
    let (root, plan, inventory) = fixture();
    let original = store::digest(
        &root
            .path()
            .join("gallery-00-state-00-standard-normal--light-page-00.png"),
    )
    .unwrap();
    let ids = gallery::validate_and_prepare(root.path(), &plan, &inventory).unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(
        store::digest(&root.path().join(format!("{}.png", ids[0]))).unwrap(),
        original
    );
}

#[test]
fn missing_extra_reordered_and_changed_ranges_fail_before_renames() {
    for change in 0..4 {
        let (root, plan, inventory) = fixture();
        let marker = root.path().join("coverage.tsv");
        let text = fs::read_to_string(&marker).unwrap();
        let changed = match change {
            0 => text.lines().next().unwrap().to_owned() + "\n",
            1 => text.clone() + text.lines().next().unwrap() + "\n",
            2 => text.lines().rev().collect::<Vec<_>>().join("\n") + "\n",
            _ => text.replace("\t0\t1\t1", "\t0\t2\t2"),
        };
        fs::write(marker, changed).unwrap();
        assert!(gallery::validate_and_prepare(root.path(), &plan, &inventory).is_err());
        assert!(!root.path().join("inventory.json").exists());
        assert!(root
            .path()
            .join("gallery-00-state-00-standard-normal--light-page-00.png")
            .exists());
    }
}

#[test]
fn live_registry_cannot_silently_grow_or_shrink() {
    let (root, plan, inventory) = fixture();
    for changed in [
        String::new(),
        inventory.clone() + "extra--light\tExtra\tdefault\n",
        inventory.clone() + &inventory,
    ] {
        assert!(gallery::validate_and_prepare(root.path(), &plan, &changed).is_err());
    }
}

#[test]
fn unexpected_files_symlinks_precision_and_size_are_rejected() {
    let (root, plan, inventory) = fixture();
    let extra = root.path().join("extra.png");
    fs::write(&extra, b"no").unwrap();
    assert!(gallery::validate_and_prepare(root.path(), &plan, &inventory).is_err());
    fs::remove_file(extra).unwrap();
    let png = root
        .path()
        .join("gallery-00-state-00-standard-normal--light-page-00.png");
    image::RgbaImage::new(1, 1).save(&png).unwrap();
    assert!(gallery::validate_and_prepare(root.path(), &plan, &inventory).is_err());
    image::ImageBuffer::<image::Rgba<u16>, Vec<u16>>::new(2400, 2000)
        .save(&png)
        .unwrap();
    assert!(gallery::validate_and_prepare(root.path(), &plan, &inventory).is_err());
    fs::remove_file(&png).unwrap();
    std::os::unix::fs::symlink(root.path().join("coverage.tsv"), &png).unwrap();
    assert!(gallery::validate_and_prepare(root.path(), &plan, &inventory).is_err());
}

#[test]
fn authored_catalogs_cover_required_pairs_and_keep_unproven_coverage_visible() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let plan: Plan =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery-plan.json")).unwrap();
    let gallery: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery.json")).unwrap();
    gallery.validate().unwrap();
    assert_eq!(plan.cases.len(), 630);
    assert_eq!(
        plan.cases.iter().map(|c| c.ranges.len()).sum::<usize>(),
        936
    );
    assert_eq!(
        gallery
            .snapshots
            .iter()
            .filter(|s| s.state == State::Required)
            .count(),
        936
    );
    assert_eq!(
        gallery
            .snapshots
            .iter()
            .filter(|s| s.state == State::Withheld)
            .count(),
        31
    );
    let browser: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/browser.json")).unwrap();
    browser.validate().unwrap();
    let required: Vec<_> = browser
        .snapshots
        .iter()
        .filter(|s| s.state == State::Required)
        .flat_map(|s| [format!("{}--light", s.base), format!("{}--dark", s.base)])
        .collect();
    assert_eq!(required, browser_cases());
}

#[test]
fn populated_library_is_required_independently_of_producer_output() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let browser: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/browser.json")).unwrap();
    let entry = browser
        .snapshots
        .iter()
        .find(|s| s.base == "library-populated")
        .expect("populated production Saved Pages must be an independent requirement");
    assert_eq!(entry.state, State::Required);
    for base in [
        "library-collections",
        "library-selection-single",
        "library-selection-multiple",
        "library-move-sheet",
        "library-remove-sheet",
        "library-collection-create",
        "history-search",
        "history-search-empty",
        "history-clear-range-menu",
        "history-recently-closed",
        "settings-general-menu",
        "settings-search-menu",
        "settings-motion-menu",
        "settings-profile-create",
        "settings-profile-edit",
        "settings-shortcuts-scroll-1",
        "settings-shortcuts-scroll-2",
        "settings-shortcuts-scroll-3",
    ] {
        assert!(
            browser
                .snapshots
                .iter()
                .any(|s| s.base == base && s.state == State::Required),
            "{base}"
        );
    }
    for base in [
        "remaining-app-surfaces",
        "library-remaining-states",
        "history-remaining-states",
        "settings-remaining-states",
        "portal-surfaces",
        "permission-surfaces",
        "welcome-about",
    ] {
        let entry = browser
            .snapshots
            .iter()
            .find(|s| s.base == base)
            .expect("explicit missing family");
        assert_eq!(entry.state, State::Withheld);
        assert!(!entry.reason.as_ref().unwrap().is_empty());
    }
    assert_eq!(browser_cases().len(), 66);
    for mode in ["light", "dark"] {
        assert!(browser_cases().contains(&format!("library-populated--{mode}")));
    }
}

#[test]
fn newly_registered_gallery_stories_remain_explicitly_withheld_until_audited() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let gallery: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery.json")).unwrap();
    gallery.validate().unwrap();
    // Independently authored source-audit requirements, not producer inventory
    // or guessed page counts. Existing historical requirements stay unchanged.
    for (base, label, limitation) in [
        ("gallery-49-audit-pending", "Delete Space sheet", "shadow"),
        ("gallery-50-audit-pending", "Peek presentation", "stand-in"),
        ("gallery-51-audit-pending", "Toast and zoom readout", "fade"),
        ("gallery-52-audit-pending", "Media card", "six"),
    ] {
        let entry = gallery
            .snapshots
            .iter()
            .find(|s| s.base == base)
            .unwrap_or_else(|| {
                panic!("new registered story lacks an explicit coverage blocker: {base}")
            });
        assert_eq!(entry.label, label);
        assert_eq!(
            entry.state,
            State::Withheld,
            "{base} is not proven coverage or an inapplicable story"
        );
        assert!(entry.reason.as_deref().unwrap().contains(limitation));
        assert!(
            entry.page.is_none() && entry.sequence.is_none() && entry.moment.is_none(),
            "source audit cannot invent captured pages/moments"
        );
    }
}

#[test]
fn source_projected_states_and_missing_assets_never_become_coverage() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let gallery: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery.json")).unwrap();
    gallery.validate().unwrap();
    for (base, reason) in [
        ("gallery-07-state-projection", "favicon"),
        ("gallery-08-iconography-gaps", "Search"),
        ("gallery-core-control-applicability", "contrast"),
        ("gallery-18-radio-check-asset", "icons/check.svg"),
    ] {
        let entry = gallery
            .snapshots
            .iter()
            .find(|s| s.base == base)
            .unwrap_or_else(|| panic!("missing concrete source-audit blocker: {base}"));
        assert_eq!(entry.state, State::Withheld);
        assert!(entry.reason.as_deref().unwrap().contains(reason));
        assert!(entry.page.is_none() && entry.sequence.is_none() && entry.moment.is_none());
    }
}

#[test]
fn tab_and_library_source_gaps_remain_explicit_non_capture_requirements() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let gallery: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery.json")).unwrap();
    gallery.validate().unwrap();
    for (base, reasons) in [
        (
            "gallery-22-stale-matrix-exclusions",
            ["bitmap", "split", "focus"],
        ),
        (
            "gallery-30-selection-collection-gaps",
            ["selected", "toolbar", "collection"],
        ),
        ("gallery-30-windowed-framing", ["24", "520", "scroll"]),
    ] {
        let entry = gallery
            .snapshots
            .iter()
            .find(|s| s.base == base)
            .unwrap_or_else(|| panic!("missing concrete source-audit blocker: {base}"));
        assert_eq!(entry.state, State::Withheld);
        for reason in reasons {
            assert!(
                entry.reason.as_deref().unwrap().contains(reason),
                "{base}: {reason}"
            );
        }
        assert!(entry.page.is_none() && entry.sequence.is_none() && entry.moment.is_none());
    }
}

#[test]
fn address_and_sidebar_component_gaps_remain_visible_without_invented_capture_cases() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let gallery: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery.json")).unwrap();
    gallery.validate().unwrap();
    for (base, reasons) in [
        (
            "gallery-31-address-production-gaps",
            ["zoom", "progress", "device"],
        ),
        (
            "gallery-32-staging-edit-gaps",
            ["selection", "marked", "end_of"],
        ),
        (
            "gallery-33-favorite-image-pop-framing",
            ["image", "pop", "300"],
        ),
        (
            "gallery-34-folder-hidden-count-states",
            ["reveal_count", "drop_target", "rename"],
        ),
        (
            "gallery-35-footer-focus-strip-framing",
            ["focus", "scroll", "fixture"],
        ),
        (
            "gallery-37-plus-menu-renderer-drift",
            ["border_l_2", "menu_cursor", "elev2"],
        ),
    ] {
        let entry = gallery
            .snapshots
            .iter()
            .find(|s| s.base == base)
            .unwrap_or_else(|| panic!("missing concrete source-audit blocker: {base}"));
        assert_eq!(entry.state, State::Withheld);
        for reason in reasons {
            assert!(
                entry.reason.as_deref().unwrap().contains(reason),
                "{base}: {reason}"
            );
        }
        assert!(entry.page.is_none() && entry.sequence.is_none() && entry.moment.is_none());
    }
}

#[test]
fn surface_audit_gaps_cannot_be_reported_as_captured_or_inapplicable() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let gallery: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery.json")).unwrap();
    gallery.validate().unwrap();
    for (base, reasons) in [
        (
            "gallery-38-download-scroll-moments",
            ["ScrollHandle", "check", "8..22"],
        ),
        (
            "gallery-39-find-wrapper-bounds",
            ["wrapper", "control", "focus"],
        ),
        (
            "gallery-40-offline-illustration-gap",
            ["illustration", "Lifering", "Offline"],
        ),
        (
            "gallery-41-unresponsive-pane-gap",
            ["unresponsive_panel", "Wait", "260"],
        ),
        (
            "gallery-43-context-menu-geometry",
            ["148", "row", "padding"],
        ),
    ] {
        let entry = gallery
            .snapshots
            .iter()
            .find(|s| s.base == base)
            .unwrap_or_else(|| panic!("missing concrete source-audit blocker: {base}"));
        assert_eq!(entry.state, State::Withheld);
        for reason in reasons {
            assert!(
                entry.reason.as_deref().unwrap().contains(reason),
                "{base}: {reason}"
            );
        }
        assert!(entry.page.is_none() && entry.sequence.is_none() && entry.moment.is_none());
    }
}

#[test]
fn composition_source_gaps_remain_withheld_without_replacing_historical_pages() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let gallery: Catalog =
        store::read_json(&repo.join("scripts/visual/catalogs/gallery.json")).unwrap();
    gallery.validate().unwrap();
    for (base, reasons) in [
        (
            "gallery-44-sidebar-production-branches",
            ["closed", "status_visible", "floating"],
        ),
        (
            "gallery-45-command-edit-row-gaps",
            ["end_of", "url=None", "glide"],
        ),
        (
            "gallery-46-space-empty-name-gap",
            ["empty", "Space name", "recorder"],
        ),
        (
            "gallery-47-tab-menu-state-gaps",
            ["Unpin Tab", "Remove Bookmark", "Close Tab"],
        ),
        (
            "gallery-48-page-menu-geometry",
            ["148", "padding", "bookmark"],
        ),
    ] {
        let entry = gallery
            .snapshots
            .iter()
            .find(|s| s.base == base)
            .unwrap_or_else(|| panic!("missing concrete composition source-audit blocker: {base}"));
        assert_eq!(entry.state, State::Withheld);
        for reason in reasons {
            assert!(
                entry.reason.as_deref().unwrap().contains(reason),
                "{base}: {reason}"
            );
        }
        assert!(entry.page.is_none() && entry.sequence.is_none() && entry.moment.is_none());
    }
}

#[test]
fn pinned_complete_export_matches_and_output_paths_fail_closed() {
    let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
    provenance::verify(&repo).unwrap();
    for raw in ["relative", "/tmp/foreign", "/tmp/../outside"] {
        assert!(runner::output_directory(&repo, Suite::Gallery, Path::new(raw)).is_err());
    }
    assert!(runner::output_directory(
        &repo,
        Suite::Gallery,
        &repo.join("reports/visual/gallery-runs/new-contract-output")
    )
    .is_ok());
}

#[test]
fn canonical_identity_does_not_accept_malformed_appearance_or_page_numbers() {
    assert_eq!(
        gallery::canonical_id("gallery-00--light-page-00").unwrap(),
        "gallery-00-page-00--light"
    );
    for raw in [
        "gallery-00--blue-page-00",
        "gallery-00--light-page-0",
        "../outside--light-page-00",
        "no-page",
    ] {
        assert!(gallery::canonical_id(raw).is_err());
    }
}
