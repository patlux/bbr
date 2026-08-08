//! Find-in-page composition (task packet `find-in-page-portal-bar`).
//!
//! One `Cmd-F` find presentation per browser window: the portable
//! [`FindSession`] state machine, the native WebKit adapter bound to the
//! **active** web view, and the typed render/snapshot state. The find bar
//! renders inside the one `NativeAboveWeb` portal window (never a second
//! host, never a GPUI overlay behind WebKit, never a screenshot composite);
//! the portal window is non-activating and never key by design, so keyboard
//! input is handled by the main window's GPUI focus (`find_handle`) and the
//! portal renders the same live state.
//!
//! Contract invariants (enforced by construction):
//!
//! - Opening, stepping, and closing never create, destroy, move, or
//!   reselect a web session: the adapter is bound once at open to the
//!   active view's `WKWebView` (via `WebEngineWk::web_view`, which only
//!   clones the retained view) and dropped at close.
//! - Counts and ordinals come from WebKit through the portable contract
//!   (the adapter echoes portable request ids and drops stale completions;
//!   the portable session rejects stale ids independently).
//! - Exactly one active find presentation per window; a tab/space/session
//!   switch discards it ([`BbrApp::find_discard_if_unbound`]), while a
//!   navigation of the bound view invalidates it in place.
//! - Escape and close restore the focus domain captured at open
//!   (`origin_chrome_focused`) and clear highlights.
//!
//! The bar geometry is the §11.4 command-surface family (compact clamp
//! width 360 pt, `density:row` height, `radius:lg`, `material:overlay`,
//! `elev:2`), anchored to the content area's top-right edge with a `sp:2`
//! gap — all settled S1 tokens, no invented sizes.

use bbr_web::find_in_page::{FindDirection, FindError, FindOptions, FindSession};
use gpui::{Context, Window};

#[cfg(target_os = "macos")]
use bbr_web_macos::find_adapter::NativeFindAdapter;
#[cfg(target_os = "macos")]
use bbr_web_macos::find_mapping::FindCompletion;

use crate::portal::geometry::{Edges, FitMode, PortalAnchor, Pt, Sz};
use crate::portal::state::{DismissalPolicy, PortalPlacement, PortalRequest};
use crate::theme::Theme;

use super::BbrApp;

/// Stable §15 ids of the find bar (DESIGN.md §15; exposed through the
/// portal snapshot's typed `ids` field, registered in
/// [`crate::portal::probe::PortalProbeState::ids`]).
pub const FIND_BAR_ID: &str = "bbr-find-bar";
pub const FIND_FIELD_ID: &str = "bbr-find-field";
pub const FIND_TEXT_ID: &str = "bbr-find-text";
pub const FIND_COUNT_ID: &str = "bbr-find-count";
pub const FIND_PREVIOUS_ID: &str = "bbr-find-previous";
pub const FIND_NEXT_ID: &str = "bbr-find-next";
pub const FIND_CLOSE_ID: &str = "bbr-find-close";

/// The stable presentation id of the find bar, allocated in the portal
/// presentation-id space next to the probe (`0xB9`) and the command
/// surfaces (`0xC1`/`0xC2`).
pub const FIND_PRESENTATION_ID: crate::portal::state::PresentationId =
    crate::portal::state::PresentationId(0xC3);
/// The find bar's portal draw priority (same slot as the command surfaces).
pub const FIND_PORTAL_PRIORITY: usize = 2;
/// The find bar's width in points: the §11.4 command-surface family's
/// compact clamp minimum ("width = field width clamped to 360–win:palette",
/// DESIGN.md §11.4) — settled chrome geometry, not an invented size.
pub const FIND_BAR_WIDTH: f32 = 360.0;

/// Snapshot-visible find state (mirrored on the wire as
/// `bbr_testkit::protocol::FindSnapshot`). Plain data; the portal render
/// and the testkit snapshot both derive from this same state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FindState {
    /// A find presentation is open for this window.
    pub open: bool,
    /// The find field owns keyboard focus (the find focus handle).
    pub focused: bool,
    /// The live query text (what the field renders).
    pub query: String,
    /// Total matches reported by WebKit (`None` until a result lands).
    pub match_count: Option<usize>,
    /// Current 1-based ordinal (`None` for zero matches / no result).
    pub current_match: Option<usize>,
    /// A start/step request is awaiting a WebKit completion.
    pub pending: bool,
    /// A document change made the reported results stale.
    pub stale: bool,
}

impl FindState {
    /// Reset the report fields, keeping the presentation open (used by the
    /// empty-query path: the field is empty so no count is shown).
    fn clear_report(&mut self) {
        self.match_count = None;
        self.current_match = None;
        self.pending = false;
        self.stale = false;
    }
}

/// The app-side find presentation: the portable session, the native adapter
/// (macOS), the bound view, and the snapshot state.
pub struct FindController {
    pub(crate) state: FindState,
    session: FindSession,
    /// The `WebViewId` the adapter is bound to (the active view at open).
    /// `None` while closed.
    pub(crate) bound_view: Option<bbr_web::WebViewId>,
    /// The focus domain (chrome vs web) captured at open; Escape/close
    /// restores it deterministically.
    origin_chrome_focused: bool,
    #[cfg(target_os = "macos")]
    adapter: Option<NativeFindAdapter>,
    #[cfg(target_os = "macos")]
    completion_tx: smol::channel::Sender<FindCompletion>,
}

impl FindController {
    /// A fresh controller. `completion_tx` (macOS) receives adapter
    /// completions; the pump routes them back into the app.
    pub fn new(
        #[cfg(target_os = "macos")] completion_tx: smol::channel::Sender<FindCompletion>,
    ) -> Self {
        Self {
            state: FindState::default(),
            session: FindSession::default(),
            bound_view: None,
            origin_chrome_focused: false,
            #[cfg(target_os = "macos")]
            adapter: None,
            #[cfg(target_os = "macos")]
            completion_tx,
        }
    }
}

/// The immutable find-bar data one portal-window render consumes (mirrors
/// the command-surface model pattern: the portal reads the app's live state
/// per render, so the count/query can never drift from the app).
#[derive(Clone, Debug)]
pub struct FindSurface {
    pub theme: Theme,
    pub focused: bool,
    pub query: String,
    pub match_count: Option<usize>,
    pub current_match: Option<usize>,
}

impl BbrApp {
    /// Whether a find presentation is currently open (testkit seam).
    #[cfg(feature = "testkit")]
    pub fn find_state_open(&self) -> bool {
        self.find.state.open
    }

    /// The current find surface for the portal render and the snapshot.
    pub fn find_surface(&self) -> FindSurface {
        let state = &self.find.state;
        FindSurface {
            theme: self.theme(),
            focused: state.open && state.focused,
            query: state.query.clone(),
            match_count: state.match_count,
            current_match: state.current_match,
        }
    }

    /// `Cmd-F`: open (or re-focus) the find presentation over the active web
    /// view. Opening replaces any open command surface (exactly one active
    /// portal presentation per window) and captures the origin focus domain
    /// for deterministic restoration on close.
    pub fn find_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.find.state.open {
            self.find_focus_field(window, cx);
            return;
        }
        let Some(view) = self.controller.snapshot().active_view else {
            log::info!("find open: no active native view");
            return;
        };
        #[cfg(target_os = "macos")]
        {
            let Some(web_view) = self
                .find_engine
                .as_ref()
                .and_then(|engine| engine.borrow().web_view(view))
            else {
                log::warn!("find open: no native web view for {view:?}");
                return;
            };
            let mtm = objc2::MainThreadMarker::new().expect("find open on the main thread");
            let adapter = NativeFindAdapter::for_web_view(mtm, web_view);
            let tx = self.find.completion_tx.clone();
            adapter.set_completion_sink(Box::new(move |completion| {
                let _ = tx.try_send(completion);
            }));
            self.find.adapter = Some(adapter);
        }
        // A find presentation replaces any open command surface (the portal
        // state keeps exactly one active presentation).
        self.dismiss_command_portal(cx);
        self.find.state.open = true;
        self.find.bound_view = Some(view);
        self.find.origin_chrome_focused = self.chrome_focused(window);
        self.present_find_portal(window, cx);
        self.find_focus_field(window, cx);
        cx.notify();
    }

    /// Escape/close: tear down the presentation and restore the focus domain
    /// captured at open, clearing highlights on the bound view.
    pub fn find_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.find.state.open {
            return;
        }
        let origin_chrome_focused = self.find.origin_chrome_focused;
        self.find_discard(cx);
        if origin_chrome_focused {
            self.focus_command_bar(window, cx);
        } else {
            self.focus_web(window, cx);
        }
    }

    /// Close the find presentation WITHOUT touching focus (the invoking
    /// mutation — a tab/space switch or a command-surface entry — owns the
    /// focus choreography afterwards).
    pub(crate) fn find_discard(&mut self, cx: &mut Context<Self>) {
        if !self.find.state.open {
            return;
        }
        self.teardown_find_adapter();
        self.find.session.finish();
        self.find.bound_view = None;
        self.find.state = FindState::default();
        self.portal_dismiss(cx);
        cx.notify();
    }

    /// Discard the find presentation when its bound view is no longer the
    /// active view (the canonical interaction contract: tab/space/session
    /// switches end the one-per-window presentation). No-op when the bound
    /// view is still active (e.g. a same-tab reorder keeps the bar).
    pub(crate) fn find_discard_if_unbound(&mut self, cx: &mut Context<Self>) {
        if !self.find.state.open {
            return;
        }
        if self.find.bound_view != self.controller.snapshot().active_view {
            self.find_discard(cx);
        }
    }

    /// Set the find query and start a fresh search (typing, backspace, and
    /// the testkit `FindSetQuery` request all route here).
    pub fn find_set_query(&mut self, query: &str, cx: &mut Context<Self>) {
        if !self.find.state.open {
            return;
        }
        self.find.state.query = query.to_owned();
        self.run_find(query, cx);
        cx.notify();
    }

    /// Navigate to the next match (Enter / the next button).
    pub fn find_next(&mut self, cx: &mut Context<Self>) {
        self.find_step(FindDirection::Next, cx);
    }

    /// Navigate to the previous match (Shift-Enter / the previous button).
    pub fn find_previous(&mut self, cx: &mut Context<Self>) {
        self.find_step(FindDirection::Previous, cx);
    }

    /// Focus the find field (main-window GPUI focus; the portal window is
    /// non-activating and never key by design, so keystrokes are handled by
    /// the main window's root listener in find mode).
    fn find_focus_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.find.state.focused = true;
        window.focus(&self.find_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Handle find-mode keystrokes from the root key-down listener.
    pub(crate) fn on_find_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ks = &event.keystroke;
        match ks.key.as_str() {
            "enter" => {
                cx.stop_propagation();
                if ks.modifiers.shift {
                    self.find_previous(cx);
                } else {
                    self.find_next(cx);
                }
            }
            "backspace" => {
                cx.stop_propagation();
                let mut query = self.find.state.query.clone();
                // UTF-8-safe: `String::pop` removes the last char.
                query.pop();
                self.find_set_query(&query, cx);
            }
            "escape" => {
                // The global `escape → FocusWeb` binding normally routes
                // here via `cancel_command_bar`; this arm is the fallback
                // when Escape reaches the root directly.
                cx.stop_propagation();
                self.find_close(window, cx);
            }
            _ => {
                if let Some(ch) = ks.key_char.as_deref() {
                    if !ch.is_empty() {
                        cx.stop_propagation();
                        let mut query = self.find.state.query.clone();
                        query.push_str(ch);
                        self.find_set_query(&query, cx);
                    }
                }
            }
        }
        cx.notify();
    }

    /// A document change in the bound view: the reported results are stale,
    /// in-flight completions from the old document are dropped by the
    /// adapter's correlation, and the portable session rejects further
    /// navigation until a fresh query starts. The bar stays open.
    pub(crate) fn find_navigation_invalidated(&mut self, cx: &mut Context<Self>) {
        self.find.session.invalidate();
        self.find.state.stale = true;
        self.find.state.pending = false;
        #[cfg(target_os = "macos")]
        if let Some(adapter) = self.find.adapter.as_ref() {
            adapter.invalidate();
        }
        cx.notify();
    }

    fn run_find(&mut self, query: &str, cx: &mut Context<Self>) {
        let options = FindOptions::default();
        match self.find.session.start(query, options) {
            Ok(request_id) => {
                self.find.state.pending = true;
                self.find.state.stale = false;
                #[cfg(target_os = "macos")]
                if let Some(adapter) = self.find.adapter.as_ref() {
                    if let Err(error) = adapter.start(request_id, query, options) {
                        log::warn!("find start rejected: {error}");
                    }
                }
            }
            Err(FindError::EmptyQuery) => {
                // Empty query: clear highlights and any previous report.
                self.find.state.clear_report();
                #[cfg(target_os = "macos")]
                if let Some(adapter) = self.find.adapter.as_ref() {
                    adapter.finish();
                }
            }
            Err(error) => log::warn!("find start rejected: {error}"),
        }
        cx.notify();
    }

    fn find_step(&mut self, direction: FindDirection, cx: &mut Context<Self>) {
        if !self.find.state.open || self.find.state.query.trim().is_empty() {
            return;
        }
        let request_id = match direction {
            FindDirection::Next => self.find.session.find_next(),
            FindDirection::Previous => self.find.session.find_previous(),
        };
        match request_id {
            Ok(id) => {
                self.find.state.pending = true;
                #[cfg(target_os = "macos")]
                if let Some(adapter) = self.find.adapter.as_ref() {
                    let result = match direction {
                        FindDirection::Next => adapter.find_next(id),
                        FindDirection::Previous => adapter.find_previous(id),
                    };
                    if let Err(error) = result {
                        log::warn!("find step rejected: {error}");
                    }
                }
            }
            Err(
                FindError::NoMatches
                | FindError::ResultsPending
                | FindError::Inactive
                | FindError::StaleResults,
            ) => {
                // Deterministic no-op: the portable session rejects the
                // step; the UI keeps its current report.
            }
            Err(error) => log::warn!("find step rejected: {error}"),
        }
        cx.notify();
    }

    /// Present the find bar through the one native portal host. The portal
    /// render derives the live surface from the app entity each frame, so a
    /// completion or query change only needs an app notify.
    fn present_find_portal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let viewport = Sz::new(
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        );
        let theme = self.theme();
        let request = PortalRequest::native(
            FIND_PRESENTATION_ID,
            find_placement(viewport, &theme),
            FIND_PORTAL_PRIORITY,
            DismissalPolicy::any(),
        );
        self.portal_probe
            .borrow_mut()
            .present_find(request, viewport);
        self.activate_portal_request(request, cx);
    }

    /// Adapter completions (macOS): feed the portable session, publish the
    /// report, and reflect document changes as stale.
    #[cfg(target_os = "macos")]
    pub fn on_find_completion(&mut self, completion: FindCompletion, cx: &mut Context<Self>) {
        match completion {
            FindCompletion::Reported {
                request_id,
                result,
                mutated,
            } => match self.find.session.report(request_id, result) {
                Ok(()) => {
                    self.find.state.match_count = Some(result.match_count);
                    self.find.state.current_match = result.current_match;
                    self.find.state.pending = false;
                    if mutated {
                        self.find.session.invalidate();
                        self.find.state.stale = true;
                    }
                    // A pending find benchmark (`find_query_result`/
                    // `find_step`) commits only from a render of the portal
                    // bar that drew the armed WebKit report: schedule the
                    // render-inclusive commit now that the report landed.
                    #[cfg(feature = "testkit")]
                    if self.performance.find_async_benchmark_pending() {
                        self.schedule_find_render_commit(cx);
                    }
                }
                Err(FindError::StaleResults) | Err(FindError::Inactive) => {
                    // The portable session rejected the id (superseded or
                    // invalidated); the report is dropped, like the
                    // adapter's own stale correlation.
                }
                Err(error) => log::warn!("find report rejected: {error}"),
            },
            FindCompletion::Failed { .. } => {
                self.find.state.pending = false;
                self.find.session.invalidate();
                self.find.state.stale = true;
            }
            FindCompletion::StaleDropped => {}
        }
        cx.notify();
    }

    #[cfg(target_os = "macos")]
    fn teardown_find_adapter(&mut self) {
        if let Some(adapter) = self.find.adapter.take() {
            // Clear highlights on the bound view and drop in-flight
            // completions; late completions become no-ops (the router sink
            // is cleared with the adapter).
            adapter.finish();
        }
    }
}

/// The find bar's anchored placement: top-right of the content area with a
/// `sp:2` gap, snapped to the window keeping the margin. Height is the
/// settled `density:row` token; width is the §11.4 command-surface compact
/// clamp ([`FIND_BAR_WIDTH`]); both come from settled tokens, never local
/// literals.
fn find_placement(viewport: Sz, theme: &Theme) -> PortalPlacement {
    let gap = f32::from(crate::theme::Spacing::sp2());
    let row = f32::from(theme.density.row());
    PortalPlacement::Anchored(crate::portal::state::AnchoredPlacement {
        anchor_point: Pt::new(viewport.width - gap, gap),
        anchor: PortalAnchor::TopRight,
        size: Sz::new(FIND_BAR_WIDTH, row),
        offset: Pt::new(0.0, 0.0),
        fit: FitMode::SnapToWindowWithMargin(Edges::all(gap)),
    })
}
