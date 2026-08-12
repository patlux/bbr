//! The bounded profile-bound download start (task packet
//! `real-favicon-pipeline.md`).
//!
//! One best candidate downloads through a **session bound to the owning
//! profile's data store**: the profile's [`WKHTTPCookieStore`] seeds a
//! private [`NSHTTPCookieStorage`] that backs a bounded ephemeral
//! [`NSURLSession`], so the request carries exactly that profile's cookies
//! and two profiles can never share a fetch, a cache entry, or a session.
//! The async cookie fetch holds the already-consumed concurrency slot; a
//! stale generation at the fetch completion releases it without creating a
//! session.

use std::ptr::NonNull;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{msg_send, AnyThread};
use objc2_foundation::{
    NSArray, NSHTTPCookie, NSHTTPCookieStorage, NSOperationQueue, NSString, NSURLSession,
    NSURLSessionConfiguration, NSURL,
};

use bbr_web::WebViewId;

use super::bounds::FAVICON_DOWNLOAD_DEADLINE;
use super::delegate::{FaviconDownloadDelegate, FaviconDownloadIvars};
use super::{FaviconAcquisitionHost, PendingDownload};

/// Begin one download: seed a private cookie storage from the owning
/// profile's data store, then run the bounded ephemeral session. The
/// async cookie fetch holds the already-consumed concurrency slot; a stale
/// generation at the fetch completion releases it without creating a
/// session.
pub(super) fn begin_download(host: &FaviconAcquisitionHost, pending: PendingDownload) {
    let PendingDownload {
        view,
        generation,
        url,
        web_view,
    } = pending;
    // SAFETY: main thread, valid view; `configuration` is a retained copy
    // of the live view's configuration whose `websiteDataStore` is the
    // owning profile's isolated store.
    let store = unsafe { web_view.configuration().websiteDataStore() };
    let cookie_store = unsafe { store.httpCookieStore() };
    let host = host.clone();
    let seeded_storage = NSHTTPCookieStorage::new();
    let completion = RcBlock::new(move |cookies: NonNull<NSArray<NSHTTPCookie>>| {
        seed_and_download(&host, view, generation, &url, &seeded_storage, cookies);
    });
    // SAFETY: main thread; `cookie_store` is retained and alive; the block
    // is main-thread delivered (WebKit guarantees it).
    unsafe {
        cookie_store.getAllCookies(&completion);
    }
}

/// Seed the private cookie storage from the profile's cookie store and
/// start the bounded session/task. A stale generation or a vanished record
/// releases the slot silently (the view was destroyed/re-registered while
/// the cookie fetch was in flight).
fn seed_and_download(
    host: &FaviconAcquisitionHost,
    view: WebViewId,
    generation: u64,
    url: &str,
    storage: &NSHTTPCookieStorage,
    cookies: NonNull<NSArray<NSHTTPCookie>>,
) {
    let current = {
        let inner = host.shared.inner.borrow();
        inner
            .records
            .get(&view)
            .is_some_and(|record| record.generation == generation && record.active)
    };
    if !current {
        host.release_view(view, generation);
        host.release_slot();
        return;
    }
    // SAFETY: `cookies` is the NonNull array WebKit delivered; it is alive
    // for this main-thread call. Each cookie lands in the private storage,
    // so the download carries exactly this profile's cookies.
    let array = unsafe { cookies.as_ref() };
    let count = array.count();
    for index in 0..count {
        // SAFETY: `index < count` on the live, unmutated array.
        let cookie = unsafe { array.objectAtIndex_unchecked(index) };
        storage.setCookie(cookie.as_ref());
    }

    let Some(ns_url) = NSURL::URLWithString(&NSString::from_str(url)) else {
        host.release_view(view, generation);
        host.release_slot();
        return;
    };

    // The bounded session configuration: ephemeral (no shared cache or
    // cookie storage), the seeded private cookie storage, and the bounded
    // request + resource deadlines.
    let config = NSURLSessionConfiguration::ephemeralSessionConfiguration();
    config.setTimeoutIntervalForRequest(FAVICON_DOWNLOAD_DEADLINE.as_secs_f64());
    config.setTimeoutIntervalForResource(FAVICON_DOWNLOAD_DEADLINE.as_secs_f64());
    config.setHTTPCookieStorage(Some(storage));
    config.setHTTPShouldSetCookies(true);

    // The bounded download delegate (owns the payload accumulation and the
    // size/status enforcement; the session strongly retains it). The class
    // is `AnyThread` (the protocol requires `Send + Sync`), so allocation
    // needs no main-thread marker.
    let delegate = FaviconDownloadDelegate::alloc().set_ivars(FaviconDownloadIvars {
        view,
        generation,
        host: host.host_ref,
        bytes: Mutex::new(Vec::new()),
        terminated: AtomicBool::new(false),
    });
    // SAFETY: `init` is the NSObject initializer; we are on the main thread.
    let delegate: Retained<FaviconDownloadDelegate> = unsafe { msg_send![super(delegate), init] };

    // SAFETY: main thread; the delegate protocol object lives as long as
    // the session (the session retains its delegate); `mainQueue` delivers
    // every delegate callback on the main thread.
    let session = unsafe {
        NSURLSession::sessionWithConfiguration_delegate_delegateQueue(
            &config,
            Some(ProtocolObject::from_ref(&*delegate)),
            Some(&NSOperationQueue::mainQueue()),
        )
    };
    host.shared
        .inner
        .borrow_mut()
        .sessions
        .insert(view, session.clone());
    // SAFETY: main thread; the session is retained (host map + delegate
    // ownership) and alive; the task is retained by the session.
    let task = session.dataTaskWithURL(&ns_url);
    task.resume();
}
