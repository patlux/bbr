//! Blueprint D10: the §15 ids the History page draws (the typed snapshot's
//! visible-id list reads them from the same model the render uses).

use super::{
    history_row_id, HistoryModel, CLEAR_DATA_ID, HISTORY_CLEAR_ID, HISTORY_ID, HISTORY_SEARCH_ID,
};

/// Every §15 id the page draws for `model` (the render's own ids).
pub fn history_visible_ids(model: &HistoryModel) -> Vec<String> {
    let mut ids: Vec<String> = [
        HISTORY_ID,
        "bbr-history-all",
        "bbr-history-closed",
        HISTORY_SEARCH_ID,
        HISTORY_CLEAR_ID,
        "bbr-history-list",
    ]
    .iter()
    .map(|id| id.to_string())
    .collect();
    if model.view == 0 {
        ids.extend((0..3).map(|index| format!("bbr-history-range-{index}")));
    }
    if model.empty.is_some() {
        ids.push("bbr-history-empty".into());
    }
    for group in &model.groups {
        for row in &group.rows {
            ids.push(history_row_id(row.id, model.view == 1));
            if model.view == 0 && model.selected == Some(row.id) {
                ids.push(format!("bbr-history-remove-{}", row.id));
            }
        }
    }
    if let Some(clear) = &model.clear {
        ids.push(CLEAR_DATA_ID.into());
        ids.push("bbr-clear-data-range".into());
        if clear.range_open {
            ids.push("bbr-clear-data-range-menu".into());
            ids.extend(
                (0..clear.ranges.len()).map(|index| format!("bbr-clear-data-range-{index}")),
            );
        }
        ids.extend((0..clear.options.len()).map(|index| format!("bbr-clear-data-option-{index}")));
        ids.push("bbr-clear-data-cancel".into());
        ids.push("bbr-clear-data-confirm".into());
    }
    ids
}
