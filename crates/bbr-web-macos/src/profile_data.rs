//! Blueprint D06/D10: website-data removal for a profile's WebKit store —
//! deleting a deleted profile's whole persistent store, and clearing kinds
//! of data (cookies and site data, caches) since a date for Clear Browsing
//! Data. The removals are asynchronous in WebKit; failures are logged.

use std::collections::HashMap;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::{AnyThread, MainThreadMarker};
use objc2_foundation::{NSDate, NSError, NSSet, NSString, NSUUID};
use objc2_web_kit::WKWebsiteDataStore;

use bbr_web::{ProfileId, WebsiteDataKinds};

use crate::native_session::profile_store_uuid;

/// Forget the cached store and, when persistent, delete it on disk.
pub(crate) fn delete_store(
    profile: ProfileId,
    non_persistent: bool,
    stores: &mut HashMap<ProfileId, Retained<WKWebsiteDataStore>>,
) {
    stores.remove(&profile);
    if non_persistent {
        return;
    }
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let uuid_string = NSString::from_str(&profile_store_uuid(profile));
    let Some(uuid) = NSUUID::initWithUUIDString(NSUUID::alloc(), &uuid_string) else {
        return;
    };
    let done = RcBlock::new(move |error: *mut NSError| {
        if !error.is_null() {
            // SAFETY: WebKit hands a valid NSError when non-null.
            let error = unsafe { &*error };
            log::warn!("could not delete the store of {profile:?}: {error:?}");
        }
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
