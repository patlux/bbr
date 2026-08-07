//! The §11.5 command result palette (DESIGN.md §11.5).
//!
//! Renders the typed result-state surface from the same app state the
//! snapshot reports: the ranked result rows with the selected accent rail
//! inside the list and the destructive error tint, plus the truthful
//! loading / empty / activation-error rows. The palette lives in
//! the sidebar rail (native web views are real child views, so GPUI chrome
//! cannot visually overlap the glass); it is dismissible with Escape and
//! carries the stable §15 ids.

use gpui::{div, prelude::*, px, Div, SharedString, Stateful, Styled};

use crate::theme::{Radius, Spacing, Theme};

use super::command_rank::{CommandResultRow, CommandResultState};
use super::BbrApp;

/// The stable §15 ids of the typed state rows (DESIGN.md §11.5).
pub const STATE_ID_LOADING: &str = "bbr-command-loading";
pub const STATE_ID_EMPTY: &str = "bbr-command-empty";
pub const STATE_ID_ERROR: &str = "bbr-command-error";

/// The stable ids the palette renders for the current ranking state: the
/// surface container plus either the result rows or the typed state row.
/// Shared by the render and the §15 visible-id derivation so E2E ids can
/// never drift from the shell.
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub fn surface_ids(state: CommandResultState, results: &[CommandResultRow]) -> Vec<String> {
    let mut ids = Vec::new();
    match state {
        CommandResultState::Inactive => {}
        CommandResultState::Results => {
            ids.push("bbr-command-results".into());
            ids.extend(
                results
                    .iter()
                    .map(|row| format!("bbr-command-result-{}", row.key)),
            );
        }
        CommandResultState::Loading => {
            ids.push("bbr-command-results".into());
            ids.push(STATE_ID_LOADING.into());
        }
        CommandResultState::Empty => {
            ids.push("bbr-command-results".into());
            ids.push(STATE_ID_EMPTY.into());
        }
        CommandResultState::ActivationError => {
            ids.push("bbr-command-results".into());
            ids.push(STATE_ID_ERROR.into());
        }
    }
    ids
}

impl BbrApp {
    /// The §11.5 result palette, rendered directly beneath the command field
    /// in the rail while the location/new-tab surface is open. `None` when
    /// the surface is inactive — or when a §11.4 palette/overlay
    /// presentation is open, because the floating surface renders the same
    /// engine's results itself (the in-rail list would duplicate them).
    pub fn command_result_surface(&self, theme: &Theme) -> Option<Stateful<Div>> {
        let ranking = &self.command_ranking;
        if ranking.state == CommandResultState::Inactive
            || self.command_surface != super::palette::CommandSurface::None
        {
            return None;
        }
        let p = theme.palette;
        let rows: Vec<Stateful<Div>> = match ranking.state {
            CommandResultState::Results => ranking
                .results
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    self.command_result_row(
                        theme,
                        row,
                        index == ranking.selected,
                        "bbr-command-result",
                    )
                })
                .collect(),
            CommandResultState::Loading => {
                vec![state_row(
                    theme,
                    SharedString::from(STATE_ID_LOADING),
                    "Ranking…",
                )]
            }
            CommandResultState::Empty => {
                vec![state_row(
                    theme,
                    SharedString::from(STATE_ID_EMPTY),
                    "Type a destination",
                )]
            }
            CommandResultState::ActivationError => vec![state_row(
                theme,
                SharedString::from(STATE_ID_ERROR),
                ranking
                    .activation_error
                    .as_deref()
                    .unwrap_or("Activation failed"),
            )],
            CommandResultState::Inactive => return None,
        };
        Some(
            div()
                .id("bbr-command-results")
                .mx(Spacing::sp3())
                .mb(Spacing::sp2())
                .rounded(theme.radius.large())
                .bg(p.surface_overlay)
                .border_1()
                .border_color(p.hairline)
                .py(Spacing::sp1())
                .children(rows),
        )
    }

    /// One §11.5 result row: `[group glyph] [title] [mono url trunc] [key
    /// hint]`. The selected row renders the accent rail inside the list with
    /// the selection tint; destructive rows render the error/destructive
    /// text language (tint only — no confirmation dialog is invented).
    /// `id_prefix` selects the surface's stable §15 id family
    /// (`bbr-command-result-<k>` for the in-rail bar and the field-anchored
    /// overlay, `bbr-palette-result-<k>` for the centered palette).
    pub(super) fn command_result_row(
        &self,
        theme: &Theme,
        row: &CommandResultRow,
        selected: bool,
        id_prefix: &str,
    ) -> Stateful<Div> {
        let p = theme.palette;
        let title_color = if row.destructive {
            p.status_error
        } else {
            p.ink
        };
        div()
            .id(SharedString::from(format!("{id_prefix}-{}", row.key)))
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::sp2())
            .px(Spacing::sp3())
            .py(Spacing::sp1())
            .when(selected, |el| {
                // §11.5 selected rail + selection tint inside the list.
                el.bg(p.tint_selected).border_l_2().border_color(p.accent)
            })
            .child(
                div()
                    .size(px(14.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(Radius.small())
                    .bg(if row.destructive {
                        p.status_error_surface
                    } else {
                        p.surface_field
                    })
                    .text_size(theme.type_.key.size)
                    .font_weight(theme.type_.key.weight)
                    .text_color(if row.destructive {
                        p.status_error
                    } else {
                        p.ink_faint
                    })
                    .child(SharedString::from(group_glyph(row))),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{id_prefix}-title-{}", row.key)))
                    .max_w(px(140.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(theme.type_.title.size)
                    .font_weight(theme.type_.title.weight)
                    .text_color(title_color)
                    .child(SharedString::from(row.title.clone())),
            )
            .when(row.subtitle.is_some(), |el| {
                let subtitle = row.subtitle.clone().expect("checked above");
                el.child(
                    div()
                        .id(SharedString::from(format!("{id_prefix}-url-{}", row.key)))
                        .flex_1()
                        .overflow_hidden()
                        .text_ellipsis()
                        .text_size(theme.type_.mono.size)
                        .text_color(p.ink_secondary)
                        .child(SharedString::from(subtitle)),
                )
            })
            .when(row.destructive || selected, |el| {
                el.child(
                    div()
                        .text_size(theme.type_.key.size)
                        .text_color(if row.destructive {
                            p.status_error
                        } else {
                            p.ink_faint
                        })
                        .child(SharedString::from(if row.destructive {
                            "destructive"
                        } else {
                            "↵"
                        })),
                )
            })
    }

    /// The ranked result rows (or the truthful typed state row) of the open
    /// §11.4 surface, with the surface-specific §15 id families:
    /// `result_prefix` for rows (`bbr-palette-result` / `bbr-command-result`),
    /// `state_prefix` for the loading/empty/error rows (`bbr-palette` /
    /// `bbr-command`). Shared by the centered palette and the field-anchored
    /// overlay presentations.
    pub(super) fn surface_results(
        &self,
        theme: &Theme,
        result_prefix: &str,
        state_prefix: &str,
    ) -> Vec<Stateful<Div>> {
        let ranking = &self.command_ranking;
        match ranking.state {
            CommandResultState::Results => ranking
                .results
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    self.command_result_row(theme, row, index == ranking.selected, result_prefix)
                })
                .collect(),
            CommandResultState::Loading => {
                let id = format!("{state_prefix}-loading");
                vec![state_row(theme, id.clone().into(), "Ranking…")]
            }
            CommandResultState::Empty => {
                let id = format!("{state_prefix}-empty");
                vec![state_row(theme, id.clone().into(), "Type a destination")]
            }
            CommandResultState::ActivationError => {
                let id = format!("{state_prefix}-error");
                vec![state_row(
                    theme,
                    id.into(),
                    ranking
                        .activation_error
                        .as_deref()
                        .unwrap_or("Activation failed"),
                )]
            }
            CommandResultState::Inactive => Vec::new(),
        }
    }
}

/// The stable leading glyph of a result row: the group's one-letter mark
/// (`T` tab / `S` space / `A` action / `H` history / `→` fallback).
fn group_glyph(row: &CommandResultRow) -> String {
    match row.group.as_str() {
        "tab" | "exact_tab" | "current_pinned" | "current_tab" | "other_space" => "T".into(),
        "space" => "S".into(),
        "action" => "A".into(),
        "history" => "H".into(),
        _ => "→".into(),
    }
}

/// One typed state row (loading/empty/activation-error) with its
/// stable §15 id; error rows render the destructive tint. Shared by the
/// in-rail result surface and the §11.4 palette/overlay presentations
/// (each passes its own surface-specific id family).
pub(super) fn state_row(theme: &Theme, id: SharedString, text: &str) -> Stateful<Div> {
    let error = id.ends_with("-error");
    div()
        .id(id)
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .text_size(theme.type_.meta.size)
        .text_color(if error {
            theme.palette.status_error
        } else {
            theme.palette.ink_faint
        })
        .child(SharedString::from(text.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_command::rank::Activation;

    fn row(key: &str, group: &str) -> CommandResultRow {
        CommandResultRow {
            key: key.into(),
            group: group.into(),
            title: "Title".into(),
            subtitle: Some("https://example.com".into()),
            destructive: false,
            activation: Activation::Navigate {
                url: "https://example.com".into(),
            },
        }
    }

    #[test]
    fn surface_ids_mirror_the_typed_states() {
        assert!(surface_ids(CommandResultState::Inactive, &[]).is_empty());
        let ids = surface_ids(
            CommandResultState::Results,
            &[row("tab:0", "exact_tab"), row("fallback", "fallback")],
        );
        assert_eq!(
            ids,
            vec![
                "bbr-command-results".to_string(),
                "bbr-command-result-tab:0".to_string(),
                "bbr-command-result-fallback".to_string(),
            ]
        );
        assert_eq!(
            surface_ids(CommandResultState::Loading, &[]),
            vec![
                "bbr-command-results".to_string(),
                "bbr-command-loading".to_string(),
            ]
        );
        assert_eq!(
            surface_ids(CommandResultState::Empty, &[]),
            vec![
                "bbr-command-results".to_string(),
                "bbr-command-empty".to_string(),
            ]
        );
        assert_eq!(
            surface_ids(CommandResultState::ActivationError, &[]),
            vec![
                "bbr-command-results".to_string(),
                "bbr-command-error".to_string(),
            ]
        );
    }

    #[test]
    fn group_glyph_covers_every_group() {
        for (group, glyph) in [
            ("exact_tab", "T"),
            ("current_pinned", "T"),
            ("current_tab", "T"),
            ("other_space", "T"),
            ("space", "S"),
            ("action", "A"),
            ("history", "H"),
            ("fallback", "→"),
        ] {
            assert_eq!(group_glyph(&row("k", group)), glyph, "glyph for {group}");
        }
    }
}
