//! Unit tests for the Space-configuration surface behavior (DESIGN.md
//! §11.3, packet `space-creation-appearance-ui.md`). The rename-draft
//! character set is the pure rule owned by the surface module; the
//! exact-SpaceId/stale-source and commit authorities are covered by the
//! controller/domain contracts and the gated real-app E2E suite.

use super::*;

#[test]
fn editable_characters_cover_the_section_4_chrome_set() {
    for ch in "Reading Work-Notes 2026_plan.xlsx (draft)".chars() {
        assert!(is_editable_char(ch), "expected editable: {ch:?}");
    }
    assert!(!is_editable_char('\n'));
    assert!(!is_editable_char('🙂'));
    assert!(!is_editable_char('\u{200d}'));
}
