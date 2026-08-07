//! # Per-site zoom lifecycle
//!
//! [`SiteZoomLifecycle`] owns the in-memory zoom state of one
//! [`ProfileSiteKey`] and the deterministic path from a user change to a
//! confirmed persistence write:
//!
//! ```text
//! load(stored) ──> zoom_in / zoom_out / set_zoom / reset
//!                        │
//!                        ▼
//!              pending write (versioned)
//!                        │
//!                        ▼
//!               persist(store) ──> ack(version)
//! ```
//!
//! ## Rules
//!
//! - **Defaults never create records.** A change that lands exactly on the
//!   documented default schedules a *removal* ([`PersistKind::Remove`]), and
//!   a `Set` write is never issued for the default value. Loading a site with
//!   no record performs no store operation at all.
//! - **Reset removes an override.** [`SiteZoomLifecycle::reset`] returns zoom
//!   to the documented default and schedules the override record's removal.
//! - **Supersession.** Every change bumps a version counter; a later change
//!   supersedes the earlier pending write, and `persist` writes only the
//!   latest intent.
//! - **Stale acknowledgements cannot mutate newer state.** [`ack`] accepts
//!   only the version of the outstanding pending write. An older version is
//!   reported [`AckResult::Stale`] and leaves every field untouched; a
//!   version with no outstanding write is [`AckResult::Mismatched`].
//! - **Failures never corrupt state.** A failing [`persist`] or
//!   [`clear_override`] propagates the error and keeps the pending write (or
//!   the previous state) intact for a retry.

use crate::site_settings::{
    ProfileSiteKey, SiteSettingsOverrides, SiteSettingsStore, StoreError, ZoomPercent,
};

/// Deterministic zoom lifecycle for exactly one profile + site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteZoomLifecycle {
    key: ProfileSiteKey,
    effective: ZoomPercent,
    /// Last override confirmed by the persistence layer (`None` = default).
    acknowledged: Option<ZoomPercent>,
    /// Monotonic generation counter of user changes since load.
    version: u64,
    /// The latest scheduled but unacknowledged write, if any.
    pending: Option<PendingPersist>,
}

/// A persistence write scheduled but not yet acknowledged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingPersist {
    version: u64,
    kind: PersistKind,
}

/// What the pending write does to the store record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistKind {
    /// Persist an explicit override at this zoom level.
    Set(ZoomPercent),
    /// Remove the override record (zoom back to default).
    Remove,
}

/// Outcome of confirming a persistence write by version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckResult {
    /// The write matched the outstanding pending change and is acknowledged;
    /// the acknowledged override is now the written value.
    Applied { version: u64 },
    /// The write is older than the outstanding pending change; it was
    /// superseded and ignored. No state changed.
    Stale { version: u64 },
    /// No outstanding write matches this version (a write that was never
    /// issued, already acknowledged, or from the future). No state changed.
    Mismatched { version: u64 },
}

impl SiteZoomLifecycle {
    /// Load the lifecycle from the stored override record.
    ///
    /// `None` (no record) applies the documented defaults. Loading never
    /// creates a record and never touches the store.
    pub fn load(key: ProfileSiteKey, stored: Option<SiteSettingsOverrides>) -> Self {
        let stored_zoom = stored.and_then(SiteSettingsOverrides::zoom);
        Self {
            key,
            effective: stored_zoom.unwrap_or_default(),
            acknowledged: stored_zoom,
            version: 0,
            pending: None,
        }
    }

    /// The profile + site address this lifecycle owns.
    pub fn key(&self) -> &ProfileSiteKey {
        &self.key
    }

    /// The current effective zoom level.
    pub const fn effective_zoom(&self) -> ZoomPercent {
        self.effective
    }

    /// The version of the latest user change (`0` = none since load).
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Whether an unpersisted change is pending.
    pub const fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// The version of the pending write, if any.
    pub const fn pending_version(&self) -> Option<u64> {
        match self.pending {
            Some(pending) => Some(pending.version),
            None => None,
        }
    }

    /// The last stored override confirmed by the persistence layer.
    pub const fn acknowledged_zoom(&self) -> Option<ZoomPercent> {
        self.acknowledged
    }

    /// Zoom in one step and schedule the resulting override write.
    pub fn zoom_in(&mut self) -> ZoomPercent {
        self.record_change(self.effective.zoom_in())
    }

    /// Zoom out one step and schedule the resulting override write.
    pub fn zoom_out(&mut self) -> ZoomPercent {
        self.record_change(self.effective.zoom_out())
    }

    /// Set the zoom level explicitly and schedule the resulting override
    /// write (a removal when the level equals the documented default).
    pub fn set_zoom(&mut self, zoom: ZoomPercent) -> ZoomPercent {
        self.record_change(zoom)
    }

    /// Reset zoom to the documented default and schedule removal of any
    /// override record.
    pub fn reset(&mut self) -> ZoomPercent {
        self.record_change(ZoomPercent::default())
    }

    /// Flush the pending write through the store.
    ///
    /// Returns the written version, or `None` when nothing is pending. The
    /// write is not considered confirmed until the matching
    /// [`ack`](Self::ack); a failure leaves the pending write intact.
    pub fn persist(
        &mut self,
        store: &mut dyn SiteSettingsStore,
    ) -> Result<Option<u64>, StoreError> {
        let Some(pending) = self.pending else {
            return Ok(None);
        };
        match pending.kind {
            PersistKind::Set(zoom) => {
                store.put(&self.key, SiteSettingsOverrides::from_zoom(zoom))?
            }
            PersistKind::Remove => store.remove(&self.key)?,
        }
        Ok(Some(pending.version))
    }

    /// Confirm a persistence write by version.
    ///
    /// Only the version of the outstanding pending write is applied; older
    /// (`Stale`) and unmatched (`Mismatched`) acknowledgements never mutate
    /// state.
    pub fn ack(&mut self, version: u64) -> AckResult {
        match self.pending {
            None => AckResult::Mismatched { version },
            Some(pending) if version == pending.version => {
                self.acknowledged = match pending.kind {
                    PersistKind::Set(zoom) => Some(zoom),
                    PersistKind::Remove => None,
                };
                self.pending = None;
                AckResult::Applied { version }
            }
            Some(pending) if version < pending.version => AckResult::Stale { version },
            Some(_) => AckResult::Mismatched { version },
        }
    }

    /// Remove the override record through the store immediately and cancel
    /// any pending write.
    ///
    /// On success zoom returns to the default, the acknowledged override is
    /// cleared, and no write remains outstanding. On failure nothing changes.
    pub fn clear_override(&mut self, store: &mut dyn SiteSettingsStore) -> Result<(), StoreError> {
        store.remove(&self.key)?;
        self.effective = ZoomPercent::default();
        self.acknowledged = None;
        self.pending = None;
        Ok(())
    }

    /// Discard any pending write and revert to the last acknowledged state
    /// (the stored override, or the default when none exists). The store is
    /// not touched.
    pub fn clear(&mut self) {
        self.pending = None;
        self.effective = self.acknowledged.unwrap_or_default();
    }

    /// Apply a user change: update the effective zoom, supersede any earlier
    /// pending write, and schedule the resulting store intent.
    fn record_change(&mut self, zoom: ZoomPercent) -> ZoomPercent {
        self.effective = zoom;
        self.version += 1;
        let kind = if zoom == ZoomPercent::default() {
            PersistKind::Remove
        } else {
            PersistKind::Set(zoom)
        };
        self.pending = Some(PendingPersist {
            version: self.version,
            kind,
        });
        zoom
    }
}

#[cfg(test)]
mod tests;
