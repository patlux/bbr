use super::*;

/// Regression guard: the sidebar plus-menu/favorites/folder-toggle
/// scenarios must carry their warm contract budgets in the seed so a future
/// rail-regression trips the gate instead of being silently unreported dead
/// code. All three are warm next-frame presentation class (50/100 from the
/// packet).
#[test]
fn u2_sidebar_budgets_are_seeded_and_enforced() {
    let b = Baseline::seed();
    for kind in [
        "sidebar_plus_menu",
        "sidebar_favorites",
        "sidebar_folder_toggle",
    ] {
        let entry = b
            .entries
            .get(kind)
            .unwrap_or_else(|| panic!("seed baseline must carry the {kind} contract budget"));
        assert_eq!(entry.budget_p95_ms, 50.0, "{kind} warm p95 ceiling");
        assert_eq!(entry.budget_max_ms, Some(100.0), "{kind} warm max ceiling");
        let mut slow = BTreeMap::new();
        slow.insert(kind.into(), measured(120.0, 160.0));
        let r = evaluate(&slow, &b, false);
        assert!(
            r.iter().any(|x| x.rule == "abs_p95"),
            "{kind} absolute p95 ceiling enforced"
        );
        assert!(
            r.iter().any(|x| x.rule == "abs_max"),
            "{kind} absolute max ceiling enforced"
        );
    }
}
