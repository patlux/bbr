//! Blueprint D16 testkit seams: the open Settings window on the wire and
//! one Settings input through the method the controls call.

use bbr_ui::surfaces::settings::{SettingsControl, SettingsEvent, SettingsPane};
use gpui::{Context, Window};

use super::BbrApp;

/// The testkit view of the open Settings window.
impl BbrApp {
    pub(crate) fn settings_snapshot(&self) -> Option<bbr_testkit::protocol::SettingsSnapshot> {
        self.settings.read().window?;
        let model = self.settings_model();
        let rows = model
            .sections
            .iter()
            .flat_map(|section| section.rows.iter())
            .map(|row| bbr_testkit::protocol::SettingsRowSnapshot {
                id: row.id.to_string(),
                title: row.title.to_string(),
                detail: row.detail.as_ref().map(|detail| detail.to_string()),
                value: control_value(&row.control),
            })
            .collect();
        Some(bbr_testkit::protocol::SettingsSnapshot {
            pane: model.pane.key().to_string(),
            rows,
            open_select: model.open_select.map(|row| row.to_string()),
            active_slider: self
                .settings
                .read()
                .active_slider
                .as_ref()
                .map(|row| row.to_string()),
            sheet: model
                .sheet
                .map(|sheet| bbr_testkit::protocol::SettingsSheetSnapshot {
                    title: sheet.title.to_string(),
                    fields: sheet.fields.iter().map(|f| f.value.to_string()).collect(),
                    focused: sheet.fields.iter().position(|f| f.focused),
                    swatch: sheet.swatches.map(|(_, chosen)| chosen),
                    message: sheet.message.map(|m| m.to_string()),
                    confirm: sheet.confirm.to_string(),
                    confirm_enabled: sheet.confirm_enabled,
                }),
        })
    }

    /// One testkit Settings input through the controls' method.
    pub(crate) fn settings_input(
        &mut self,
        input: bbr_testkit::protocol::SettingsInput,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        use bbr_testkit::protocol::SettingsInput;
        let event = match input {
            SettingsInput::Pane { pane } => match SettingsPane::parse(&pane) {
                Some(pane) => SettingsEvent::Pane(pane),
                None => return false,
            },
            SettingsInput::Toggle { row } => SettingsEvent::Toggle(row.into()),
            SettingsInput::Choose { row, index } => SettingsEvent::Choose(row.into(), index),
            SettingsInput::Press { row } => SettingsEvent::Press(row.into()),
            SettingsInput::OpenSelect { row } => SettingsEvent::OpenSelect(row.into()),
            SettingsInput::Slide { row, value } => SettingsEvent::Slide(row.into(), value),
            SettingsInput::Dismiss => SettingsEvent::Dismiss,
            SettingsInput::Select { row } => SettingsEvent::Select(row.into()),
            SettingsInput::SheetField { index } => SettingsEvent::SheetField(index),
            SettingsInput::SheetSwatch { index } => SettingsEvent::SheetSwatch(index),
            SettingsInput::SheetCancel => SettingsEvent::SheetCancel,
            SettingsInput::SheetConfirm => SettingsEvent::SheetConfirm,
            SettingsInput::Key { key, text } => SettingsEvent::Key {
                key: key.into(),
                text: text.map(Into::into),
            },
        };
        self.on_settings_event(event, Some(window), cx);
        true
    }
}

/// What a control shows, on the wire.
fn control_value(control: &SettingsControl) -> String {
    match control {
        SettingsControl::None => String::new(),
        SettingsControl::Badge(text) => text.to_string(),
        SettingsControl::Toggle(on) => if *on { "on" } else { "off" }.to_string(),
        SettingsControl::Select { value, options }
        | SettingsControl::Segmented { value, options } => options
            .get(*value)
            .map(|o| o.to_string())
            .unwrap_or_default(),
        SettingsControl::Button { label, .. } | SettingsControl::Link(label) => label.to_string(),
        SettingsControl::Keys(keys) => keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join(" "),
        SettingsControl::Density(value) => match value {
            None => "automatic".to_string(),
            Some(0) => "small".to_string(),
            Some(1) => "normal".to_string(),
            Some(_) => "large".to_string(),
        },
        SettingsControl::Slider { label, .. } => label.to_string(),
        SettingsControl::Chip { letter, .. } => letter.to_string(),
        SettingsControl::ChipButton { letter, label, .. } => format!("{letter} {label}"),
    }
}
