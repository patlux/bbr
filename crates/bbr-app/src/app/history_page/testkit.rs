//! Blueprint D10 testkit seams: the open History page on the wire and one
//! History input through the method the page's controls call.

use bbr_testkit::protocol::{
    ClearDataSnapshot, ClearOptionSnapshot, HistoryGroupSnapshot, HistoryInput, HistoryRowSnapshot,
    HistorySnapshot,
};
use bbr_ui::surfaces::history::HistoryEvent;
use gpui::{Context, Window};

use crate::app::BbrApp;

impl BbrApp {
    /// The open History page as drawn (`None` while it is closed).
    pub(crate) fn history_snapshot(&self, window: &Window) -> Option<HistorySnapshot> {
        if !self.history_open() {
            return None;
        }
        let model = self.history_model(self.library_handle.is_focused(window));
        Some(HistorySnapshot {
            view: if model.view == 1 { "closed" } else { "all" }.into(),
            range: ["today", "week", "all"][model.range.min(2)].into(),
            search: model.search.to_string(),
            closed_count: model.closed_count,
            groups: model
                .groups
                .iter()
                .map(|group| HistoryGroupSnapshot {
                    title: group.title.to_string(),
                    rows: group
                        .rows
                        .iter()
                        .map(|row| HistoryRowSnapshot {
                            id: row.id,
                            time: row.time.as_ref().map(|t| t.to_string()),
                            title: row.title.to_string(),
                            url: row.url.to_string(),
                        })
                        .collect(),
                })
                .collect(),
            selected: model.selected,
            empty: model.empty.map(|e| e.to_string()),
            focused: model.focused,
            clear: model.clear.map(|clear| ClearDataSnapshot {
                profile: clear.profile.to_string(),
                range: clear.ranges[clear.range].to_string(),
                range_open: clear.range_open,
                options: clear
                    .options
                    .iter()
                    .map(|option| ClearOptionSnapshot {
                        label: option.label.to_string(),
                        checked: option.checked,
                        enabled: option.enabled,
                    })
                    .collect(),
                confirm_enabled: clear.confirm_enabled,
            }),
        })
    }

    /// One testkit History input through the controls' method.
    pub(crate) fn history_input(
        &mut self,
        input: HistoryInput,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.history_open() {
            return false;
        }
        let event = match input {
            HistoryInput::View { view } => HistoryEvent::View(view),
            HistoryInput::Range { range } => HistoryEvent::Range(range),
            HistoryInput::Open { id } => HistoryEvent::Open(id),
            HistoryInput::Remove { id } => HistoryEvent::Remove(id),
            HistoryInput::ClearOpen => HistoryEvent::ClearOpen,
            HistoryInput::ClearRangeMenu => HistoryEvent::ClearRangeMenu,
            HistoryInput::ClearRange { index } => HistoryEvent::ClearRange(index),
            HistoryInput::ClearToggle { index } => HistoryEvent::ClearToggle(index),
            HistoryInput::ClearCancel => HistoryEvent::ClearCancel,
            HistoryInput::ClearConfirm => HistoryEvent::ClearConfirm,
        };
        self.on_history_event(event, window, cx);
        true
    }
}
