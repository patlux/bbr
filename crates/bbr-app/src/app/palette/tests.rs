//! Unit tests for the §11.4 command-surface state, ids, and wire conversion.

use super::*;
use bbr_command::rank::Activation;
use bbr_ui::surfaces::command::{
    OVERLAY_CURSOR_ID, OVERLAY_FIELD_ID, OVERLAY_ID, OVERLAY_STATE_ERROR, OVERLAY_STATE_LOADING,
    OVERLAY_TEXT_ID, PALETTE_CURSOR_ID, PALETTE_FIELD_ID, PALETTE_ID, PALETTE_MODE_ID,
    PALETTE_RESULT_PREFIX, PALETTE_STATE_ERROR, PALETTE_STATE_LOADING, PALETTE_TEXT_ID,
};

fn row(key: &str, group: &str) -> CommandResultRow {
    CommandResultRow {
        key: key.into(),
        group: group.into(),
        title: "Title".into(),
        subtitle: Some("https://example.com".into()),
        url: None,
        collection: None,
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
        vec![
            OVERLAY_ID,
            OVERLAY_FIELD_ID,
            OVERLAY_TEXT_ID,
            OVERLAY_CURSOR_ID,
        ]
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
    assert!(overlay_loading.contains(&OVERLAY_STATE_LOADING.to_string()));
    let overlay_error = surface_ids(
        CommandSurface::Overlay,
        CommandResultState::ActivationError,
        &[],
    );
    assert!(overlay_error.contains(&OVERLAY_STATE_ERROR.to_string()));
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
fn overlay_portal_placement_covers_the_address_control() {
    let field = WebRect::new(12.0, 48.0, 280.0, 34.0);
    // §11.4: the anchored overlay width clamps to a 360 pt floor, and the
    // overlay **covers** the address control — top and leading edges flush
    // with the control's, never a dropdown below it.
    let placement = overlay_placement(field, 360.0);
    let PortalPlacement::Anchored(placement) = placement else {
        panic!("overlay must use anchored portal placement");
    };
    assert_eq!(placement.anchor_point, Pt::new(12.0, 48.0));
    assert_eq!(placement.anchor, PortalAnchor::TopLeft);
    assert_eq!(placement.size, Sz::new(360.0, 320.0));
    assert_eq!(placement.offset, Pt::new(0.0, 0.0));
    assert_eq!(
        placement.fit,
        FitMode::SnapToWindowWithMargin(Edges::all(8.0))
    );
}
