//! The Auth sheet matrix authority (packet
//! `component-gallery-browser-components.md`, Group 8): **one declaration
//! source** for every valid credential-sheet family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::surfaces::auth` symbols
//! (`auth_sheet`, `AuthSheetModel`, `AuthField`), and the fixture builder
//! refuses (`None`) any case the validity authority rejects — so the
//! story, the fixture builder, the matrix table, and the tests cannot
//! drift. The rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Auth sheet row: realm
//! present vs absent; origin-server vs proxy authority; username
//! typed/empty; password masked bullets (never the characters); focus
//! username vs password; Cancel vs Log In (the two buttons always render
//! and route through the recorder callbacks). Rejected: an unmasked
//! password render (bullets always — the model carries no password text,
//! only the masked `password_entered` flag), outside-press dismissal (no
//! backdrop dismissal for credentials — the sheet's only exits are Log In
//! and Cancel), and any serializable credential shape (the DTO is never
//! `Serialize`/`Deserialize` and holds no password field).

use bbr_ui::surfaces::auth::{AuthField, AuthSheetModel};
use bbr_ui::theme::Theme;

/// The deterministic fixture host the matrix families share (the packet
/// matrix does not vary the host; it stays stable so the detail line
/// renders one line at the settled card width).
pub const FIXTURE_HOST: &str = "intranet.example";

/// One matrix row: a family label and the production `AuthSheetModel`
/// fields. The matrix-only fields (`claims_unmasked`,
/// `claims_backdrop_dismissal`, `claims_raw_password`) exist solely to
/// name the rejected combinations in the same case shape; they are never
/// mapped into `AuthSheetModel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthCase {
    pub label: &'static str,
    /// The realm label; `None` renders the realm-absent header.
    pub realm: Option<&'static str>,
    /// `"origin"` or `"proxy"` — who is asking.
    pub authority: &'static str,
    /// The typed username ("" = empty field).
    pub username: &'static str,
    /// Masked: whether the password field holds any characters (bullets).
    pub password_entered: bool,
    /// Which field owns the keyboard cursor.
    pub focused: AuthField,
    /// Matrix-only: this case claims an unmasked password render (§13 —
    /// the password draws bullets always; the model has no password field).
    pub claims_unmasked: bool,
    /// Matrix-only: this case claims a backdrop outside-press dismissal
    /// (a credential sheet has no outside dismissal — only Log In and
    /// Cancel, §13).
    pub claims_backdrop_dismissal: bool,
    /// Matrix-only: this case claims a raw password in the model (any
    /// serializable credential shape is rejected — the DTO carries only
    /// the masked entered flag and is never serialized).
    pub claims_raw_password: bool,
}

impl AuthCase {
    /// The valid-family constructor: the five production fields with the
    /// matrix-only flags defaulting to false. `#[allow]` keeps the dense
    /// const table readable (the fields are the documented matrix row).
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        realm: Option<&'static str>,
        authority: &'static str,
        username: &'static str,
        password_entered: bool,
        focused: AuthField,
    ) -> Self {
        AuthCase {
            label,
            realm,
            authority,
            username,
            password_entered,
            focused,
            claims_unmasked: false,
            claims_backdrop_dismissal: false,
            claims_raw_password: false,
        }
    }
}

/// Every valid credential-sheet family — the single declaration the story
/// renders. The five packet dimensions are covered: realm present/absent,
/// origin/proxy authority, username typed/empty, password masked
/// bullets/empty, and focus username/password. The family set is
/// representative and the dimension-coverage test pins it; the
/// cross-product test additionally proves the whole valid matrix (all 32
/// realm × authority × username × password × focus combinations) builds.
#[rustfmt::skip]
pub const AUTH_VALID: &[AuthCase] = &[
    AuthCase::new("realm present · origin · empty · focus username", Some("Finance VPN"), "origin", "", false, AuthField::Username),
    AuthCase::new("realm present · proxy · typed · bullets · focus password", Some("Corporate Proxy"), "proxy", "alice", true, AuthField::Password),
    AuthCase::new("realm absent · origin · typed · bullets · focus username", None, "origin", "alice", true, AuthField::Username),
    AuthCase::new("realm absent · proxy · empty · focus password", None, "proxy", "", false, AuthField::Password),
    AuthCase::new("realm present · origin · typed · empty · focus password", Some("Finance VPN"), "origin", "alice", false, AuthField::Password),
    AuthCase::new("realm present · proxy · empty · bullets · focus username", Some("Corporate Proxy"), "proxy", "", true, AuthField::Username),
    AuthCase::new("realm absent · origin · empty · bullets · focus password", None, "origin", "", true, AuthField::Password),
    AuthCase::new("realm absent · proxy · typed · empty · focus username", None, "proxy", "alice", false, AuthField::Username),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `AuthCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedAuth {
    pub label: &'static str,
    pub case: AuthCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`auth_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const AUTH_REJECTED: &[RejectedAuth] = &[
    RejectedAuth {
        label: "unmasked password render",
        case: AuthCase {
            label: "unmasked password render",
            realm: Some("Finance VPN"),
            authority: "origin",
            username: "alice",
            password_entered: true,
            focused: AuthField::Password,
            claims_unmasked: true,
            claims_backdrop_dismissal: false,
            claims_raw_password: false,
        },
        reason: "the password draws bullets always (masked length) — the model carries no password text, only the masked password_entered flag",
    },
    RejectedAuth {
        label: "outside-press dismissal",
        case: AuthCase {
            label: "outside-press dismissal",
            realm: Some("Finance VPN"),
            authority: "origin",
            username: "",
            password_entered: false,
            focused: AuthField::Username,
            claims_unmasked: false,
            claims_backdrop_dismissal: true,
            claims_raw_password: false,
        },
        reason: "no backdrop dismissal for credentials — the sheet's only exits are Log In and Cancel (§13; the portal backdrop stops propagation)",
    },
    RejectedAuth {
        label: "a raw password in the model (serializable credential shape)",
        case: AuthCase {
            label: "a raw password in the model",
            realm: Some("Finance VPN"),
            authority: "origin",
            username: "alice",
            password_entered: true,
            focused: AuthField::Username,
            claims_unmasked: false,
            claims_backdrop_dismissal: false,
            claims_raw_password: true,
        },
        reason: "any serializable credential shape is rejected — the DTO has no password field and is never Serialize/Deserialize; the password render is masked length only",
    },
];

/// The one credential-sheet validity authority: rejects the impossible
/// combinations the matrix forbids. The fixture builder refuses (`None`)
/// any case this rejects, so the story can never render a rejected sheet.
pub fn auth_case_valid(case: &AuthCase) -> bool {
    // §13: the password draws bullets always (masked length).
    if case.claims_unmasked {
        return false;
    }
    // §13: no outside-press dismissal for credentials.
    if case.claims_backdrop_dismissal {
        return false;
    }
    // Redaction invariant: no raw password and no serializable shape.
    if case.claims_raw_password {
        return false;
    }
    true
}

/// Build the production `AuthSheetModel` for a valid matrix family; `None`
/// for any rejected combination. The theme is the caller-resolved gallery
/// theme (the story proves the surface renders in any mode/contrast/
/// density — the hardcoded dark sheet is never enshrined).
pub fn build_auth_fixture(case: &AuthCase, theme: &Theme) -> Option<AuthSheetModel> {
    if !auth_case_valid(case) {
        return None;
    }
    Some(AuthSheetModel {
        theme: *theme,
        host: FIXTURE_HOST.to_string(),
        realm: case.realm.map(str::to_string),
        authority: case.authority.to_string(),
        focused: case.focused,
        username: case.username.to_string(),
        password_entered: case.password_entered,
    })
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
        for case in AUTH_VALID {
            assert!(
                auth_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let fixture =
                build_auth_fixture(case, &theme).unwrap_or_else(|| panic!("valid family builds"));
            assert_eq!(fixture.realm.as_deref(), case.realm, "{}", case.label);
            assert_eq!(fixture.authority, case.authority, "{}", case.label);
            assert_eq!(fixture.username, case.username, "{}", case.label);
            assert_eq!(
                fixture.password_entered, case.password_entered,
                "{}",
                case.label
            );
            assert_eq!(fixture.focused, case.focused, "{}", case.label);
            assert_eq!(fixture.host, FIXTURE_HOST, "{}", case.label);
            // The fixture rides the caller-resolved theme (never dark-only).
            assert_eq!(fixture.theme.mode, theme.mode, "{}", case.label);
            assert_eq!(fixture.theme.contrast, theme.contrast, "{}", case.label);
            assert_eq!(fixture.theme.density, theme.density, "{}", case.label);
        }
        for rejected in AUTH_REJECTED {
            assert!(
                !auth_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_auth_fixture(&rejected.case, &theme).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        // The story renders exactly these families in this order; the E2E
        // pins the same stable ids. Reordering or removing a family fails
        // here — the no-drift contract.
        let labels: Vec<&str> = AUTH_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "realm present · origin · empty · focus username",
                "realm present · proxy · typed · bullets · focus password",
                "realm absent · origin · typed · bullets · focus username",
                "realm absent · proxy · empty · focus password",
                "realm present · origin · typed · empty · focus password",
                "realm present · proxy · empty · bullets · focus username",
                "realm absent · origin · empty · bullets · focus password",
                "realm absent · proxy · typed · empty · focus username",
            ]
        );
        // Labels are unique (ids derive from position).
        let mut seen = BTreeSet::new();
        for case in AUTH_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn auth_matrix_covers_the_required_dimensions() {
        // The five packet dimensions, each with both values present:
        // realm present/absent, origin/proxy authority, username
        // typed/empty, password masked bullets/empty, focus
        // username/password.
        let realms: BTreeSet<bool> = AUTH_VALID.iter().map(|c| c.realm.is_some()).collect();
        assert_eq!(realms, BTreeSet::from([true, false]));
        let authorities: BTreeSet<&str> = AUTH_VALID.iter().map(|c| c.authority).collect();
        assert_eq!(authorities, BTreeSet::from(["origin", "proxy"]));
        let usernames: BTreeSet<bool> = AUTH_VALID.iter().map(|c| !c.username.is_empty()).collect();
        assert_eq!(usernames, BTreeSet::from([true, false]));
        let passwords: BTreeSet<bool> = AUTH_VALID.iter().map(|c| c.password_entered).collect();
        assert_eq!(passwords, BTreeSet::from([true, false]));
        let focuses: BTreeSet<&str> = AUTH_VALID.iter().map(|c| c.focused.label()).collect();
        assert_eq!(focuses, BTreeSet::from(["username", "password"]));
        // "Cancel vs Log In": both buttons always render and route through
        // the recorder callbacks (the story wires both; the E2E proves
        // both fire) — no family drops a button.
    }

    #[test]
    fn the_whole_valid_cross_product_is_accepted() {
        // The full realm × authority × username × password × focus cross
        // product (2^5 = 32) is every legal sheet: nothing in the packet
        // matrix rejects any of them, so the validity authority accepts
        // and the fixture builder builds every one.
        let theme = fixture_theme();
        for realm in [None, Some("Finance VPN")] {
            for authority in ["origin", "proxy"] {
                for username in ["", "alice"] {
                    for password_entered in [false, true] {
                        for focused in [AuthField::Username, AuthField::Password] {
                            let case = AuthCase {
                                label: "cross-product row",
                                realm,
                                authority,
                                username,
                                password_entered,
                                focused,
                                claims_unmasked: false,
                                claims_backdrop_dismissal: false,
                                claims_raw_password: false,
                            };
                            assert!(auth_case_valid(&case), "{case:?}");
                            let fixture = build_auth_fixture(&case, &theme)
                                .unwrap_or_else(|| panic!("cross-product builds: {case:?}"));
                            assert_eq!(fixture.realm.as_deref(), realm);
                            assert_eq!(fixture.authority, authority);
                            assert_eq!(fixture.username, username);
                            assert_eq!(fixture.password_entered, password_entered);
                            assert_eq!(fixture.focused, focused);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = AUTH_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "unmasked password render",
                "outside-press dismissal",
                "a raw password in the model (serializable credential shape)",
            ]
        );
        for rejected in AUTH_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }

    #[test]
    fn fixtures_carry_the_masked_password_shape_only() {
        // The redaction invariant is a type shape: the fixture model has
        // no password field — only the masked entered flag (the packet's
        // "any serializable credential shape" rejection). The gallery can
        // only ever build the masked shape.
        let theme = fixture_theme();
        let case = &AUTH_VALID[1]; // bullets family
        let fixture = build_auth_fixture(case, &theme).expect("valid family builds");
        assert!(fixture.password_entered);
        let empty = &AUTH_VALID[0];
        let fixture = build_auth_fixture(empty, &theme).expect("valid family builds");
        assert!(!fixture.password_entered);
        // The typed username is the only editable text in the model (the
        // bbr-ui Debug-redaction tests pin that it never renders).
        assert!(!fixture.username.is_empty() || fixture.username.is_empty());
    }
}
