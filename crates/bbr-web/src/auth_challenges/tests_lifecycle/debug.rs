//! Debug-shape tests for the response and lifecycle types: exact redacted
//! handle shapes and absence of secret-shaped material.

use super::support::*;
use crate::auth_challenges::{AuthEvent, AuthResponse, AuthState};

/// Response/state/event Debug output carries only redacted handle shapes and
/// never secret-shaped material.
#[test]
fn lifecycle_debug_shapes_are_exact_and_leak_nothing() {
    assert_eq!(
        format!("{:?}", AuthResponse::UseCredential(handle(7))),
        "UseCredential(CredentialHandle(#7))"
    );
    assert_eq!(
        format!("{:?}", AuthResponse::UseClientIdentity(identity(9))),
        "UseClientIdentity(ClientIdentityHandle(#9))"
    );

    let rendered = [
        format!(
            "{:?}",
            AuthState::Resolved {
                response: AuthResponse::UseCredential(handle(7)),
            }
        ),
        format!(
            "{:?}",
            AuthEvent::Respond {
                id: id(1),
                response: AuthResponse::UseClientIdentity(identity(9)),
            }
        ),
        format!(
            "{:?}",
            AuthEvent::Supersede {
                id: id(1),
                by: id(2)
            }
        ),
    ];
    for shape in rendered {
        for needle in [
            "BEGIN",
            "password",
            "secret",
            "private",
            "certificate",
            "-----",
        ] {
            assert!(!shape.contains(needle), "{shape} leaks {needle:?}");
        }
    }
}
