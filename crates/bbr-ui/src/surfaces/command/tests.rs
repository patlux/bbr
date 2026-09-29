//! Unit tests for the §11.4 floating command presentation (packet
//! `component-gallery-compositions.md`): the plain model validation, the
//! stable ids, the UI-local states, the geometry/render construction, and
//! the rejected missing-key / mutual-exclusion cases.

use super::*;

fn row(key: &str, group: &str) -> CommandResultRow {
    CommandResultRow {
        key: key.into(),
        group: group.into(),
        title: "Title".into(),
        subtitle: Some("https://example.com".into()),
        url: None,
        destructive: false,
    }
}

fn model(presentation: CommandPresentation) -> CommandModel {
    CommandModel {
        presentation,
        theme: crate::theme::Theme::new(
            crate::theme::ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::Normal,
        ),
        chrome_focused: presentation.is_open(),
        text: "https://example.com".into(),
        edit: crate::components::EditState::end_of("https://example.com"),
        line_sink: None,
        state: CommandResultState::Results,
        results: vec![row("tab:0", "exact_tab"), row("fallback", "fallback")],
        selected: 0,
        entrance: None,
        glide: None,
    }
}

#[test]
fn presentation_discriminant_is_closed_by_default_and_reflects_openness() {
    assert_eq!(CommandPresentation::default(), CommandPresentation::Closed);
    assert!(!CommandPresentation::Closed.is_open());
    assert!(CommandPresentation::CenteredPalette.is_open());
    assert!(CommandPresentation::AnchoredOverlay.is_open());
}

#[test]
fn presentation_discriminant_round_trips_its_pinned_raw_values() {
    // The pinned raw values: 0 = closed, 1 = centered palette, 2 = anchored
    // overlay. Every valid value round-trips.
    for presentation in [
        CommandPresentation::Closed,
        CommandPresentation::CenteredPalette,
        CommandPresentation::AnchoredOverlay,
    ] {
        assert_eq!(
            CommandPresentation::from_raw(presentation.raw()),
            Ok(presentation)
        );
    }
}

#[test]
fn an_invalid_presentation_discriminant_is_rejected() {
    // The mutual-exclusion contract: exactly one presentation can be open,
    // so any raw value outside the three valid discriminants (including a
    // hypothetical "centered + anchored simultaneously" encoding) is
    // refused.
    for raw in [3, 4, u8::MAX] {
        assert_eq!(
            CommandPresentation::from_raw(raw),
            Err(CommandModelError::InvalidPresentation),
            "raw {raw} is not a valid presentation"
        );
    }
}

#[test]
fn a_model_with_a_missing_or_empty_row_key_is_rejected() {
    let mut m = model(CommandPresentation::CenteredPalette);
    assert_eq!(m.validate(), Ok(()));

    m.results[0].key.clear();
    assert_eq!(m.validate(), Err(CommandModelError::MissingRowKey));

    m.results[0].key = " ".into();
    assert_eq!(m.validate(), Err(CommandModelError::MissingRowKey));

    m.results[0].key = "tab:0".into();
    m.results[1].key.clear();
    assert_eq!(m.validate(), Err(CommandModelError::MissingRowKey));
}

#[test]
fn every_state_renders_an_exact_stable_state_row_id() {
    // The typed state rows keep their §11.5 ids per surface: the palette's
    // own `bbr-palette-loading` / `-empty` / `-error`, and the overlay's
    // `bbr-command-loading` / `-empty` / `-error` (the same strings the
    // in-rail surface uses).
    for (presentation, loading, empty, error) in [
        (
            CommandPresentation::CenteredPalette,
            "bbr-palette-loading",
            "bbr-palette-empty",
            "bbr-palette-error",
        ),
        (
            CommandPresentation::AnchoredOverlay,
            "bbr-command-loading",
            "bbr-command-empty",
            "bbr-command-error",
        ),
    ] {
        assert_eq!(state_id(&model(presentation), "loading"), loading);
        assert_eq!(state_id(&model(presentation), "empty"), empty);
        assert_eq!(state_id(&model(presentation), "error"), error);
    }
}

#[test]
fn result_and_state_ids_are_deterministic_and_surface_scoped() {
    // Result rows render under the surface's own prefix with the stable
    // key; the two surfaces never share a row-id family.
    assert_eq!(PALETTE_RESULT_PREFIX, "bbr-palette-result");
    assert_eq!(
        format!("{PALETTE_RESULT_PREFIX}-{}", row("tab:0", "exact_tab").key),
        "bbr-palette-result-tab:0"
    );
    assert_eq!(
        format!("bbr-command-result-{}", row("action:2", "action").key),
        "bbr-command-result-action:2"
    );
    assert_ne!(PALETTE_RESULT_PREFIX, "bbr-command-result");
}

#[test]
fn the_open_surface_ids_are_exact() {
    assert_eq!(PALETTE_ID, "bbr-palette");
    assert_eq!(PALETTE_FIELD_ID, "bbr-palette-field");
    assert_eq!(PALETTE_TEXT_ID, "bbr-palette-text");
    assert_eq!(PALETTE_MODE_ID, "bbr-palette-mode");
    assert_eq!(PALETTE_CURSOR_ID, "bbr-palette-cursor");
    assert_eq!(OVERLAY_ID, "bbr-command-overlay");
    assert_eq!(OVERLAY_FIELD_ID, "bbr-command-overlay-field");
    assert_eq!(OVERLAY_TEXT_ID, "bbr-command-overlay-text");
    assert_eq!(OVERLAY_CURSOR_ID, "bbr-command-overlay-cursor");
    // The overlay fade is a render-internal element id, never a §15 id.
    assert_eq!(OVERLAY_FADE_ELEMENT_ID, "bbr-command-overlay-fade");
}

#[test]
fn the_closed_presentation_is_a_valid_renderable_model() {
    // The closed presentation is the default and carries no rows; the
    // render path is proven by the Gallery E2E (the closed family renders
    // the `bbr-command-portal-closed` card). Here we pin the model shape.
    let closed = CommandModel {
        presentation: CommandPresentation::Closed,
        theme: crate::theme::Theme::new(
            crate::theme::ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::Normal,
        ),
        chrome_focused: false,
        text: String::new(),
        edit: crate::components::EditState::default(),
        line_sink: None,
        state: CommandResultState::Inactive,
        results: Vec::new(),
        selected: 0,
        entrance: None,
        glide: None,
    };
    assert_eq!(closed.validate(), Ok(()));
    assert!(!closed.chrome_focused);
    assert!(closed.results.is_empty());
}

#[test]
fn geometry_rect_is_plain_point_data() {
    let rect = CommandRect::new(12.0, 48.0, 360.0, 320.0);
    assert_eq!(rect.x, 12.0);
    assert_eq!(rect.y, 48.0);
    assert_eq!(rect.width, 360.0);
    assert_eq!(rect.height, 320.0);
    let rect2 = CommandRect::new(rect.x, rect.y, rect.width, rect.height);
    assert_eq!(rect, rect2);
    let default_rect = CommandRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
    assert_eq!(CommandRect::new(0.0, 0.0, 0.0, 0.0), default_rect);
}

#[test]
fn ui_local_state_enum_matches_the_typed_surface_states() {
    // The UI-local result-state enum is the closed set the renderer draws:
    // no unreachable no-match state exists (the production Location/NewTab
    // fallback makes no-result unreachable).
    assert_eq!(
        [
            CommandResultState::Inactive,
            CommandResultState::Empty,
            CommandResultState::Loading,
            CommandResultState::Results,
            CommandResultState::ActivationError,
        ]
        .len(),
        5
    );
    assert_eq!(CommandResultState::default(), CommandResultState::Inactive);
}

#[test]
fn saved_page_results_use_the_drawn_bookmark_mark() {
    // Regression: the palette/overlay renderer had no Saved Page branch and
    // showed the `→` fallback; the in-rail surface showed the text `★`.
    // A Saved Page result carries no letter — it renders `Icon::Bookmark`.
    assert_eq!(super::rows::group_glyph(&row("k", "saved_page")), None);
    for (group, letter) in [
        ("exact_tab", "T"),
        ("space", "S"),
        ("action", "A"),
        ("history", "H"),
        ("fallback", "→"),
    ] {
        assert_eq!(
            super::rows::group_glyph(&row("k", group)),
            Some(letter),
            "letter for {group}"
        );
    }
}

/// S12: the query's first case-insensitive occurrence is the highlighted
/// range; no match (or an empty query) highlights nothing.
#[test]
fn match_range_finds_the_query_in_the_title() {
    use super::rows::match_range;
    assert_eq!(
        match_range("GPUI render pipeline", "gpui render"),
        Some((0, 11))
    );
    assert_eq!(match_range("YouTube Music", "tube"), Some((3, 7)));
    assert_eq!(match_range("YouTube", "  "), None);
    assert_eq!(match_range("YouTube", "vimeo"), None);
    assert_eq!(match_range("Ä", "äb"), None);
}

/// S12: the trailing hint is `↵` on the selected row and the kind
/// otherwise; destructive rows always say so.
#[test]
fn kind_hint_names_the_result_kind() {
    use super::rows::kind_hint;
    assert_eq!(kind_hint(&row("k", "current_tab"), true), "↵");
    assert_eq!(kind_hint(&row("k", "current_tab"), false), "Tab");
    assert_eq!(kind_hint(&row("k", "history"), false), "History");
    assert_eq!(kind_hint(&row("k", "saved_page"), false), "Saved");
    let mut destructive = row("k", "action");
    destructive.destructive = true;
    assert_eq!(kind_hint(&destructive, true), "destructive");
    let mut search = row("k", "fallback");
    search.url = None;
    assert_eq!(kind_hint(&search, false), "Search");
    search.url = Some("https://example.com".into());
    assert_eq!(kind_hint(&search, false), "URL");
}

/// Blueprint D04: an action shows its shortcut in the kind column; the `>`
/// prefix is a mode switch, not part of the match needle.
#[test]
fn actions_show_their_shortcut_and_the_prefix_is_not_the_needle() {
    use super::rows::kind_hint;
    let mut action = row("action:4", "action");
    action.subtitle = Some("⇧⌘P".into());
    assert_eq!(kind_hint(&action, false), "⇧⌘P");
    assert_eq!(kind_hint(&action, true), "↵");
    action.subtitle = None;
    assert_eq!(kind_hint(&action, false), "Action");
    assert_eq!(super::actions_query("> pin"), "pin");
    assert_eq!(super::actions_query("pin"), "pin");
}
