//! U1 command-surface visible-id regressions.

use super::*;

#[test]
fn visible_ids_expose_the_open_command_surface_only_while_open() {
    // §11.4/§15: the centered palette ids and the field-anchored overlay
    // id appear only while the respective presentation is open, and they
    // appear even with the sidebar rail hidden (the overlay floats above
    // the window content, elev:2 — §13).
    let browser = space_snapshot();
    use crate::app::palette::CommandSurface as Surface;

    let palette = visible_ids(
        &browser,
        true,
        crate::command_bar::CommandBarMode::NewTab,
        false,
        None,
        None,
        Surface::Palette,
        crate::app::command_rank::CommandResultState::Inactive,
        &[],
        &[],
        None,
        None,
        dense_all(0),
        None,
    );
    for id in [
        "bbr-palette",
        "bbr-palette-field",
        "bbr-palette-text",
        "bbr-palette-mode",
        "bbr-palette-cursor",
    ] {
        assert!(
            palette.contains(&id.to_string()),
            "missing {id}: {palette:?}"
        );
    }
    assert!(!palette.iter().any(|id| id == "bbr-command-overlay"));

    let overlay = visible_ids(
        &browser,
        true,
        crate::command_bar::CommandBarMode::Location,
        false,
        None,
        None,
        Surface::Overlay,
        crate::app::command_rank::CommandResultState::Inactive,
        &[],
        &[],
        None,
        None,
        dense_all(0),
        None,
    );
    assert!(overlay.contains(&"bbr-command-overlay".to_string()));
    assert!(!overlay.iter().any(|id| id.starts_with("bbr-palette")));

    // Closed (the default): neither surface advertises ids.
    let closed = visible_ids(
        &browser,
        false,
        crate::command_bar::CommandBarMode::Inactive,
        true,
        None,
        None,
        closed(),
        crate::app::command_rank::CommandResultState::Inactive,
        &[],
        &[],
        None,
        None,
        dense_all(0),
        None,
    );
    assert!(!closed.iter().any(|id| id.starts_with("bbr-palette")));
    assert!(!closed.iter().any(|id| id == "bbr-command-overlay"));
}

#[test]
fn visible_ids_expose_the_open_surface_result_rows_when_published() {
    // §11.4/§15: once the shared engine publishes ranked rows, the open
    // surface exposes its own result ids (the palette's
    // `bbr-palette-result-<k>`, the overlay's `bbr-command-result-<k>`)
    // even with the sidebar rail hidden; the typed state rows use the
    // surface-specific id families.
    let browser = space_snapshot();
    use crate::app::palette::CommandSurface as Surface;
    let rows = [
        crate::app::command_rank::CommandResultRow {
            key: "tab:0".into(),
            group: "exact_tab".into(),
            title: "Alpha".into(),
            subtitle: Some("https://a.example".into()),
            destructive: false,
            activation: bbr_command::rank::Activation::Navigate {
                url: "https://a.example".into(),
            },
        },
        crate::app::command_rank::CommandResultRow {
            key: "fallback".into(),
            group: "fallback".into(),
            title: "Search DuckDuckGo".into(),
            subtitle: None,
            destructive: false,
            activation: bbr_command::rank::Activation::Search {
                terms: "alpha".into(),
            },
        },
    ];
    let state = crate::app::command_rank::CommandResultState::Results;

    let palette = visible_ids(
        &browser,
        true,
        crate::command_bar::CommandBarMode::NewTab,
        false,
        None,
        None,
        Surface::Palette,
        state,
        &rows,
        &[],
        None,
        None,
        dense_all(0),
        None,
    );
    assert!(palette.contains(&"bbr-palette-result-tab:0".to_string()));
    assert!(palette.contains(&"bbr-palette-result-fallback".to_string()));
    assert!(
        !palette.iter().any(|id| id == "bbr-command-results"),
        "the in-rail result list ids are never advertised while a
        surface is open: {palette:?}"
    );

    let overlay = visible_ids(
        &browser,
        true,
        crate::command_bar::CommandBarMode::Location,
        false,
        None,
        None,
        Surface::Overlay,
        state,
        &rows,
        &[],
        None,
        None,
        dense_all(0),
        None,
    );
    assert!(overlay.contains(&"bbr-command-result-tab:0".to_string()));
    assert!(overlay.contains(&"bbr-command-result-fallback".to_string()));
    assert!(
        !overlay
            .iter()
            .any(|id| id.starts_with("bbr-palette-result")),
        "the overlay never advertises palette result ids"
    );

    // The typed loading/error state rows use the surface id families.
    let loading = visible_ids(
        &browser,
        true,
        crate::command_bar::CommandBarMode::NewTab,
        false,
        None,
        None,
        Surface::Palette,
        crate::app::command_rank::CommandResultState::Loading,
        &[],
        &[],
        None,
        None,
        dense_all(0),
        None,
    );
    assert!(loading.contains(&"bbr-palette-loading".to_string()));
    let overlay_error = visible_ids(
        &browser,
        true,
        crate::command_bar::CommandBarMode::Location,
        false,
        None,
        None,
        Surface::Overlay,
        crate::app::command_rank::CommandResultState::ActivationError,
        &[],
        &[],
        None,
        None,
        dense_all(0),
        None,
    );
    assert!(overlay_error.contains(&"bbr-command-error".to_string()));
}
