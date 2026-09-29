//! Test driver for a view's `NSTextInputClient` conformance (blueprint D02
//! command-field IME evidence; `testkit` feature only).
//!
//! An input method talks to the key view through `NSTextInputClient`:
//! `setMarkedText:selectedRange:replacementRange:` while composing,
//! `insertText:replacementRange:` to commit, `unmarkText`, and the
//! `markedRange` / `selectedRange` / `firstRectForCharacterRange:` queries
//! it uses to place its candidate window. This driver sends exactly those
//! messages to the GPUI root view, so the real-app E2E exercises the same
//! AppKit → GPUI → input-handler path a Japanese or Chinese input method
//! takes, without installing an input source or moving the user's cursor.
//!
//! Every call must run on the main thread **outside** any GPUI app borrow:
//! the view forwards into the registered GPUI input handler, which updates
//! the app itself.

use std::ffi::c_void;

use objc2::encode::{Encode, Encoding, RefEncode};
use objc2::msg_send;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSNotFound, NSRect, NSString};

/// `NSRange` as GPUI's view declares it (`{NSRange=QQ}`); objc2's `NSRange`
/// encodes AppKit's `{_NSRange=QQ}` tag, which objc2's message verification
/// rejects against GPUI's method signatures.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NSRange {
    location: usize,
    length: usize,
}

impl NSRange {
    fn new(location: usize, length: usize) -> Self {
        Self { location, length }
    }
}

// SAFETY: `#[repr(C)]` two `NSUInteger`s, the layout of AppKit's `NSRange`.
unsafe impl Encode for NSRange {
    const ENCODING: Encoding = Encoding::Struct("NSRange", &[usize::ENCODING, usize::ENCODING]);
}

// SAFETY: as above.
unsafe impl RefEncode for NSRange {
    const ENCODING_REF: Encoding = Encoding::Pointer(&<Self as Encode>::ENCODING);
}

/// What the text-input client reports after an edit (UTF-16 ranges as
/// `[start, end]`; `first_rect` in screen points, bottom-left origin).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextInputReadback {
    pub marked: Option<[usize; 2]>,
    pub selected: Option<[usize; 2]>,
    pub first_rect: Option<[f64; 4]>,
    /// The view's window frame (screen points, bottom-left origin).
    pub window_frame: [f64; 4],
}

fn not_found() -> NSRange {
    NSRange::new(NSNotFound as usize, 0)
}

fn as_view<'a>(view: *mut c_void) -> &'a AnyObject {
    // SAFETY (caller contract): `view` is a live NSView on the main thread.
    unsafe { &*view.cast::<AnyObject>() }
}

fn range_pair(range: NSRange) -> Option<[usize; 2]> {
    (range.location != NSNotFound as usize).then(|| [range.location, range.location + range.length])
}

/// `setMarkedText:selectedRange:replacementRange:` — compose `text` with the
/// UTF-16 `selected` range inside it, replacing the current composition (or
/// the selection).
///
/// # Safety
/// `view` must be a live `NSView*` conforming to `NSTextInputClient`, used
/// on the main thread outside any GPUI app borrow.
pub unsafe fn set_marked_text(view: *mut c_void, text: &str, selected: [usize; 2]) {
    let string = NSString::from_str(text);
    let selected = NSRange::new(selected[0], selected[1].saturating_sub(selected[0]));
    let _: () = unsafe {
        msg_send![as_view(view), setMarkedText: &*string, selectedRange: selected, replacementRange: not_found()]
    };
}

/// `insertText:replacementRange:` — commit `text` over the composition (or
/// the selection).
///
/// # Safety
/// As [`set_marked_text`].
pub unsafe fn insert_text(view: *mut c_void, text: &str) {
    let string = NSString::from_str(text);
    let _: () =
        unsafe { msg_send![as_view(view), insertText: &*string, replacementRange: not_found()] };
}

/// `unmarkText` — keep the composition as typed text.
///
/// # Safety
/// As [`set_marked_text`].
pub unsafe fn unmark_text(view: *mut c_void) {
    let _: () = unsafe { msg_send![as_view(view), unmarkText] };
}

/// Read back the client's marked and selected ranges and the screen rect of
/// the composition (or the selection) — where an input method would open
/// its candidate window.
///
/// # Safety
/// As [`set_marked_text`].
pub unsafe fn read_back(view: *mut c_void) -> TextInputReadback {
    let view = as_view(view);
    let marked: NSRange = unsafe { msg_send![view, markedRange] };
    let selected: NSRange = unsafe { msg_send![view, selectedRange] };
    let probe = if marked.location != NSNotFound as usize {
        Some(marked)
    } else if selected.location != NSNotFound as usize {
        Some(selected)
    } else {
        None
    };
    let first_rect = probe.map(|range| {
        let rect: NSRect = unsafe {
            msg_send![view, firstRectForCharacterRange: range, actualRange: std::ptr::null_mut::<AnyObject>()]
        };
        [rect.origin.x, rect.origin.y, rect.size.width, rect.size.height]
    });
    let window: *mut AnyObject = unsafe { msg_send![view, window] };
    let window_frame = if window.is_null() {
        [0.0; 4]
    } else {
        let frame: NSRect = unsafe { msg_send![&*window, frame] };
        [
            frame.origin.x,
            frame.origin.y,
            frame.size.width,
            frame.size.height,
        ]
    };
    TextInputReadback {
        marked: range_pair(marked),
        selected: range_pair(selected),
        first_rect,
        window_frame,
    }
}
