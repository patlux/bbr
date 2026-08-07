//! Unit tests for the §11.4 command-surface state, ids, and wire conversion.

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
fn closed_by_default_and_is_open_reflects_the_presentation() {
    assert_eq!(CommandSurface::default(), CommandSurface::None);
    assert!(!CommandSurface::None.is_open());
    assert!(CommandSurface::Palette.is_open());
    assert!(CommandSurface::Overlay.is_open());
}

#[test]
fn each_surface_couples_to_its_command_bar_mode() {
    assert_eq!(CommandSurface::None.mode(), CommandBarMode::Inactive);
    assert_eq!(CommandSurface::Palette.mode(), CommandBarMode::NewTab);
    assert_eq!(CommandSurface::Overlay.mode(), CommandBarMode::Location);
}

#[test]
fn coupled_surface_keeps_only_the_matching_presentation() {
    use CommandBarMode::{Inactive, Location, MoveTab, NewSpace, NewTab};
    // A mode entry preserves the open surface only when it re-enters
    // the coupled mode; every other combination closes to the in-rail
    // bar (mutual exclusion by construction).
    assert_eq!(
        CommandSurface::Palette.coupled_surface(NewTab),
        CommandSurface::Palette
    );
    assert_eq!(
        CommandSurface::Overlay.coupled_surface(Location),
        CommandSurface::Overlay
    );
    assert_eq!(
        CommandSurface::None.coupled_surface(NewTab),
        CommandSurface::None
    );
    assert_eq!(
        CommandSurface::None.coupled_surface(Location),
        CommandSurface::None
    );
    assert_eq!(
        CommandSurface::Palette.coupled_surface(Location),
        CommandSurface::None
    );
    assert_eq!(
        CommandSurface::Overlay.coupled_surface(NewTab),
        CommandSurface::None
    );
    assert_eq!(
        CommandSurface::Palette.coupled_surface(MoveTab),
        CommandSurface::None
    );
    assert_eq!(
        CommandSurface::Overlay.coupled_surface(NewSpace),
        CommandSurface::None
    );
    assert_eq!(
        CommandSurface::Palette.coupled_surface(Inactive),
        CommandSurface::None
    );
}

#[test]
fn surface_ids_expose_palette_or_overlay_only_while_open() {
    let closed_state = CommandResultState::Inactive;
    assert!(surface_ids(CommandSurface::None, closed_state, &[]).is_empty());
    assert_eq!(
        surface_ids(CommandSurface::Palette, closed_state, &[]),
        vec![
            PALETTE_ID,
            PALETTE_FIELD_ID,
            PALETTE_TEXT_ID,
            PALETTE_MODE_ID,
            PALETTE_CURSOR_ID,
        ]
    );
    assert_eq!(
        surface_ids(CommandSurface::Overlay, closed_state, &[]),
        vec![OVERLAY_ID]
    );
}

#[test]
fn surface_ids_expose_result_and_state_rows_when_the_engine_publishes() {
    let rows = [row("tab:0", "exact_tab"), row("fallback", "fallback")];
    // The centered palette: `bbr-palette-result-<k>` rows under the
    // base ids, or its typed state rows.
    let palette_results = surface_ids(CommandSurface::Palette, CommandResultState::Results, &rows);
    assert!(palette_results.contains(&"bbr-palette-result-tab:0".to_string()));
    assert!(palette_results.contains(&"bbr-palette-result-fallback".to_string()));
    let palette_loading = surface_ids(CommandSurface::Palette, CommandResultState::Loading, &[]);
    assert!(palette_loading.contains(&PALETTE_STATE_LOADING.to_string()));
    let palette_error = surface_ids(
        CommandSurface::Palette,
        CommandResultState::ActivationError,
        &[],
    );
    assert!(palette_error.contains(&PALETTE_STATE_ERROR.to_string()));
    // The field-anchored overlay keeps the §11.5 result ids.
    let overlay_results = surface_ids(CommandSurface::Overlay, CommandResultState::Results, &rows);
    assert!(overlay_results.contains(&"bbr-command-result-tab:0".to_string()));
    assert!(overlay_results.contains(&"bbr-command-result-fallback".to_string()));
    assert!(
        !overlay_results
            .iter()
            .any(|id| id.starts_with(PALETTE_RESULT_PREFIX)),
        "the overlay never advertises palette result ids"
    );
    let overlay_loading = surface_ids(CommandSurface::Overlay, CommandResultState::Loading, &[]);
    assert!(overlay_loading.contains(&STATE_ID_LOADING.to_string()));
    let overlay_error = surface_ids(
        CommandSurface::Overlay,
        CommandResultState::ActivationError,
        &[],
    );
    assert!(overlay_error.contains(&STATE_ID_ERROR.to_string()));
}

#[test]
fn protocol_conversion_maps_the_wire_enum_one_to_one() {
    let wire: bbr_testkit::protocol::CommandSurface = CommandSurface::None.into();
    assert_eq!(wire, bbr_testkit::protocol::CommandSurface::None);
    let wire: bbr_testkit::protocol::CommandSurface = CommandSurface::Palette.into();
    assert_eq!(wire, bbr_testkit::protocol::CommandSurface::Palette);
    let wire: bbr_testkit::protocol::CommandSurface = CommandSurface::Overlay.into();
    assert_eq!(wire, bbr_testkit::protocol::CommandSurface::Overlay);
}

#[test]
fn overlay_left_inset_keeps_the_card_flush_with_the_field() {
    // The §11.4 overlay card is positioned inside the §13 surface layer,
    // whose containing block starts at `x = sidebar_width`. The measured
    // field rect is window-absolute, so the inset must subtract the rail
    // width: `sidebar_width + inset` must land exactly on the field's
    // leading edge (DESIGN.md §11.4). A bug that used `rect.x` directly
    // placed the card a full rail width right of the field.
    let sidebar_width = 260.0;
    for field_x in [0.0, 12.0, 260.0, 320.0, 500.0] {
        let inset = overlay_left_inset(field_x, sidebar_width);
        assert_eq!(
            sidebar_width + inset,
            field_x,
            "flush for field x={field_x}"
        );
    }
    // Rail collapsed: the inset is the window-absolute x itself.
    assert_eq!(overlay_left_inset(12.0, 0.0), 12.0);
}
