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
        4
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
