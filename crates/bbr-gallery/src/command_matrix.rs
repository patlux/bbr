//! The Command composition matrix authority (packet
//! `component-gallery-compositions.md`): **one declaration source** for
//! every valid floating-command family and every rejected combination. The
//! gallery story renders exactly the valid families through the production
//! `bbr_ui::surfaces::command::command_card` composition, and the fixture
//! builder refuses (`None`) any case the validity authority rejects — so
//! the story, the fixture builder, the matrix table, and the tests cannot
//! drift.
//!
//! Matrix rows come straight from the packet's Command row: closed/inactive;
//! centered Cmd-T palette and field-anchored Cmd-L/click overlay; the typed
//! loading/empty/results/activation-error states; selected and destructive
//! rows; long-title truncation; reduced-motion; contrast standard/increased
//! — all with the exact stable §11.4/§11.5 ids and deterministic keys.
//! Rejected: centered + anchored simultaneously (the one presentation
//! discriminant refuses any invalid raw value) and a result row with a
//! missing/empty stable key. Missing-icon and cross-profile-hint are
//! explicitly inapplicable to this production model (the group glyph is
//! used; the current portal model has no profile-hint field), so no such
//! family exists here.
//!
//! The plain [`bbr_ui::surfaces::command::CommandModel`] construction
//! lives in the focused [`crate::command_fixture`] module (a bounded split
//! so this authority stays under the ~500-line module budget); the fixture
//! builds the model only — every visual decision stays in `bbr-ui`/DESIGN,
//! and no activation callback exists (production activation stays
//! app/Portal-owned; the Gallery observes presentation only).

use bbr_ui::surfaces::command::{CommandPresentation, CommandResultState};
use bbr_ui::theme::{ContrastMode, Density};

/// One plain result-row descriptor (the fields the production renderer
/// actually presents: stable key / group / title / subtitle / destructive).
#[derive(Debug, Clone, Copy)]
pub struct RowSpec {
    /// The stable external key (`tab:<id>`, `space:<id>`, `action:<id>`,
    /// `fallback`, …) — the `bbr-palette-result-<key>` /
    /// `bbr-command-result-<key>` render-id suffix.
    pub key: &'static str,
    /// The §11.5 group name driving the group glyph.
    pub group: &'static str,
    pub title: &'static str,
    pub subtitle: Option<&'static str>,
    pub destructive: bool,
}

impl RowSpec {
    /// The deterministic result rows of the "results" families (the §11.5
    /// row anatomy with a selected/destructive mix and a fallback row —
    /// the production Location/NewTab query always carries one).
    pub const fn results() -> &'static [RowSpec] {
        &[
            RowSpec {
                key: "tab:1",
                group: "exact_tab",
                title: "BBR Home",
                subtitle: None,
                destructive: false,
            },
            RowSpec {
                key: "tab:2",
                group: "current_tab",
                title: "Docs — GPUI",
                subtitle: Some("https://gpui.dev/docs"),
                destructive: false,
            },
            RowSpec {
                key: "space:3",
                group: "space",
                title: "Work",
                subtitle: Some("Space"),
                destructive: false,
            },
            RowSpec {
                key: "action:1",
                group: "action",
                title: "New Space",
                subtitle: None,
                destructive: false,
            },
            RowSpec {
                key: "action:2",
                group: "action",
                title: "Close Tab",
                subtitle: None,
                destructive: true,
            },
            RowSpec {
                key: "fallback",
                group: "fallback",
                title: "https://bbr.dev/",
                subtitle: Some("Navigate"),
                destructive: false,
            },
        ]
    }
}

/// One matrix row: a family label, the production model fields, and the
/// matrix-only claims that name the rejected combinations in the same case
/// shape.
#[derive(Debug, Clone, Copy)]
pub struct CommandCase {
    pub label: &'static str,
    /// The UI-local presentation discriminant (one open surface at a time).
    pub presentation: CommandPresentation,
    /// The UI-local typed result state.
    pub state: CommandResultState,
    /// Whether the open surface owns chrome focus (the §11.4 block cursor).
    pub chrome_focused: bool,
    /// The selected result index (`None`-like position when the family has
    /// no results).
    pub selected: usize,
    /// The field text (the overlay opens with the full URL preselected).
    pub text: &'static str,
    /// The result rows of a `Results` family (`&[]` otherwise).
    pub rows: &'static [RowSpec],
    /// §4: long one-line titles (truncation).
    pub long_titles: bool,
    /// §15: the reduced-motion mode field.
    pub reduced_motion: bool,
    /// §3: standard/increased contrast.
    pub contrast: ContrastMode,
    /// The density mode (drives the palette radius/width).
    pub density: Density,
    // Matrix-only claims (the rejected combinations).
    pub claims_missing_key: bool,
    pub claims_invalid_presentation: bool,
}

impl CommandCase {
    /// The valid-family constructor: the production fields with the
    /// matrix-only claims defaulting to false.
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        presentation: CommandPresentation,
        state: CommandResultState,
        selected: usize,
        text: &'static str,
        rows: &'static [RowSpec],
        long_titles: bool,
        reduced_motion: bool,
        contrast: ContrastMode,
    ) -> Self {
        Self {
            label,
            presentation,
            state,
            chrome_focused: presentation.is_open(),
            selected,
            text,
            rows,
            long_titles,
            reduced_motion,
            contrast,
            density: Density::Normal,
            claims_missing_key: false,
            claims_invalid_presentation: false,
        }
    }

    /// The claim-only constructor for a rejected combination: the same
    /// case shape with exactly one matrix-only claim set.
    const fn rejected(label: &'static str, claims_missing_key: bool) -> Self {
        Self {
            label,
            presentation: CommandPresentation::CenteredPalette,
            state: CommandResultState::Results,
            chrome_focused: true,
            selected: 0,
            text: "https://bbr.dev/",
            rows: RowSpec::results(),
            long_titles: false,
            reduced_motion: false,
            contrast: ContrastMode::Standard,
            density: Density::Normal,
            claims_missing_key,
            claims_invalid_presentation: !claims_missing_key,
        }
    }
}

/// One rejected-combination row: the label the story shows and the exact
/// reason.
#[derive(Debug, Clone, Copy)]
pub struct RejectedCommandCase {
    pub label: &'static str,
    pub reason: &'static str,
    pub case: CommandCase,
}

/// The valid families (declaration order = the story's state-selector
/// order; pinned by a matrix test): closed/inactive; centered palette and
/// anchored overlay across the typed loading/empty/results/activation-error
/// states; selected and destructive rows; long-title truncation;
/// reduced-motion; standard/increased contrast.
pub const COMMAND_VALID: &[CommandCase] = &[
    CommandCase::new(
        "closed / inactive — no surface",
        CommandPresentation::Closed,
        CommandResultState::Inactive,
        0,
        "",
        &[],
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "centered palette — results (selected 1)",
        CommandPresentation::CenteredPalette,
        CommandResultState::Results,
        1,
        "bbr",
        RowSpec::results(),
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "centered palette — loading",
        CommandPresentation::CenteredPalette,
        CommandResultState::Loading,
        0,
        "bbr",
        &[],
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "centered palette — empty",
        CommandPresentation::CenteredPalette,
        CommandResultState::Empty,
        0,
        "",
        &[],
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "centered palette — activation error",
        CommandPresentation::CenteredPalette,
        CommandResultState::ActivationError,
        0,
        "bbr",
        &[],
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "anchored overlay — results (selected 0)",
        CommandPresentation::AnchoredOverlay,
        CommandResultState::Results,
        0,
        "https://bbr.dev/",
        RowSpec::results(),
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "anchored overlay — loading",
        CommandPresentation::AnchoredOverlay,
        CommandResultState::Loading,
        0,
        "https://bbr.dev/",
        &[],
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "anchored overlay — empty",
        CommandPresentation::AnchoredOverlay,
        CommandResultState::Empty,
        0,
        "",
        &[],
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "anchored overlay — activation error",
        CommandPresentation::AnchoredOverlay,
        CommandResultState::ActivationError,
        0,
        "https://bbr.dev/",
        &[],
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "centered palette — selected + destructive rows",
        CommandPresentation::CenteredPalette,
        CommandResultState::Results,
        4,
        "close",
        RowSpec::results(),
        false,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "anchored overlay — long-title truncation",
        CommandPresentation::AnchoredOverlay,
        CommandResultState::Results,
        1,
        "https://bbr.dev/",
        RowSpec::results(),
        true,
        false,
        ContrastMode::Standard,
    ),
    CommandCase::new(
        "centered palette — reduced-motion + increased contrast",
        CommandPresentation::CenteredPalette,
        CommandResultState::Results,
        0,
        "bbr",
        RowSpec::results(),
        false,
        true,
        ContrastMode::Increased,
    ),
];

/// The rejected combinations with their reasons — the story shows the whole
/// matrix contract, and `command_case_valid`/`build_command_fixture` refuse
/// every one.
pub const COMMAND_REJECTED: &[RejectedCommandCase] = &[
    RejectedCommandCase {
        label: "centered + anchored simultaneously",
        reason: "exactly one presentation discriminant (`CommandPresentation`); the palette and the overlay are mutually exclusive, and any invalid raw discriminant is refused by `CommandPresentation::from_raw`",
        case: CommandCase::rejected("centered + anchored simultaneously", false),
    },
    RejectedCommandCase {
        label: "result row without a stable key",
        reason: "every rendered row carries the stable `bbr-*` key (`CommandModel::validate` refuses a missing/empty key)",
        case: CommandCase::rejected("result row without a stable key", true),
    },
];

/// The validity authority: a case is valid only when it does not claim a
/// rejected combination. The presentation mutual exclusion is additionally
/// enforced by construction (`CommandPresentation::from_raw` refuses any
/// invalid discriminant) and the missing-key claim is refused by the
/// fixture builder through the production `CommandModel::validate`.
pub fn command_case_valid(case: &CommandCase) -> bool {
    !case.claims_missing_key && !case.claims_invalid_presentation
}

/// The state-selector options: the valid family labels in declaration
/// order (pinned by a matrix test so the options can never drift from
/// [`COMMAND_VALID`]).
pub const COMMAND_STATE_OPTIONS: &[&str] = &[
    "closed / inactive — no surface",
    "centered palette — results (selected 1)",
    "centered palette — loading",
    "centered palette — empty",
    "centered palette — activation error",
    "anchored overlay — results (selected 0)",
    "anchored overlay — loading",
    "anchored overlay — empty",
    "anchored overlay — activation error",
    "centered palette — selected + destructive rows",
    "anchored overlay — long-title truncation",
    "centered palette — reduced-motion + increased contrast",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_fixture::build_command_fixture;

    #[test]
    fn every_valid_family_builds_a_model_through_the_production_validation() {
        for case in COMMAND_VALID {
            let model = build_command_fixture(case)
                .unwrap_or_else(|| panic!("valid family {} must build", case.label));
            // The production validation authority accepts the model.
            assert_eq!(
                model.validate(),
                Ok(()),
                "family {} fails the production validation",
                case.label
            );
            // The presentation discriminant round-trips its pinned raw
            // value (the mutual exclusion by construction).
            assert_eq!(
                CommandPresentation::from_raw(case.presentation.raw()),
                Ok(case.presentation),
                "family {} presentation round-trips",
                case.label
            );
        }
    }

    #[test]
    fn every_rejected_family_refuses_construction() {
        for rejected in COMMAND_REJECTED {
            assert!(
                !command_case_valid(&rejected.case),
                "rejected {} passes the validity authority",
                rejected.label
            );
            assert!(
                build_command_fixture(&rejected.case).is_none(),
                "rejected {} builds a model",
                rejected.label
            );
        }
    }

    #[test]
    fn the_state_options_never_drift_from_the_valid_families() {
        let labels: Vec<&str> = COMMAND_VALID.iter().map(|c| c.label).collect();
        assert_eq!(COMMAND_STATE_OPTIONS, labels.as_slice());
    }

    #[test]
    fn the_presentation_mutual_exclusion_and_missing_key_are_the_only_rejections() {
        // Exactly the two packet-rejected combinations exist: an invalid
        // presentation discriminant (the simultaneous palette+overlay is
        // impossible to construct) and a missing/empty row key.
        assert_eq!(COMMAND_REJECTED.len(), 2);
        assert_eq!(
            bbr_ui::surfaces::command::CommandPresentation::from_raw(3),
            Err(bbr_ui::surfaces::command::CommandModelError::InvalidPresentation)
        );
        // Every valid family carries deterministic stable keys.
        for case in COMMAND_VALID {
            for spec in case.rows {
                assert!(!spec.key.is_empty(), "{} carries an empty key", case.label);
            }
        }
    }

    #[test]
    fn missing_icon_and_cross_profile_hint_families_do_not_exist() {
        // The packet marks missing-icon and cross-profile-hint explicitly
        // inapplicable to this production model (the group glyph is used
        // and the current portal model has no profile-hint field); the
        // matrix declares no such family.
        for case in COMMAND_VALID {
            assert!(
                !case.label.contains("missing-icon"),
                "no missing-icon family for the floating command surface"
            );
            assert!(
                !case.label.contains("profile-hint") && !case.label.contains("cross-profile"),
                "no cross-profile-hint family for the floating command surface"
            );
        }
    }
}
