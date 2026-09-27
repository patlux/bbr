//! The §11.5 command result palette (DESIGN.md §11.5).
//!
//! Renders the typed result-state surface from the same app state the
//! snapshot reports: the ranked result rows with the selected accent rail
//! inside the list and the destructive error tint, plus the truthful
//! loading / empty / activation-error rows. The palette lives in
//! the sidebar rail (native web views are real child views, so GPUI chrome
//! cannot visually overlap the glass); it is dismissible with Escape and
//! carries the stable §15 ids.

use gpui::{div, prelude::*, px, AnyElement, Div, SharedString, Stateful, Styled};

use bbr_ui::icons::Icon;
use bbr_ui::theme::{Radius, Spacing, Theme};

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
    /// the `tint.accent`; destructive rows render the error/destructive
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
                // §11.5 selected rail + `tint.accent` inside the list.
                el.bg(p.tint_accent).border_l_2().border_color(p.accent)
            })
            .child(
                div()
                    .size(theme.density.icon())
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
                    .child(group_mark(theme, row)),
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
            .when(row.collection.is_some(), |el| {
                // §11.16 optional one-level collection label (`type:meta`).
                let collection = row.collection.clone().expect("checked above");
                el.child(
                    div()
                        .id(SharedString::from(format!(
                            "{id_prefix}-collection-{}",
                            row.key
                        )))
                        .max_w(px(96.0))
                        .overflow_hidden()
                        .text_ellipsis()
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_faint)
                        .child(SharedString::from(collection)),
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
}

/// The leading mark of a result row: the drawn §11.16 bookmark glyph for a
/// Saved Page (never a Unicode `★`), otherwise the group letter.
fn group_mark(theme: &Theme, row: &CommandResultRow) -> AnyElement {
    match group_glyph(row) {
        Some(letter) => SharedString::from(letter).into_any_element(),
        None => Icon::Bookmark
            .canvas(theme, theme.density.icon() - px(4.0))
            .into_any_element(),
    }
}

/// The stable one-letter group mark (`T` tab / `S` space / `H` history /
/// `A` action / `→` fallback); `None` for a Saved Page, which renders the
/// drawn bookmark glyph instead.
fn group_glyph(row: &CommandResultRow) -> Option<&'static str> {
    match row.group.as_str() {
        "tab" | "exact_tab" | "current_pinned" | "current_tab" | "other_space" => Some("T"),
        "space" => Some("S"),
        "saved_page" => None,
        "action" => Some("A"),
        "history" => Some("H"),
        _ => Some("→"),
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
            collection: None,
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
            assert_eq!(
                group_glyph(&row("k", group)),
                Some(glyph),
                "glyph for {group}"
            );
        }
        // A Saved Page renders the drawn bookmark glyph, never a letter or `★`.
        assert_eq!(group_glyph(&row("k", "saved_page")), None);
    }
}
