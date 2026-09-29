//! # Per-site zoom composition (queue item 11, part A)
//!
//! Wires `Cmd-Plus` / `Cmd-Minus` / `Cmd-0` and per-site persistence to the
//! merged portable contract (`crates/bbr-web/src/site_settings`, task packet
//! `site-settings-zoom-contract`): the site key identity, the bounded
//! integer-percent ladder, and the [`SiteZoomLifecycle`] state machine.
//!
//! ## Ownership
//!
//! [`ZoomController`] is the app-side authority for the zoom level of the
//! active view's site. It keeps one [`SiteZoomLifecycle`] per
//! [`ProfileSiteKey`] (profile-scoped, origin-scoped — never shared across
//! profiles or hosts), backed by an in-memory [`MemorySiteSettingsStore`]
//! that implements the portable [`SiteSettingsStore`] port. The later
//! persistence repository slice replaces the memory store; the lifecycle
//! contract (load → change → persist → ack) is identical either way.
//!
//! Native application goes through an injected target (`Rc<dyn Fn(...)>`);
//! the composition root binds it to the concrete `WebKitZoomAdapter` on
//! macOS. The controller itself never touches WebKit, so every rule is
//! unit-testable with a recording fake target.
//!
//! ## Rules enforced here
//!
//! - Levels come from the contract ladder only ([`ZoomPercent::STEP`] = 25,
//!   bounds 25..=400, default 100) — no invented steps, no continuous zoom.
//! - Applying a level never creates, destroys, moves, or reselects a web
//!   session: the target binds the retained live view, and an unknown view
//!   is a no-op.
//! - A navigation within the same site keeps the level (same lifecycle);
//!   a navigation to a different site loads that site's own level; a fresh
//!   view for a stored site restores the stored level on load and on
//!   activation.
//! - Defaults never create records; reset removes the override (the
//!   lifecycle schedules a `Remove`, never a `Set(100)`).

#[cfg(test)]
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use bbr_web::site_settings::{
    ProfileSiteKey, SiteKey, SiteSettingsOverrides, SiteSettingsStore, SiteZoomLifecycle,
    StoreError, ZoomPercent,
};
use bbr_web::{ProfileId, WebViewId};

/// Native application seam: applies `zoom` to the live view `view` and
/// returns whether the level was actually applied to a live native view
/// (`false` = no live view/session for that id — the caller must NOT record
/// the level as applied). macOS binds this to `WebKitZoomAdapter` via the
/// concrete engine handle; tests inject a recording fake.
pub type ZoomTarget = Rc<dyn Fn(WebViewId, ZoomPercent) -> bool>;

/// In-memory [`SiteSettingsStore`] implementation (the persistence
/// repository slice replaces this later; the port contract is authoritative).
///
/// Records are keyed by [`ProfileSiteKey`]: profile + normalized origin.
/// `get` returns `Ok(None)` for an absent record, `remove` of an absent key
/// succeeds, and the lifecycle never stores a default or empty record.
#[derive(Debug, Default)]
pub struct MemorySiteSettingsStore {
    records: BTreeMap<ProfileSiteKey, SiteSettingsOverrides>,
}

impl MemorySiteSettingsStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SiteSettingsStore for MemorySiteSettingsStore {
    fn get(&mut self, key: &ProfileSiteKey) -> Result<Option<SiteSettingsOverrides>, StoreError> {
        Ok(self.records.get(key).copied())
    }

    fn put(
        &mut self,
        key: &ProfileSiteKey,
        overrides: SiteSettingsOverrides,
    ) -> Result<(), StoreError> {
        if overrides.is_empty() {
            return Err(StoreError::Rejected(
                "the lifecycle never persists an empty override record".into(),
            ));
        }
        self.records.insert(key.clone(), overrides);
        Ok(())
    }

    fn remove(&mut self, key: &ProfileSiteKey) -> Result<(), StoreError> {
        self.records.remove(key);
        Ok(())
    }
}

/// The zoom level state the E2E snapshot reports (mirrored on the wire as
/// `bbr_testkit::protocol::ZoomSnapshot`).
#[cfg(any(feature = "testkit", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomSnapshotState {
    /// Effective zoom percent of the active view's site (the contract
    /// default 100 when no site is resolvable yet).
    pub level: u16,
    /// The normalized origin the level belongs to, if resolvable.
    pub site: Option<String>,
    /// The view the level applies to (identity-preservation evidence).
    pub view: Option<WebViewId>,
    /// The level actually applied to that native view (`None` when the view
    /// has no live session or no site was resolved yet).
    pub applied: Option<u16>,
    /// A persistence write is scheduled but not yet acknowledged.
    pub pending: bool,
}

/// The app-side per-site zoom authority.
pub struct ZoomController {
    store: MemorySiteSettingsStore,
    lifecycles: BTreeMap<ProfileSiteKey, SiteZoomLifecycle>,
    /// Level actually applied to each live view (the native `pageZoom`).
    applied: BTreeMap<WebViewId, ZoomPercent>,
    /// The profile + origin of the active view (resolved from the last
    /// navigation event).
    active_key: Option<ProfileSiteKey>,
    target: ZoomTarget,
}

impl std::fmt::Debug for ZoomController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZoomController")
            .field("lifecycles", &self.lifecycles.len())
            .field("applied", &self.applied)
            .field("active_key", &self.active_key)
            .finish_non_exhaustive()
    }
}

impl ZoomController {
    /// A fresh controller with the native application target.
    pub fn new(target: ZoomTarget) -> Self {
        Self {
            store: MemorySiteSettingsStore::new(),
            lifecycles: BTreeMap::new(),
            applied: BTreeMap::new(),
            active_key: None,
            target,
        }
    }

    /// Resolve the profile-scoped site key of a URL, or `None` when the URL
    /// is not a valid `http(s)` origin (`about:blank`, empty, malformed).
    /// Path/query/fragment are ignored; the port is normalized.
    pub fn resolve_key(profile: ProfileId, url: &str) -> Option<ProfileSiteKey> {
        SiteKey::parse(url)
            .ok()
            .map(|site| ProfileSiteKey::new(profile, site))
    }

    /// A navigation landed on `view` (the `UrlChanged` seam): resolve the
    /// destination site, load its lifecycle (stored level, or the default
    /// when no record exists), and apply the level to the live view. A
    /// same-site navigation keeps the already-applied level; a different
    /// site switches to its own; a fresh view restores the stored level.
    pub fn on_navigation(&mut self, view: WebViewId, profile: ProfileId, url: &str) {
        let Some(key) = Self::resolve_key(profile, url) else {
            return;
        };
        self.active_key = Some(key.clone());
        self.ensure_loaded(&key);
        let level = self
            .lifecycles
            .get(&key)
            .map(SiteZoomLifecycle::effective_zoom)
            .unwrap_or_default();
        if self.applied.get(&view) != Some(&level) {
            self.apply(view, level);
        }
    }

    /// The active view was (re)activated — e.g. a tab switch: apply the
    /// site's current level to the freshly shown view so a hidden view that
    /// missed an earlier change never presents a stale level.
    pub fn on_activated(&mut self, view: WebViewId, profile: ProfileId, url: &str) {
        let Some(key) = Self::resolve_key(profile, url) else {
            return;
        };
        self.active_key = Some(key.clone());
        self.ensure_loaded(&key);
        let level = self
            .lifecycles
            .get(&key)
            .map(SiteZoomLifecycle::effective_zoom)
            .unwrap_or_default();
        if self.applied.get(&view) != Some(&level) {
            self.apply(view, level);
        }
    }

    /// `Cmd-Plus`: one contract step up on the active view's site, clamped
    /// at [`ZoomPercent::MAX`]. Returns the new effective level, or `None`
    /// when no site is resolvable for the active view.
    pub fn zoom_in(&mut self, view: WebViewId, profile: ProfileId, url: &str) -> Option<u16> {
        self.step(view, profile, url, |lifecycle| lifecycle.zoom_in())
    }

    /// `Cmd-Minus`: one contract step down on the active view's site,
    /// clamped at [`ZoomPercent::MIN`].
    pub fn zoom_out(&mut self, view: WebViewId, profile: ProfileId, url: &str) -> Option<u16> {
        self.step(view, profile, url, |lifecycle| lifecycle.zoom_out())
    }

    /// `Cmd-0`: reset the active view's site to the documented default and
    /// remove the override record.
    pub fn zoom_reset(&mut self, view: WebViewId, profile: ProfileId, url: &str) -> Option<u16> {
        self.step(view, profile, url, |lifecycle| lifecycle.reset())
    }

    /// The active site's effective zoom percent (the documented default
    /// when no site is active).
    pub fn active_level(&self) -> u16 {
        self.active_key
            .as_ref()
            .and_then(|key| self.lifecycles.get(key))
            .map(SiteZoomLifecycle::effective_zoom)
            .unwrap_or_default()
            .as_u16()
    }

    /// The snapshot state for the active view.
    #[cfg(any(feature = "testkit", test))]
    pub fn snapshot(&self, view: WebViewId) -> ZoomSnapshotState {
        let level = self
            .active_key
            .as_ref()
            .and_then(|key| self.lifecycles.get(key))
            .map(SiteZoomLifecycle::effective_zoom)
            .unwrap_or_default();
        ZoomSnapshotState {
            level: level.as_u16(),
            site: self.active_key.as_ref().map(|key| key.to_string()),
            view: Some(view),
            applied: self.applied.get(&view).map(|applied| applied.as_u16()),
            pending: self
                .active_key
                .as_ref()
                .and_then(|key| self.lifecycles.get(key))
                .map(SiteZoomLifecycle::has_pending)
                .unwrap_or(false),
        }
    }

    /// Load the lifecycle for `key` from the store when not already loaded.
    /// Absent records load the documented default; loading never writes.
    fn ensure_loaded(&mut self, key: &ProfileSiteKey) {
        if self.lifecycles.contains_key(key) {
            return;
        }
        let stored = self.store.get(key).ok().flatten();
        let lifecycle = SiteZoomLifecycle::load(key.clone(), stored);
        self.lifecycles.insert(key.clone(), lifecycle);
    }

    /// One user zoom change: mutate the lifecycle, flush the pending write
    /// through the store, acknowledge it, and apply the new level natively.
    fn step(
        &mut self,
        view: WebViewId,
        profile: ProfileId,
        url: &str,
        change: impl FnOnce(&mut SiteZoomLifecycle) -> ZoomPercent,
    ) -> Option<u16> {
        let key = Self::resolve_key(profile, url)?;
        self.active_key = Some(key.clone());
        self.ensure_loaded(&key);
        let lifecycle = self.lifecycles.get_mut(&key).expect("just ensured");
        let level = change(lifecycle);
        // Flush the pending write through the portable store port and
        // acknowledge it. A store failure leaves the pending write intact
        // (the contract's retry path); the effective level is still applied
        // because it is the user-visible truth.
        match lifecycle.persist(&mut self.store) {
            Ok(Some(version)) => {
                let _ = lifecycle.ack(version);
            }
            Ok(None) => {}
            Err(error) => {
                log::warn!("zoom persistence failed, write stays pending: {error}");
            }
        }
        self.apply(view, level);
        Some(level.as_u16())
    }

    /// Apply `level` to `view` through the native target and record it —
    /// only when the target actually applied it to a live native view, so
    /// the snapshot's `applied` field is never a claim without a native
    /// backing.
    fn apply(&mut self, view: WebViewId, level: ZoomPercent) {
        if (self.target)(view, level) {
            self.applied.insert(view, level);
        }
    }
}

/// The `BbrApp` zoom composition: the production command paths the
/// `Cmd-Plus`/`Cmd-Minus`/`Cmd-0` actions route to, the navigation and
/// activation hooks, and the typed snapshot accessor. All zoom mutations are
/// synchronous (the native `pageZoom` set is a property write) and never
/// create, destroy, move, or reselect a web session.
impl super::BbrApp {
    /// `Cmd-Plus`: one contract step up on the active view's site.
    pub fn zoom_in(&mut self, cx: &mut gpui::Context<Self>) {
        self.zoom_step(cx, |zoom, view, profile, url| {
            zoom.zoom_in(view, profile, url)
        });
    }

    /// `Cmd-Minus`: one contract step down on the active view's site.
    pub fn zoom_out(&mut self, cx: &mut gpui::Context<Self>) {
        self.zoom_step(cx, |zoom, view, profile, url| {
            zoom.zoom_out(view, profile, url)
        });
    }

    /// `Cmd-0`: reset the active view's site to the documented default and
    /// remove the override record.
    pub fn zoom_reset(&mut self, cx: &mut gpui::Context<Self>) {
        self.zoom_step(cx, |zoom, view, profile, url| {
            zoom.zoom_reset(view, profile, url)
        });
    }

    /// One production zoom change on the active view's site: resolve the
    /// active view + profile + URL, run the contract step, then record the
    /// resulting zoom state for the presentation probe.
    fn zoom_step(
        &mut self,
        cx: &mut gpui::Context<Self>,
        change: impl FnOnce(
            &mut crate::app::zoom::ZoomController,
            bbr_web::WebViewId,
            bbr_web::ProfileId,
            &str,
        ) -> Option<u16>,
    ) {
        let Some((view, profile, url)) = self.zoom_active() else {
            return;
        };
        change(&mut self.zoom, view, profile, &url);
        #[cfg(feature = "testkit")]
        self.observe_zoom_presentation();
        // Blueprint D08: a transient readout confirms each step.
        let percent = u32::from(self.address_zoom_level());
        self.show_zoom_readout(percent, cx);
        cx.notify();
    }

    /// A navigation landed on `view` (the `UrlChanged` seam): switch the
    /// per-site level, keeping the level on same-site navigations and using
    /// the destination site's own level otherwise.
    pub(crate) fn zoom_navigation(&mut self, view: bbr_web::WebViewId, url: &str) {
        let Some(profile) = self
            .controller
            .snapshot()
            .all_tabs
            .iter()
            .find(|tab| tab.web_view == Some(view))
            .map(|tab| tab.profile)
        else {
            return;
        };
        self.zoom.on_navigation(view, profile, url);
    }

    /// The active view changed (tab open/select): apply the site's current
    /// level to the freshly shown view so a hidden view that missed an
    /// earlier change never presents a stale level.
    pub(crate) fn zoom_after_activation(&mut self) {
        let Some((view, profile, url)) = self.zoom_active() else {
            return;
        };
        self.zoom.on_activated(view, profile, &url);
    }

    /// The §17.14 zoom chip the address field presents right now: only
    /// while the rail shows the address field (not a staging surface) and
    /// the zoom is off 100 %.
    #[cfg(feature = "testkit")]
    pub(crate) fn address_zoom_chip(&self) -> Option<u16> {
        let staging = self.command_surface == crate::app::palette::CommandSurface::None
            && self.command_bar_mode.is_active();
        if !self.sidebar_visible || staging {
            return None;
        }
        bbr_ui::components::address::zoom_accessory(self.address_zoom_level())
    }

    /// Blueprint D03: the address control's leading identity mark while
    /// the address renders (never on the staging surface or with the rail
    /// hidden). Testkit evidence only: the render derives the mark itself.
    #[cfg(feature = "testkit")]
    pub(crate) fn rendered_address_mark(&self) -> Option<bbr_ui::components::address::AddressMark> {
        let staging = self.command_surface == crate::app::palette::CommandSurface::None
            && self.command_bar_mode.is_active();
        if !self.sidebar_visible || staging {
            return None;
        }
        bbr_ui::components::address::AddressMark::for_identity(&bbr_ui::text::address_identity(
            &self.active_url(),
        ))
    }

    /// The active site's effective zoom percent (100 without an active
    /// view) — the §17.14 address accessory input.
    pub(crate) fn address_zoom_level(&self) -> u16 {
        if self.zoom_active().is_none() {
            return 100;
        }
        self.zoom.active_level()
    }

    /// The zoom state of the active view for the typed E2E snapshot.
    #[cfg(feature = "testkit")]
    pub fn zoom_snapshot(&self) -> crate::app::zoom::ZoomSnapshotState {
        let Some((view, _, _)) = self.zoom_active() else {
            return crate::app::zoom::ZoomSnapshotState {
                level: 100,
                site: None,
                view: None,
                applied: None,
                pending: false,
            };
        };
        self.zoom.snapshot(view)
    }

    /// Resolve the active view, its profile, and its tab URL — the address
    /// zoom acts on. `None` when no active native view exists.
    fn zoom_active(&self) -> Option<(bbr_web::WebViewId, bbr_web::ProfileId, String)> {
        let snap = self.controller.snapshot();
        let view = snap.active_view?;
        let tab = snap.tabs.get(snap.active_index?)?;
        Some((view, snap.active_profile, tab.url.clone()))
    }

    /// Record the zoom state for the presentation probe after a production
    /// zoom mutation (the zoom benchmarks commit only from this observation).
    #[cfg(feature = "testkit")]
    fn observe_zoom_presentation(&self) {
        let s = self.zoom_snapshot();
        self.performance
            .observe_zoom_state(s.level, s.applied, s.pending);
    }
}

/// The recorded native-application log shared by the recording fake target.
#[cfg(test)]
type TargetLog = Rc<RefCell<Vec<(WebViewId, ZoomPercent)>>>;

/// A recording fake target for unit tests.
#[cfg(test)]
pub(crate) fn recording_target() -> (ZoomTarget, TargetLog) {
    let log: TargetLog = Rc::new(RefCell::new(Vec::new()));
    let target_log = log.clone();
    let target: ZoomTarget = Rc::new(move |view, level| {
        target_log.borrow_mut().push((view, level));
        true
    });
    (target, log)
}

/// A target that never applies — the composition root replaces it with the
/// macOS `WebKitZoomAdapter` binding via [`ZoomController::set_target`] as
/// soon as the concrete engine handle is attached (the same deferred wiring
/// the portal host and the find adapter use). While it is installed, no
/// level is recorded as applied.
pub fn noop_target() -> ZoomTarget {
    Rc::new(|_, _| false)
}

impl ZoomController {
    /// Replace the native application target (the composition root wires the
    /// concrete engine-bound adapter here once the engine is attached).
    pub fn set_target(&mut self, target: ZoomTarget) {
        self.target = target;
    }
}

#[cfg(test)]
mod tests;
