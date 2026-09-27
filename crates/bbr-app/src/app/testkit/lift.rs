//! D2 lift evidence (DESIGN §2, §18.1): which rail surfaces render the
//! static `elev:lift`, resolved from the same row states, folder-member
//! selection and chip state the render paints with — never re-derived from
//! pixels.

use bbr_ui::components::{row_lift, FolderMemberVisual, RowVisuals};
use bbr_ui::sidebar::strip::{chip_lifts, space_chip_id};
use bbr_ui::state::SpaceChipVisuals;

/// The §15 ids of the lifted rail surfaces: selected tab rows
/// (`bbr-tab-row-<n>`), selected folder members
/// (`bbr-folder-item-<n>-<m>`), then the active space chip
/// (`bbr-space-chip-<space>`), in render order.
pub(super) fn lifted_ids(
    rows: &[RowVisuals],
    members: &[FolderMemberVisual],
    chips: &[SpaceChipVisuals],
) -> Vec<String> {
    let rows = rows
        .iter()
        .filter(|row| row_lift(row.state))
        .map(|row| format!("{}-{}", bbr_ui::components::tab_row::TAB_ROW_ID, row.index));
    let members = members
        .iter()
        .filter(|member| member.selected)
        .map(|member| {
            bbr_ui::components::folders::folder_item_id(member.folder_index, member.member)
                .to_string()
        });
    let chips = chips
        .iter()
        .filter(|chip| chip_lifts(chip.selected, chip.drag_target))
        .map(|chip| space_chip_id(chip.space).to_string());
    rows.chain(members).chain(chips).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::state::TabVisualState;

    fn row(index: usize, state: TabVisualState) -> RowVisuals {
        RowVisuals {
            index,
            title: "Example".into(),
            state,
            favicon: None,
            pinned: false,
            show_close: true,
            reveal_close: false,
        }
    }

    fn member(folder_index: usize, member: usize, selected: bool) -> FolderMemberVisual {
        FolderMemberVisual {
            folder: bbr_domain::FolderId(1),
            folder_index,
            member,
            title: "Member".into(),
            letter: 'M',
            hue: 0,
            image: None,
            selected,
        }
    }

    fn chip(space: u64, selected: bool, drag_target: bool) -> SpaceChipVisuals {
        SpaceChipVisuals {
            space: bbr_browser::SpaceId(space),
            title: "Space".into(),
            profile_title: "Default".into(),
            identity: "book".into(),
            theme: bbr_domain::SpaceTheme::Cobalt,
            selected,
            drag_target,
            companion: None,
        }
    }

    #[test]
    fn only_selected_rows_members_and_the_active_chip_report_the_lift() {
        let rows = [
            row(0, TabVisualState::Hover),
            row(1, TabVisualState::Selected),
            row(2, TabVisualState::Active),
            row(3, TabVisualState::DragTarget),
        ];
        let members = [member(0, 0, false), member(0, 1, true)];
        let chips = [chip(1, true, false), chip(2, false, false)];
        assert_eq!(
            lifted_ids(&rows, &members, &chips),
            vec![
                "bbr-tab-row-1".to_string(),
                "bbr-folder-item-0-1".to_string(),
                "bbr-space-chip-1".to_string()
            ]
        );
        assert!(lifted_ids(&rows[..1], &members[..1], &chips[1..]).is_empty());
        // A drag-targeted active chip shows the §10.9 treatment, not the lift.
        assert!(lifted_ids(&[], &[], &[chip(1, true, true)]).is_empty());
    }
}
