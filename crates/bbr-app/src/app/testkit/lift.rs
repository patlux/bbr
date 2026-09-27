//! D2 lift evidence (DESIGN §2, §18.1): which rail surfaces render the
//! static `elev:lift`, resolved from the same row states and folder-member
//! selection the render paints with — never re-derived from pixels.

use bbr_ui::components::{row_lift, FolderMemberVisual, RowVisuals};

/// The §15 ids of the lifted rail surfaces: selected tab rows
/// (`bbr-tab-row-<n>`) then selected folder members
/// (`bbr-folder-item-<n>-<m>`), in render order.
pub(super) fn lifted_ids(rows: &[RowVisuals], members: &[FolderMemberVisual]) -> Vec<String> {
    rows.iter()
        .filter(|row| row_lift(row.state))
        .map(|row| format!("{}-{}", bbr_ui::components::tab_row::TAB_ROW_ID, row.index))
        .chain(
            members
                .iter()
                .filter(|member| member.selected)
                .map(|member| {
                    bbr_ui::components::folders::folder_item_id(member.folder_index, member.member)
                        .to_string()
                }),
        )
        .collect()
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

    #[test]
    fn only_selected_rows_and_members_report_the_lift() {
        let rows = [
            row(0, TabVisualState::Hover),
            row(1, TabVisualState::Selected),
            row(2, TabVisualState::Active),
            row(3, TabVisualState::DragTarget),
        ];
        let members = [member(0, 0, false), member(0, 1, true)];
        assert_eq!(
            lifted_ids(&rows, &members),
            vec![
                "bbr-tab-row-1".to_string(),
                "bbr-folder-item-0-1".to_string()
            ]
        );
        assert!(lifted_ids(&rows[..1], &members[..1]).is_empty());
    }
}
