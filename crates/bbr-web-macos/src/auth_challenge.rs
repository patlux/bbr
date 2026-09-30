//! WebKit's authentication-challenge wrapper.
//!
//! The navigation delegate receives a `WKNSURLAuthenticationChallenge`, an
//! `NSProxy` subclass (`WKObject`) that implements none of
//! `NSURLAuthenticationChallenge`'s methods itself: it forwards them to a
//! real challenge it creates on first use (`forwardingTargetForSelector:`).
//! objc2 checks every message in debug builds by looking the method up on
//! the receiver's class, so messaging the wrapper aborts a debug build
//! inside the delegate callback; release builds skip the check and reach
//! the real challenge through forwarding. The delegate therefore messages
//! the forwarding target, in every build.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{msg_send, sel, Message};
use objc2_foundation::NSURLAuthenticationChallenge;

/// The object that implements the challenge's methods: `challenge` itself,
/// or — for WebKit's proxy — the real challenge it forwards to.
pub(crate) fn real_challenge(
    challenge: &NSURLAuthenticationChallenge,
) -> Retained<NSURLAuthenticationChallenge> {
    let object: &AnyObject = challenge;
    let class = object.class();
    if class.instance_method(sel!(protectionSpace)).is_some()
        || class
            .instance_method(sel!(forwardingTargetForSelector:))
            .is_none()
    {
        return challenge.retain();
    }
    // SAFETY: the receiver's class implements `forwardingTargetForSelector:`
    // (checked above), which returns the object a message is forwarded to,
    // or nil; the downcast checks its class before it is used as one.
    let target: Option<Retained<AnyObject>> =
        unsafe { msg_send![object, forwardingTargetForSelector: sel!(protectionSpace)] };
    target
        .and_then(|target| target.downcast::<NSURLAuthenticationChallenge>().ok())
        .unwrap_or_else(|| challenge.retain())
}
