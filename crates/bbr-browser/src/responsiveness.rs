//! Portable renderer-availability monitor (renderer-isolation lane).
//!
//! A hung or dead renderer must never block the BBR chrome. This module
//! owns the **portable** availability model for live sessions: the typed
//! [`RendererAvailability`] state, the bounded liveness-probe lifecycle
//! (arm → in-flight → complete/timeout → backoff), the fail-closed deadline
//! policy, and the closed recovery-action set (Wait / Reload / Close). The
//! platform adapter owns the real probe mechanics
//! (`bbr-web-macos::responsiveness`); this module owns the policy.
//!
//! ## Semantics
//!
//! - Probes run **only for visible sessions** on a bounded cadence with
//!   backoff; hidden/warm tabs are never probed (their resource policy stays
//!   `bbr-domain` lifecycle-owned). A session is armed when it becomes
//!   visible and re-armed after a completion (`probe_interval`) or a timeout
//!   (`timeout_backoff`). An idle window (no due probe) arms nothing.
//! - A probe that cannot complete inside its bounded deadline marks the view
//!   [`Unresponsive`](RendererAvailability::Unresponsive) — fail-closed,
//!   never a blocking wait, never an unbounded poll. The deadline is also
//!   enforced portably in [`ResponsivenessMonitor::pump`] as a
//!   belt-and-braces layer over the adapter's own deadline timer.
//! - `Crashed` stays owned by the existing `on_page_crash` authority
//!   (DESIGN §11.8 crash panel): a termination marks the session crashed
//!   and destroys it. `Unresponsive` is a distinct sibling state that keeps
//!   the session and tab alive under the §11.8a recovery surface.
//! - Stale-event discipline: completions/timeouts for unknown, destroyed, or
//!   re-registered views are dropped; the per-view probe **generation** makes
//!   a late answer from a previous probe unable to mutate a newer probe or a
//!   recreated session (mirrors the `ControllerEvent::UnknownWebView` path).
//! - After any `PageCrashed` or unresponsive timeout, a bounded
//!   reconciliation sweep re-arms the other live sessions so each sibling is
//!   marked independently (WebKit shares WebContent processes; the policy
//!   never assumes a one-process-per-tab guarantee).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use bbr_web::WebViewId;

/// The availability of one session's renderer.
///
/// `Unknown` is the fresh-session default (no probe has completed yet);
/// `Responsive` is a completing probe; `Unresponsive` is a bounded probe
/// that could not complete inside its deadline (fail-closed); `Crashed` is
/// owned by the existing `on_page_crash` crash authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RendererAvailability {
    #[default]
    Unknown,
    Responsive,
    Unresponsive,
    Crashed,
}

/// The bounded, configurable monitor constants (the detection-latency
/// budget is recorded as a design constant; measured acceptance is deferred
/// to the canonical runner).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponsivenessConfig {
    /// How long one probe may stay in flight before it is declared
    /// unresponsive (fail-closed deadline). The adapter runs the same
    /// deadline as a timer; the monitor enforces it portably too.
    pub probe_deadline: Duration,
    /// Re-arm cadence after a completing probe (bounded; a responsive
    /// visible session is re-probed at this interval, never continuously).
    pub probe_interval: Duration,
    /// Re-arm delay after a timeout (backoff: a hung renderer is not
    /// hammered by probe traffic).
    pub timeout_backoff: Duration,
}

impl Default for ResponsivenessConfig {
    fn default() -> Self {
        Self {
            probe_deadline: Duration::from_millis(2000),
            probe_interval: Duration::from_millis(1000),
            timeout_backoff: Duration::from_millis(3000),
        }
    }
}

/// A portable monotonic clock (injectable so the deadline/backoff policy is
/// unit-testable without sleeping).
pub type Clock = Box<dyn Fn() -> Instant>;

/// One live session's probe record.
#[derive(Debug, Clone, PartialEq)]
struct ProbeRecord {
    /// The per-view probe generation: incremented on every arm, so a late
    /// completion from a previous probe (or from a re-registered session)
    /// can never mutate the current probe.
    generation: u64,
    /// A probe is in flight and its deadline is armed.
    in_flight: bool,
    /// The fail-closed deadline of the in-flight probe.
    deadline: Option<Instant>,
    /// Earliest time the next probe may be armed (cadence/backoff gate).
    next_probe_at: Option<Instant>,
    /// The last committed availability state.
    state: RendererAvailability,
}

impl ProbeRecord {
    fn fresh() -> Self {
        Self {
            generation: 0,
            in_flight: false,
            deadline: None,
            next_probe_at: None,
            state: RendererAvailability::Unknown,
        }
    }
}

/// The portable liveness monitor: per-session availability plus the bounded
/// probe lifecycle. Owned by the [`super::BrowserController`].
pub struct ResponsivenessMonitor {
    config: ResponsivenessConfig,
    records: HashMap<WebViewId, ProbeRecord>,
    clock: Clock,
}

impl std::fmt::Debug for ResponsivenessMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponsivenessMonitor")
            .field("config", &self.config)
            .field("records", &self.records)
            .finish_non_exhaustive()
    }
}

impl Default for ResponsivenessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponsivenessMonitor {
    /// A monitor with the default bounded constants and the system clock.
    pub fn new() -> Self {
        Self::with_config(ResponsivenessConfig::default(), Box::new(Instant::now))
    }

    /// A monitor with explicit constants and an injectable clock.
    pub fn with_config(config: ResponsivenessConfig, clock: Clock) -> Self {
        Self {
            config,
            records: HashMap::new(),
            clock,
        }
    }

    /// The availability state of one live session (`Unknown` for a
    /// session this monitor does not know — callers must treat that like
    /// the stale-id discipline: no mutation).
    pub fn state(&self, view: WebViewId) -> RendererAvailability {
        self.records
            .get(&view)
            .map_or(RendererAvailability::Unknown, |record| record.state)
    }

    /// The bounded constants in force (tests read the cadence/backoff).
    #[cfg(test)]
    pub(crate) fn config(&self) -> ResponsivenessConfig {
        self.config
    }

    /// Replace the bounded constants (unit tests exercise alternate
    /// deadlines/backoffs without sleeping).
    #[cfg(test)]
    pub(crate) fn set_config(&mut self, config: ResponsivenessConfig) {
        self.config = config;
    }

    /// Whether `view` is currently reported unresponsive.
    pub fn is_unresponsive(&self, view: WebViewId) -> bool {
        self.state(view) == RendererAvailability::Unresponsive
    }

    /// The views currently reported unresponsive (bounded; the recovery
    /// surface and the snapshot consume this).
    pub fn unresponsive_views(&self) -> Vec<WebViewId> {
        let mut views: Vec<WebViewId> = self
            .records
            .iter()
            .filter(|(_, record)| record.state == RendererAvailability::Unresponsive)
            .map(|(view, _)| *view)
            .collect();
        views.sort();
        views
    }

    /// The current probe generation of `view` (stale-completion tests).
    pub fn generation(&self, view: WebViewId) -> Option<u64> {
        self.records.get(&view).map(|record| record.generation)
    }

    /// Register a fresh live session: availability `Unknown`, probe due
    /// immediately (the session is probed as soon as it is visible).
    pub fn register(&mut self, view: WebViewId) {
        let mut record = ProbeRecord::fresh();
        record.next_probe_at = Some((self.clock)());
        self.records.insert(view, record);
    }

    /// Forget a destroyed/recreated session: its record (and any in-flight
    /// probe or pending deadline) is dropped, so a late completion for the
    /// id cannot mutate the newer session.
    pub fn forget(&mut self, view: WebViewId) {
        self.records.remove(&view);
    }

    /// Advance the probe lifecycle for one live session.
    ///
    /// - A hidden session is never probed (no arm, no timeout).
    /// - An in-flight probe whose deadline passed is declared
    ///   [`Unresponsive`](RendererAvailability::Unresponsive) (portable
    ///   fail-closed deadline; the adapter timer is the primary path, this
    ///   is belt-and-braces) and the siblings are re-armed.
    /// - An idle due probe is armed: `arm(view)` is called exactly once and
    ///   the record moves to in-flight with a fresh generation + deadline.
    ///
    /// Returns the state transition produced, if any (the caller emits it
    /// as an observation; the snapshot is the truth).
    pub fn pump(
        &mut self,
        view: WebViewId,
        visible: bool,
        arm: &mut dyn FnMut(WebViewId),
    ) -> Option<RendererAvailability> {
        let now = (self.clock)();
        let record = self.records.get_mut(&view)?;
        if !visible {
            return None;
        }
        if record.in_flight {
            if record.deadline.is_some_and(|deadline| now >= deadline) {
                record.in_flight = false;
                record.deadline = None;
                record.state = RendererAvailability::Unresponsive;
                record.next_probe_at = Some(now + self.config.timeout_backoff);
                let transition = RendererAvailability::Unresponsive;
                self.reconcile(now, view);
                return Some(transition);
            }
            return None;
        }
        if record.next_probe_at.is_some_and(|due| now >= due) {
            record.generation += 1;
            record.in_flight = true;
            record.deadline = Some(now + self.config.probe_deadline);
            record.next_probe_at = None;
            arm(view);
        }
        None
    }

    /// A probe completed for `view` (the adapter's
    /// `UnresponsiveChanged(view, false)`): the renderer answers again.
    /// `generation` must match the record's current generation; completions
    /// for unknown/forgotten views, mismatched generations, or crashed
    /// sessions are dropped. Returns the transition when the state changed.
    pub fn probe_completed(
        &mut self,
        view: WebViewId,
        generation: u64,
    ) -> Option<RendererAvailability> {
        let record = self.records.get_mut(&view)?;
        if record.generation != generation || record.state == RendererAvailability::Crashed {
            return None;
        }
        if record.in_flight {
            record.in_flight = false;
            record.deadline = None;
        }
        let now = (self.clock)();
        record.next_probe_at = Some(now + self.config.probe_interval);
        if record.state == RendererAvailability::Responsive {
            return None;
        }
        record.state = RendererAvailability::Responsive;
        Some(RendererAvailability::Responsive)
    }

    /// A probe timed out for `view` (the adapter's
    /// `UnresponsiveChanged(view, true)` — or the portable deadline in
    /// [`Self::pump`]): fail-closed [`Unresponsive`](RendererAvailability::Unresponsive)
    /// plus backoff and a sibling reconciliation sweep. Same stale-id and
    /// generation guards as [`Self::probe_completed`].
    pub fn probe_timed_out(
        &mut self,
        view: WebViewId,
        generation: u64,
    ) -> Option<RendererAvailability> {
        let record = self.records.get_mut(&view)?;
        if record.generation != generation
            || record.state == RendererAvailability::Crashed
            || !record.in_flight
        {
            return None;
        }
        let now = (self.clock)();
        record.in_flight = false;
        record.deadline = None;
        record.state = RendererAvailability::Unresponsive;
        record.next_probe_at = Some(now + self.config.timeout_backoff);
        self.reconcile(now, view);
        Some(RendererAvailability::Unresponsive)
    }

    /// The crash authority (`on_page_crash`) owns a termination: mark the
    /// view `Crashed`, drop any in-flight probe, and re-arm every other
    /// live session (bounded reconciliation sweep — WebKit may share the
    /// dead WebContent process, so each sibling is re-probed and marked
    /// independently).
    pub fn on_crash(&mut self, view: WebViewId) {
        if let Some(record) = self.records.get_mut(&view) {
            record.state = RendererAvailability::Crashed;
            record.in_flight = false;
            record.deadline = None;
            record.next_probe_at = None;
        }
        let now = (self.clock)();
        self.reconcile(now, view);
    }

    /// The Wait recovery action: keep monitoring the unresponsive session.
    /// The state stays `Unresponsive` until a probe answers; the next probe
    /// is due after one probe interval (no probe spam on a hung renderer).
    pub fn wait(&mut self, view: WebViewId) {
        let Some(record) = self.records.get_mut(&view) else {
            return;
        };
        let now = (self.clock)();
        record.in_flight = false;
        record.deadline = None;
        record.next_probe_at = Some(now + self.config.probe_interval);
    }

    /// Force one immediate probe for a live view, cancelling any pending
    /// deadline (the testkit `ProbeResponsiveness` seam: "refresh the
    /// adapter's bounded liveness probe for one live view"). Returns whether
    /// a probe was armed; unknown/destroyed views are no-ops.
    pub fn arm_now(&mut self, view: WebViewId, arm: &mut dyn FnMut(WebViewId)) -> bool {
        let Some(record) = self.records.get_mut(&view) else {
            return false;
        };
        record.generation += 1;
        record.in_flight = true;
        record.deadline = Some((self.clock)() + self.config.probe_deadline);
        record.next_probe_at = None;
        arm(view);
        true
    }

    /// Re-arm every other live session now (the bounded reconciliation
    /// sweep after a crash or an unresponsive timeout): each sibling's
    /// in-flight probe is cancelled and its next probe forced due, so the
    /// next pump re-probes and marks each independently — the policy never
    /// assumes a one-process-per-tab topology. `excluded` is the triggering
    /// view: its own record keeps its backoff/state (a timed-out view is not
    /// re-probed immediately, a crashed view is already terminal).
    fn reconcile(&mut self, now: Instant, excluded: WebViewId) {
        for (view, record) in self.records.iter_mut() {
            if *view == excluded || record.state == RendererAvailability::Crashed {
                continue;
            }
            record.in_flight = false;
            record.deadline = None;
            record.next_probe_at = Some(now);
        }
    }
}
