//! Blueprint D06/D10: website-data removal for a profile's WebKit store —
//! deleting a deleted profile's whole persistent store, and clearing kinds
//! of data (cookies and site data, caches) since a date for Clear Browsing
//! Data. The removals are asynchronous in WebKit; failures are logged.

use std::collections::HashMap;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::{AnyThread, MainThreadMarker};
use std::ptr::NonNull;

use objc2_foundation::{NSDate, NSError, NSSet, NSString, NSTimer, NSUUID};
use objc2_web_kit::WKWebsiteDataStore;

use bbr_web::{ProfileId, WebsiteDataKinds};

use crate::store_kind::{store_identifier, StoreKind};

/// Forget the cached store and, when persistent, delete it on disk.
pub(crate) fn delete_store(
    profile: ProfileId,
    kind: StoreKind,
    stores: &mut HashMap<ProfileId, Retained<WKWebsiteDataStore>>,
) {
    stores.remove(&profile);
    let Some(identifier) = store_identifier(kind, profile) else {
        return;
    };
    let uuid_string = NSString::from_str(&identifier);
    let Some(uuid) = NSUUID::initWithUUIDString(NSUUID::alloc(), &uuid_string) else {
        return;
    };
    remove_store(uuid, profile, 0);
}

/// How often a removal is tried while WebKit reports the store in use,
/// and the pause between tries (10 s in all).
const REMOVE_ATTEMPTS: u32 = 40;
const REMOVE_RETRY_SECONDS: f64 = 0.25;

/// `WKWebsiteDataStore`'s error while a web view of the store is alive.
const STORE_IN_USE: isize = 1;

/// Remove a deleted profile's store. The profile's web views are closed
/// first, but WebKit releases their pages asynchronously and reports the
/// store "in use" until then — the first try right after the close always
/// failed, leaving the deleted profile's cookies and storage on disk
/// (found by the persistent website-data E2E, reality-gaps item F). WebKit
/// has no public "released" signal, so the removal is tried again on that
/// error, from the main run loop, up to [`REMOVE_ATTEMPTS`] times.
fn remove_store(uuid: Retained<NSUUID>, profile: ProfileId, attempt: u32) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let retry = uuid.clone();
    let done = RcBlock::new(move |error: *mut NSError| {
        if error.is_null() {
            log::info!("removed the website data of {profile:?}");
            return;
        }
        // SAFETY: WebKit hands a valid NSError when non-null.
        let error = unsafe { &*error };
        if error.code() == STORE_IN_USE && attempt + 1 < REMOVE_ATTEMPTS {
            let uuid = retry.clone();
            let again = RcBlock::new(move |_timer: NonNull<NSTimer>| {
                remove_store(uuid.clone(), profile, attempt + 1);
            });
            // SAFETY: main thread; the block is non-Send (main-thread
            // delivery) and the timer is non-repeating.
            let _ = unsafe {
                NSTimer::scheduledTimerWithTimeInterval_repeats_block(
                    REMOVE_RETRY_SECONDS,
                    false,
                    &again,
                )
            };
            return;
        }
        log::warn!("could not delete the store of {profile:?}: {error:?}");
    });
    // SAFETY: main thread; the identifier and the block live for the call
    // (WebKit copies the block).
    unsafe {
        WKWebsiteDataStore::removeDataStoreForIdentifier_completionHandler(&uuid, &done, mtm)
    };
}

/// Remove `kinds` of data modified since `since` from the profile's store.
pub(crate) fn clear_data(store: &WKWebsiteDataStore, kinds: WebsiteDataKinds, since: Option<f64>) {
    let mut types: Vec<Retained<NSString>> = Vec::new();
    if kinds.cookies_and_site_data {
        for name in [
            "WKWebsiteDataTypeCookies",
            "WKWebsiteDataTypeLocalStorage",
            "WKWebsiteDataTypeSessionStorage",
            "WKWebsiteDataTypeIndexedDBDatabases",
            "WKWebsiteDataTypeWebSQLDatabases",
            "WKWebsiteDataTypeServiceWorkerRegistrations",
            "WKWebsiteDataTypeFileSystem",
        ] {
            types.push(NSString::from_str(name));
        }
    }
    if kinds.caches {
        for name in [
            "WKWebsiteDataTypeDiskCache",
            "WKWebsiteDataTypeMemoryCache",
            "WKWebsiteDataTypeFetchCache",
            "WKWebsiteDataTypeOfflineWebApplicationCache",
        ] {
            types.push(NSString::from_str(name));
        }
    }
    if types.is_empty() {
        return;
    }
    let refs: Vec<&NSString> = types.iter().map(|t| &**t).collect();
    let set = NSSet::from_slice(&refs);
    let date = NSDate::dateWithTimeIntervalSince1970(since.unwrap_or(0.0));
    let done = RcBlock::new(|| {});
    // SAFETY: main thread (engine methods are main-thread-only); the set,
    // date and block live for the call (WebKit copies the block).
    unsafe { store.removeDataOfTypes_modifiedSince_completionHandler(&set, &date, &done) };
}

/// Blueprint D11: whether a data record (named by its registrable domain,
/// or an IP host) belongs to `host`.
pub(crate) fn record_matches(display_name: &str, host: &str) -> bool {
    let name = display_name.trim().to_ascii_lowercase();
    let host = host.trim().to_ascii_lowercase();
    !name.is_empty() && (host == name || host.ends_with(&format!(".{name}")))
}

/// Blueprint D11: remove every kind of data the store holds for `host`
/// (Clear Site Data… in site settings). Asynchronous in WebKit.
pub(crate) fn clear_site(store: &WKWebsiteDataStore, host: &str) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    // SAFETY: main thread.
    let types = unsafe { WKWebsiteDataStore::allWebsiteDataTypes(mtm) };
    let host = host.to_string();
    let target = objc2::Message::retain(store);
    let remove_types = types.clone();
    let fetched = RcBlock::new(
        move |records: std::ptr::NonNull<
            objc2_foundation::NSArray<objc2_web_kit::WKWebsiteDataRecord>,
        >| {
            // SAFETY: WebKit hands a valid array for the call.
            let records = unsafe { records.as_ref() };
            let matching: Vec<Retained<objc2_web_kit::WKWebsiteDataRecord>> = (0..records.count())
                .map(|index| records.objectAtIndex(index))
                // SAFETY: a live record from WebKit's array.
                .filter(|record| {
                    record_matches(&unsafe { record.displayName() }.to_string(), &host)
                })
                .collect();
            if matching.is_empty() {
                return;
            }
            let refs: Vec<&objc2_web_kit::WKWebsiteDataRecord> =
                matching.iter().map(|r| &**r).collect();
            let array = objc2_foundation::NSArray::from_slice(&refs);
            let done = RcBlock::new(|| {});
            // SAFETY: main thread (WebKit calls back on it); the arguments
            // live for the call and WebKit copies the block.
            unsafe {
                target.removeDataOfTypes_forDataRecords_completionHandler(
                    &remove_types,
                    &array,
                    &done,
                )
            };
        },
    );
    // SAFETY: main thread; WebKit copies the block.
    unsafe { store.fetchDataRecordsOfTypes_completionHandler(&types, &fetched) };
}

#[cfg(test)]
mod tests {
    use super::record_matches;

    #[test]
    fn records_match_their_domain_and_its_subdomains() {
        assert!(record_matches("example.com", "example.com"));
        assert!(record_matches("example.com", "meet.example.com"));
        assert!(record_matches("127.0.0.1", "127.0.0.1"));
        assert!(!record_matches("example.com", "notexample.com"));
        assert!(!record_matches("", "example.com"));
    }
}
