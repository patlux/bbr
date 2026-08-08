//! Credential-sheet presentation identity for the presentation probe
//! (queue item 10).
//!
//! The `auth_sheet_open` benchmark measures the production presentation of
//! the credential sheet (the path a challenge arrival drives) and commits
//! only from a *render* of the sheet that carried the exact armed identity:
//! the presented challenge id with the sheet open. The observation is
//! render-gated — the portal window's `PortalRoot` records the redacted
//! sheet state it actually drew, so a frame that omitted the sheet can
//! never commit a sample.
//!
//! Redaction: the observation carries **no credential text** — only the
//! challenge id, the focused field label, and masked entered flags. The
//! username/password never enter the probe, the measurement, or any report
//! shape.

use super::{Pending, PerformanceProbe, State};

/// The exact credential-sheet presentation a pending sample must commit
/// from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AuthExpectation {
    /// The sheet is open presenting exactly this challenge id.
    Open { challenge: u64 },
}

/// The credential-sheet presentation the portal render actually drew —
/// redacted by construction (no credential text can exist in these fields).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct AuthObservation {
    pub open: bool,
    pub challenge: Option<u64>,
    pub focused: Option<String>,
    pub username_entered: bool,
    pub password_entered: bool,
}

impl PerformanceProbe {
    /// Arm the exact credential-sheet presentation the pending sample must
    /// commit from: the sheet open with `challenge` presented. Arming clears
    /// any prior observation: a render that drew a different (or earlier)
    /// sheet can never satisfy a newer expectation.
    pub fn expect_auth_sheet(&self, challenge: u64) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.auth = Some(AuthExpectation::Open { challenge });
            state.latest_auth = None;
        }
    }

    /// Record the redacted sheet state the portal render actually drew (the
    /// render milestone; the portal `PortalRoot` calls this after consuming
    /// the app's live `AuthSheetSurface`). No observation is recorded for a
    /// frame that omitted the sheet, so a pending sample can never commit
    /// from a render that did not present it. Credentials never reach this
    /// probe — only masked flags and the challenge id.
    pub fn observe_auth_sheet(
        &self,
        open: bool,
        challenge: Option<u64>,
        focused: Option<&str>,
        username_entered: bool,
        password_entered: bool,
    ) {
        let mut state = self.inner.borrow_mut();
        state.latest_auth = Some(AuthObservation {
            open,
            challenge,
            focused: focused.map(str::to_string),
            username_entered,
            password_entered,
        });
    }
}

/// The credential-sheet part of the presentation predicate: true when no
/// auth expectation is armed, or when the observed rendered sheet carries
/// exactly the armed challenge identity. An armed expectation with no render
/// observation yet can never commit.
pub(super) fn auth_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.auth.as_ref() else {
        return true;
    };
    let observed = state.latest_auth.as_ref();
    match (expected, observed) {
        (AuthExpectation::Open { challenge }, Some(observed)) => {
            observed.open && observed.challenge == Some(*challenge)
        }
        (_, None) => false,
    }
}
