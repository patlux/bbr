//! Pure incident lifecycle: a closed state machine plus per-web-view
//! identity guards.
//!
//! This module fixes **what may happen to an incident after it is reported**.
//! The state set ([`LifecycleState`]) and event set ([`LifecycleEvent`]) are
//! closed, and the transition function [`apply_event`] is pure and
//! deterministic: the same state, event, and category always produce the
//! same next state or the same rejection. [`Lifecycle`] binds a state to one
//! concrete incident and rejects events that name a different, older, or
//! already-known incident.
//!
//! ## State machine
//!
//! ```text
//! Reported ──RequestRecovery{a}──▶ RecoveryRequested{a} ──Succeeded{a}──▶ Recovered
//!                                   RecoveryRequested{a} ──Failed{a}───▶ RecoveryFailed
//! Reported ──Dismiss──▶ Dismissed        RecoveryRequested{a} ──Dismiss──▶ Dismissed
//! Reported ──Supersede──▶ Superseded     RecoveryFailed ──Dismiss──▶ Dismissed
//! RecoveryRequested{a} ──Supersede──▶ Superseded
//! RecoveryFailed ──Supersede──▶ Superseded
//! ```
//!
//! `Recovered` and `Dismissed` are terminal: no event changes them. A newer
//! incident on the same web view does not supersede a resolved one — the
//! tracker simply starts tracking the new incident ([`Lifecycle::supersede_with`]).
//! A failed recovery does not allow another request on the same incident: a
//! fresh attempt is a new incident with a higher sequence and supersedes this
//! one.
//!
//! ## Recovery requests are validated
//!
//! A recovery request is accepted only when the action is in the
//! incident-derived legal set of its category (the same matrix the surface
//! menu uses) **minus `Dismiss`**: dismissal is a lifecycle event of its
//! own, not a recovery request. A process crash therefore accepts only
//! `Restore` — never `Retry`/`Reload`, because a crash is not an ordinary
//! navigation retry — and TLS never gains a bypass action, because the closed
//! action set has no bypass variant to request.
//!
//! ## Identity guards
//!
//! [`Lifecycle`] tracks the incidents of exactly one web view (one
//! `(ProfileId, WebViewId)`) and rejects:
//!
//! - **duplicate** reports: the same [`IncidentId`] reported twice;
//! - **stale** reports/callbacks: an id older than the tracked incident;
//! - **mismatched** callbacks: a completion that names an incident that is
//!   not the tracked one, or an action that was not the requested one.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

use super::identity::{Incident, IncidentId};
use super::recovery::{is_legal, legal_actions_for_category, RecoveryAction};
use super::taxonomy::IncidentCategory;

/// Pure lifecycle state of one web-error incident.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    /// The incident was reported and is awaiting resolution.
    Reported,
    /// The incident was replaced by a newer incident on the same web view;
    /// it can no longer transition.
    Superseded,
    /// A legal recovery action was requested and is in flight. The requested
    /// action is kept so a completion can be checked against it.
    RecoveryRequested { action: RecoveryAction },
    /// The requested recovery action completed successfully. Terminal.
    Recovered,
    /// The requested recovery action failed. The incident stays visible and
    /// may be dismissed or superseded by a newer incident.
    RecoveryFailed,
    /// The incident was dismissed. Terminal.
    Dismissed,
}

impl LifecycleState {
    /// The in-flight recovery action, when the state is [`LifecycleState::RecoveryRequested`].
    pub fn action(&self) -> Option<RecoveryAction> {
        match self {
            Self::RecoveryRequested { action } => Some(*action),
            _ => None,
        }
    }
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Reported => "reported".to_string(),
            Self::Superseded => "superseded".to_string(),
            Self::RecoveryRequested { action } => format!("recovery requested ({action})"),
            Self::Recovered => "recovered".to_string(),
            Self::RecoveryFailed => "recovery failed".to_string(),
            Self::Dismissed => "dismissed".to_string(),
        };
        f.write_str(&text)
    }
}

/// Closed set of lifecycle events for a web-error incident.
///
/// `RecoverySucceeded`/`RecoveryFailed` carry the action that completed so
/// the lifecycle can reject a completion that does not match the in-flight
/// request. Deliberately non-serde, like the validation rejections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleEvent {
    /// A newer incident on the same web view replaced this one.
    Supersede,
    /// Request the given recovery action for the incident.
    RequestRecovery { action: RecoveryAction },
    /// The requested recovery action completed successfully.
    RecoverySucceeded { action: RecoveryAction },
    /// The requested recovery action failed.
    RecoveryFailed { action: RecoveryAction },
    /// The incident surface was dismissed.
    Dismiss,
}

impl fmt::Display for LifecycleEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Supersede => "supersede".to_string(),
            Self::RequestRecovery { action } => format!("request recovery ({action})"),
            Self::RecoverySucceeded { action } => format!("recovery succeeded ({action})"),
            Self::RecoveryFailed { action } => format!("recovery failed ({action})"),
            Self::Dismiss => "dismiss".to_string(),
        };
        f.write_str(&text)
    }
}

/// Why a lifecycle transition or identity guard rejected an event.
///
/// Deliberately non-serde, like [`super::recovery::ActionRejection`] and
/// [`super::identity::DefinitionError`]: this type reports validation
/// failures to the controller and is not a wire shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleError {
    /// The same incident was reported a second time.
    DuplicateIncident { id: IncidentId },
    /// The reported/callback incident is older than the tracked incident.
    StaleIncident {
        current: IncidentId,
        received: IncidentId,
    },
    /// A callback or report names an incident that is not the tracked one
    /// (a stale superseded id, a wrong web view, or a wrong profile).
    MismatchedCallback {
        tracked: IncidentId,
        received: IncidentId,
    },
    /// The requested recovery action is not in the incident's requestable
    /// set (category-legal actions minus `Dismiss`).
    IllegalAction {
        action: RecoveryAction,
        category: IncidentCategory,
    },
    /// The completed action does not match the in-flight requested action.
    MismatchedAction {
        requested: RecoveryAction,
        completed: RecoveryAction,
    },
    /// The current state does not permit the event.
    InvalidState {
        state: LifecycleState,
        event: LifecycleEvent,
    },
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateIncident { id } => {
                write!(f, "incident {id} was already reported")
            }
            Self::StaleIncident { current, received } => {
                write!(
                    f,
                    "incident {received} is stale; current incident is {current}"
                )
            }
            Self::MismatchedCallback { tracked, received } => {
                write!(
                    f,
                    "callback for incident {received} does not match tracked incident {tracked}"
                )
            }
            Self::IllegalAction { action, category } => {
                write!(
                    f,
                    "action {action} is not a legal recovery request for category {category}"
                )
            }
            Self::MismatchedAction {
                requested,
                completed,
            } => {
                write!(
                    f,
                    "completed action {completed} does not match requested action {requested}"
                )
            }
            Self::InvalidState { state, event } => {
                write!(f, "cannot apply {event} from state {state}")
            }
        }
    }
}

/// Pure, deterministic lifecycle transition.
///
/// `category` is the incident's category (derived from its subreason, the
/// taxonomy's single source of truth) and is used to validate a
/// [`LifecycleEvent::RequestRecovery`] against the closed requestable set.
/// State eligibility is checked before action legality: an illegal action on
/// an ineligible state reports the state violation.
pub fn apply_event(
    state: LifecycleState,
    event: LifecycleEvent,
    category: IncidentCategory,
) -> Result<LifecycleState, LifecycleError> {
    use LifecycleEvent::{
        Dismiss, RecoveryFailed as EventFailed, RecoverySucceeded as EventSucceeded,
        RequestRecovery, Supersede,
    };
    use LifecycleState::{
        Dismissed, Recovered, RecoveryFailed, RecoveryRequested, Reported, Superseded,
    };

    let next = match (state, event) {
        (Reported | RecoveryRequested { .. } | RecoveryFailed, Supersede) => Superseded,
        (Reported, RequestRecovery { action }) => {
            if action == RecoveryAction::Dismiss || !is_legal(category, action) {
                return Err(LifecycleError::IllegalAction { action, category });
            }
            RecoveryRequested { action }
        }
        (RecoveryRequested { action: requested }, EventSucceeded { action })
            if action == requested =>
        {
            Recovered
        }
        (RecoveryRequested { action: requested }, EventFailed { action })
            if action == requested =>
        {
            RecoveryFailed
        }
        (RecoveryRequested { action: requested }, EventSucceeded { action })
        | (RecoveryRequested { action: requested }, EventFailed { action }) => {
            return Err(LifecycleError::MismatchedAction {
                requested,
                completed: action,
            });
        }
        (Reported | RecoveryRequested { .. } | RecoveryFailed, Dismiss) => Dismissed,
        (state, event) => {
            return Err(LifecycleError::InvalidState { state, event });
        }
    };
    Ok(next)
}

/// The recovery actions that may be *requested* for a category: its
/// category-legal actions minus `Dismiss`, which is a lifecycle event of its
/// own rather than a recovery.
fn requestable_actions(category: IncidentCategory) -> Vec<RecoveryAction> {
    legal_actions_for_category(category)
        .into_iter()
        .filter(|action| *action != RecoveryAction::Dismiss)
        .collect()
}

/// One tracked web-error incident bound to its lifecycle state.
///
/// A `Lifecycle` belongs to exactly one web view: the controller keeps one
/// per `(ProfileId, WebViewId)`. [`Lifecycle::report`] starts the first
/// incident; [`Lifecycle::supersede_with`] advances to the next incident on
/// the same web view, rejecting duplicate and stale ids and leaving the
/// previous incident superseded (or untouched when it was already terminal).
#[derive(Debug)]
pub struct Lifecycle {
    incident: Incident,
    state: LifecycleState,
}

impl Lifecycle {
    /// Start tracking a freshly reported incident in the `Reported` state.
    pub fn report(incident: Incident) -> Self {
        Self {
            incident,
            state: LifecycleState::Reported,
        }
    }

    /// Advance to a newer incident on the same web view.
    ///
    /// Rejects a duplicate report (same id), a stale report (sequence not
    /// greater than the tracked one), and a wrong-scope report (different
    /// profile or web view). An active tracked incident ends `Superseded`
    /// (see [`apply_event`]); a resolved one is simply replaced.
    pub fn supersede_with(&self, newer: Incident) -> Result<Self, LifecycleError> {
        let current = self.id();
        if newer.id == current {
            return Err(LifecycleError::DuplicateIncident { id: current });
        }
        if newer.id.profile != current.profile || newer.id.web_view != current.web_view {
            return Err(LifecycleError::MismatchedCallback {
                tracked: current,
                received: newer.id,
            });
        }
        if newer.id.sequence <= current.sequence {
            return Err(LifecycleError::StaleIncident {
                current,
                received: newer.id,
            });
        }
        Ok(Self::report(newer))
    }

    /// Request a recovery action for the tracked incident. The action must be
    /// in the incident's requestable set (category-legal actions minus
    /// `Dismiss`); the request may only start from `Reported`.
    pub fn request_recovery(&mut self, action: RecoveryAction) -> Result<(), LifecycleError> {
        let next = apply_event(
            self.state,
            LifecycleEvent::RequestRecovery { action },
            self.category(),
        )?;
        self.state = next;
        Ok(())
    }

    /// Report that a requested recovery completed successfully. The callback
    /// must name the tracked incident and the requested action.
    pub fn recovered(
        &mut self,
        id: IncidentId,
        action: RecoveryAction,
    ) -> Result<(), LifecycleError> {
        self.expect_tracked(id)?;
        let next = apply_event(
            self.state,
            LifecycleEvent::RecoverySucceeded { action },
            self.category(),
        )?;
        self.state = next;
        Ok(())
    }

    /// Report that a requested recovery failed. The callback must name the
    /// tracked incident and the requested action.
    pub fn recovery_failed(
        &mut self,
        id: IncidentId,
        action: RecoveryAction,
    ) -> Result<(), LifecycleError> {
        self.expect_tracked(id)?;
        let next = apply_event(
            self.state,
            LifecycleEvent::RecoveryFailed { action },
            self.category(),
        )?;
        self.state = next;
        Ok(())
    }

    /// Dismiss the incident surface. Legal from `Reported`,
    /// `RecoveryRequested`, and `RecoveryFailed`.
    pub fn dismiss(&mut self) -> Result<(), LifecycleError> {
        let next = apply_event(self.state, LifecycleEvent::Dismiss, self.category())?;
        self.state = next;
        Ok(())
    }

    /// The tracked incident.
    pub fn incident(&self) -> &Incident {
        &self.incident
    }

    /// The tracked incident's id.
    pub fn id(&self) -> IncidentId {
        self.incident.id
    }

    /// The current lifecycle state.
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// The tracked incident's category, derived from the subreason (the
    /// taxonomy's single source of truth).
    fn category(&self) -> IncidentCategory {
        self.incident.reason.category()
    }

    /// Identity guard for completion callbacks: the callback must name the
    /// tracked incident, otherwise it is stale or mismatched.
    fn expect_tracked(&self, id: IncidentId) -> Result<(), LifecycleError> {
        if id == self.id() {
            Ok(())
        } else {
            Err(LifecycleError::MismatchedCallback {
                tracked: self.id(),
                received: id,
            })
        }
    }
}

impl fmt::Display for Lifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.incident, self.state)
    }
}

/// The requestable actions of a category, kept public for contract tests.
///
/// The lifecycle validates [`LifecycleEvent::RequestRecovery`] against this
/// set: the category-legal actions minus `Dismiss`.
pub fn lifecycle_requestable_actions(category: IncidentCategory) -> Vec<RecoveryAction> {
    requestable_actions(category)
}
