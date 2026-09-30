//! Test-run website-data stores (reality-gaps packet, item F; `testkit`
//! feature only).
//!
//! An E2E run that proves persistence gives its profiles identifier-backed
//! stores in its own namespace (`crate::store_kind`). These helpers list
//! every identifier-backed store WebKit keeps for the app
//! (`fetchAllDataStoreIdentifiers:`) and remove the test runs' stores
//! (`removeDataStoreForIdentifier:`), so a run can prove a deleted profile's
//! store is gone and leave nothing behind. Both answer asynchronously on
//! the main thread.

use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

use block2::RcBlock;
use objc2::MainThreadMarker;
use objc2_foundation::{NSArray, NSError, NSUUID};
use objc2_web_kit::WKWebsiteDataStore;

use crate::store_kind::TEST_STORE_PREFIX;

/// List every identifier-backed store of the app (lowercase UUIDs).
pub fn fetch_identifiers(done: impl Fn(Vec<String>) + 'static) {
    let Some(mtm) = MainThreadMarker::new() else {
        done(Vec::new());
        return;
    };
    let block = RcBlock::new(move |identifiers: NonNull<NSArray<NSUUID>>| {
        // SAFETY: WebKit hands a valid array for the call.
        let identifiers = unsafe { identifiers.as_ref() };
        done(
            identifiers
                .iter()
                .map(|uuid| uuid.UUIDString().to_string().to_lowercase())
                .collect(),
        );
    });
    // SAFETY: main thread; WebKit copies the block.
    unsafe { WKWebsiteDataStore::fetchAllDataStoreIdentifiers(&block, mtm) };
}

/// Remove every store of every test run (identifiers starting with
/// `TEST_STORE_PREFIX`); `done` gets the number removed, or the first
/// error. A store still used by a web view cannot be removed.
pub fn remove_test_stores(done: impl Fn(Result<usize, String>) + 'static) {
    let done = Rc::new(done);
    fetch_identifiers(move |identifiers| {
        let Some(mtm) = MainThreadMarker::new() else {
            done(Err("not on the main thread".into()));
            return;
        };
        let targets: Vec<String> = identifiers
            .into_iter()
            .filter(|id| id.starts_with(TEST_STORE_PREFIX))
            .collect();
        if targets.is_empty() {
            done(Ok(0));
            return;
        }
        let remaining = Rc::new(RefCell::new((targets.len(), None::<String>)));
        let total = targets.len();
        for identifier in targets {
            let uuid = NSUUID::from_string(&objc2_foundation::NSString::from_str(&identifier));
            let Some(uuid) = uuid else {
                done(Err(format!("not a UUID: {identifier}")));
                return;
            };
            let remaining = remaining.clone();
            let done = done.clone();
            let block = RcBlock::new(move |error: *mut NSError| {
                let mut state = remaining.borrow_mut();
                if !error.is_null() && state.1.is_none() {
                    // SAFETY: WebKit hands a valid NSError when non-null.
                    let error = unsafe { &*error };
                    state.1 = Some(error.localizedDescription().to_string());
                }
                state.0 -= 1;
                if state.0 == 0 {
                    match state.1.take() {
                        Some(error) => done(Err(error)),
                        None => done(Ok(total)),
                    }
                }
            });
            // SAFETY: main thread; the identifier and block live for the
            // call (WebKit copies the block).
            unsafe {
                WKWebsiteDataStore::removeDataStoreForIdentifier_completionHandler(
                    &uuid, &block, mtm,
                )
            };
        }
    });
}
