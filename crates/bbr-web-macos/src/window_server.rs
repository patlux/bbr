//! Minimal cross-process Window Server queries for E2E evidence (task
//! packet `native-portal-hardening`).
//!
//! The real-app E2E suites run in the *test* process, but the windows they
//! must observe belong to the *app* process. `NSApplication.windows` only
//! enumerates the current process, so the suites enumerate the Window
//! Server's on-screen window list through the public CoreGraphics API and
//! filter by the app's pid. This is the native-boundary complement to the
//! app-side diagnostics: it lets the close scenario prove that closing the
//! browser window while a portal presentation is open leaves **no** BBR
//! window on screen — the portal is never left orphaned, never occluding
//! the desktop, never intercepting input.
//!
//! The CoreGraphics/CoreFoundation surface used here is tiny and read-only:
//! `CGWindowListCopyWindowInfo` plus the array/dictionary/number accessors.
//! All unsafe is contained in this module (the crate's unsafe rule).

use std::ffi::c_void;

/// `kCGWindowListOptionOnScreenOnly` — return only windows currently on
/// screen (the list is otherwise polluted with ordered-out and stale
/// entries that survive for the whole app lifetime).
const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
/// `kCGNullWindowID` — no relative-to-window filter.
const K_CG_NULL_WINDOW_ID: u32 = 0;
/// `kCFNumberSInt64Type` — the pid is read back as a signed 64-bit int.
const K_CF_NUMBER_SINT64_TYPE: u32 = 4;

/// Opaque CoreFoundation handle types (never dereferenced here; every
/// access goes through the CF accessors below). The `_private` field keeps
/// them FFI-safe opaque handles (a bare zero-sized struct is not).
#[repr(C)]
struct __CFArray {
    _private: [u8; 0],
}
#[repr(C)]
struct __CFDictionary {
    _private: [u8; 0],
}
#[repr(C)]
struct __CFString {
    _private: [u8; 0],
}
#[repr(C)]
struct __CFNumber {
    _private: [u8; 0],
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> *const __CFArray;
    static kCGWindowOwnerPID: *const __CFString;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFArrayGetCount(the_array: *const __CFArray) -> isize;
    fn CFArrayGetValueAtIndex(the_array: *const __CFArray, index: isize) -> *const c_void;
    fn CFDictionaryGetValue(
        the_dict: *const __CFDictionary,
        key: *const __CFString,
    ) -> *const c_void;
    fn CFNumberGetValue(number: *const __CFNumber, the_type: u32, value_ptr: *mut c_void) -> bool;
}

/// Count the windows currently on screen that belong to `pid` (the app
/// process under test). Read-only Window Server query; returns 0 when the
/// process has no on-screen window or is no longer alive.
pub fn onscreen_window_count(pid: i32) -> usize {
    // SAFETY: `CGWindowListCopyWindowInfo` is a read-only Window Server
    // snapshot call (no retained handles leak: the returned array is
    // autoreleased and never stored); the CF accessors are the standard
    // read-only CF API family. The returned handles are only ever passed
    // back to CF read functions and never dereferenced.
    unsafe {
        let list =
            CGWindowListCopyWindowInfo(K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY, K_CG_NULL_WINDOW_ID);
        if list.is_null() {
            return 0;
        }
        let count = CFArrayGetCount(list);
        let mut matching = 0usize;
        for index in 0..count {
            let entry = CFArrayGetValueAtIndex(list, index);
            if entry.is_null() {
                continue;
            }
            let owner = CFDictionaryGetValue(entry.cast::<__CFDictionary>(), kCGWindowOwnerPID);
            if owner.is_null() {
                continue;
            }
            let mut owner_pid: i64 = 0;
            if CFNumberGetValue(
                owner.cast::<__CFNumber>(),
                K_CF_NUMBER_SINT64_TYPE,
                (&mut owner_pid as *mut i64).cast::<c_void>(),
            ) && owner_pid == pid as i64
            {
                matching += 1;
            }
        }
        matching
    }
}
