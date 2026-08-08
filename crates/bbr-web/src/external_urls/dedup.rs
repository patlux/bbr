//! Burst deduplication for rapidly re-delivered external URLs.
//!
//! The operating system can deliver the same URL several times in a quick
//! burst (a double-click on a link in another app, a universal link
//! delivered twice). Within a bounded window the store routes the URL once
//! and reports later arrivals as duplicates, so a burst of `n` identical
//! opens causes exactly one navigation.
//!
//! ## Determinism without a wall clock
//!
//! The contract contains no time source. The caller supplies a **monotonic
//! tick** for every arrival (typically the adapter's per-event counter), and
//! the window is measured in ticks: an entry first seen at tick `t` is still
//! a duplicate while `current_tick - t <= window_ticks` and has expired once
//! `current_tick - t > window_ticks`. A duplicate never refreshes the
//! window — it is anchored to the **first** observation, so a stream of
//! identical arrivals can never extend the burst; after the window elapses
//! the URL is fresh again and routes anew.
//!
//! ## The dedup key is privacy-safe
//!
//! The tracker retains a key built from the **reduced origin**
//! (`scheme://host[:port]`, lower-cased, userinfo stripped — the same
//! reduction the sibling web-error contract establishes) plus a
//! non-cryptographic FNV-1a hash of the untransformed remainder
//! (path+query+fragment). Identical URLs always collide; any byte difference
//! in the remainder (including the fragment) does not; scheme/host case
//! differences collapse because they denote the same origin. Full URLs,
//! credentials, and query strings are never retained.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, VecDeque};

use crate::external_urls::{ExternalUrlRequestId, WebUrl};

/// Bounded-burst configuration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BurstConfig {
    window_ticks: u64,
    max_remembered: usize,
}

impl BurstConfig {
    /// `window_ticks` ≥ 1 (a burst of zero width is meaningless) and
    /// `max_remembered` ≥ 1 (the tracker must remember at least one URL).
    pub fn new(window_ticks: u64, max_remembered: usize) -> Result<Self, BurstConfigError> {
        if window_ticks == 0 {
            return Err(BurstConfigError::ZeroWindowTicks);
        }
        if max_remembered == 0 {
            return Err(BurstConfigError::ZeroCapacity);
        }
        Ok(Self {
            window_ticks,
            max_remembered,
        })
    }
}

/// Why a [`BurstConfig`] was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurstConfigError {
    ZeroWindowTicks,
    ZeroCapacity,
}

/// Whether an observed URL is a first arrival or a repeat within the burst.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupVerdict {
    /// First arrival within the window; the caller may route.
    Fresh,
    /// The same URL was already observed inside the window; do not route
    /// again.
    Duplicate {
        /// The request id of the first observation, still in the window.
        original: ExternalUrlRequestId,
        /// Total observations of this URL so far, including this one.
        observations: u64,
    },
}

/// Deterministic burst tracker. See the module docs for the window model.
///
/// The caller guarantees non-decreasing ticks (the coordinator enforces it);
/// with non-decreasing ticks the age `tick - first_seen_tick` can never
/// underflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BurstTracker {
    config: BurstConfig,
    entries: BTreeMap<DedupKey, SeenEntry>,
    /// Insertion order (oldest first) for deterministic expiry and capacity
    /// eviction; ticks are non-decreasing, so age order equals tick order.
    oldest: VecDeque<DedupKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SeenEntry {
    first_seen_tick: u64,
    original_id: ExternalUrlRequestId,
    observations: u64,
}

/// Privacy-safe dedup key: the reduced origin plus a hash of the raw
/// remainder. See the module docs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DedupKey {
    origin: String,
    tail_hash: u64,
}

impl DedupKey {
    fn from_web_url(url: &WebUrl) -> Self {
        Self {
            origin: url.reduced_origin(),
            tail_hash: fnv1a(url.remainder.as_bytes()),
        }
    }
}

impl BurstTracker {
    /// A tracker with the given bounded-burst configuration.
    pub fn new(config: BurstConfig) -> Self {
        Self {
            config,
            entries: BTreeMap::new(),
            oldest: VecDeque::new(),
        }
    }

    /// Observe one arrival of `url` at `tick` under request identity `id`.
    ///
    /// Prunes expired entries first, then reports a duplicate when the URL is
    /// still in the window, otherwise records a fresh arrival (evicting the
    /// oldest entry when the remembered set is at capacity).
    pub fn observe(&mut self, tick: u64, url: &WebUrl, id: ExternalUrlRequestId) -> DedupVerdict {
        let key = DedupKey::from_web_url(url);
        self.prune(tick);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.observations += 1;
            return DedupVerdict::Duplicate {
                original: entry.original_id,
                observations: entry.observations,
            };
        }
        self.evict_if_full();
        let entry = SeenEntry {
            first_seen_tick: tick,
            original_id: id,
            observations: 1,
        };
        self.entries.insert(key.clone(), entry);
        self.oldest.push_back(key);
        DedupVerdict::Fresh
    }

    /// Drop entries whose window elapsed: `tick - first_seen_tick > window_ticks`.
    /// The window is anchored to the first observation; duplicates never
    /// refresh it.
    fn prune(&mut self, tick: u64) {
        while let Some(front) = self.oldest.front().cloned() {
            let expired = match self.entries.get(&front) {
                Some(entry) => tick - entry.first_seen_tick > self.config.window_ticks,
                // Defensive: a stale index is always droppable.
                None => true,
            };
            if !expired {
                break;
            }
            self.oldest.pop_front();
            self.entries.remove(&front);
        }
    }

    /// Bound the remembered set at `max_remembered`, evicting the oldest
    /// entries first (insertion order).
    fn evict_if_full(&mut self) {
        while self.entries.len() >= self.config.max_remembered {
            let Some(front) = self.oldest.pop_front() else {
                break;
            };
            self.entries.remove(&front);
        }
    }
}

/// FNV-1a 64-bit (offset basis and prime from the FNV spec). A
/// non-cryptographic structural fingerprint: deterministic across builds
/// (unlike the std `DefaultHasher`), used only to tell identical remainders
/// apart without retaining them.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
