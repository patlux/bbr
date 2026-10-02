//! Authored static pilot requirements, not generated capture coverage.
use bbr_visual::{gallery_selection::Schedule, Suite};
use cargo_gpui_visual::{config::Config, selection::Selection, store};
use std::path::PathBuf;

#[test]
fn tokens_pilot_selects_every_authored_pair_without_enabling_other_stories() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let config = Config::load(&repo.join(Suite::Gallery.config())).unwrap();
    let selection = Selection::resolve(&repo, &config, &["gallery-00".into()]).unwrap();
    let sources = store::source_hashes(&repo, &config.source_policy, None).unwrap();
    let schedule = Schedule::resolve(
        &repo,
        &sources,
        "scripts/visual/catalogs/gallery-plan.json",
        &selection,
    )
    .unwrap();
    assert_eq!(schedule.story_ids(), &[0]);
    assert_eq!(schedule.plan().cases.len(), 6);
    let mut expected = Vec::new();
    for contrast in ["standard", "increased"] {
        for density in ["small", "normal", "extralarge"] {
            expected.push(format!("gallery-00-state-00-{contrast}-{density}-page-00"));
        }
    }
    assert_eq!(selection.stories.len(), 1);
    assert_eq!(selection.stories[0].cases, expected);
    assert!(selection.stories[0].sequences.is_empty());
    for case in &schedule.plan().cases {
        assert_eq!(case.label, "Tokens");
        assert_eq!(case.state, "default");
        assert_eq!(case.ranges, vec![[0, 10]]);
        assert!(case.dark_ranges.is_none());
        assert!(case.endpoint.is_none());
    }
    schedule.check_sources(&repo, &sources).unwrap();
    for ids in [
        vec!["gallery-16".into()],
        vec!["gallery-44".into()],
        vec!["gallery-00".into(), "gallery-00".into()],
    ] {
        assert!(Selection::resolve(&repo, &config, &ids).is_err());
    }
    // Configured discovery does not silently enable a persisted capture filter.
    assert!(config.selection.is_none());
    assert!(bbr_visual::runner::require_vm_execution().is_err());
}
