//! Bounded per-profile, host-keyed favicon cache (task packet
//! `real-favicon-pipeline.md`).
//!
//! The controller owns the one favicon acquisition record: every
//! [`bbr_web::WebEvent::FaviconChanged`] image lands here keyed by
//! `(ProfileId, host)`, and the tab presentation model ([`TabFavicon`])
//! plus the Favorite and folder-item presentations all derive from this
//! same record — one acquisition, every presentation.
//!
//! ## Invariants
//!
//! - **Per-profile isolation:** the composite `(ProfileId, host)` key means
//!   two profiles never share a cache entry, exactly like their web
//!   sessions never share a fetch.
//! - **Bounded:** each profile keeps at most [`FAVICON_CACHE_CAP`] records;
//!   eviction is deterministic FIFO (insertion order), so the cache never
//!   grows without bound and a repeated host renders instantly.
//! - **No disk persistence:** this slice owns no favicon blob on disk
//!   (Phase 7 owns persisted state; Saved Pages explicitly store no favicon
//!   blob, DESIGN §11.16).
//! - **Silent fallback:** a missing entry means the §7.2 monogram renders;
//!   acquisition failures never produce a broken row or a retry storm (the
//!   engine emits no event on failure).

use std::collections::{HashMap, VecDeque};

use bbr_web::ProfileId;

/// Maximum favicon image records kept per profile. Bounded so the cache
/// never grows without bound; a repeated host renders instantly from its
/// entry until eviction.
pub(crate) const FAVICON_CACHE_CAP: usize = 64;

/// The registrable host of a URL — the favicon cache key. Mirrors the
/// presentation monogram host extraction (`bbr-ui` `host_of`): scheme
/// stripped, first path/query/fragment segment, explicit port dropped.
/// Empty when unparsable (no favicon is keyed for such URLs).
pub(crate) fn favicon_host(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    host.split(':').next().unwrap_or(host).trim().to_string()
}

/// One profile's bounded host-keyed favicon records.
#[derive(Debug, Default)]
struct ProfileFavicons {
    /// host → resolved image bytes.
    entries: HashMap<String, Vec<u8>>,
    /// Insertion order (FIFO eviction; deterministic).
    order: VecDeque<String>,
}

/// The bounded per-profile favicon store.
#[derive(Debug, Default)]
pub(crate) struct FaviconStore {
    profiles: HashMap<ProfileId, ProfileFavicons>,
    /// Monotonic change counter, bumped on every insert or eviction. The
    /// presentation layer compares it cheaply to know when its decoded
    /// mirror must be refreshed.
    version: u64,
}

impl FaviconStore {
    /// Store the resolved favicon bytes for `(profile, host)`. A repeated
    /// host replaces its record (the newest acquisition wins) without
    /// reordering; a fresh host appends and evicts the profile's oldest
    /// record when the per-profile bound is reached.
    pub(crate) fn insert(&mut self, profile: ProfileId, host: &str, bytes: Vec<u8>) {
        if bytes.is_empty() {
            // An empty payload is not a favicon; treat as acquisition
            // failure (silent monogram fallback).
            return;
        }
        let record = self.profiles.entry(profile).or_default();
        if record.entries.contains_key(host) {
            record.entries.insert(host.to_string(), bytes);
            self.version += 1;
            return;
        }
        if record.order.len() >= FAVICON_CACHE_CAP {
            if let Some(oldest) = record.order.pop_front() {
                record.entries.remove(&oldest);
            }
        }
        record.entries.insert(host.to_string(), bytes);
        record.order.push_back(host.to_string());
        self.version += 1;
    }

    /// The resolved image bytes for `(profile, host)`, if cached.
    pub(crate) fn image_for(&self, profile: ProfileId, host: &str) -> Option<&[u8]> {
        self.profiles
            .get(&profile)
            .and_then(|record| record.entries.get(host))
            .map(Vec::as_slice)
    }

    /// The monotonic change counter.
    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    /// The number of records held for `profile` (bounded-store evidence).
    pub(crate) fn entry_count(&self, profile: ProfileId) -> usize {
        self.profiles
            .get(&profile)
            .map(|record| record.entries.len())
            .unwrap_or(0)
    }

    /// All records of `profile` in deterministic insertion order.
    pub(crate) fn entries_for(&self, profile: ProfileId) -> Vec<(String, Vec<u8>)> {
        let Some(record) = self.profiles.get(&profile) else {
            return Vec::new();
        };
        record
            .order
            .iter()
            .filter_map(|host| {
                record
                    .entries
                    .get(host)
                    .map(|bytes| (host.clone(), bytes.clone()))
            })
            .collect()
    }

    /// Every record across all profiles as `(profile, host, bytes)` (the
    /// presentation layer's decoded-image mirror source).
    pub(crate) fn entries_all(&self) -> Vec<(ProfileId, String, Vec<u8>)> {
        let mut profiles: Vec<(&ProfileId, &ProfileFavicons)> = self.profiles.iter().collect();
        // Deterministic order: profile id, then insertion order within the
        // profile (never map iteration order).
        profiles.sort_by_key(|(profile, _)| profile.0);
        profiles
            .into_iter()
            .flat_map(|(profile, record)| {
                record.order.iter().filter_map(move |host| {
                    record
                        .entries
                        .get(host)
                        .map(|bytes| (*profile, host.clone(), bytes.clone()))
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_extraction_mirrors_the_presentation_key() {
        assert_eq!(favicon_host("https://example.com/a/b"), "example.com");
        assert_eq!(favicon_host("http://localhost:8080/x"), "localhost");
        assert_eq!(favicon_host("https://github.com/"), "github.com");
        assert_eq!(favicon_host("about:blank"), "about");
        assert_eq!(favicon_host(""), "");
    }

    #[test]
    fn repeated_host_replaces_without_reordering_or_duplication() {
        let mut store = FaviconStore::default();
        store.insert(ProfileId(1), "a.example", vec![1]);
        store.insert(ProfileId(1), "b.example", vec![2]);
        store.insert(ProfileId(1), "a.example", vec![9]);
        assert_eq!(
            store.entries_for(ProfileId(1)),
            vec![
                ("a.example".to_string(), vec![9]),
                ("b.example".to_string(), vec![2]),
            ],
            "the newest acquisition wins and insertion order is preserved"
        );
    }

    #[test]
    fn profiles_never_share_entries() {
        let mut store = FaviconStore::default();
        store.insert(ProfileId(1), "shared.example", vec![1]);
        store.insert(ProfileId(2), "shared.example", vec![2]);
        assert_eq!(
            store.image_for(ProfileId(1), "shared.example"),
            Some(&[1][..])
        );
        assert_eq!(
            store.image_for(ProfileId(2), "shared.example"),
            Some(&[2][..])
        );
        // A profile's bound counts only its own records.
        assert_eq!(store.entry_count(ProfileId(1)), 1);
        assert_eq!(store.entry_count(ProfileId(2)), 1);
    }

    #[test]
    fn empty_payload_is_not_a_favicon() {
        let mut store = FaviconStore::default();
        store.insert(ProfileId(1), "a.example", Vec::new());
        assert_eq!(store.image_for(ProfileId(1), "a.example"), None);
        assert_eq!(store.entry_count(ProfileId(1)), 0);
    }

    #[test]
    fn per_profile_bound_evicts_fifo() {
        let mut store = FaviconStore::default();
        for index in 0..=FAVICON_CACHE_CAP {
            store.insert(
                ProfileId(1),
                &format!("host-{index}.example"),
                vec![index as u8],
            );
        }
        assert_eq!(store.entry_count(ProfileId(1)), FAVICON_CACHE_CAP);
        // FIFO: the oldest (host-0) was evicted, the newest retained.
        assert_eq!(store.image_for(ProfileId(1), "host-0.example"), None);
        assert_eq!(
            store.image_for(ProfileId(1), &format!("host-{}.example", FAVICON_CACHE_CAP)),
            Some(&[FAVICON_CACHE_CAP as u8][..])
        );
        // Another profile is untouched.
        assert_eq!(store.entry_count(ProfileId(2)), 0);
    }

    #[test]
    fn version_bumps_on_insert_and_eviction() {
        let mut store = FaviconStore::default();
        assert_eq!(store.version(), 0);
        store.insert(ProfileId(1), "a.example", vec![1]);
        assert_eq!(store.version(), 1);
        store.insert(ProfileId(1), "a.example", vec![2]);
        assert_eq!(store.version(), 2, "replacement bumps the version");
    }
}
