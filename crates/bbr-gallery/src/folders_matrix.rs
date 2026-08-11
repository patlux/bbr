//! The Folders matrix authority (packet
//! `component-gallery-browser-components.md`, Group 4): **one declaration
//! source** for every valid §11.11 folders family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::components::folders` symbols
//! (`folder_row`, `folder_item_row`), and the fixture builder refuses
//! (`None`) any case the validity authority rejects — so the story, the
//! fixture builder, the matrix table, and the tests cannot drift. The
//! rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Folders row: collapsed /
//! expanded; nested rows with the §11.1 anatomy (the full §10 vocabulary
//! on nested rows is a DESIGN state the current `FolderMemberVisual`
//! carries no field for — documented in the story, never fabricated);
//! empty collapsible folder (a folder whose tabs were all closed remains
//! as an empty collapsible row); the count `tnum` column; hairlines
//! between folder groups (two folder rows in one family). Rejected:
//! nested rows in a collapsed folder (they render only when expanded,
//! §11.11) and persisted folders (session-local only, §11.11).
//!
//! Folder rows stay **session-local** in the gallery exactly like the
//! production model: the fixture carries no persistence claim, and the
//! rejected combination names the §11.11 no-persistence rule.

use bbr_domain::FolderId;
use bbr_ui::components::{FolderMemberVisual, FolderRowSpec};

/// The folder-row-index offset per matrix family: family `n` renders its
/// rows at `bbr-folder-row-<n * FOLDERS_FAMILY_OFFSET + row>` (and the
/// nested ids at `bbr-folder-item-<same>-<m>`) so every id is unique
/// across the story (the E2E targets one exact row/toggle/member). Pinned
/// by the matrix test so ids can never drift from the declaration.
pub const FOLDERS_FAMILY_OFFSET: usize = 10;

/// One matrix row: a family label, the folder rows and expansion state
/// the fixture builds, and the matrix-only claims that name the rejected
/// combinations in the same case shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderCase {
    pub label: &'static str,
    /// The folder rows in creation order: `(title, member count)`. The
    /// fixture builds one `FolderRowSpec` per entry with the `tnum` count.
    pub rows: &'static [(&'static str, usize)],
    /// The row positions (0-based within `rows`) expanded in this family.
    pub expanded: &'static [usize],
    /// The nested member visuals: `(row position, member title)`. Only
    /// expanded rows may carry members (§11.11 — nested rows render only
    /// when expanded).
    pub members: &'static [(usize, &'static str)],
    /// Matrix-only: this case claims persisted folders (§11.11 — folders
    /// are session-local in v1, never persisted).
    pub claims_persisted: bool,
    /// Matrix-only: this case claims nested rows inside a collapsed
    /// folder (§11.11 — nested rows render only when expanded).
    pub claims_nested_in_collapsed: bool,
}

/// Every valid folders family — the single declaration the story renders.
#[rustfmt::skip]
pub const FOLDERS_VALID: &[FolderCase] = &[
    FolderCase {
        label: "collapsed folder",
        rows: &[("Work", 2)],
        expanded: &[],
        members: &[],
        claims_persisted: false,
        claims_nested_in_collapsed: false,
    },
    FolderCase {
        label: "expanded folder with nested rows",
        rows: &[("Work", 2)],
        expanded: &[0],
        members: &[(0, "Example"), (0, "Local")],
        claims_persisted: false,
        claims_nested_in_collapsed: false,
    },
    FolderCase {
        label: "empty collapsible folder",
        rows: &[("Archive", 0)],
        expanded: &[],
        members: &[],
        claims_persisted: false,
        claims_nested_in_collapsed: false,
    },
    FolderCase {
        label: "count tnum",
        rows: &[("Reading", 12)],
        expanded: &[],
        members: &[],
        claims_persisted: false,
        claims_nested_in_collapsed: false,
    },
    FolderCase {
        label: "hairlines between groups",
        rows: &[("Work", 2), ("Personal", 1)],
        expanded: &[0],
        members: &[(0, "Example")],
        claims_persisted: false,
        claims_nested_in_collapsed: false,
    },
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `FolderCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedFolder {
    pub label: &'static str,
    pub case: FolderCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`folder_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const FOLDERS_REJECTED: &[RejectedFolder] = &[
    RejectedFolder {
        label: "nested rows in a collapsed folder",
        case: FolderCase {
            label: "nested rows in a collapsed folder",
            rows: &[("Work", 2)],
            expanded: &[],
            members: &[(0, "Example")],
            claims_persisted: false,
            claims_nested_in_collapsed: true,
        },
        reason: "§11.11: nested rows render only when the folder is expanded — a collapsed folder exposes no nested rows",
    },
    RejectedFolder {
        label: "persisted folders",
        case: FolderCase {
            label: "persisted folders",
            rows: &[("Work", 2)],
            expanded: &[],
            members: &[],
            claims_persisted: true,
            claims_nested_in_collapsed: false,
        },
        reason: "§11.11: a folder is a session-local container of tab identities — in-memory in v1, never persisted (Phase 7 owns restoration)",
    },
];

/// The one folders validity authority: rejects the impossible combinations
/// the matrix forbids. The fixture builder refuses (`None`) any case this
/// rejects, so the story can never render a rejected folder list.
pub fn folder_case_valid(case: &FolderCase) -> bool {
    // §11.11: folders are session-local, never persisted.
    if case.claims_persisted {
        return false;
    }
    // §11.11: nested rows render only when the folder is expanded.
    if case.claims_nested_in_collapsed {
        return false;
    }
    // A member row whose folder is not expanded is the same impossible
    // combination expressed through the data: refuse it structurally.
    for (folder_pos, _) in case.members {
        if !case.expanded.contains(folder_pos) {
            return false;
        }
    }
    // A member row for a folder that does not exist cannot render.
    for (folder_pos, _) in case.members {
        if *folder_pos >= case.rows.len() {
            return false;
        }
    }
    true
}

/// Build the production `FolderRowSpec`/`FolderMemberVisual` lists for a
/// valid matrix family; `None` for any rejected combination. Rows keep
/// deterministic creation order (`bbr-folder-row-<family*OFFSET+pos>`),
/// distinct domain `FolderId`s, the `tnum` count, and the declared
/// expansion state; expanded rows carry their nested §11.1 member visuals
/// at deterministic `bbr-folder-item-<row>-<m>` ids.
pub fn build_folders_fixture(
    case: &FolderCase,
) -> Option<(Vec<FolderRowSpec>, Vec<FolderMemberVisual>)> {
    if !folder_case_valid(case) {
        return None;
    }
    let family = FOLDERS_VALID
        .iter()
        .position(|c| c.label == case.label)
        .unwrap_or(0);
    let offset = family * FOLDERS_FAMILY_OFFSET;
    let rows: Vec<FolderRowSpec> = case
        .rows
        .iter()
        .enumerate()
        .map(|(pos, (title, count))| FolderRowSpec {
            index: offset + pos,
            id: FolderId((offset + pos) as u64 + 1),
            title: title.to_string(),
            count: *count,
            expanded: case.expanded.contains(&pos),
        })
        .collect();
    let mut members = Vec::new();
    for (folder_pos, title) in case.members {
        let folder_row = rows
            .iter()
            .find(|row| row.index == offset + folder_pos)
            .expect("member folder row built above");
        let member_pos = members
            .iter()
            .filter(|m: &&FolderMemberVisual| m.folder_index == folder_row.index)
            .count();
        let letter = title
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase())
            .unwrap_or('?');
        members.push(FolderMemberVisual {
            folder: folder_row.id,
            folder_index: folder_row.index,
            member: member_pos,
            title: title.to_string(),
            letter,
            hue: member_pos % 6,
        });
    }
    Some((rows, members))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::components::folders::folders_ids;
    use std::collections::BTreeSet;

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in FOLDERS_VALID {
            assert!(
                folder_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let (rows, members) = build_folders_fixture(case)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(rows.len(), case.rows.len(), "{}", case.label);
            for row in &rows {
                assert_ne!(row.id, FolderId(0), "{}", case.label);
                assert!(!row.title.is_empty(), "{}", case.label);
            }
            // Members belong to expanded rows only.
            for member in &members {
                let row = rows
                    .iter()
                    .find(|r| r.id == member.folder)
                    .expect("member folder row");
                assert!(row.expanded, "{}", case.label);
                assert_eq!(row.index, member.folder_index, "{}", case.label);
            }
        }
        for rejected in FOLDERS_REJECTED {
            assert!(
                !folder_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_folders_fixture(&rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        let labels: Vec<&str> = FOLDERS_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "collapsed folder",
                "expanded folder with nested rows",
                "empty collapsible folder",
                "count tnum",
                "hairlines between groups",
            ]
        );
        let mut seen = BTreeSet::new();
        for case in FOLDERS_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn nested_rows_render_only_inside_expanded_folders() {
        // The expanded family exposes its nested ids; every other family
        // exposes none.
        let by_label = |label: &str| {
            FOLDERS_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let collapsed = build_folders_fixture(by_label("collapsed folder")).unwrap();
        assert!(collapsed.1.is_empty(), "collapsed folder has no members");
        let expanded = build_folders_fixture(by_label("expanded folder with nested rows")).unwrap();
        assert_eq!(expanded.1.len(), 2);
        // The ids expose the nested rows only for the expanded folder.
        let collapsed_ids = folders_ids(&collapsed.0, &collapsed.1);
        assert!(
            !collapsed_ids
                .iter()
                .any(|id| id.starts_with("bbr-folder-item")),
            "collapsed folders expose no nested ids"
        );
        let expanded_ids = folders_ids(&expanded.0, &expanded.1);
        assert!(expanded_ids.contains(&"bbr-folder-item-10-0".to_string()));
        assert!(expanded_ids.contains(&"bbr-folder-item-10-0-title".to_string()));
        assert!(expanded_ids.contains(&"bbr-folder-item-10-0-favicon".to_string()));
        assert!(expanded_ids.contains(&"bbr-folder-item-10-0-close".to_string()));
        assert!(expanded_ids.contains(&"bbr-folder-item-10-1".to_string()));
    }

    #[test]
    fn ids_are_unique_across_families_and_stable() {
        // Family `n` renders at `bbr-folder-row-<n * OFFSET + pos>` so the
        // E2E targets one exact row/toggle/member id without collisions.
        let by_label = |label: &str| {
            FOLDERS_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let first = build_folders_fixture(by_label("collapsed folder")).unwrap();
        let second = build_folders_fixture(by_label("expanded folder with nested rows")).unwrap();
        assert_eq!(first.0[0].index, 0, "family 0 starts at bbr-folder-row-0");
        assert_eq!(
            second.0[0].index, 10,
            "family 1 starts at bbr-folder-row-10 (the pinned offset)"
        );
        let all_indices: BTreeSet<usize> = FOLDERS_VALID
            .iter()
            .flat_map(|c| build_folders_fixture(c).expect("valid family").0)
            .map(|r| r.index)
            .collect();
        let total_rows: usize = FOLDERS_VALID.iter().map(|c| c.rows.len()).sum();
        assert_eq!(all_indices.len(), total_rows, "row indices are unique");
        // Distinct domain FolderIds per row: session-local identity that
        // commands key on (Z2-1: FolderId keys state/commands).
        let ids: BTreeSet<FolderId> = FOLDERS_VALID
            .iter()
            .flat_map(|c| build_folders_fixture(c).expect("valid family").0)
            .map(|r| r.id)
            .collect();
        assert_eq!(ids.len(), total_rows, "FolderIds are unique");
        // Stable: rebuilding yields identical specs.
        assert_eq!(
            build_folders_fixture(by_label("expanded folder with nested rows")),
            build_folders_fixture(by_label("expanded folder with nested rows"))
        );
    }

    #[test]
    fn counts_carry_the_tnum_and_the_empty_folder_stays_collapsible() {
        let by_label = |label: &str| {
            FOLDERS_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let count = build_folders_fixture(by_label("count tnum")).unwrap();
        assert_eq!(count.0[0].count, 12, "the §11.11 tnum count column");
        let empty = build_folders_fixture(by_label("empty collapsible folder")).unwrap();
        assert_eq!(
            empty.0[0].count, 0,
            "empty collapsible folder keeps the row"
        );
        assert!(!empty.0[0].expanded);
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = FOLDERS_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec!["nested rows in a collapsed folder", "persisted folders"]
        );
        for rejected in FOLDERS_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
