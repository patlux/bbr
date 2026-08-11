//! The Address/Input matrix authority (packet
//! `component-gallery-browser-components.md`, Group 3): **one declaration
//! source** for every valid §11.5 address-pill and command-staging family
//! and every rejected combination. The gallery stories render exactly the
//! valid families through the production `bbr_ui::components::address`
//! symbols (`address_control`, `staging_surface`), and the fixture
//! builders refuse (`None`) any case the validity authority rejects — so
//! the story, the fixture builder, the matrix table, and the tests cannot
//! drift. The rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Address/input row: domain
//! pill (with/without `www.`, IDN punycode→Unicode, explicit port); the
//! accessory slot reserved-empty only (Copy Link is future, §17.14 open —
//! never invented); the hover fill step (`hovered: true`/`false`, §10.2
//! wash at `dur:fine`); the profile-hint chip presence (§11.3 transient,
//! `Some`/`None`); and the staging field (New Space / Move Tab / the
//! §11.8 empty New Tab surface) with block cursor `show_cursor`
//! true/false, hint present/absent, and profile-hint present/absent
//! (production `StagingModel.profile_hint`/`AddressProfileHint`).
//! Rejected: pill-as-text-field (no editing state, no cursor, no
//! selection — §11.5), a focus ring on the pill (never, any state —
//! §11.5), and a Copy-Link accessory glyph (§17.14 open; v1 ships the
//! slot reserved-empty only).

use bbr_ui::components::{AddressModel, AddressProfileHint, StagingModel};
use bbr_ui::text::{address_domain, address_domain_budget_chars, mid_ellipsis};
use bbr_ui::theme::Theme;
use gpui::SharedString;

/// The §11.3 profile-hint fixture: a deterministic §12 chip (creation-
/// order index, monogram, hue) plus the hint label, built as a plain
/// `bbr-ui` DTO — never the app's controller-derived chip.
pub fn profile_hint_fixture() -> AddressProfileHint {
    AddressProfileHint {
        profile_index: 1,
        spec: crate::fixtures::profile_chip_spec(bbr_domain::ProfileHue::Violet),
        label: "Switch to Private".into(),
    }
}

/// One matrix row: a family label and the production `AddressModel`
/// fields. The matrix-only fields (`claims_editing`,
/// `claims_focus_ring`, `claims_copy_link`) exist solely to name the
/// rejected combinations in the same case shape; they are never mapped
/// into `AddressModel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressCase {
    pub label: &'static str,
    /// The live URL the §11.5 domain derivation runs on (the pill shows
    /// the domain only; the full address lives in the palette). The
    /// fixture derives the label through the production
    /// `bbr_ui::text::address_domain` + budget + `mid_ellipsis` chain.
    pub url: &'static str,
    /// §10.2 hover wash: the pointer hovers the pill.
    pub hovered: bool,
    /// §11.3 cross-context profile-hint chip presence.
    pub profile_hint: bool,
    /// Matrix-only: this case claims an editing state (cursor/selection)
    /// on the pill (§11.5 rejects it — the pill is a display control).
    pub claims_editing: bool,
    /// Matrix-only: this case claims a focus ring (§11.5 rejects it — no
    /// border and no focus ring at any time, any state).
    pub claims_focus_ring: bool,
    /// Matrix-only: this case claims a Copy-Link accessory glyph (§17.14
    /// open — v1 ships the accessory slot reserved-empty only).
    pub claims_copy_link: bool,
}

impl AddressCase {
    /// The valid-family constructor: the four production fields with the
    /// matrix-only flags defaulting to false. `#[allow]` keeps the dense
    /// const table readable (the fields are the documented matrix row).
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        url: &'static str,
        hovered: bool,
        profile_hint: bool,
    ) -> Self {
        AddressCase {
            label,
            url,
            hovered,
            profile_hint,
            claims_editing: false,
            claims_focus_ring: false,
            claims_copy_link: false,
        }
    }
}

/// Every valid address-pill family — the single declaration the story
/// renders. The `hovered: false`/`profile_hint: None` rest sides are the
/// domain families; the hover step and the profile-hint chip are their own
/// families (packet matrix: "hover fill step", "profile-hint chip
/// presence").
#[rustfmt::skip]
pub const ADDRESS_VALID: &[AddressCase] = &[
    AddressCase::new("domain — no www", "https://example.com/", false, false),
    AddressCase::new("domain — with www (elided)", "https://www.example.com/", false, false),
    AddressCase::new("domain — IDN (punycode → Unicode)", "https://xn--bcher-kva.example/", false, false),
    AddressCase::new("domain — explicit port (dropped)", "https://example.com:8443/", false, false),
    AddressCase::new("hover fill step (hovered)", "https://example.com/", true, false),
    AddressCase::new("profile-hint chip (present)", "https://example.com/", false, true),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `AddressCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedAddress {
    pub label: &'static str,
    pub case: AddressCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`address_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const ADDRESS_REJECTED: &[RejectedAddress] = &[
    RejectedAddress {
        label: "pill-as-text-field (editing state, cursor, selection)",
        case: AddressCase {
            label: "pill-as-text-field",
            url: "https://example.com/",
            hovered: false,
            profile_hint: false,
            claims_editing: true,
            claims_focus_ring: false,
            claims_copy_link: false,
        },
        reason: "§11.5: the address is a display control, not a text field — no editing state, no cursor, no selection, never keyboard text",
    },
    RejectedAddress {
        label: "a focus ring on the pill",
        case: AddressCase {
            label: "a focus ring on the pill",
            url: "https://example.com/",
            hovered: false,
            profile_hint: false,
            claims_editing: false,
            claims_focus_ring: true,
            claims_copy_link: false,
        },
        reason: "§11.5: no border and no focus ring at any time, any state — the pill participates in the chrome focus domain as a button, never a field",
    },
    RejectedAddress {
        label: "Copy-Link accessory",
        case: AddressCase {
            label: "Copy-Link accessory",
            url: "https://example.com/",
            hovered: false,
            profile_hint: false,
            claims_editing: false,
            claims_focus_ring: false,
            claims_copy_link: true,
        },
        reason: "§17.14 open: v1 ships the accessory slot reserved-empty only; Copy Link is a future decision and is never invented here",
    },
];

/// The one pill validity authority: rejects the impossible combinations
/// the matrix forbids. The fixture builder refuses (`None`) any case this
/// rejects, so the story can never render a rejected pill.
pub fn address_case_valid(case: &AddressCase) -> bool {
    // §11.5: the pill is a display control — no editing state, no cursor,
    // no selection, no keyboard text.
    if case.claims_editing {
        return false;
    }
    // §11.5: no border and no focus ring at any time, any state.
    if case.claims_focus_ring {
        return false;
    }
    // §17.14 open: the accessory slot is reserved-empty only in v1.
    if case.claims_copy_link {
        return false;
    }
    true
}

/// Build the production `AddressModel` for a valid matrix family; `None`
/// for any rejected combination. The domain label derives through the
/// production `bbr_ui::text` chain the app uses (§11.5: `address_domain`
/// → `address_domain_budget_chars` → `mid_ellipsis`), so the fixture and
/// the production display can never drift.
pub fn build_address_fixture(case: &AddressCase, theme: &Theme) -> Option<AddressModel> {
    if !address_case_valid(case) {
        return None;
    }
    let domain = address_domain(case.url);
    let budget = address_domain_budget_chars(theme, true);
    let label = mid_ellipsis(&domain, budget);
    Some(AddressModel {
        domain: SharedString::from(label),
        hovered: case.hovered,
        profile_hint: if case.profile_hint {
            Some(profile_hint_fixture())
        } else {
            None
        },
    })
}

/// One staging matrix row: a family label and the production
/// `StagingModel` fields. The staging surface (`bbr-command-bar`) shows
/// only while a non-address staged mode is active — New Space, Move Tab,
/// and the §11.8 empty New Tab surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StagingCase {
    pub label: &'static str,
    /// §9: whether the browser chrome is focused (the field background and
    /// the accent border follow it).
    pub focused: bool,
    /// The staged command text (empty renders the "Type a URL…"
    /// placeholder — the §11.8 empty New Tab surface).
    pub text: &'static str,
    /// The §11.3 hint suffix, if any (production: Move Tab only).
    pub hint: Option<&'static str>,
    /// Whether the block cursor renders — `chrome_focused && mode.is_active()`
    /// resolved by the caller, mirrored here as focused→cursor.
    pub show_cursor: bool,
    /// The §11.3 cross-context profile-hint chip presence, carried through
    /// the production `StagingModel.profile_hint`/`AddressProfileHint`.
    pub profile_hint: bool,
}

impl StagingCase {
    /// The valid-family constructor.
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        focused: bool,
        text: &'static str,
        hint: Option<&'static str>,
        show_cursor: bool,
        profile_hint: bool,
    ) -> Self {
        StagingCase {
            label,
            focused,
            text,
            hint,
            show_cursor,
            profile_hint,
        }
    }
}

/// Every valid staging family — the single declaration the story renders.
/// The cursor true/false, hint present/absent, and profile-hint
/// present/absent dimensions are covered: New Space and Move Tab collect
/// with the cursor (Move Tab carries the production hint), the §11.8 empty
/// New Tab surface shows the placeholder with the cursor, unfocused
/// staging drops the cursor (`show_cursor = chrome_focused && is_active`),
/// the unfocused Move Tab keeps the hint without the cursor, and the
/// profile-hint chip rides both a focused cursor+hint family and an
/// unfocused no-hint family — independent of cursor and hint.
#[rustfmt::skip]
pub const STAGING_VALID: &[StagingCase] = &[
    StagingCase::new("New Space — cursor (no hint)", true, "Reading", None, true, false),
    StagingCase::new("Move Tab — cursor + hint", true, "Reading", Some("→ Reading"), true, false),
    StagingCase::new("empty New Tab — placeholder + cursor", true, "", None, true, false),
    StagingCase::new("unfocused staging — no cursor", false, "Reading", None, false, false),
    StagingCase::new("unfocused Move Tab — hint, no cursor", false, "Reading", Some("→ Reading"), false, false),
    StagingCase::new("Move Tab — cursor + hint + profile hint", true, "Reading", Some("→ Reading"), true, true),
    StagingCase::new("unfocused New Space — profile hint, no cursor", false, "Reading", None, false, true),
];

/// Build the production `StagingModel` for a staging family. The staging
/// surface has no rejected combination in the packet matrix (the rejected
/// column is the pill's), so every declared row builds; the profile-hint
/// presence maps to the production `StagingModel.profile_hint` through the
/// same §11.3 `AddressProfileHint` fixture the pill uses.
pub fn build_staging_fixture(case: &StagingCase) -> StagingModel {
    StagingModel {
        focused: case.focused,
        text: case.text.to_string(),
        hint: case.hint.map(SharedString::from),
        show_cursor: case.show_cursor,
        profile_hint: if case.profile_hint {
            Some(profile_hint_fixture())
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
    use std::collections::BTreeSet;

    /// The representative fixture theme (the canvas default).
    fn fixture_theme() -> Theme {
        Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal)
    }

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        let theme = fixture_theme();
        for case in ADDRESS_VALID {
            assert!(
                address_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let fixture = build_address_fixture(case, &theme)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(fixture.hovered, case.hovered, "{}", case.label);
            assert_eq!(
                fixture.profile_hint.is_some(),
                case.profile_hint,
                "{}",
                case.label
            );
            assert!(
                !fixture.domain.is_empty(),
                "the pill always renders a domain label: {}",
                case.label
            );
        }
        for case in STAGING_VALID {
            let fixture = build_staging_fixture(case);
            assert_eq!(fixture.focused, case.focused, "{}", case.label);
            assert_eq!(fixture.text, case.text, "{}", case.label);
            assert_eq!(fixture.hint.as_deref(), case.hint, "{}", case.label);
            assert_eq!(fixture.show_cursor, case.show_cursor, "{}", case.label);
            assert_eq!(
                fixture.profile_hint.is_some(),
                case.profile_hint,
                "{}",
                case.label
            );
            if let Some(carried) = fixture.profile_hint.as_ref() {
                assert_eq!(carried.profile_index, 1, "{}", case.label);
                assert_eq!(
                    carried.label.as_ref(),
                    "Switch to Private",
                    "{}",
                    case.label
                );
            }
        }
        for rejected in ADDRESS_REJECTED {
            assert!(
                !address_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_address_fixture(&rejected.case, &theme).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        // The stories render exactly these families in this order; the
        // E2E pins the same stable ids. Reordering or removing a family
        // fails here — the no-drift contract.
        let pill_labels: Vec<&str> = ADDRESS_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            pill_labels,
            vec![
                "domain — no www",
                "domain — with www (elided)",
                "domain — IDN (punycode → Unicode)",
                "domain — explicit port (dropped)",
                "hover fill step (hovered)",
                "profile-hint chip (present)",
            ]
        );
        let staging_labels: Vec<&str> = STAGING_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            staging_labels,
            vec![
                "New Space — cursor (no hint)",
                "Move Tab — cursor + hint",
                "empty New Tab — placeholder + cursor",
                "unfocused staging — no cursor",
                "unfocused Move Tab — hint, no cursor",
                "Move Tab — cursor + hint + profile hint",
                "unfocused New Space — profile hint, no cursor",
            ]
        );
        // Labels are unique (ids derive from position).
        let mut seen = BTreeSet::new();
        for case in ADDRESS_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
        let mut seen = BTreeSet::new();
        for case in STAGING_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn staging_matrix_covers_the_required_dimensions() {
        // The review finding: cursor=false + hint present (unfocused Move
        // Tab) is a valid staging family.
        assert!(
            STAGING_VALID
                .iter()
                .any(|c| !c.show_cursor && c.hint.is_some()),
            "cursor=false + hint present (unfocused Move Tab) is covered"
        );
        // The exact (cursor, hint, profile-hint) dimension coverage, pinned
        // so the matrix cannot regress: cursor true/false, hint
        // present/absent, profile-hint present/absent, and profile-hint
        // presence independent of cursor and hint (present with
        // cursor=true+hint and with cursor=false+no-hint).
        let triples: BTreeSet<(bool, bool, bool)> = STAGING_VALID
            .iter()
            .map(|c| (c.show_cursor, c.hint.is_some(), c.profile_hint))
            .collect();
        assert_eq!(
            triples,
            BTreeSet::from([
                (true, false, false),
                (true, true, false),
                (false, false, false),
                (false, true, false),
                (true, true, true),
                (false, false, true),
            ])
        );
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = ADDRESS_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "pill-as-text-field (editing state, cursor, selection)",
                "a focus ring on the pill",
                "Copy-Link accessory",
            ]
        );
        for rejected in ADDRESS_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }

    #[test]
    fn fixtures_use_the_production_domain_derivation() {
        // The fixture derives the pill label through the production
        // `bbr_ui::text` chain, so www-elision, IDN decoding, and port
        // dropping are the production's (the bbr-ui text tests already pin
        // the derivation; the gallery only proves the fixture rides it).
        let theme = fixture_theme();
        let by_label = |label: &str| {
            ADDRESS_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let no_www = build_address_fixture(by_label("domain — no www"), &theme).unwrap();
        assert_eq!(no_www.domain, "example.com");
        let with_www =
            build_address_fixture(by_label("domain — with www (elided)"), &theme).unwrap();
        assert_eq!(with_www.domain, "example.com", "leading www elided");
        let idn =
            build_address_fixture(by_label("domain — IDN (punycode → Unicode)"), &theme).unwrap();
        assert_eq!(idn.domain, "bücher.example", "IDN decoded to Unicode");
        let port =
            build_address_fixture(by_label("domain — explicit port (dropped)"), &theme).unwrap();
        assert_eq!(port.domain, "example.com", "explicit port dropped");
        let hovered = build_address_fixture(by_label("hover fill step (hovered)"), &theme).unwrap();
        assert!(hovered.hovered);
        let hint = build_address_fixture(by_label("profile-hint chip (present)"), &theme).unwrap();
        let carried = hint.profile_hint.expect("hint chip set");
        assert_eq!(carried.profile_index, 1);
        assert_eq!(carried.spec.letter, "Vi");
        assert_eq!(carried.label, "Switch to Private");
    }

    #[test]
    fn profile_hint_fixture_is_a_plain_stable_dto() {
        let hint = profile_hint_fixture();
        assert_eq!(hint, profile_hint_fixture(), "stable per call");
        assert_eq!(hint.profile_index, 1);
        assert!(!hint.spec.letter.is_empty());
        assert!(!hint.label.is_empty());
    }
}
